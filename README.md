# CAELES

**CAELES** é um motor de **cápsulas WebAssembly** focado em **Android**.  

<p align="center">
  <img src="./caeleslogo.png" alt="Logo CAELES" width="320" />
</p>

## 🔍 O que é o CAELES?

O **CAELES** é uma plataforma para executar **cápsulas** – pequenos módulos compilados para **WebAssembly** – de forma:

- 🔒 isolada (sandbox WebAssembly)  
- 📱 pensada primeiro para **Android**  
- ⚡ leve e portátil (o mesmo `.wasm` pode rodar em vários hosts)
- 🛡️ segura (com limites de recursos e permissões)

Você escreve a lógica da cápsula (por exemplo em Rust), gera um `.wasm`, descreve tudo em um **manifesto CAELES**, e o **núcleo CAELES** cuida de carregar e executar.

## ⚡ Quick Start

```bash
# Clone e execute
git clone https://github.com/jardelva96/caeles.git
cd caeles
./quickstart.sh

# Ou use o Makefile
make help           # Ver todos os comandos
make build-all      # Compilar tudo
make test           # Executar testes
make run-hello      # Executar exemplo
```

---

## 🧩 Conceitos principais

### Cápsula

Uma **cápsula CAELES** é a unidade básica do sistema.  
Ela é composta por:

- `capsule.wasm` – binário WebAssembly (`wasm32-unknown-unknown`)  
- `capsule.manifest.json` – arquivo declarando como e com quais permissões ela roda

Exemplo **simplificado** de manifesto (formato ainda em evolução):

```json
{
  "id": "com.caeles.example.demo",
  "name": "Cápsula Demo",
  "version": "0.1.0",

  "entry": "capsule.wasm",

  "permissions": {
    "notifications": false,
    "network": false
  },

  "lifecycle": {
    "kind": "on_demand"
  }
}
```

### Núcleo CAELES (runtime)

O núcleo CAELES é o "motor" que:

- lê e valida o manifesto
- localiza e carrega o `.wasm`
- prepara o ambiente de execução WebAssembly
- aplica permissões conforme o manifesto
- faz a ponte com o sistema host (Android, desktop, etc.)

A implementação é em Rust, usando WebAssembly como base.

---

## 🏗️ Arquitetura (alto nível)

```
[ Android / Desktop / Outro host ]
               │
               ▼
        [ Núcleo CAELES ]
           (Rust + WASM)
               │
      carrega e executa
               │
               ▼
        [ Cápsula WASM ]
     (wasm32-unknown-unknown)
```

No Android, o CAELES deve ser embutido em um app host, que chama o núcleo nativo.

Em desktop, o núcleo pode ser usado para desenvolvimento, debug e testes de cápsulas.

---

## 🚦 Estado atual

✅ **Funcional e testado**

O projeto já possui:

- ✅ **Runtime funcional**: `caeles-runtime` carrega e executa cápsulas WASM
- ✅ **SDK básico**: `caeles-sdk` com funções para log e notificações
- ✅ **Sistema de manifesto**: Formato JSON validado e carregado
- ✅ **Sistema de permissões**: Controle de acesso (ex: notifications)
- ✅ **Registry de cápsulas**: Gerenciamento centralizado de cápsulas disponíveis
- ✅ **Exemplos funcionais**: `hello-capsule` e `logger-capsule`
- ✅ **Limites de recursos**: Fuel para prevenir loops infinitos (100M instruções)
- ✅ **Validação de segurança**: Limites de memória, validação de ponteiros
- ✅ **Testes automatizados**: Suite de testes para manifest e runtime
- ✅ **Build automation**: Makefile com comandos comuns
- ✅ **CLI completa**: Listagem de cápsulas, help, versão

**Em desenvolvimento:**

- 🚧 Integração com Android
- 🚧 Mais host functions (filesystem, network, etc.)
- 🚧 Sistema de dependências entre cápsulas

A API, o formato de manifesto e a estrutura do código ainda podem mudar.

---

## 📖 API do SDK

O `caeles-sdk` fornece funções para interagir com o host:

### `log(msg: &str)`

Envia uma mensagem de log para o host.

```rust
use caeles_sdk::log;

log("Iniciando processamento...");
```

### `notify(msg: &str)`

Envia uma notificação para o host. Requer permissão `notifications: true` no manifesto.

```rust
use caeles_sdk::notify;

notify("Processamento concluído!");
```

Se a permissão não estiver ativada, a notificação será bloqueada pelo runtime.

---

## 🚀 Como usar

### Pré-requisitos

- Rust 1.70+ (instale via [rustup](https://rustup.rs/))
- Target WASM: `rustup target add wasm32-unknown-unknown`

### 1. Compilar o runtime

```bash
cargo build --release
```

Isso gera o executável `target/release/caeles-runtime`.

### 2. Criar uma cápsula

Uma cápsula é um projeto Rust que compila para WebAssembly.

**Exemplo: `Cargo.toml`**

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

**Exemplo: `src/lib.rs`**

```rust
use caeles_sdk::{log, notify};

#[no_mangle]
pub extern "C" fn caeles_main() {
    log("Olá do CAELES! 👋");
    notify("Algo importante aconteceu! 🔔");
}
```

### 3. Compilar a cápsula para WASM

```bash
cargo build --target wasm32-unknown-unknown --release
```

Isso gera: `target/wasm32-unknown-unknown/release/minha_capsule.wasm`

### 4. Criar o manifesto

Crie um arquivo `manifest.json`:

```json
{
  "id": "com.exemplo.minha-capsule",
  "name": "Minha Cápsula",
  "version": "0.1.0",
  "entry": "../../target/wasm32-unknown-unknown/release/minha_capsule.wasm",
  "permissions": {
    "notifications": true,
    "network": false
  },
  "lifecycle": {
    "kind": "on_demand"
  }
}
```

### 5. Executar a cápsula

**Listar cápsulas disponíveis:**

```bash
./target/release/caeles-runtime --list
```

**Opção 1: Diretamente pelo manifesto**

```bash
./target/release/caeles-runtime --manifest path/to/manifest.json
```

**Opção 2: Pelo ID via registry**

Adicione sua cápsula ao `capsules/registry.json`:

```json
[
  {
    "id": "com.exemplo.minha-capsule",
    "name": "Minha Cápsula",
    "manifest": "path/to/manifest.json"
  }
]
```

Execute:

```bash
./target/release/caeles-runtime --capsule-id com.exemplo.minha-capsule
```

**Ver versão e ajuda:**

```bash
./target/release/caeles-runtime --version
./target/release/caeles-runtime --help
```

### Exemplos incluídos

O repositório já inclui duas cápsulas de exemplo:

```bash
# Usando Makefile (recomendado)
make run-hello
make run-logger

# Ou manualmente
cargo build --target wasm32-unknown-unknown --workspace

# Executar hello-capsule
./target/debug/caeles-runtime --capsule-id com.caeles.example.hello

# Executar logger-capsule
./target/debug/caeles-runtime --capsule-id com.caeles.example.logger
```

---

## 🤝 Contribuição

Contribuições são bem-vindas! Áreas de interesse:

- 🎯 Novas host functions (filesystem, network, etc.)
- 📱 Integração com Android via JNI
- 🔒 Melhorias no sistema de permissões
- 📦 Sistema de empacotamento de cápsulas
- 🧪 Testes e exemplos
- 📚 Documentação

Para contribuir:

1. Faça um fork do repositório
2. Crie uma branch para sua feature (`git checkout -b feature/nova-funcionalidade`)
3. Commit suas mudanças (`git commit -am 'Adiciona nova funcionalidade'`)
4. Push para a branch (`git push origin feature/nova-funcionalidade`)
5. Abra um Pull Request

## 📚 Estrutura do projeto

```
caeles/
├── crates/
│   ├── caeles-runtime/    # Runtime que executa cápsulas
│   └── caeles-sdk/        # SDK para criar cápsulas
├── capsules/
│   ├── hello-capsule/     # Exemplo básico
│   ├── logger-capsule/    # Exemplo com múltiplos logs
│   └── registry.json      # Registry de cápsulas disponíveis
├── Makefile               # Comandos de build e execução
├── quickstart.sh          # Script de início rápido
└── README.md
```

## 🔧 Solução de Problemas

Encontrou algum problema? Consulte o [Guia de Solução de Problemas](TROUBLESHOOTING.md) que cobre:

- Erros comuns e soluções
- Problemas de compilação
- Questões de permissão
- Debug de cápsulas
- FAQs

## 📚 Documentação Adicional

- [CONTRIBUTING.md](CONTRIBUTING.md) - Como contribuir para o projeto
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Guia de solução de problemas
- [crates/caeles-runtime/README.md](crates/caeles-runtime/README.md) - Documentação do runtime
- [crates/caeles-sdk/README.md](crates/caeles-sdk/README.md) - Documentação do SDK

## 📄 Licença

Este projeto está em desenvolvimento. Licença a ser definida.
