(module
  (import "wasi_snapshot_preview1" "path_open"
    (func $path_open (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "attempt.txt\00")
  (data (i32.const 32) "\00\00\00\00")

  (func (export "caeles_main")
    (local $errno i32)
    (local.set $errno
      (call $path_open
        (i32.const 3)   ;; fd do primeiro preopen
        (i32.const 0)   ;; dirflags
        (i32.const 0)   ;; ptr do path
        (i32.const 11)  ;; len do path
        (i32.const 0)   ;; oflags
        (i64.const 2)   ;; rights base (write)
        (i64.const 2)   ;; rights inheriting (write)
        (i32.const 0)   ;; fs_flags
        (i32.const 32)  ;; onde colocar fd resultante
      )
    )
    ;; Se errno == 0, então conseguiu abrir para escrita => trap
    (if (i32.eqz (local.get $errno))
      (then unreachable)
      (else nop)))
)
