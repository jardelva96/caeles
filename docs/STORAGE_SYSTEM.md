# 💾 Sistema de Storage Persistente CAELES

Este documento descreve o sistema de armazenamento persistente para cápsulas CAELES.

## 📋 Visão Geral

O Storage System do CAELES gerencia a instalação, armazenamento e remoção de cápsulas de forma persistente no sistema de arquivos.

### Estrutura de Diretórios

```
~/.caeles/                          # Diretório raiz
├── capsules/                       # Cápsulas instaladas
│   ├── com_caeles_example_hello/   # ID sanitizado (. → _)
│   │   ├── capsule.wasm           # Binário WASM
│   │   ├── manifest.json          # Manifest da cápsula
│   │   └── metadata.json          # Metadata de instalação
│   └── com_caeles_example_logger/
│       ├── capsule.wasm
│       ├── manifest.json
│       └── metadata.json
├── logs/                           # Logs de execução (futuro)
└── data/                           # Dados persistentes (futuro)
```

## 🚀 Comandos Disponíveis

### 1. `caeles install`

Instala uma cápsula no sistema.

```bash
# Instalar do diretório atual (usa capsule.manifest.json)
caeles install

# Instalar de manifest específico
caeles install --manifest path/to/manifest.json

# Instalar de projeto específico
caeles install --path ./my-capsule

# Forçar reinstalação
caeles install --force
```

**Processo:**
1. Carrega manifest
2. Verifica se WASM existe
3. Verifica se já está instalada
4. Copia WASM e manifest para `~/.caeles/capsules/{id}/`
5. Cria metadata de instalação

**Output:**
```
📦 CAELES Install

📄 Manifest: Hello Capsule
🆔 ID: com.caeles.example.hello
📌 Versão: 0.1.0

📥 Instalando cápsula...
✅ Cápsula 'com.caeles.example.hello' instalada com sucesso!

💡 Próximos passos:
   caeles list              # Ver cápsulas instaladas
   caeles start com.caeles.example.hello  # Iniciar cápsula (futuro)
```

---

### 2. `caeles list`

Lista cápsulas instaladas.

```bash
# Lista compacta (padrão)
caeles list

# Lista detalhada
caeles list --verbose

# Output JSON
caeles list --format json
```

**Modo Compacto:**
```
📦 Cápsulas Instaladas (2):

ID                                       NOME                      VERSÃO
─────────────────────────────────────────────────────────────────────────────
com.caeles.example.hello                 Hello Capsule             0.1.0
com.caeles.example.logger                Logger Capsule            0.2.0

📊 Storage: 2 instaladas, 0.24 MB em /home/user/.caeles
```

**Modo Verbose:**
```
📦 Cápsulas Instaladas (2):

─────────────────────────────────────
ID:       com.caeles.example.hello
Nome:     Hello Capsule
Versão:   0.1.0
WASM:     124 KB
Execuções: 0
Instalado: 5 minutos atrás
─────────────────────────────────────
ID:       com.caeles.example.logger
Nome:     Logger Capsule
Versão:   0.2.0
WASM:     128 KB
Execuções: 0
Instalado: 10 minutos atrás
─────────────────────────────────────

📊 Storage: 2 instaladas, 0.24 MB em /home/user/.caeles
```

**Formato JSON:**
```json
[
  {
    "id": "com.caeles.example.hello",
    "name": "Hello Capsule",
    "version": "0.1.0",
    "installed_at": 1704896400,
    "run_count": 0
  },
  {
    "id": "com.caeles.example.logger",
    "name": "Logger Capsule",
    "version": "0.2.0",
    "installed_at": 1704896100,
    "run_count": 0
  }
]
```

---

### 3. `caeles remove`

Remove uma cápsula instalada.

```bash
# Remover com confirmação
caeles remove com.caeles.example.hello

# Remover sem confirmação
caeles remove com.caeles.example.hello --yes
```

**Output:**
```
🗑️  Remover cápsula:
   ID:      com.caeles.example.hello
   Nome:    Hello Capsule
   Versão:  0.1.0

Tem certeza? (s/N): s

🗑️  Removendo...
✅ Cápsula 'com.caeles.example.hello' removida com sucesso!
```

---

## 🎯 Workflow Completo

### Desenvolvimento e Instalação

```bash
# 1. Criar e implementar cápsula
cargo new --lib my-capsule
cd my-capsule
# ... implementar código ...

# 2. Build
caeles build

# 3. Instalar
caeles install

# 4. Verificar instalação
caeles list

# 5. Executar (quando implementado)
caeles start com.caeles.my-capsule
```

### Atualização de Cápsula

```bash
# 1. Fazer alterações no código
# 2. Rebuild
caeles build

# 3. Forçar reinstalação
caeles install --force

# Ou: remover e instalar
caeles remove com.caeles.my-capsule --yes
caeles install
```

### Limpeza

```bash
# Remover cápsula específica
caeles remove com.caeles.my-capsule

# Ver o que sobrou
caeles list
```

---

## 📊 Metadata de Instalação

Cada cápsula instalada tem um arquivo `metadata.json`:

```json
{
  "capsule_id": "com.caeles.example.hello",
  "installed_at": 1704896400,
  "install_count": 1,
  "last_run": null,
  "run_count": 0
}
```

**Campos:**
- `capsule_id`: ID único da cápsula
- `installed_at`: Unix timestamp da instalação
- `install_count`: Número de vezes reinstalada
- `last_run`: Timestamp da última execução (null se nunca executada)
- `run_count`: Total de execuções

---

## 🏗️ Arquitetura Técnica

### CapsuleStorage

```rust
pub struct CapsuleStorage {
    root_dir: PathBuf,  // ~/.caeles
}

impl CapsuleStorage {
    // Criar storage no home do usuário
    pub fn new() -> Result<Self>

    // Instalar cápsula
    pub fn install_capsule(&self, capsule_id: &str, wasm_path: &Path, manifest_path: &Path) -> Result<()>

    // Listar instaladas
    pub fn list_installed(&self) -> Result<Vec<String>>

    // Remover cápsula
    pub fn remove_capsule(&self, capsule_id: &str) -> Result<()>

    // Verificar instalação
    pub fn is_installed(&self, capsule_id: &str) -> bool

    // Obter paths
    pub fn get_wasm_path(&self, capsule_id: &str) -> Result<PathBuf>
    pub fn get_manifest_path(&self, capsule_id: &str) -> Result<PathBuf>

    // Metadata
    pub fn get_metadata(&self, capsule_id: &str) -> Result<InstallMetadata>

    // Estatísticas
    pub fn stats(&self) -> Result<StorageStats>
}
```

### Sanitização de IDs

IDs de cápsulas usam formato reverse-domain (`com.caeles.app`), mas filesystems têm restrições. A sanitização converte para nomes de diretório válidos:

```
com.caeles.example.hello  →  com_caeles_example_hello
```

**Regra:** Substitui `.` por `_`

### Operações Atômicas

O storage realiza operações de forma segura:

1. **Install:**
   - Cria diretório da cápsula
   - Copia WASM
   - Copia manifest
   - Cria metadata
   - Falha em qualquer erro (não deixa instalação parcial)

2. **Remove:**
   - Verifica se existe
   - Remove diretório completo recursivamente

---

## 🔒 Validações e Segurança

### Install

✓ Verifica se manifest é válido
✓ Verifica se WASM existe
✓ Verifica se já está instalada (evita duplicatas)
✓ Valida IDs únicos
✓ Cria estrutura de diretórios se não existe

### Remove

✓ Verifica se cápsula existe
✓ Confirmação interativa (pode ser bypassed com --yes)
✓ Remove apenas o diretório da cápsula (não afeta outras)

---

## 📈 Estatísticas

O comando `list` exibe estatísticas do storage:

```rust
pub struct StorageStats {
    pub total_capsules: usize,      // Número de cápsulas
    pub total_size_bytes: u64,      // Tamanho total em bytes
    pub storage_path: PathBuf,      // Caminho do storage
}
```

**Cálculo de Tamanho:**
- Recursivo através de todos os arquivos
- Inclui WASM + manifests + metadata
- Exibido em KB ou MB conforme tamanho

---

## 🚨 Troubleshooting

### Erro: "Cápsula já está instalada"

```
Solução 1: Usar --force
caeles install --force

Solução 2: Remover primeiro
caeles remove {id}
caeles install
```

### Erro: "Arquivo WASM não encontrado"

```
Causa: Manifest aponta para WASM que não existe

Solução: Executar build primeiro
caeles build
caeles install
```

### Erro: "Cápsula não está instalada"

```
Causa: Tentou remover cápsula não instalada

Verificar: caeles list
```

### Erro: "Não foi possível determinar diretório home"

```
Causa: Variável HOME não definida (raro)

Solução: Definir HOME manualmente ou usar root customizado
export HOME=/caminho/para/home
```

---

## 🔮 Próximos Passos (Futuro)

### Storage Melhorias:

**1. Versionamento Completo**
```bash
caeles install --version 1.0.0  # Manter múltiplas versões
caeles list --all-versions       # Ver todas as versões
caeles switch {id} --version 1.0.0  # Trocar versão ativa
```

**2. Backup e Restore**
```bash
caeles backup {id} --output backup.tar.gz
caeles restore backup.tar.gz
```

**3. Storage Compactação**
```bash
caeles gc  # Garbage collection
caeles prune --older-than 30d  # Remover antigas
```

**4. Importação de Pacotes**
```bash
caeles install app.capsule  # Instalar de arquivo .capsule
```

**5. Storage Remoto**
```bash
caeles push {id}  # Enviar para registry
caeles pull {registry-id}  # Baixar de registry
```

---

## 📖 Exemplos Práticos

### Exemplo 1: Workflow Básico

```bash
# Desenvolver
cd my-project
caeles build

# Instalar localmente
caeles install

# Verificar
caeles list -v

# Remover quando não precisar mais
caeles remove com.mycompany.myapp
```

### Exemplo 2: CI/CD

```bash
#!/bin/bash
# Script de build e deploy

# Build
caeles build --release

# Remover versão antiga (se existir)
caeles remove com.app.production --yes || true

# Instalar nova versão
caeles install

# Verificar instalação
caeles list --format json | jq '.[] | select(.id == "com.app.production")'

# Start (futuro)
# caeles start com.app.production
```

### Exemplo 3: Múltiplos Ambientes

```bash
# Dev
cd dev-capsule
caeles build
caeles install

# Staging
cd ../staging-capsule
caeles build
caeles install

# Production
cd ../prod-capsule
caeles build --release
caeles install

# Ver todos
caeles list
```

---

## 💡 Boas Práticas

### IDs de Cápsulas

✅ **Bom:** `com.empresa.projeto.componente`
❌ **Ruim:** `my-app` (muito genérico)

✅ **Bom:** `com.caeles.utils.logger`
❌ **Ruim:** `logger123` (sem contexto)

### Instalação

✅ Sempre executar `build` antes de `install`
✅ Usar `--force` apenas quando necessário
✅ Verificar com `list` após instalar

### Remoção

✅ Usar `list` antes de remover para confirmar ID
✅ Usar `--yes` apenas em scripts automatizados
✅ Fazer backup se necessário antes de remover

---

## 🎓 Comparação com Docker

| Aspecto | Docker | CAELES |
|---------|--------|--------|
| **Armazenamento** | `/var/lib/docker` | `~/.caeles` |
| **Instalação** | `docker pull` | `caeles install` |
| **Listagem** | `docker images` | `caeles list` |
| **Remoção** | `docker rmi` | `caeles remove` |
| **Formato** | Layers (OCI) | WASM + Manifest |
| **Tamanho** | MB-GB | KB-MB |
| **Isolamento** | Containers | WASM Sandbox |

---

**Sistema de Storage implementado com sucesso! 🎉**

Next: Lifecycle Management (start/stop/status)
