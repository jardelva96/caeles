use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

fn default_path_buf() -> PathBuf {
    PathBuf::new()
}

#[derive(Debug, Deserialize)]
pub struct Permissions {
    pub notifications: bool,
    pub network: bool,
    pub metrics: bool,
    pub storage: bool,
}

#[derive(Debug, Deserialize)]
pub struct CapsuleManifest {
    pub id: String,
    pub name: String,
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
        Ok(manifest)
    }

    /// Caminho completo para o arquivo .wasm
    pub fn wasm_path(&self) -> PathBuf {
        self.base_dir.join(&self.entry)
    }
}
