# Android Host PoC (Started)

This directory now contains an initial Android host scaffold:

- `native/` Rust JNI bridge PoC (`cdylib`)
- `app/` Kotlin bridge declarations

## Current PoC Deliverables

- JNI health endpoint: `nativeHealth()`
- JNI list placeholder: `nativeList(registryPath)`
- JNI run placeholder: `nativeRun(manifestPath)`

Rust JNI exports are implemented in:

- `android/poc-host/native/src/lib.rs`

Kotlin bridge declarations are in:

- `android/poc-host/app/src/main/java/com/caeles/host/CaelesBridge.kt`

## Next Step (Bridge Runtime APIs)

Current runtime is CLI-first (`main.rs`).
To make Android integration real, extract reusable runtime service APIs into a shared library module and call those APIs from both:

- CLI subcommands
- JNI bridge functions

## Suggested follow-up tasks

1. Move command logic into `crates/caeles-runtime/src/lib.rs` service functions.
2. Keep `src/main.rs` as thin CLI adapter.
3. Update JNI PoC to call service functions for `list` and `run`.
4. Return structured JSON from JNI bridge for Android UI consumption.
