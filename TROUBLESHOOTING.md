# Guia de Solução de Problemas

Este guia ajuda a resolver problemas comuns ao usar o CAELES.

## Problemas Comuns

### 1. "Arquivo WASM não encontrado"

**Erro:**
```
Error: Arquivo WASM não encontrado: target/wasm32-unknown-unknown/debug/minha_capsule.wasm
Verifique se a cápsula foi compilada corretamente.
```

**Solução:**

1. Certifique-se de que o target está instalado:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. Compile a cápsula para WASM:
   ```bash
   cargo build --target wasm32-unknown-unknown --package minha-capsule
   ```

3. Verifique se o caminho no `manifest.json` está correto (relativo ao diretório do manifest).

### 2. "Função 'caeles_main' não encontrada"

**Erro:**
```
Error: Função 'caeles_main' não encontrada. Certifique-se de que a cápsula exporta essa função com #[no_mangle]
```

**Solução:**

Sua cápsula deve ter uma função entry point com exatamente este formato:

```rust
#[no_mangle]
pub extern "C" fn caeles_main() {
    // Seu código aqui
}
```

**Pontos importantes:**
- Use `#[no_mangle]` para evitar name mangling
- Função deve ser `pub extern "C"`
- Nome deve ser exatamente `caeles_main`

### 3. "Permissão bloqueada" para notificações

**Mensagem:**
```
[capsule-notify BLOQUEADA] Permissão 'notifications' = false. Mensagem seria: ...
```

**Solução:**

Defina `notifications: true` no `manifest.json`:

```json
{
  "permissions": {
    "notifications": true,
    "network": false
  }
}
```

### 4. Cápsula não importa do caeles-sdk

**Erro:**
```
error[E0432]: unresolved import `caeles_sdk`
```

**Solução:**

Adicione ao `Cargo.toml` da cápsula:

```toml
[dependencies]
caeles-sdk = { path = "../../crates/caeles-sdk" }
```

E configure como biblioteca dinâmica:

```toml
[lib]
crate-type = ["cdylib"]
```

### 5. "Cápsula não exporta memória"

**Erro:**
```
[caeles-runtime] cápsula não exporta memória "memory"
```

**Causa:** Isso pode acontecer se a cápsula for compilada para o target errado.

**Solução:**

Use sempre:
```bash
cargo build --target wasm32-unknown-unknown
```

**NÃO use:** `wasm32-wasi` (é um target diferente)

### 6. Registry JSON inválido

**Erro:**
```
Error: Registry JSON inválido
```

**Solução:**

Verifique o formato do `registry.json`:

```json
[
  {
    "id": "com.exemplo.capsule",
    "name": "Minha Cápsula",
    "manifest": "path/to/manifest.json"
  }
]
```

**Checklist:**
- ✅ É um array `[]`
- ✅ Cada entrada tem `id`, `name` e `manifest`
- ✅ Caminho do manifest está correto
- ✅ JSON válido (sem vírgulas extras, etc.)

### 7. Fuel esgotado (loop infinito)

Se você criar um loop infinito na cápsula, o runtime vai parar a execução quando o fuel acabar.

**Comportamento esperado:**
```
Error: all fuel consumed by WebAssembly
```

**Solução:**
- Remova loops infinitos
- Use lógica que termina

### 8. Cápsula consome muita memória

**Erro:**
```
[caeles-runtime] string muito grande: 2097152 bytes (limite: 1048576 bytes)
```

**Causa:** Tentativa de passar string maior que 1MB.

**Solução:**
- Divida dados em chunks menores
- Use múltiplas chamadas

## Comandos Úteis para Debug

### Verificar se WASM foi gerado
```bash
ls -lh target/wasm32-unknown-unknown/debug/*.wasm
```

### Inspecionar cápsula WASM
```bash
wasm-objdump -x target/wasm32-unknown-unknown/debug/minha_capsule.wasm | grep export
```

### Validar JSON
```bash
cat manifest.json | jq .
```

### Listar cápsulas disponíveis
```bash
./target/debug/caeles-runtime --list
```

### Executar com mais informação
```bash
RUST_BACKTRACE=1 ./target/debug/caeles-runtime --capsule-id com.exemplo.id
```

## Perguntas Frequentes

### Como sei se minha cápsula compilou corretamente?

Execute:
```bash
cargo build --target wasm32-unknown-unknown --package minha-capsule
```

Se não houver erros e você ver:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs
```

Então está ok.

### Posso usar bibliotecas externas?

Sim, mas apenas bibliotecas que funcionam em `no_std` ou que têm suporte a `wasm32-unknown-unknown`.

**Bibliotecas comuns que funcionam:**
- `serde` (com `no_std`)
- `serde_json` (com `no_std`)
- Crates de matemática
- Crates de lógica pura

**NÃO funcionam:**
- Crates que usam I/O direto
- Crates que usam threads nativos
- Crates específicas de sistema operacional

### Como depurar minha cápsula?

Use a função `log` do SDK:

```rust
use caeles_sdk::log;

log("Iniciando processamento");
log(&format!("Valor: {}", x));
log("Processamento concluído");
```

### Fuel é o que exatamente?

"Fuel" é uma medida de instruções executadas. O runtime define um limite de 100 milhões de instruções para evitar loops infinitos. Você verá o fuel consumido no final:

```
> Fuel consumido: 87 instruções
```

## Ainda com Problemas?

1. Verifique os exemplos em `capsules/`
2. Execute os testes: `make test`
3. Abra uma issue no repositório com:
   - Descrição do problema
   - Mensagem de erro completa
   - Código relevante
   - Passos para reproduzir

## Recursos Adicionais

- [README.md](README.md) - Visão geral do projeto
- [CONTRIBUTING.md](CONTRIBUTING.md) - Como contribuir
- Exemplos: `capsules/hello-capsule` e `capsules/logger-capsule`
