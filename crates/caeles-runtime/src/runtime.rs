use crate::manifest::CapsuleManifest;
use anyhow::Result;
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
    // Engine do CAELES
    let engine = Engine::default();

    let module_path = manifest.wasm_path();
    if !module_path.exists() {
        anyhow::bail!(
            "Arquivo WASM não encontrado em '{}'. Compile a cápsula com \
             `cargo build --target wasm32-unknown-unknown` e verifique o campo \
             `entry` do manifest.",
            module_path.display()
        );
    }
    println!("> Carregando cápsula: {}", module_path.display());

    // Carrega o módulo WASM da cápsula (wasm32-unknown-unknown)
    let module = Module::from_file(&engine, &module_path)?;

    // O runtime atual não fornece WASI. Se a cápsula importar WASI, falhamos
    // explicitamente com uma mensagem clara para evitar erros de link em tempo
    // de execução e tentativas de usar wasmtime_wasi com APIs antigas.
    if let Some(import) = module
        .imports()
        .find(|import| import.module().starts_with("wasi"))
    {
        anyhow::bail!(
            "Cápsula requer WASI (módulo de importação: '{}'). Construa a cápsula \
             para wasm32-unknown-unknown ou adicione suporte WASI ao runtime antes de executar.",
            import.module()
        );
    }

    // Store sem estado (por enquanto não temos contexto customizado)
    let mut store = Store::new(&engine, ());

    // Linker para registrar imports que a cápsula espera
    let mut linker = Linker::new(&engine);

    // -------------------------
    // Import "caeles"."host_log"
    // -------------------------
    linker.func_wrap(
        "caeles",
        "host_log",
        |caller: Caller<'_, ()>, ptr: i32, len: i32| {
            if let Some(msg) = read_string_from_memory(caller, ptr, len) {
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

    // Instancia o módulo com os imports registrados
    let instance = linker.instantiate(&mut store, &module)?;

    // A "entrypoint" padrão da cápsula CAELES será a função exportada `caeles_main`
    let func = instance.get_typed_func::<(), ()>(&mut store, "caeles_main")?;

    println!("> Chamando caeles_main da cápsula...");
    func.call(&mut store, ())?;
    println!("> caeles_main terminou.");

    Ok(())
}
