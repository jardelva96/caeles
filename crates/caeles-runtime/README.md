# CAELES Runtime

Runtime WebAssembly para executar cápsulas CAELES.

## Descrição

O `caeles-runtime` é o motor de execução do ecossistema CAELES. Ele carrega, valida e executa cápsulas WebAssembly com controle de permissões e isolamento.

## Funcionalidades

- ✅ Carregamento de módulos WebAssembly (wasm32-unknown-unknown)
- ✅ Validação de manifesto JSON
- ✅ Sistema de permissões (notifications, network)
- ✅ Registry de cápsulas para fácil gerenciamento
- ✅ Host functions: `host_log` e `host_notify`
- ✅ Sandbox de execução via Wasmtime

## Uso

### Executar via manifesto direto

```bash
caeles-runtime --manifest path/to/manifest.json
```

### Executar via ID no registry

```bash
caeles-runtime --capsule-id com.caeles.example.hello --registry capsules/registry.json
```

O registry padrão é `capsules/registry.json`.

## Formato do Manifesto

```json
{
  "id": "com.exemplo.capsule",
  "name": "Minha Cápsula",
  "version": "0.1.0",
  "entry": "path/to/capsule.wasm",
  "permissions": {
    "notifications": true,
    "network": false
  },
  "lifecycle": {
    "kind": "on_demand"
  }
}
```

### Campos

- `id`: Identificador único da cápsula (convenção: reverse domain notation)
- `name`: Nome legível da cápsula
- `version`: Versão semântica
- `entry`: Caminho relativo para o arquivo `.wasm` (relativo ao diretório do manifesto)
- `permissions`: Objeto com permissões booleanas
  - `notifications`: Permite o uso da função `host_notify`
  - `network`: (Reservado para uso futuro)
- `lifecycle`: Configuração do ciclo de vida
  - `kind`: Tipo de execução (atualmente apenas `"on_demand"`)

## Formato do Registry

```json
[
  {
    "id": "com.caeles.example.hello",
    "name": "Hello Capsule",
    "manifest": "capsules/hello-capsule/manifest.json"
  }
]
```

## Host Functions

O runtime fornece as seguintes funções para as cápsulas:

### `caeles.host_log(ptr: i32, len: i32)`

Imprime uma mensagem de log no console do host.

### `caeles.host_notify(ptr: i32, len: i32)`

Envia uma notificação. Requer `permissions.notifications = true`.

## Compilação

```bash
cargo build --release
```

## Dependências principais

- [wasmtime](https://wasmtime.dev/) - Runtime WebAssembly
- [clap](https://docs.rs/clap/) - Parser de argumentos CLI
- [serde](https://serde.rs/) - Serialização/deserialização JSON
