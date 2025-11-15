use caeles_sdk::{log, notify};

/// Entry point da cápsula CAELES.
///
/// É essa função que o runtime chama via wasmtime.
#[no_mangle]
pub extern "C" fn caeles_main() {
    log("Hello from CAELES capsule via ABI + host_log! 🚀");
    notify("Notificação da cápsula: algo aconteceu aqui dentro! 🔔");
}
