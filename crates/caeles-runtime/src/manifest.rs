use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

fn default_path_buf() -> PathBuf {
    PathBuf::new()
}

#[derive(Debug, Deserialize)]
pub struct Permissions {
    pub notifications: bool,
    /// Reserved for future use
    #[allow(dead_code)]
    pub network: bool,
}

#[derive(Debug, Deserialize)]
pub struct CapsuleManifest {
    /// Unique identifier of the capsule (reserved for future validation)
    #[allow(dead_code)]
    pub id: String,
    /// Human-readable name (reserved for future display features)
    #[allow(dead_code)]
    pub name: String,
    /// Version string (reserved for future compatibility checks)
    #[allow(dead_code)]
    pub version: String,
    pub entry: String,
    pub permissions: Permissions,

    #[serde(skip, default = "default_path_buf")]
    base_dir: PathBuf,
}

impl CapsuleManifest {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;
        let mut manifest: CapsuleManifest = serde_json::from_str(&text)?;
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        manifest.base_dir = base.to_path_buf();
        
        // Validações básicas
        manifest.validate()?;
        
        Ok(manifest)
    }

    /// Valida os campos do manifesto
    fn validate(&self) -> anyhow::Result<()> {
        // Valida ID (deve ser formato reverse domain)
        if self.id.is_empty() {
            anyhow::bail!("Campo 'id' não pode estar vazio");
        }
        if !self.id.contains('.') {
            eprintln!("⚠️  Aviso: ID '{}' não segue convenção de reverse domain (ex: com.empresa.app)", self.id);
        }

        // Valida nome
        if self.name.is_empty() {
            anyhow::bail!("Campo 'name' não pode estar vazio");
        }

        // Valida versão (formato básico semver)
        if self.version.is_empty() {
            anyhow::bail!("Campo 'version' não pode estar vazio");
        }
        if !self.version.chars().any(|c| c.is_numeric()) {
            eprintln!("⚠️  Aviso: Versão '{}' não parece seguir semver (ex: 1.0.0)", self.version);
        }

        // Valida entry
        if self.entry.is_empty() {
            anyhow::bail!("Campo 'entry' não pode estar vazio");
        }
        if !self.entry.ends_with(".wasm") {
            eprintln!("⚠️  Aviso: Campo 'entry' não termina com .wasm: {}", self.entry);
        }

        Ok(())
    }

    /// Caminho completo para o arquivo .wasm
    pub fn wasm_path(&self) -> PathBuf {
        self.base_dir.join(&self.entry)
    }
}
