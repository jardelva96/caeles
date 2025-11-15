use crate::manifest::CapsuleManifest;
use anyhow::Result;
use reqwest::blocking;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use wasmtime::{Caller, Engine, Extern, Linker, Module, Store};

/// Lê uma string da memória exportada "memory" da cápsula.
fn read_string_from_memory(
    caller: &mut Caller<'_, ()>,
    ptr: i32,
    len: i32,
) -> Option<String> {
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => {
            eprintln!("[caeles-runtime] cápsula não exporta memória \"memory\"");
            return None;
        }
    };

    let mut buf = vec![0u8; len as usize];
    if let Err(e) = memory.read(caller, ptr as usize, &mut buf) {
        eprintln!("[caeles-runtime] erro lendo memória da cápsula: {e}");
        return None;
    }

    match String::from_utf8(buf) {
        Ok(s) => Some(s),
        Err(_) => {
            eprintln!("[caeles-runtime] bytes não são UTF-8 válidos");
            None
        }
    }
}

pub fn run_capsule(manifest: &CapsuleManifest) -> Result<()> {
    // Engine do CAELES
    let engine = Engine::default();

    let module_path = manifest.wasm_path();
    println!("> Carregando cápsula: {}", module_path.display());

    // Carrega o módulo WASM da cápsula (wasm32-unknown-unknown)
    let module = Module::from_file(&engine, &module_path)?;

    // Store sem estado customizado (por enquanto)
    let mut store = Store::new(&engine, ());

    // Linker para registrar imports que a cápsula espera
    let mut linker = Linker::new(&engine);

    // =========================
    // Estado de MÉTRICAS no host
    // =========================
    let metrics_map: Arc<Mutex<HashMap<String, i64>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let metrics_for_import = metrics_map.clone();

    // -------------------------
    // Import "caeles"."host_log"
    // -------------------------
    linker.func_wrap(
        "caeles",
        "host_log",
        |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
            if let Some(msg) = read_string_from_memory(&mut caller, ptr, len) {
                println!("[capsule-log] {msg}");
            }
        },
    )?;

    // ----------------------------
    // Import "caeles"."host_notify"
    // Respeita permissions.notifications do manifest
    // ----------------------------
    let notifications_allowed = manifest.permissions.notifications;

    linker.func_wrap(
        "caeles",
        "host_notify",
        move |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
            if let Some(msg) = read_string_from_memory(&mut caller, ptr, len) {
                if notifications_allowed {
                    println!("[capsule-notify] {msg}");
                } else {
                    println!(
                        "[capsule-notify BLOQUEADA] Permissão 'notifications' = false. Mensagem seria: {msg}"
                    );
                }
            }
        },
    )?;

    // ----------------------------
    // Import "caeles"."host_http_get"
    // Usa permissions.network para permitir/bloquear acesso
    // ----------------------------
    let network_allowed = manifest.permissions.network;

    linker.func_wrap(
        "caeles",
        "host_http_get",
        move |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
            if let Some(url) = read_string_from_memory(&mut caller, ptr, len) {
                if !network_allowed {
                    println!(
                        "[capsule-http BLOQUEADO] Permissão 'network' = false. Requisição para: {url}"
                    );
                    return;
                }

                println!("[capsule-http] realizando GET em: {url}");

                match blocking::get(&url) {
                    Ok(resp) => {
                        let status = resp.status();
                        let text = resp
                            .text()
                            .unwrap_or_else(|_| "<erro lendo corpo>".to_string());
                        let snippet: String = text.chars().take(120).collect();
                        println!(
                            "[capsule-http] status: {status}, body (prefixo): {}",
                            snippet.replace('\n', " ")
                        );
                    }
                    Err(e) => {
                        println!("[capsule-http ERRO] Falha ao fazer GET: {e}");
                    }
                }
            }
        },
    )?;

    // ----------------------------
    // Import "caeles"."host_metric_inc"
    // Usa permissions.metrics para permitir/bloquear
    // ----------------------------
    let metrics_allowed = manifest.permissions.metrics;

    linker.func_wrap(
        "caeles",
        "host_metric_inc",
        move |mut caller: Caller<'_, ()>, name_ptr: i32, name_len: i32, delta: i64| {
            if let Some(name) = read_string_from_memory(&mut caller, name_ptr, name_len) {
                if !metrics_allowed {
                    println!(
                        "[capsule-metric BLOQUEADA] Métricas desabilitadas no manifest. name={name}, delta={delta}"
                    );
                    return;
                }

                let mut map = metrics_for_import
                    .lock()
                    .expect("poisoned metrics mutex");
                let entry = map.entry(name.clone()).or_insert(0);
                *entry += delta;
                println!("[capsule-metric] {name} += {delta} (total = {entry})");
            }
        },
    )?;

    // ----------------------------
    // Import "caeles"."host_store_event"
    // Usa permissions.storage para permitir/bloquear
    // ----------------------------
    let storage_allowed = manifest.permissions.storage;
    let capsule_id = manifest.id.clone();
    let data_dir = PathBuf::from("data");

    linker.func_wrap(
        "caeles",
        "host_store_event",
        move |mut caller: Caller<'_, ()>,
              key_ptr: i32,
              key_len: i32,
              payload_ptr: i32,
              payload_len: i32| {
            let key = match read_string_from_memory(&mut caller, key_ptr, key_len) {
                Some(k) => k,
                None => return,
            };

            let payload =
                match read_string_from_memory(&mut caller, payload_ptr, payload_len) {
                    Some(p) => p,
                    None => return,
                };

            if !storage_allowed {
                println!(
                    "[capsule-store BLOQUEADO] Permissão 'storage' = false. Evento: key={key}"
                );
                return;
            }

            if let Err(e) = fs::create_dir_all(&data_dir) {
                eprintln!("[caeles-runtime] erro criando pasta de dados: {e}");
                return;
            }

            let file = data_dir.join(format!("events-{}.log", capsule_id));
            let line = format!("key={key} payload={payload}\n");

            let result = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file)
                .and_then(|mut f| f.write_all(line.as_bytes()));

            match result {
                Ok(_) => {
                    println!("[capsule-store] evento gravado em {:?}", file);
                }
                Err(e) => {
                    eprintln!(
                        "[caeles-runtime] erro gravando evento em {:?}: {e}",
                        file
                    );
                }
            }
        },
    )?;

    // Instancia o módulo com os imports registrados
    let instance = linker.instantiate(&mut store, &module)?;

    // A "entrypoint" padrão da cápsula CAELES será a função exportada `caeles_main`
    let func = instance.get_typed_func::<(), ()>(&mut store, "caeles_main")?;

    println!("> Chamando caeles_main da cápsula...");
    func.call(&mut store, ())?;
    println!("> caeles_main terminou.");

    // Se houver métricas registradas, imprime um resumo no final
    let metrics_snapshot = metrics_map.lock().expect("poisoned metrics mutex");
    if !metrics_snapshot.is_empty() {
        println!("> Resumo de métricas desta execução:");
        let mut keys: Vec<_> = metrics_snapshot.keys().cloned().collect();
        keys.sort();
        for k in keys {
            if let Some(v) = metrics_snapshot.get(&k) {
                println!("  - {k} = {v}");
            }
        }
    }

    Ok(())
}
