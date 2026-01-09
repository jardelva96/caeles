# 🔨 Sistema de Build CAELES

Este documento descreve o sistema de build integrado para cápsulas CAELES.

## 📋 Visão Geral

O sistema de build CAELES automatiza o processo completo de compilação de cápsulas:

1. ✅ Detecta projeto Rust válido
2. ✅ Compila para `wasm32-unknown-unknown`
3. ✅ Valida exports do WASM (`caeles_main`, `memory`)
4. ✅ Gera/atualiza manifest automaticamente
5. ✅ Calcula checksums SHA-256
6. ✅ Organiza artefatos de build

## 🚀 Uso Básico

### Compilar Cápsula

```bash
# No diretório da cápsula
cd my-capsule

# Build em modo debug
caeles-runtime build

# Build em modo release (otimizado)
caeles-runtime build --release
```

### Opções Disponíveis

```bash
caeles-runtime build [OPTIONS]

Opções:
  --path <DIR>        Diretório do projeto (padrão: diretório atual)
  -r, --release       Compilar em modo release (otimizado)
  -o, --output <DIR>  Diretório de output para artefatos
  --no-manifest       Não gerar/atualizar manifest automaticamente
  --no-hash           Não calcular hash SHA-256 do WASM
  -h, --help          Exibir ajuda
```

## 📦 Saída do Build

O comando `build` produz:

```
✅ WASM compilado: target/wasm32-unknown-unknown/{debug|release}/{nome}.wasm
✅ Manifest gerado/atualizado: capsule.manifest.json
✅ Metadata do build: build-metadata.json (se --output especificado)
```

### Exemplo de Saída

```
🚀 CAELES Build System

🔍 Detectando projeto Rust...
✅ Projeto detectado: my-capsule v0.1.0

🔨 Compilando para wasm32-unknown-unknown...
   Compiling my-capsule v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.23s
✅ WASM gerado: target/wasm32-unknown-unknown/debug/my_capsule.wasm

🔍 Validando WASM...
📦 Imports do CAELES detectados:
   - host_log
   - host_notify
📦 Tamanho do WASM: 0.12 MB (124 KB)
✅ WASM válido (exports: caeles_main, memory)

🔐 Calculando checksum...
✅ SHA-256: a3f2b8c1...7d9e4f23

📝 Gerando manifest...
✅ Manifest: capsule.manifest.json

📦 Resumo do Build:
─────────────────────────────────────
WASM:     target/wasm32-unknown-unknown/debug/my_capsule.wasm
Tamanho:  124 KB
SHA-256:  a3f2b8c1...7d9e4f23
Manifest: capsule.manifest.json
Modo:     debug
─────────────────────────────────────

✅ Build concluído com sucesso!

💡 Próximos passos:
   1. Executar: caeles-runtime --manifest capsule.manifest.json
   2. Ou instalar no registry para execução rápida
```

## 🏗️ Arquitetura do Build System

```
BuildSystem
├── ProjectDetector    → Detecta e valida projeto Rust
├── CargoBuilder       → Executa cargo build --target wasm32
├── WasmValidator      → Valida exports e imports WASM
├── ManifestGenerator  → Gera/atualiza manifest.json
└── BuildArtifacts     → Gerencia outputs e metadata
```

## ✅ Validações Automáticas

### 1. Projeto Rust

- ✓ Verifica presença de `Cargo.toml`
- ✓ Valida `src/lib.rs` existe
- ✓ Verifica `crate-type = ["cdylib"]` no Cargo.toml
- ⚠️ Avisa se `caeles-sdk` não está nas dependências

### 2. Target WASM

- ✓ Verifica se `wasm32-unknown-unknown` está instalado
- ✓ Sugere instalação se ausente: `rustup target add wasm32-unknown-unknown`

### 3. WASM Gerado

- ✓ Valida que é módulo WASM válido
- ✓ Verifica export `caeles_main` (função de entrada)
- ✓ Verifica export `memory` (comunicação com host)
- ✗ Rejeita imports WASI (não suportado ainda)
- ⚠️ Avisa sobre WASMs muito grandes (>10MB)

### 4. Manifest

- ✓ Gera ID no formato `com.caeles.<nome-pacote>`
- ✓ Extrai versão do `Cargo.toml`
- ✓ Cria path relativo para o WASM
- ✓ Preserva permissões se manifest já existe

## 🔧 Requisitos

### Software

```bash
# Rust toolchain
rustup --version

# Target WASM
rustup target add wasm32-unknown-unknown

# CAELES SDK (na cápsula)
# Cargo.toml:
[dependencies]
caeles-sdk = "0.1"
```

### Estrutura do Projeto

```
my-capsule/
├── Cargo.toml          # [lib] com crate-type = ["cdylib"]
├── src/
│   └── lib.rs          # Contém #[no_mangle] pub extern "C" fn caeles_main()
└── (build outputs)
    ├── capsule.manifest.json
    └── target/
        └── wasm32-unknown-unknown/
            └── debug/
                └── my_capsule.wasm
```

### Cargo.toml Exemplo

```toml
[package]
name = "my-capsule"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
caeles-sdk = "0.1"
```

### src/lib.rs Exemplo

```rust
use caeles_sdk::{log, notify};

#[no_mangle]
pub extern "C" fn caeles_main() {
    log("🚀 Cápsula iniciada!");
    notify("Hello from CAELES!");
}
```

## 🎯 Workflow Completo

```bash
# 1. Criar projeto de cápsula
cargo new --lib my-capsule
cd my-capsule

# 2. Configurar Cargo.toml
# Adicionar [lib] crate-type = ["cdylib"]
# Adicionar caeles-sdk como dependência

# 3. Implementar src/lib.rs
# Adicionar função caeles_main()

# 4. Build com CAELES
caeles-runtime build

# 5. Executar
caeles-runtime --manifest capsule.manifest.json

# 6. Build otimizado para produção
caeles-runtime build --release --output ./dist
```

## 📊 Comparação de Modos

| Aspecto | Debug | Release |
|---------|-------|---------|
| Otimização | Nenhuma | Máxima |
| Tempo de build | Rápido (~1-2s) | Lento (~5-10s) |
| Tamanho WASM | Maior (~200KB) | Menor (~50KB) |
| Performance | Lenta | Rápida |
| Debug info | Incluída | Removida |
| Uso | Desenvolvimento | Produção |

## 🚨 Troubleshooting

### Erro: "Cargo.toml não encontrado"

```bash
# Certifique-se de estar no diretório do projeto
cd my-capsule
caeles-runtime build
```

### Erro: "wasm32-unknown-unknown não está instalado"

```bash
rustup target add wasm32-unknown-unknown
```

### Erro: "Módulo WASM não exporta 'caeles_main'"

Adicione a função de entrada no `src/lib.rs`:

```rust
#[no_mangle]
pub extern "C" fn caeles_main() {
    // código da cápsula
}
```

### Erro: "Módulo WASM não exporta 'memory'"

Adicione ao `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]
```

### Erro: "Módulo WASM contém imports WASI"

CAELES não suporta WASI ainda. Compile para `wasm32-unknown-unknown` (não `wasm32-wasi`).

### Aviso: "WASM muito grande"

```bash
# Use modo release
caeles-runtime build --release

# Ou use wasm-opt para otimização adicional
wasm-opt -Oz input.wasm -o output.wasm
```

## 🔬 Build Avançado

### Output Customizado

```bash
# Copiar artefatos para diretório específico
caeles-runtime build --output ./dist

# Estrutura criada:
# dist/
# ├── my_capsule.wasm
# ├── capsule.manifest.json
# └── build-metadata.json
```

### Build sem Manifest

```bash
# Apenas compilar, sem gerar manifest
caeles-runtime build --no-manifest

# Útil quando manifest já está configurado manualmente
```

### Build sem Hash

```bash
# Pular cálculo de SHA-256 (mais rápido)
caeles-runtime build --no-hash
```

### Build Múltiplas Cápsulas

```bash
# Script para build de múltiplas cápsulas
for dir in capsules/*/; do
  cd "$dir"
  caeles-runtime build --release
  cd -
done
```

## 📖 Próximos Passos

Após o build bem-sucedido:

1. **Testar localmente**: `caeles-runtime --manifest capsule.manifest.json`
2. **Instalar no registry**: `caeles install` (futuro)
3. **Empacotar para distribuição**: `caeles package` (futuro)
4. **Deploy para Android**: Integração JNI (futuro)

## 🎓 Exemplos

Veja as cápsulas de exemplo incluídas no repositório:

- [hello-capsule](../capsules/hello-capsule/) - Demonstração básica
- [logger-capsule](../capsules/logger-capsule/) - Logging exemplo

Para compilá-las:

```bash
cd capsules/hello-capsule
caeles-runtime build
caeles-runtime --manifest capsule.manifest.json
```
