use crate::manifest::CapsuleManifest;
use anyhow::{Context, Result};
use wasmtime::{Caller, Engine, Extern, Linker, Module, Store};

fn read_string_from_memory(mut caller: Caller<'_, ()>, ptr: i32, len: i32) -> Option<String> {
    if ptr < 0 || len < 0 {
        eprintln!("[caeles-runtime] invalid pointer or length (ptr={ptr}, len={len})");
        return None;
    }

    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => {
            eprintln!("[caeles-runtime] capsule does not export memory \"memory\"");
            return None;
        }
    };

    let mut buf = vec![0u8; len as usize];
    if let Err(err) = memory.read(&mut caller, ptr as usize, &mut buf) {
        eprintln!("[caeles-runtime] error reading capsule memory: {err}");
        return None;
    }

    match String::from_utf8(buf) {
        Ok(value) => Some(value),
        Err(_) => {
            eprintln!("[caeles-runtime] bytes are not valid UTF-8");
            None
        }
    }
}

pub fn run_capsule(manifest: &CapsuleManifest) -> Result<()> {
    let engine = Engine::default();

    let module_path = manifest.wasm_path();
    println!(
        "> Executing capsule '{}' (id={}, version={})",
        manifest.name, manifest.id, manifest.version
    );
    println!(
        "> Permissions: notifications={}, network={}",
        manifest.permissions.notifications, manifest.permissions.network
    );
    println!("> Loading capsule: {}", module_path.display());

    let module = Module::from_file(&engine, &module_path).with_context(|| {
        format!(
            "Failed to load WASM module '{}'. Build the capsule before running.",
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
                        "[capsule-notify BLOCKED] permission 'notifications' = false. Message: {msg}"
                    );
                }
            }
        },
    )?;

    let network_allowed = manifest.permissions.network;
    linker.func_wrap(
        "caeles",
        "host_http_get",
        move |caller: Caller<'_, ()>, ptr: i32, len: i32| -> i32 {
            let Some(url) = read_string_from_memory(caller, ptr, len) else {
                return 2;
            };

            if !network_allowed {
                println!(
                    "[capsule-network BLOCKED] permission 'network' = false. Requested URL: {url}"
                );
                return 1;
            }

            if !(url.starts_with("http://") || url.starts_with("https://")) {
                println!("[capsule-network ERROR] invalid URL (use http:// or https://): {url}");
                return 2;
            }

            match ureq::get(&url).call() {
                Ok(response) => {
                    println!("[capsule-network] GET {} -> {}", url, response.status());
                    0
                }
                Err(err) => {
                    println!("[capsule-network ERROR] GET {} failed: {}", url, err);
                    3
                }
            }
        },
    )?;

    let instance = linker.instantiate(&mut store, &module)?;
    let func = instance.get_typed_func::<(), ()>(&mut store, "caeles_main")?;

    println!("> Calling capsule caeles_main...");
    func.call(&mut store, ())?;
    println!("> caeles_main finished.");

    Ok(())
}
