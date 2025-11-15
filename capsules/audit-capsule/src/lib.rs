use caeles_sdk::{log, notify, http_get, metric_inc, store_event};

/// Entry point da cápsula de auditoria.
///
/// Ela faz:
///  - log inicial
///  - GET em https://example.com
///  - incrementa algumas métricas
///  - grava eventos de auditoria no host
///  - envia uma notificação no final
#[no_mangle]
pub extern "C" fn caeles_main() {
    log("audit-capsule: início da execução.");

    // Chama uma URL só para gerar algum tráfego
    http_get("https://example.com");

    // Simula alguns eventos de negócio
    for i in 0..3 {
        let key = format!("order_{}", i);
        let payload = format!(r#"{{"order_id": {i}, "status": "created"}}"#);

        store_event(&key, &payload);
        metric_inc("orders_created_total", 1);
        log(&format!("audit-capsule: evento gravado para {key}"));
    }

    notify("audit-capsule: execução concluída; veja eventos em data/events-com.caeles.example.audit.log");
}
