//! Sistema de storage persistente para cápsulas CAELES

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Gerenciador de storage do CAELES
pub struct CapsuleStorage {
    root_dir: PathBuf,
}

impl CapsuleStorage {
    /// Cria um novo storage na home do usuário
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .context("Não foi possível determinar diretório home do usuário")?;

        let root_dir = home.join(".caeles");

        // Criar estrutura de diretórios
        fs::create_dir_all(&root_dir)?;
        fs::create_dir_all(root_dir.join("capsules"))?;
        fs::create_dir_all(root_dir.join("logs"))?;
        fs::create_dir_all(root_dir.join("data"))?;

        Ok(Self { root_dir })
    }

    /// Cria storage em diretório customizado (para testes)
    pub fn with_root(root_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root_dir)?;
        fs::create_dir_all(root_dir.join("capsules"))?;
        fs::create_dir_all(root_dir.join("logs"))?;
        fs::create_dir_all(root_dir.join("data"))?;

        Ok(Self { root_dir })
    }

    /// Retorna o diretório raiz do storage
    pub fn root(&self) -> &Path {
        &self.root_dir
    }

    /// Retorna o diretório de cápsulas
    pub fn capsules_dir(&self) -> PathBuf {
        self.root_dir.join("capsules")
    }

    /// Retorna o diretório de logs
    pub fn logs_dir(&self) -> PathBuf {
        self.root_dir.join("logs")
    }

    /// Retorna o diretório de dados
    pub fn data_dir(&self) -> PathBuf {
        self.root_dir.join("data")
    }

    /// Retorna o diretório de uma cápsula específica
    pub fn capsule_dir(&self, capsule_id: &str) -> PathBuf {
        self.capsules_dir().join(sanitize_id(capsule_id))
    }

    /// Verifica se uma cápsula está instalada
    pub fn is_installed(&self, capsule_id: &str) -> bool {
        self.capsule_dir(capsule_id).exists()
    }

    /// Lista IDs de todas as cápsulas instaladas
    pub fn list_installed(&self) -> Result<Vec<String>> {
        let capsules_dir = self.capsules_dir();

        if !capsules_dir.exists() {
            return Ok(Vec::new());
        }

        let mut capsule_ids = Vec::new();

        for entry in fs::read_dir(capsules_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    // Reverter sanitização
                    let id = name.replace("_", ".");
                    capsule_ids.push(id);
                }
            }
        }

        Ok(capsule_ids)
    }

    /// Instala uma cápsula copiando WASM e manifest
    pub fn install_capsule(
        &self,
        capsule_id: &str,
        wasm_path: &Path,
        manifest_path: &Path,
    ) -> Result<()> {
        let capsule_dir = self.capsule_dir(capsule_id);

        // Verificar se já está instalada
        if capsule_dir.exists() {
            anyhow::bail!(
                "Cápsula '{}' já está instalada em {}\n\
                 Use 'caeles remove {}' para desinstalar primeiro.",
                capsule_id,
                capsule_dir.display(),
                capsule_id
            );
        }

        // Criar diretório da cápsula
        fs::create_dir_all(&capsule_dir)
            .context("Falha ao criar diretório da cápsula")?;

        // Copiar WASM
        let wasm_dest = capsule_dir.join("capsule.wasm");
        fs::copy(wasm_path, &wasm_dest)
            .context("Falha ao copiar arquivo WASM")?;

        // Copiar manifest
        let manifest_dest = capsule_dir.join("manifest.json");
        fs::copy(manifest_path, &manifest_dest)
            .context("Falha ao copiar manifest")?;

        // Criar metadata
        let metadata = InstallMetadata::new(capsule_id);
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(capsule_dir.join("metadata.json"), metadata_json)?;

        Ok(())
    }

    /// Remove uma cápsula instalada
    pub fn remove_capsule(&self, capsule_id: &str) -> Result<()> {
        let capsule_dir = self.capsule_dir(capsule_id);

        if !capsule_dir.exists() {
            anyhow::bail!("Cápsula '{}' não está instalada", capsule_id);
        }

        fs::remove_dir_all(&capsule_dir)
            .context("Falha ao remover diretório da cápsula")?;

        Ok(())
    }

    /// Obtém o caminho do WASM de uma cápsula instalada
    pub fn get_wasm_path(&self, capsule_id: &str) -> Result<PathBuf> {
        let wasm_path = self.capsule_dir(capsule_id).join("capsule.wasm");

        if !wasm_path.exists() {
            anyhow::bail!("WASM não encontrado para cápsula '{}'", capsule_id);
        }

        Ok(wasm_path)
    }

    /// Obtém o caminho do manifest de uma cápsula instalada
    pub fn get_manifest_path(&self, capsule_id: &str) -> Result<PathBuf> {
        let manifest_path = self.capsule_dir(capsule_id).join("manifest.json");

        if !manifest_path.exists() {
            anyhow::bail!("Manifest não encontrado para cápsula '{}'", capsule_id);
        }

        Ok(manifest_path)
    }

    /// Obtém metadata de instalação
    pub fn get_metadata(&self, capsule_id: &str) -> Result<InstallMetadata> {
        let metadata_path = self.capsule_dir(capsule_id).join("metadata.json");

        if !metadata_path.exists() {
            // Criar metadata padrão se não existir (compatibilidade)
            return Ok(InstallMetadata::new(capsule_id));
        }

        let content = fs::read_to_string(&metadata_path)?;
        let metadata = serde_json::from_str(&content)?;

        Ok(metadata)
    }

    /// Limpa todos os dados (USE COM CUIDADO!)
    pub fn clear_all(&self) -> Result<()> {
        if self.root_dir.exists() {
            fs::remove_dir_all(&self.root_dir)?;
        }

        // Recriar estrutura
        fs::create_dir_all(&self.root_dir)?;
        fs::create_dir_all(self.capsules_dir())?;
        fs::create_dir_all(self.logs_dir())?;
        fs::create_dir_all(self.data_dir())?;

        Ok(())
    }

    /// Retorna estatísticas do storage
    pub fn stats(&self) -> Result<StorageStats> {
        let total_capsules = self.list_installed()?.len();

        let total_size = calculate_dir_size(&self.capsules_dir())?;

        Ok(StorageStats {
            total_capsules,
            total_size_bytes: total_size,
            storage_path: self.root_dir.clone(),
        })
    }
}

impl Default for CapsuleStorage {
    fn default() -> Self {
        Self::new().expect("Falha ao criar storage padrão")
    }
}

/// Metadata de instalação de cápsula
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstallMetadata {
    pub capsule_id: String,
    pub installed_at: u64,
    pub install_count: u32,
    pub last_run: Option<u64>,
    pub run_count: u64,
}

impl InstallMetadata {
    pub fn new(capsule_id: &str) -> Self {
        Self {
            capsule_id: capsule_id.to_string(),
            installed_at: current_timestamp(),
            install_count: 1,
            last_run: None,
            run_count: 0,
        }
    }

    pub fn mark_run(&mut self) {
        self.last_run = Some(current_timestamp());
        self.run_count += 1;
    }
}

/// Estatísticas do storage
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_capsules: usize,
    pub total_size_bytes: u64,
    pub storage_path: PathBuf,
}

impl StorageStats {
    pub fn total_size_mb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn total_size_kb(&self) -> u64 {
        self.total_size_bytes / 1024
    }
}

/// Sanitiza ID de cápsula para nome de diretório
fn sanitize_id(id: &str) -> String {
    // Substitui pontos por underscores para filesystem
    id.replace(".", "_")
}

/// Calcula timestamp atual
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Calcula tamanho total de um diretório recursivamente
fn calculate_dir_size(path: &Path) -> Result<u64> {
    let mut total_size = 0u64;

    if !path.exists() {
        return Ok(0);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            total_size += calculate_dir_size(&entry.path())?;
        }
    }

    Ok(total_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn test_storage() -> CapsuleStorage {
        let test_dir = env::temp_dir().join("caeles-test");
        if test_dir.exists() {
            let _ = fs::remove_dir_all(&test_dir);
        }
        CapsuleStorage::with_root(test_dir).unwrap()
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(sanitize_id("com.caeles.app"), "com_caeles_app");
        assert_eq!(sanitize_id("simple"), "simple");
    }

    #[test]
    fn test_storage_creation() {
        let storage = test_storage();
        assert!(storage.capsules_dir().exists());
        assert!(storage.logs_dir().exists());
        assert!(storage.data_dir().exists());
    }

    #[test]
    fn test_list_installed_empty() {
        let storage = test_storage();
        let installed = storage.list_installed().unwrap();
        assert_eq!(installed.len(), 0);
    }

    #[test]
    fn test_is_installed() {
        let storage = test_storage();
        assert!(!storage.is_installed("com.caeles.test"));
    }
}
