use anyhow::{bail, Context};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

fn default_path_buf() -> PathBuf {
    PathBuf::new()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Permissions {
    pub notifications: bool,
    pub network: bool,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleKind {
    OnDemand,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lifecycle {
    pub kind: LifecycleKind,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapsuleManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub entry: String,
    pub permissions: Permissions,
    pub lifecycle: Lifecycle,

    #[serde(skip, default = "default_path_buf")]
    base_dir: PathBuf,
}

impl CapsuleManifest {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("Nao foi possivel ler manifest '{}'", path.display()))?;

        let mut manifest: CapsuleManifest = serde_json::from_str(&text).with_context(|| {
            format!(
                "Manifest invalido em '{}': verifique campos obrigatorios e tipos",
                path.display()
            )
        })?;
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        manifest.base_dir = base.to_path_buf();
        manifest.validate(path)?;
        Ok(manifest)
    }

    fn validate_non_empty(path: &Path, field: &str, value: &str) -> anyhow::Result<()> {
        if value.trim().is_empty() {
            bail!(
                "Manifest invalido em '{}': campo '{}' nao pode ser vazio",
                path.display(),
                field
            );
        }

        Ok(())
    }

    fn validate(&self, path: &Path) -> anyhow::Result<()> {
        Self::validate_non_empty(path, "id", &self.id)?;
        Self::validate_non_empty(path, "name", &self.name)?;
        Self::validate_non_empty(path, "version", &self.version)?;
        Self::validate_non_empty(path, "entry", &self.entry)?;

        let entry_path = Path::new(&self.entry);
        if entry_path.is_absolute() {
            bail!(
                "Manifest invalido em '{}': 'entry' deve ser caminho relativo ao manifesto",
                path.display()
            );
        }

        let is_wasm = entry_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("wasm"))
            .unwrap_or(false);
        if !is_wasm {
            bail!(
                "Manifest invalido em '{}': 'entry' deve apontar para um arquivo .wasm",
                path.display()
            );
        }

        Ok(())
    }

    /// Full path for the wasm file.
    pub fn wasm_path(&self) -> PathBuf {
        self.base_dir.join(&self.entry)
    }
}

#[cfg(test)]
mod tests {
    use super::CapsuleManifest;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("caeles-manifest-{prefix}-{suffix}"));
        fs::create_dir_all(&dir).expect("temp directory should be created");
        dir
    }

    #[test]
    fn load_accepts_valid_v0_manifest() {
        let root = temp_dir("valid");
        let manifest_path = root.join("manifest.json");

        fs::write(
            &manifest_path,
            r#"{
  "id": "com.caeles.tests.valid",
  "name": "Valid",
  "version": "0.1.0",
  "entry": "capsule.wasm",
  "permissions": { "notifications": true, "network": false },
  "lifecycle": { "kind": "on_demand" }
}"#,
        )
        .expect("manifest should be written");

        let manifest = CapsuleManifest::load(&manifest_path).expect("manifest should load");
        assert_eq!(manifest.id, "com.caeles.tests.valid");
        assert_eq!(manifest.wasm_path(), root.join("capsule.wasm"));

        fs::remove_dir_all(root).expect("temp directory should be removed");
    }

    #[test]
    fn load_rejects_invalid_lifecycle_kind() {
        let root = temp_dir("invalid-lifecycle");
        let manifest_path = root.join("manifest.json");

        fs::write(
            &manifest_path,
            r#"{
  "id": "com.caeles.tests.invalid",
  "name": "Invalid",
  "version": "0.1.0",
  "entry": "capsule.wasm",
  "permissions": { "notifications": true, "network": false },
  "lifecycle": { "kind": "boot" }
}"#,
        )
        .expect("manifest should be written");

        let err = CapsuleManifest::load(&manifest_path)
            .expect_err("manifest should fail for invalid lifecycle");
        let err_text = err.to_string();
        assert!(
            err_text.contains("Manifest invalido"),
            "error should explain invalid manifest: {err_text}"
        );
        assert!(
            err_text.contains("on_demand"),
            "error should include accepted lifecycle value: {err_text}"
        );

        fs::remove_dir_all(root).expect("temp directory should be removed");
    }
}
