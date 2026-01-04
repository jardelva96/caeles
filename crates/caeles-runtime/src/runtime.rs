use crate::manifest::{CapsuleManifest, ValidatedPreopen};
use anyhow::{Context, Result};
use wasmtime::{Caller, Config, Engine, Extern, ExternType, Instance, Linker, Module, Store};
use wasmtime_wasi::ambient_authority;
use wasmtime_wasi::preview1::{
    add_to_linker, DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView,
};
use wasmtime_wasi::Dir;

#[derive(Default)]
struct RuntimeState {
    wasi: WasiCtx,
    notifications_allowed: bool,
}

impl WasiView for RuntimeState {
    fn ctx(&self) -> &WasiCtx {
        &self.wasi
    }

    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

fn read_string_from_memory(
    mut caller: Caller<'_, RuntimeState>,
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

fn ensure_memory_export(module: &Module) -> Result<()> {
    let has_memory = module
        .exports()
        .any(|export| matches!(export.ty(), ExternType::Memory(_)) && export.name() == "memory");

    if !has_memory {
        anyhow::bail!("Módulo não exporta memória \"memory\"");
    }

    Ok(())
}

fn detect_network_imports(module: &Module) -> bool {
    module.imports().any(|import| {
        let module_name = import.module();
        let field = import.name();

        let preview1_sock = module_name == "wasi_snapshot_preview1" && field.starts_with("sock_");
        let preview1_network_ext = module_name == "wasi_snapshot_preview1"
            && (field.starts_with("tcp_") || field.starts_with("udp_"));

        let component_socket = module_name.starts_with("wasi:io/socket");
        let component_net =
            module_name.starts_with("wasi:net") || module_name.starts_with("wasi:sockets");

        preview1_sock || preview1_network_ext || component_socket || component_net
    })
}

fn configure_engine() -> Result<Engine> {
    let mut config = Config::new();
    config.wasm_multi_memory(true);
    config.wasm_component_model(true);
    Engine::new(&config).map_err(Into::into)
}

fn build_stdio_pipes(manifest: &CapsuleManifest, builder: &mut WasiCtxBuilder) -> Result<()> {
    if manifest.permissions.inherit_stdio {
        builder.inherit_stdio();
    }

    Ok(())
}

fn add_env(manifest: &CapsuleManifest, builder: &mut WasiCtxBuilder) -> Result<()> {
    for (key, value) in manifest.validated_env()? {
        builder.env(&key, &value)?;
    }
    Ok(())
}

fn open_preopen_dir(preopen: &ValidatedPreopen) -> Result<Dir> {
    let dir = Dir::open_ambient_dir(&preopen.host, ambient_authority()).with_context(|| {
        format!(
            "Falha ao abrir diretório preaberto {}",
            preopen.host.display()
        )
    })?;

    Ok(dir)
}

fn add_preopens(manifest: &CapsuleManifest, builder: &mut WasiCtxBuilder) -> Result<()> {
    for preopen in manifest.validated_preopens()? {
        let dir = open_preopen_dir(&preopen)?;
        let guest = preopen.guest.to_str().ok_or_else(|| {
            anyhow::anyhow!("guest path não é UTF-8: {}", preopen.guest.display())
        })?;

        if preopen.read_only {
            builder.preopened_dir_with_capabilities(dir, guest, DirPerms::READ, FilePerms::READ)?;
        } else {
            builder.preopened_dir(dir, guest)?;
        }
    }
    Ok(())
}

fn build_wasi_ctx(manifest: &CapsuleManifest) -> Result<WasiCtx> {
    let mut builder = WasiCtxBuilder::new();

    builder.arg(&manifest.id)?;

    add_env(manifest, &mut builder)?;
    build_stdio_pipes(manifest, &mut builder)?;

    if manifest.permissions.network {
        builder.inherit_network();
    }

    add_preopens(manifest, &mut builder)?;

    Ok(builder.build())
}

fn instantiate_module(
    manifest: &CapsuleManifest,
    engine: &Engine,
    module: &Module,
) -> Result<(Store<RuntimeState>, Instance)> {
    let wasi = build_wasi_ctx(manifest)?;
    let state = RuntimeState {
        wasi,
        notifications_allowed: manifest.permissions.notifications,
    };

    let mut store = Store::new(engine, state);
    let mut linker = Linker::new(engine);
    add_to_linker(&mut linker, |state: &mut RuntimeState| state)?;

    linker.func_wrap(
        "caeles",
        "host_log",
        |caller: Caller<'_, RuntimeState>, ptr: i32, len: i32| {
            if let Some(msg) = read_string_from_memory(caller, ptr, len) {
                println!("[capsule-log] {msg}");
            }
        },
    )?;

    linker.func_wrap(
        "caeles",
        "host_notify",
        |caller: Caller<'_, RuntimeState>, ptr: i32, len: i32| {
            if let Some(msg) = read_string_from_memory(caller, ptr, len) {
                if caller.data().notifications_allowed {
                    println!("[capsule-notify] {msg}");
                } else {
                    println!(
                        "[capsule-notify BLOQUEADA] Permissão 'notifications' = false. Mensagem seria: {msg}"
                    );
                }
            }
        },
    )?;

    let instance = linker.instantiate(&mut store, module)?;

    Ok((store, instance))
}

pub fn run_capsule(manifest: &CapsuleManifest) -> Result<()> {
    let engine = configure_engine()?;

    let module_path = manifest.wasm_path();
    println!("> Carregando cápsula: {}", module_path.display());

    let module = Module::from_file(&engine, &module_path)?;

    ensure_memory_export(&module)?;

    if detect_network_imports(&module) && !manifest.permissions.network {
        anyhow::bail!("Módulo requer APIs de rede, mas permissions.network = false");
    }

    let (mut store, instance) = instantiate_module(manifest, &engine, &module)?;

    let func = instance.get_typed_func::<(), ()>(&mut store, "caeles_main")?;

    println!("> Chamando caeles_main da cápsula...");
    func.call(&mut store, ())?;
    println!("> caeles_main terminou.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use wat::parse_str;

    fn write_wasm(temp: &TempDir, wasm_name: &str, wat_src: &str) -> std::path::PathBuf {
        let wasm_bytes = parse_str(wat_src).expect("wat válido");
        let wasm_path = temp.path().join(wasm_name);
        fs::write(&wasm_path, wasm_bytes).expect("escrever wasm");
        wasm_path
    }

    fn write_manifest(
        temp: &TempDir,
        wasm_name: &str,
        permissions: &str,
        preopen_section: &str,
        env_section: &str,
    ) -> std::path::PathBuf {
        let manifest_path = temp.path().join("capsule.manifest.json");
        let manifest = format!(
            r#"{{
  "id": "com.caeles.test",
  "name": "Test Capsule",
  "version": "0.0.0",
  "entry": "{wasm_name}",
  "permissions": {permissions},
  "env": {env_section},
  "preopened_dirs": {preopen_section},
  "lifecycle": {{"kind": "on_demand"}}
}}"#
        );
        fs::write(&manifest_path, manifest).expect("escrever manifest");
        manifest_path
    }

    fn load_manifest(path: &std::path::Path) -> CapsuleManifest {
        CapsuleManifest::load(path).expect("manifest válido")
    }

    #[test]
    fn fails_without_memory_export() {
        let temp = TempDir::new().unwrap();
        let wasm_path = write_wasm(
            &temp,
            "no_memory.wasm",
            include_str!("../tests/fixtures/no_memory.wat"),
        );
        let manifest_path = write_manifest(
            &temp,
            wasm_path.file_name().unwrap().to_string_lossy().as_ref(),
            r#"{"notifications": false, "network": false, "inherit_stdio": false}"#,
            "[]",
            "{}",
        );

        let manifest = load_manifest(&manifest_path);
        let err = run_capsule(&manifest).expect_err("sem memory deve falhar");
        assert!(
            err.to_string().contains("memória"),
            "mensagem inesperada: {err}"
        );
    }

    #[test]
    fn blocks_network_imports_when_permission_disabled() {
        let temp = TempDir::new().unwrap();
        let wasm_path = write_wasm(
            &temp,
            "network.wasm",
            include_str!("../tests/fixtures/network_socket.wat"),
        );
        let manifest_path = write_manifest(
            &temp,
            wasm_path.file_name().unwrap().to_string_lossy().as_ref(),
            r#"{"notifications": false, "network": false, "inherit_stdio": false}"#,
            "[]",
            "{}",
        );

        let manifest = load_manifest(&manifest_path);
        let err = run_capsule(&manifest).expect_err("imports de rede devem ser bloqueados");
        assert!(
            err.to_string().contains("permissions.network"),
            "mensagem inesperada: {err}"
        );
    }

    #[test]
    fn denies_writes_on_readonly_preopen() {
        let temp = TempDir::new().unwrap();
        let data_dir = temp.path().join("data");
        fs::create_dir_all(&data_dir).unwrap();

        let wasm_path = write_wasm(
            &temp,
            "readonly.wasm",
            include_str!("../tests/fixtures/readonly_write.wat"),
        );
        let manifest_path = write_manifest(
            &temp,
            wasm_path.file_name().unwrap().to_string_lossy().as_ref(),
            r#"{"notifications": false, "network": false, "inherit_stdio": false}"#,
            r#"[{"host": "./data", "guest": "/data", "read_only": true}]"#,
            "{}",
        );

        let manifest = load_manifest(&manifest_path);
        let result = run_capsule(&manifest);

        assert!(
            result.is_ok(),
            "Módulo deve rodar sem permitir escrita: {result:?}"
        );
    }
}
