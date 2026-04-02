use caeles_sdk::{log, notify};

#[no_mangle]
pub extern "C" fn caeles_main() {
    log("hello-capsule: hello from CAELES capsule via host_log");
    notify("hello-capsule: runtime notification from capsule");
}
