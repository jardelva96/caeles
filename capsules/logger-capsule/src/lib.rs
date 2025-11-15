use caeles_sdk::{log, notify};

/// Entry point da cápsula logger.
/// Mostra alguns logs em sequência e uma notificação.
#[no_mangle]
pub extern "C" fn caeles_main() {
    log("logger-capsule: início da execução.");
    log("logger-capsule: fazendo alguma lógica interna...");
    notify("logger-capsule: execução concluída com sucesso ✅");
}
