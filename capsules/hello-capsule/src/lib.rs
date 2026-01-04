use caeles_sdk::{log, notify};
use std::fs;

/// Entry point da cápsula CAELES.
///
/// É essa função que o runtime chama via wasmtime.
#[no_mangle]
pub extern "C" fn caeles_main() {
    let greeting = std::env::var("GREETING").unwrap_or_else(|_| "Olá do CAELES!".to_string());
    log(&format!("Greeting recebido do manifest/env: {greeting}"));

    match fs::read_to_string("/data/message.txt") {
        Ok(msg) => log(&format!("Conteúdo de /data/message.txt: {msg}")),
        Err(err) => log(&format!("Não foi possível ler /data/message.txt: {err}")),
    }

    notify("Notificação da cápsula: rotina principal concluída.");
}
