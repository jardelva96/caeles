use caeles_sdk::{log, notify, metric_inc};

/// Entry point da cápsula de métricas.
///
/// Ela simula o processamento de alguns "jobs" e registra métricas
/// no host (jobs_total, jobs_ok, jobs_error).
#[no_mangle]
pub extern "C" fn caeles_main() {
    log("metrics-capsule: começando processamento de jobs...");

    for i in 0..5 {
        metric_inc("jobs_total", 1);

        if i % 2 == 0 {
            metric_inc("jobs_ok", 1);
            log(&format!("metrics-capsule: job {i} OK"));
        } else {
            metric_inc("jobs_error", 1);
            log(&format!("metrics-capsule: job {i} ERRO simulado"));
        }
    }

    notify("metrics-capsule: terminou; veja o resumo de métricas impresso pelo host.");
}
