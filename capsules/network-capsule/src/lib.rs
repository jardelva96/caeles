use caeles_sdk::{log, notify, http_get};

/// Entry point da cápsula de rede.
///
/// Ela pede para o host fazer um HTTP GET e registra logs/notificações.
#[no_mangle]
pub extern "C" fn caeles_main() {
    log("network-capsule: iniciando requisição HTTP para https://example.com ...");
    http_get("https://example.com");
    notify("network-capsule: requisição HTTP concluída (veja logs do host).");
}
