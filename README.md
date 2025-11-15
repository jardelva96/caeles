
# CAELES (Dev) – Como Rodar

Guia rápido para rodar o runtime e as cápsulas na branch **Dev**.

---

## 1. Pré-requisitos
- Rust + Cargo instalados  
- Adicionar target WebAssembly:

```bash
rustup target add wasm32-unknown-unknown
```

Estar na branch Dev:
```bash
git checkout Dev
```

Build do Runtime
```bash
cargo build -p caeles-runtime
```

Build das Cápsulas (WASM)
```bash
cargo build -p hello-capsule   --target wasm32-unknown-unknown
cargo build -p logger-capsule  --target wasm32-unknown-unknown
cargo build -p network-capsule --target wasm32-unknown-unknown
cargo build -p metrics-capsule --target wasm32-unknown-unknown
```

Executando cápsulas via ID
```bash
cargo run -p caeles-runtime -- --capsule-id com.caeles.example.hello
cargo run -p caeles-runtime -- --capsule-id com.caeles.example.logger
cargo run -p caeles-runtime -- --capsule-id com.caeles.example.network
cargo run -p caeles-runtime -- --capsule-id com.caeles.example.metrics
```

Executando via manifest
```bash
cargo run -p caeles-runtime -- --manifest capsules/hello-capsule/manifest.json
```

Listar cápsulas
```bash
cargo run -p caeles-runtime -- --list-capsules
```
