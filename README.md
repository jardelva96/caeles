# CAELES 🚀

**Runtime CAELES** - Sistema de gerenciamento de cápsulas WebAssembly focado em Android

---

## 📋 Visão Geral

CAELES é um runtime profissional para cápsulas WebAssembly, com workflow completo de build, instalação, execução e lifecycle management. Pense em Docker, mas para WASM com foco em Android.

### O que é uma Cápsula?

Uma **cápsula CAELES** é composta por:

- **`capsule.wasm`** – Binário WebAssembly (`wasm32-unknown-unknown`)
- **`capsule.manifest.json`** – Configuração de execução e permissões

```json
{
  "id": "com.caeles.example.hello",
  "name": "Hello Capsule",
  "version": "0.1.0",
  "entry": "target/wasm32-unknown-unknown/debug/hello_capsule.wasm",
  "permissions": {
    "notifications": false,
    "network": false
  },
  "lifecycle": {
    "kind": "on_demand"
  }
}
```

---

## 🚀 Início Rápido

### 1. Pré-requisitos

```bash
# Rust + Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Target WebAssembly
rustup target add wasm32-unknown-unknown

# Clone e build
git checkout codex/corrigir-erros-ao-construir-caeles-runtime
cargo build -p caeles-runtime
```

### 2. Criar uma Cápsula

```bash
# Criar projeto
cargo new --lib my-capsule
cd my-capsule
```

**Configurar `Cargo.toml`:**

```toml
[package]
name = "my-capsule"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
caeles-sdk = { path = "../crates/caeles-sdk" }
```

**Implementar `src/lib.rs`:**

```rust
use caeles_sdk::{log, notify};

#[no_mangle]
pub extern "C" fn caeles_main() {
    log("🚀 Cápsula iniciada!");
    notify("Hello from CAELES!");
}
```

### 3. Workflow Completo

```bash
# 1. Build (compilar para WASM)
cargo run -p caeles-runtime -- build

# 2. Instalar no sistema
cargo run -p caeles-runtime -- install

# 3. Iniciar cápsula em background
cargo run -p caeles-runtime -- start com.caeles.my-capsule

# 4. Ver status
cargo run -p caeles-runtime -- status

# 5. Parar cápsula
cargo run -p caeles-runtime -- stop com.caeles.my-capsule

# 6. Remover quando não precisar mais
cargo run -p caeles-runtime -- remove com.caeles.my-capsule
```

---

## 📦 Comandos Disponíveis

### Build System

```bash
# Compilar cápsula
cargo run -p caeles-runtime -- build

# Build otimizado para produção
cargo run -p caeles-runtime -- build --release

# Build com output customizado
cargo run -p caeles-runtime -- build --output ./dist
```

**O que o build faz:**
- ✅ Compila para `wasm32-unknown-unknown`
- ✅ Valida exports do WASM (`caeles_main`, `memory`)
- ✅ Gera/atualiza manifest automaticamente
- ✅ Calcula checksums SHA-256
- ✅ Detecta imports do CAELES

### Storage Persistente

```bash
# Instalar cápsula
cargo run -p caeles-runtime -- install
cargo run -p caeles-runtime -- install --force  # Reinstalar

# Listar instaladas
cargo run -p caeles-runtime -- list
cargo run -p caeles-runtime -- list --verbose   # Detalhes completos
cargo run -p caeles-runtime -- list --format json

# Remover cápsula
cargo run -p caeles-runtime -- remove com.caeles.example.hello
cargo run -p caeles-runtime -- remove com.caeles.example.hello --yes
```

**Estrutura de storage:**
```
~/.caeles/
├── capsules/
│   └── com_caeles_example_hello/
│       ├── capsule.wasm
│       ├── manifest.json
│       └── metadata.json
├── state/           # Estado de instâncias
├── logs/            # Logs de execução (futuro)
└── data/            # Dados persistentes (futuro)
```

### Lifecycle Management

```bash
# Iniciar cápsula em background
cargo run -p caeles-runtime -- start com.caeles.example.hello

# Ver status de todas as instâncias
cargo run -p caeles-runtime -- status

# Ver apenas as rodando
cargo run -p caeles-runtime -- status --running

# Parar cápsula
cargo run -p caeles-runtime -- stop com.caeles.example.hello

# Output JSON
cargo run -p caeles-runtime -- status --format json
```

**Output do `status`:**
```
📊 Status de Cápsulas (2)

ID                                       STATUS      PID        UPTIME
────────────────────────────────────────────────────────────────────────────
com.caeles.example.hello                 running     12345      5m 23s
com.caeles.example.logger                stopped     -          -

💡 Comandos:
   caeles stop <id>      # Parar cápsula
   caeles status --running  # Apenas rodando
```

### Sistema de Logs

```bash
# Ver logs de uma cápsula
cargo run -p caeles-runtime -- logs com.caeles.example.hello

# Ver últimas N linhas
cargo run -p caeles-runtime -- logs com.caeles.example.hello -n 50

# Ver logs de erro (stderr)
cargo run -p caeles-runtime -- logs com.caeles.example.hello --errors

# Ver logs desde um timestamp
cargo run -p caeles-runtime -- logs com.caeles.example.hello --since 1704896400

# Seguir logs em tempo real (streaming)
cargo run -p caeles-runtime -- logs com.caeles.example.hello -f

# Limpar todos os logs de uma cápsula
cargo run -p caeles-runtime -- logs com.caeles.example.hello --clear
```

**Output do `logs`:**
```
📝 Logs de 'com.caeles.example.hello' (STDOUT)

[2025-01-09 10:30:15] 🚀 Cápsula iniciada
[2025-01-09 10:30:16] Processando requisição...
[2025-01-09 10:30:17] Operação concluída com sucesso

📊 Estatísticas:
   Arquivos:     3
   Tamanho:      0.15 MB
   Linhas atual: 142
```

**Estrutura de logs:**
```
~/.caeles/
└── logs/
    └── com_caeles_example_hello/
        ├── current.log        # Log stdout atual
        ├── error.log          # Log stderr atual
        ├── current.log.1704896400  # Log rotacionado
        └── error.log.1704896400    # Log erro rotacionado
```

### Ferramentas Auxiliares

```bash
# Wizard para criar manifest
cargo run -p caeles-runtime -- init

# Interface web para gerenciar cápsulas
cargo run -p caeles-runtime -- web
# Acesse http://127.0.0.1:8080

# Executar diretamente (modo compatibilidade)
cargo run -p caeles-runtime -- --manifest capsule.manifest.json
cargo run -p caeles-runtime -- --capsule-id com.caeles.example.hello
```

---

## 🏗️ Arquitetura

```
┌─────────────────────────────────────────────┐
│         Android / Desktop Host              │
└────────────────┬────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────┐
│         CAELES Runtime (Rust)               │
├─────────────────────────────────────────────┤
│  • Build System     • Storage Persistente   │
│  • Lifecycle Mgmt   • Log Management        │
│  • Instance Manager • Process Runner        │
└────────────────┬────────────────────────────┘
                 │ executa
                 ▼
┌─────────────────────────────────────────────┐
│       Cápsula WASM (wasm32-unknown)         │
│         com.caeles.example.hello            │
└─────────────────────────────────────────────┘
```

### Componentes Principais

**1. Build System** ([docs/BUILD_SYSTEM.md](docs/BUILD_SYSTEM.md))
- Detecção de projetos Rust
- Compilação para WASM
- Validação de exports/imports
- Geração de manifests
- Checksumming SHA-256

**2. Storage Persistente** ([docs/STORAGE_SYSTEM.md](docs/STORAGE_SYSTEM.md))
- Instalação em `~/.caeles/capsules/`
- Metadata de instalação
- Tracking de execuções
- Estatísticas de uso

**3. Lifecycle Management**
- Gerenciamento de instâncias
- Processos em background
- Tracking de PID e status
- Controle de uptime

**4. Sistema de Logs**
- Captura de stdout/stderr
- Logs persistentes em `~/.caeles/logs/`
- Rotação automática de logs
- Filtros por timestamp
- Streaming em tempo real
- Estatísticas de uso

**5. Permission System** (em desenvolvimento)
- Permissões declarativas no manifest
- Runtime enforcement
- Isolamento de recursos

---

## 📖 Exemplos Práticos

### Workflow Básico

```bash
# Desenvolver
cd my-project
cargo run -p caeles-runtime -- build

# Instalar localmente
cargo run -p caeles-runtime -- install

# Verificar
cargo run -p caeles-runtime -- list -v

# Iniciar
cargo run -p caeles-runtime -- start com.mycompany.myapp

# Monitorar
cargo run -p caeles-runtime -- status

# Parar e remover quando terminar
cargo run -p caeles-runtime -- stop com.mycompany.myapp
cargo run -p caeles-runtime -- remove com.mycompany.myapp
```

### CI/CD

```bash
#!/bin/bash
# Build script para produção

# Build otimizado
cargo run -p caeles-runtime -- build --release

# Remover versão antiga
cargo run -p caeles-runtime -- remove com.app.production --yes || true

# Instalar nova versão
cargo run -p caeles-runtime -- install --force

# Verificar instalação
cargo run -p caeles-runtime -- list --format json | jq '.[] | select(.id == "com.app.production")'

# Iniciar automaticamente
cargo run -p caeles-runtime -- start com.app.production
```

### Múltiplos Ambientes

```bash
# Dev
cd dev-capsule
cargo run -p caeles-runtime -- build
cargo run -p caeles-runtime -- install

# Staging
cd ../staging-capsule
cargo run -p caeles-runtime -- build
cargo run -p caeles-runtime -- install

# Production
cd ../prod-capsule
cargo run -p caeles-runtime -- build --release
cargo run -p caeles-runtime -- install

# Ver todos
cargo run -p caeles-runtime -- list
```

---

## 📊 Comparação com Docker

| Aspecto | Docker | CAELES |
|---------|--------|--------|
| **Armazenamento** | `/var/lib/docker` | `~/.caeles` |
| **Build** | `docker build` | `caeles build` |
| **Instalação** | `docker pull` | `caeles install` |
| **Listagem** | `docker images` | `caeles list` |
| **Execução** | `docker run` | `caeles start` |
| **Status** | `docker ps` | `caeles status` |
| **Parar** | `docker stop` | `caeles stop` |
| **Logs** | `docker logs` | `caeles logs` |
| **Remoção** | `docker rmi` | `caeles remove` |
| **Formato** | Layers (OCI) | WASM + Manifest |
| **Tamanho** | MB-GB | KB-MB |
| **Isolamento** | Containers | WASM Sandbox |
| **Target** | Servidores | Android/Mobile |

---

## 🎯 Roadmap

### ✅ Fase 1: Build System (COMPLETO)
- [x] Detecção de projetos Rust
- [x] Compilação para WASM
- [x] Validação de WASM
- [x] Geração de manifests
- [x] Checksumming SHA-256

### ✅ Fase 2: Storage Persistente (COMPLETO)
- [x] Sistema de instalação
- [x] Diretório `~/.caeles`
- [x] Metadata tracking
- [x] Comandos install/list/remove

### ✅ Fase 3: Lifecycle Management (COMPLETO)
- [x] Instance Manager
- [x] Background processes
- [x] Comandos start/stop/status
- [x] PID e uptime tracking

### ✅ Fase 4: Sistema de Logs (COMPLETO)
- [x] Logs persistentes em `~/.caeles/logs`
- [x] Comando `logs` com opções
- [x] Captura de stdout/stderr
- [x] Rotação automática de logs
- [x] Filtros por timestamp
- [x] Limpeza de logs antigos
- [x] Estatísticas de uso

### 🚧 Fase 5: Info e Inspect
- [ ] Comando `info` para detalhes
- [ ] Histórico de execuções
- [ ] Métricas de performance
- [ ] Dependências entre cápsulas

### 🚧 Fase 6: Permission Runtime
- [ ] Enforcement de permissões
- [ ] Validação em runtime
- [ ] Audit log de acessos
- [ ] Permissões granulares

### 🚧 Fase 7: Resource Limits
- [ ] Limite de memória
- [ ] Limite de CPU
- [ ] Timeout de execução
- [ ] Quotas de I/O

### 🚧 Fase 8: Package Format
- [ ] Formato `.capsule`
- [ ] Compressão de artefatos
- [ ] Assinatura digital
- [ ] Registry remoto

---

## 🛠️ Troubleshooting

### Erro: "wasm32-unknown-unknown não está instalado"

```bash
rustup target add wasm32-unknown-unknown
```

### Erro: "Módulo WASM não exporta 'caeles_main'"

Adicione ao `src/lib.rs`:

```rust
#[no_mangle]
pub extern "C" fn caeles_main() {
    // código da cápsula
}
```

### Erro: "Cápsula já está instalada"

```bash
# Forçar reinstalação
cargo run -p caeles-runtime -- install --force

# Ou remover primeiro
cargo run -p caeles-runtime -- remove {id}
cargo run -p caeles-runtime -- install
```

### Erro: "Arquivo WASM não encontrado"

```bash
# Executar build primeiro
cargo run -p caeles-runtime -- build
cargo run -p caeles-runtime -- install
```

---

## 📚 Documentação

- **[Build System](docs/BUILD_SYSTEM.md)** - Sistema de compilação e validação
- **[Storage System](docs/STORAGE_SYSTEM.md)** - Instalação e gerenciamento persistente
- **[Architecture](docs/ARCHITECTURE.md)** - Arquitetura técnica completa (futuro)
- **[API Reference](docs/API.md)** - Referência de APIs (futuro)

---

## 🤝 Contribuindo

CAELES está em desenvolvimento ativo. Contribuições são bem-vindas!

```bash
# Clone
git clone https://github.com/seu-usuario/caeles.git
cd caeles

# Checkout da branch de desenvolvimento
git checkout codex/corrigir-erros-ao-construir-caeles-runtime

# Build
cargo build

# Testes
cargo test
```

---

## 📜 Licença

[Definir licença]

---

## 🎓 Exemplos Incluídos

O repositório inclui cápsulas de exemplo:

- **hello-capsule** - Demonstração básica
- **logger-capsule** - Exemplo de logging
- **network-capsule** - Exemplo de permissões de rede
- **metrics-capsule** - Exemplo de métricas

Para compilá-las:

```bash
cd capsules/hello-capsule
cargo run -p caeles-runtime -- build
cargo run -p caeles-runtime -- install
cargo run -p caeles-runtime -- start com.caeles.example.hello
```

---

**CAELES** - Runtime profissional para cápsulas WebAssembly focado em Android 🚀
