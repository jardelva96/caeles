use crate::manifest::CapsuleManifest;
use anyhow::{Context, Result};
use wasmtime::{Caller, Engine, Extern, Linker, Module, Store};

fn read_string_from_memory(mut caller: Caller<'_, ()>, ptr: i32, len: i32) -> Option<String> {
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => {
            eprintln!("[caeles-runtime] cápsula não exporta memória \"memory\"");
            return None;
        }
    };

    let mut buf = vec![0u8; len as usize];
    if let Err(e) = memory.read(&mut caller, ptr as usize, &mut buf) {
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
    let engine = Engine::default();

    let module_path = manifest.wasm_path();
    println!(
        "> Executando cápsula '{}' (id={}, version={})",
        manifest.name, manifest.id, manifest.version
    );
    println!(
        "> Permissões: notifications={}, network={}",
        manifest.permissions.notifications, manifest.permissions.network
    );
    println!("> Carregando cápsula: {}", module_path.display());

    let module = Module::from_file(&engine, &module_path).with_context(|| {
        format!(
            "Falha ao carregar módulo WASM '{}'. Compile a cápsula antes de executar.",
            module_path.display()
        )
    })?;

    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);

    linker.func_wrap(
        "caeles",
        "host_log",
        |caller: Caller<'_, ()>, ptr: i32, len: i32| {
            if let Some(msg) = read_string_from_memory(caller, ptr, len) {
                println!("[capsule-log] {msg}");
            }
        },
    )?;

    let notifications_allowed = manifest.permissions.notifications;
    linker.func_wrap(
        "caeles",
        "host_notify",
        move |caller: Caller<'_, ()>, ptr: i32, len: i32| {
            if let Some(msg) = read_string_from_memory(caller, ptr, len) {
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

    let instance = linker.instantiate(&mut store, &module)?;
    let func = instance.get_typed_func::<(), ()>(&mut store, "caeles_main")?;

    println!("> Chamando caeles_main da cápsula...");
    func.call(&mut store, ())?;
    println!("> caeles_main terminou.");

    Ok(())
}
