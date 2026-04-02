#[link(wasm_import_module = "caeles")]
extern "C" {
    fn host_log(ptr: *const u8, len: u32);
    fn host_notify(ptr: *const u8, len: u32);
    fn host_http_get(ptr: *const u8, len: u32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkError {
    BlockedByPermission,
    InvalidRequest,
    HostFailure,
}

/// Send a log line to the CAELES host runtime.
pub fn log(msg: &str) {
    unsafe {
        host_log(msg.as_ptr(), msg.len() as u32);
    }
}

/// Send a notification to the CAELES host runtime.
pub fn notify(msg: &str) {
    unsafe {
        host_notify(msg.as_ptr(), msg.len() as u32);
    }
}

/// Perform a host-mediated HTTP GET request.
///
/// The runtime enforces the `permissions.network` flag from the manifest.
pub fn http_get(url: &str) -> Result<(), NetworkError> {
    let code = unsafe { host_http_get(url.as_ptr(), url.len() as u32) };
    match code {
        0 => Ok(()),
        1 => Err(NetworkError::BlockedByPermission),
        2 => Err(NetworkError::InvalidRequest),
        _ => Err(NetworkError::HostFailure),
    }
}
