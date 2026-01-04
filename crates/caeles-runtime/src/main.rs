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
    /// Caminho para o arquivo de manifest JSON da cápsula
    #[arg(long, conflicts_with = "capsule_id")]
    manifest: Option<PathBuf>,

    /// ID da cápsula (procurado no registry JSON)
    #[arg(long, conflicts_with = "manifest")]
    capsule_id: Option<String>,

    /// Caminho para o arquivo de registry de cápsulas
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,
}

fn load_manifest_from_registry(registry_path: &Path, id: &str) -> anyhow::Result<CapsuleManifest> {
    let text = fs::read_to_string(registry_path)?;
    let entries: Vec<RegistryEntry> = serde_json::from_str(&text)?;

    let entry = entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| anyhow::anyhow!(format!("Capsule id '{id}' não encontrado no registry")))?;

    println!(
        "> Registro encontrado: {} (id: {})",
        entry.name,
        entry.id
    );

    let manifest_path = Path::new(&entry.manifest);
    CapsuleManifest::load(manifest_path)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let manifest = if let Some(path) = args.manifest {
        // Caminho direto para o manifest
        CapsuleManifest::load(&path)?
    } else if let Some(id) = args.capsule_id {
        // Resolve via registry
        load_manifest_from_registry(&args.registry, &id)?
    } else {
        anyhow::bail!("Use --manifest <arquivo> ou --capsule-id <id-da-capsula>");
    };

    println!(
        "> Manifest carregado: {} v{} (id: {})",
        manifest.name, manifest.version, manifest.id
    );
    println!(
        "> Permissões: notifications={}, network={}",
        manifest.permissions.notifications, manifest.permissions.network
    );

    runtime::run_capsule(&manifest)
}
