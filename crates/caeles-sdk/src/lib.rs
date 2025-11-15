/// Módulo de FFI com o host CAELES.
///
/// Este módulo declara as funções que a cápsula importa do runtime.
/// O atributo `wasm_import_module = "caeles"` diz ao compilador que
/// estas funções vêm do módulo de import "caeles" no WASM.
#[link(wasm_import_module = "caeles")]
extern "C" {
    fn host_log(ptr: *const u8, len: u32);
    fn host_notify(ptr: *const u8, len: u32);
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
