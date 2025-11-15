/// Módulo de FFI com o host CAELES.
///
/// Este módulo declara as funções que a cápsula importa do runtime.
/// O atributo `wasm_import_module = "caeles"` diz ao compilador que
/// estas funções vêm do módulo de import "caeles" no WASM.
#[link(wasm_import_module = "caeles")]
extern "C" {
    fn host_log(ptr: *const u8, len: u32);
    fn host_notify(ptr: *const u8, len: u32);
    fn host_http_get(ptr: *const u8, len: u32);
    fn host_metric_inc(ptr: *const u8, len: u32, delta: i64);
    fn host_store_event(key_ptr: *const u8, key_len: u32, payload_ptr: *const u8, payload_len: u32);
}

/// Envia uma string de log para o host CAELES.
///
/// No runtime, isso é implementado por uma função Rust registrada em wasmtime.
pub fn log(msg: &str) {
    unsafe {
        host_log(msg.as_ptr(), msg.len() as u32);
    }
}

/// Envia uma notificação para o host CAELES.
///
/// O runtime pode aplicar regras de permissão com base no manifest.json.
pub fn notify(msg: &str) {
    unsafe {
        host_notify(msg.as_ptr(), msg.len() as u32);
    }
}

/// Pede para o host fazer um HTTP GET na URL informada.
///
/// O resultado (status + trecho do body) é logado no host (stdout).
/// Se a permissão `network` estiver false no manifest, o host bloqueia.
pub fn http_get(url: &str) {
    unsafe {
        host_http_get(url.as_ptr(), url.len() as u32);
    }
}

/// Incrementa uma métrica no host.
///
/// Se a permissão `metrics` estiver false, o host apenas registra que foi bloqueado.
pub fn metric_inc(name: &str, delta: i64) {
    unsafe {
        host_metric_inc(name.as_ptr(), name.len() as u32, delta);
    }
}

/// Grava um evento de auditoria / negócio no host.
///
/// Se a permissão `storage` estiver false, o host apenas registra que foi bloqueado.
pub fn store_event(key: &str, payload: &str) {
    unsafe {
        host_store_event(
            key.as_ptr(),
            key.len() as u32,
            payload.as_ptr(),
            payload.len() as u32,
        );
    }
}
