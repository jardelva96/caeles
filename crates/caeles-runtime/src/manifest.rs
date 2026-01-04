use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

fn default_path_buf() -> PathBuf {
    PathBuf::new()
}

fn default_false() -> bool {
    false
}

#[derive(Debug, Deserialize)]
pub struct Permissions {
    #[serde(default = "default_false")]
    pub notifications: bool,
    #[serde(default = "default_false")]
    pub network: bool,
    /// Se true, o runtime herda stdin/stdout/stderr do host.
    #[serde(default = "default_false")]
    pub inherit_stdio: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PreopenedDir {
    /// Caminho relativo ao diretório do manifest.
    pub host: PathBuf,
    /// Caminho visto pela cápsula (ex: "/data").
    pub guest: PathBuf,
    /// Se true, o runtime monta o diretório como somente leitura.
    #[serde(default = "default_false")]
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub struct ValidatedPreopen {
    pub host: PathBuf,
    pub guest: PathBuf,
    pub read_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct CapsuleManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub entry: String,
    pub permissions: Permissions,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub preopened_dirs: Vec<PreopenedDir>,

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

    pub fn validated_env(&self) -> anyhow::Result<Vec<(String, String)>> {
        for (key, value) in &self.env {
            if key.is_empty() {
                anyhow::bail!("Chave de env não pode ser vazia");
            }
            if key.contains('\0') || key.contains('=') {
                anyhow::bail!("Chave de env inválida (contém '=' ou NUL): {key}");
            }
            if value.contains('\0') {
                anyhow::bail!("Valor de env inválido (contém NUL) para chave {key}");
            }
        }
        Ok(self
            .env
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }

    fn canonicalize_base_dir(&self) -> anyhow::Result<PathBuf> {
        let base = fs::canonicalize(&self.base_dir).map_err(|e| {
            anyhow::anyhow!(
                "Não foi possível canonizar base_dir {:?}: {e}",
                self.base_dir
            )
        })?;
        Ok(base)
    }

    fn canonicalize_guest_path(&self, guest: &Path) -> anyhow::Result<PathBuf> {
        if !guest.has_root() {
            anyhow::bail!(
                "guest path de preopen deve ser absoluto: {}",
                guest.display()
            );
        }

        for comp in guest.components() {
            match comp {
                Component::ParentDir => {
                    anyhow::bail!(
                        "guest path de preopen não pode conter '..': {}",
                        guest.display()
                    );
                }
                _ => {}
            }
        }

        Ok(guest.to_path_buf())
    }

    pub fn validated_preopens(&self) -> anyhow::Result<Vec<ValidatedPreopen>> {
        let base = self.canonicalize_base_dir()?;
        let mut results = Vec::new();

        for entry in &self.preopened_dirs {
            let host_path = base.join(&entry.host);
            if !host_path.exists() {
                anyhow::bail!(
                    "Diretório preaberto não existe (host): {}",
                    host_path.display()
                );
            }
            if !host_path.is_dir() {
                anyhow::bail!(
                    "Diretório preaberto aponta para algo que não é diretório (host): {}",
                    host_path.display()
                );
            }

            let canonical_host = fs::canonicalize(&host_path).map_err(|e| {
                anyhow::anyhow!(
                    "Erro ao canonizar diretório preaberto {}: {e}",
                    host_path.display()
                )
            })?;

            if !canonical_host.starts_with(&base) {
                anyhow::bail!(
                    "Diretório preaberto sai de base_dir: {} não está dentro de {}",
                    canonical_host.display(),
                    base.display()
                );
            }

            let guest = self.canonicalize_guest_path(&entry.guest)?;

            results.push(ValidatedPreopen {
                host: canonical_host,
                guest,
                read_only: entry.read_only,
            });
        }

        Ok(results)
    }
}
