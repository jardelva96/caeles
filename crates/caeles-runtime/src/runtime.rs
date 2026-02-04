use crate::manifest::CapsuleManifest;
use anyhow::{Context, Result};
use wasmtime::{Caller, Engine, Extern, Linker, Module, Store};

// Configurações de limites de recursos para segurança
const MAX_STRING_LEN: i32 = 1024 * 1024; // 1 MB

fn read_string_from_memory(
    mut caller: Caller<'_, ()>,
    ptr: i32,
    len: i32,
) -> Option<String> {
    if ptr < 0 || len < 0 {
        eprintln!("[caeles-runtime] ponteiros inválidos: ptr={}, len={}", ptr, len);
        return None;
    }

    // Limite de tamanho de string para evitar DoS
    if len > MAX_STRING_LEN {
        eprintln!(
            "[caeles-runtime] string muito grande: {} bytes (limite: {} bytes)",
            len, MAX_STRING_LEN
        );
        return None;
    }

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
    // Engine do CAELES com configurações de segurança
    let mut config = wasmtime::Config::new();
    config.consume_fuel(true); // Habilita medição de "fuel" para limitar execução
    
    let engine = Engine::new(&config)
        .context("Falha ao criar engine WebAssembly")?;

    let module_path = manifest.wasm_path();
    
    // Verifica se o arquivo WASM existe
    if !module_path.exists() {
        anyhow::bail!(
            "Arquivo WASM não encontrado: {}\nVerifique se a cápsula foi compilada corretamente.",
            module_path.display()
        );
    }
    
    println!("> Carregando cápsula: {}", module_path.display());

    // Carrega o módulo WASM da cápsula (wasm32-unknown-unknown)
    let module = Module::from_file(&engine, &module_path)
        .with_context(|| format!("Falha ao carregar módulo WASM: {}", module_path.display()))?;

    // Store com contexto vazio mas com fuel habilitado
    let mut store = Store::new(&engine, ());
    
    // Define fuel para evitar loops infinitos (aproximadamente 100M instruções)
    store.set_fuel(100_000_000)
        .context("Falha ao configurar fuel")?;

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
    )
    .context("Falha ao registrar host_log")?;

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
    )
    .context("Falha ao registrar host_notify")?;

    // Instancia o módulo com os imports registrados
    let instance = linker.instantiate(&mut store, &module)
        .context("Falha ao instanciar módulo WASM")?;

    // A "entrypoint" padrão da cápsula CAELES será a função exportada `caeles_main`
    let func = instance
        .get_typed_func::<(), ()>(&mut store, "caeles_main")
        .context("Função 'caeles_main' não encontrada. Certifique-se de que a cápsula exporta essa função com #[no_mangle]")?;

    println!("> Chamando caeles_main da cápsula...");
    func.call(&mut store, ())
        .context("Erro durante execução da cápsula")?;
    
    let remaining_fuel = store.get_fuel()
        .context("Falha ao obter fuel restante")?;
    let consumed_fuel = 100_000_000 - remaining_fuel;
    
    println!("> caeles_main terminou.");
    println!("> Fuel consumido: {} instruções", consumed_fuel);

    Ok(())
}
