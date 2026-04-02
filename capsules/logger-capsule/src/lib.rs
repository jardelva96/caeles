use caeles_sdk::{http_get, log, notify, NetworkError};

#[no_mangle]
pub extern "C" fn caeles_main() {
    log("logger-capsule: start");
    log("logger-capsule: emitting sequential logs");

    match http_get("https://example.com") {
        Ok(()) => log("logger-capsule: network request succeeded"),
        Err(NetworkError::BlockedByPermission) => {
            log("logger-capsule: network request blocked by manifest permission")
        }
        Err(NetworkError::InvalidRequest) => {
            log("logger-capsule: network request rejected (invalid URL)")
        }
        Err(NetworkError::HostFailure) => {
            log("logger-capsule: network request failed at host runtime")
        }
    }

    notify("logger-capsule: execution completed");
}
