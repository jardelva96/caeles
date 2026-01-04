(module
  (import "wasi_snapshot_preview1" "sock_accept" (func $sock_accept (param i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (func (export "caeles_main")
    (drop (call $sock_accept (i32.const 0) (i32.const 0) (i32.const 0)))))
