mod manifest;
mod runtime;

use crate::manifest::CapsuleManifest;
use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub manifest: String,
}

#[derive(Debug, Parser)]
#[command(
    name = "caeles-runtime",
    about = "CAELES runtime: executa cápsulas a partir de manifest ou ID"
)]
struct Args {
    /// Lista as cápsulas disponíveis no registry
    #[arg(long, default_value_t = false)]
    list: bool,

    /// Caminho para o arquivo de manifest JSON da cápsula
    #[arg(long, conflicts_with_all = ["capsule_id", "list"])]
    manifest: Option<PathBuf>,

    /// ID da cápsula (procurado no registry JSON)
    #[arg(long, conflicts_with_all = ["manifest", "list"])]
    capsule_id: Option<String>,

    /// Caminho para o arquivo de registry de cápsulas
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,
}

fn load_registry_entries(registry_path: &Path) -> anyhow::Result<Vec<RegistryEntry>> {
    let text = fs::read_to_string(registry_path)?;
    Ok(serde_json::from_str(&text)?)
}

fn resolve_manifest_path(registry_path: &Path, manifest: &str) -> PathBuf {
    let manifest_path = Path::new(manifest);

    if manifest_path.is_absolute() {
        return manifest_path.to_path_buf();
    }

    let registry_dir = registry_path.parent().unwrap_or_else(|| Path::new("."));

    if manifest_path.starts_with(registry_dir) {
        return manifest_path.to_path_buf();
    }

    registry_dir.join(manifest_path)
}

fn load_manifest_from_registry(registry_path: &Path, id: &str) -> anyhow::Result<CapsuleManifest> {
    let entries = load_registry_entries(registry_path)?;

    let entry = entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| anyhow::anyhow!(format!("Capsule id '{id}' não encontrado no registry")))?;

    let manifest_path = resolve_manifest_path(registry_path, &entry.manifest);
    CapsuleManifest::load(&manifest_path)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.list {
        let entries = load_registry_entries(&args.registry)?;

        if entries.is_empty() {
            println!(
                "Nenhuma cápsula encontrada no registry: {}",
                args.registry.display()
            );
            return Ok(());
        }

        println!("Cápsulas em {}:", args.registry.display());
        for entry in entries {
            let manifest_path = resolve_manifest_path(&args.registry, &entry.manifest);
            println!("- {} ({})", entry.id, entry.name);
            println!("  manifest: {}", manifest_path.display());
        }

        return Ok(());
    }

    let manifest = if let Some(path) = args.manifest {
        // Caminho direto para o manifest
        CapsuleManifest::load(&path)?
    } else if let Some(id) = args.capsule_id {
        // Resolve via registry
        load_manifest_from_registry(&args.registry, &id)?
    } else {
        anyhow::bail!("Use --manifest <arquivo> ou --capsule-id <id-da-capsula>");
    };

    runtime::run_capsule(&manifest)
}

#[cfg(test)]
mod tests {
    use super::resolve_manifest_path;
    use std::path::Path;

    #[test]
    fn resolve_manifest_path_keeps_existing_relative_manifest() {
        let registry = Path::new("capsules/registry.json");
        let manifest = "capsules/hello-capsule/manifest.json";

        let resolved = resolve_manifest_path(registry, manifest);

        assert_eq!(resolved, Path::new("capsules/hello-capsule/manifest.json"));
    }

    #[test]
    fn resolve_manifest_path_uses_registry_dir_for_registry_relative_manifest() {
        let registry = Path::new("capsules/registry.json");
        let manifest = "hello-capsule/manifest.json";

        let resolved = resolve_manifest_path(registry, manifest);

        assert_eq!(resolved, Path::new("capsules/hello-capsule/manifest.json"));
    }

    #[test]
    fn resolve_manifest_path_keeps_absolute_manifest() {
        let registry = Path::new("capsules/registry.json");
        let manifest = "/tmp/manifest.json";

        let resolved = resolve_manifest_path(registry, manifest);

        assert_eq!(resolved, Path::new("/tmp/manifest.json"));
    }
}
