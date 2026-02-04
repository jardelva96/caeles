# Hello Capsule

Exemplo básico de uma cápsula CAELES.

## Descrição

Esta é a cápsula de exemplo mais simples do CAELES. Demonstra:

- Como criar uma cápsula básica
- Uso da função `log()`
- Uso da função `notify()`
- Configuração de permissões no manifesto

## Código

```rust
use caeles_sdk::{log, notify};

#[no_mangle]
pub extern "C" fn caeles_main() {
    log("Hello from CAELES capsule via ABI + host_log! 🚀");
    notify("Notificação da cápsula: algo aconteceu aqui dentro! 🔔");
}
```

## Compilação

```bash
cargo build --target wasm32-unknown-unknown
```

## Execução

```bash
# Da raiz do projeto
./target/debug/caeles-runtime --capsule-id com.caeles.example.hello
```

Ou diretamente pelo manifesto:

```bash
./target/debug/caeles-runtime --manifest capsules/hello-capsule/manifest.json
```

## Output esperado

```
> Carregando cápsula: capsules/hello-capsule/../../target/wasm32-unknown-unknown/debug/hello_capsule.wasm
> Chamando caeles_main da cápsula...
[capsule-log] Hello from CAELES capsule via ABI + host_log! 🚀
[capsule-notify] Notificação da cápsula: algo aconteceu aqui dentro! 🔔
> caeles_main terminou.
```

## Manifesto

A cápsula possui `notifications: true`, permitindo o uso de `notify()`:

```json
{
  "id": "com.caeles.example.hello",
  "name": "Hello Capsule",
  "version": "0.1.0",
  "entry": "../../target/wasm32-unknown-unknown/debug/hello_capsule.wasm",
  "permissions": {
    "notifications": true,
    "network": false
  },
  "lifecycle": {
    "kind": "on_demand"
  }
}
```
