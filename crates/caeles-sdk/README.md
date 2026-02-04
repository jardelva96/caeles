# CAELES SDK

SDK para criar cápsulas CAELES em Rust.

## Descrição

O `caeles-sdk` fornece a interface para que cápsulas WebAssembly interajam com o runtime CAELES. Ele exporta funções que se conectam aos host functions fornecidos pelo runtime.

## Instalação

Adicione ao seu `Cargo.toml`:

```toml
[dependencies]
caeles-sdk = { path = "../../crates/caeles-sdk" }
```

Para publicar cápsulas, o SDK será disponibilizado no crates.io no futuro.

## Uso

### Estrutura básica de uma cápsula

```rust
use caeles_sdk::{log, notify};

/// Entry point da cápsula CAELES
#[no_mangle]
pub extern "C" fn caeles_main() {
    log("Cápsula iniciada!");
    notify("Algo importante aconteceu! 🔔");
}
```

### Configuração do Cargo.toml

Sua cápsula deve ser compilada como uma biblioteca dinâmica:

```toml
[package]
name = "minha-capsule"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
caeles-sdk = { path = "../../crates/caeles-sdk" }
```

### Compilação

```bash
cargo build --target wasm32-unknown-unknown --release
```

Isso gera o arquivo `.wasm` em:
```
target/wasm32-unknown-unknown/release/minha_capsule.wasm
```

## API

### `log(msg: &str)`

Envia uma mensagem de log para o host.

```rust
use caeles_sdk::log;

log("Iniciando processamento...");
log("Dados carregados com sucesso!");
```

**Output no host:**
```
[capsule-log] Iniciando processamento...
[capsule-log] Dados carregados com sucesso!
```

### `notify(msg: &str)`

Envia uma notificação para o host. Requer a permissão `notifications: true` no manifesto da cápsula.

```rust
use caeles_sdk::notify;

notify("Tarefa concluída!");
```

**Output no host (se permissão ativada):**
```
[capsule-notify] Tarefa concluída!
```

**Output no host (se permissão desativada):**
```
[capsule-notify BLOQUEADA] Permissão 'notifications' = false. Mensagem seria: Tarefa concluída!
```

## Detalhes de implementação

O SDK utiliza FFI (Foreign Function Interface) para chamar funções do host:

```rust
#[link(wasm_import_module = "caeles")]
extern "C" {
    fn host_log(ptr: *const u8, len: u32);
    fn host_notify(ptr: *const u8, len: u32);
}
```

As funções públicas `log()` e `notify()` são wrappers seguros em Rust que gerenciam os ponteiros de memória automaticamente.

## Entry Point

Toda cápsula CAELES deve exportar uma função `caeles_main`:

```rust
#[no_mangle]
pub extern "C" fn caeles_main() {
    // Sua lógica aqui
}
```

O atributo `#[no_mangle]` garante que o nome da função não seja alterado pelo compilador, permitindo que o runtime a encontre.

## Exemplos

Veja as cápsulas de exemplo no repositório:

- `capsules/hello-capsule` - Exemplo básico
- `capsules/logger-capsule` - Exemplo com múltiplos logs

## Compilação

O SDK não precisa ser compilado separadamente. Ele é incluído como dependência nas cápsulas.

Para verificar se compila:

```bash
cargo check
```

## Futuro

Funções planejadas para versões futuras:

- `fs_read()`, `fs_write()` - Acesso a filesystem (sandboxed)
- `http_get()`, `http_post()` - Requisições HTTP
- `storage_get()`, `storage_set()` - Armazenamento persistente
- `time_now()` - Obter timestamp atual
- `random()` - Geração de números aleatórios
