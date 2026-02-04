# Logger Capsule

Exemplo de cápsula CAELES com múltiplos logs.

## Descrição

Esta cápsula demonstra:

- Uso sequencial de múltiplos `log()`
- Simulação de etapas de processamento
- Notificação de conclusão com `notify()`

## Código

```rust
use caeles_sdk::{log, notify};

#[no_mangle]
pub extern "C" fn caeles_main() {
    log("logger-capsule: início da execução.");
    log("logger-capsule: fazendo alguma lógica interna...");
    notify("logger-capsule: execução concluída com sucesso ✅");
}
```

## Compilação

```bash
cargo build --target wasm32-unknown-unknown
```

## Execução

```bash
# Da raiz do projeto
./target/debug/caeles-runtime --capsule-id com.caeles.example.logger
```

Ou diretamente pelo manifesto:

```bash
./target/debug/caeles-runtime --manifest capsules/logger-capsule/manifest.json
```

## Output esperado

```
> Carregando cápsula: capsules/logger-capsule/../../target/wasm32-unknown-unknown/debug/logger_capsule.wasm
> Chamando caeles_main da cápsula...
[capsule-log] logger-capsule: início da execução.
[capsule-log] logger-capsule: fazendo alguma lógica interna...
[capsule-notify] logger-capsule: execução concluída com sucesso ✅
> caeles_main terminou.
```

## Manifesto

```json
{
  "id": "com.caeles.example.logger",
  "name": "Logger Capsule",
  "version": "0.1.0",
  "entry": "../../target/wasm32-unknown-unknown/debug/logger_capsule.wasm",
  "permissions": {
    "notifications": true,
    "network": false
  },
  "lifecycle": {
    "kind": "on_demand"
  }
}
```

## Possíveis extensões

Esta cápsula poderia ser estendida para:

- Registrar informações de debug durante processamento
- Medir tempo de execução de diferentes etapas
- Reportar progresso para o host
- Demonstrar tratamento de erros
