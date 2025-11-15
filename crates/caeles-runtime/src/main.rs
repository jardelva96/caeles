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
    #[arg(long)]
    manifest: Option<PathBuf>,

    /// ID da cápsula (procurado no registry JSON)
    #[arg(long)]
    capsule_id: Option<String>,

    /// Caminho para o arquivo de registry de cápsulas
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,

    /// Lista as cápsulas cadastradas no registry e sai
    #[arg(long)]
    list_capsules: bool,
}

fn load_manifest_from_registry(registry_path: &Path, id: &str) -> anyhow::Result<CapsuleManifest> {
    let text = fs::read_to_string(registry_path)?;
    let entries: Vec<RegistryEntry> = serde_json::from_str(&text)?;

    let entry = entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| anyhow::anyhow!(format!("Capsule id '{id}' não encontrado no registry")))?;

    let manifest_path = Path::new(&entry.manifest);
    CapsuleManifest::load(manifest_path)
}

fn list_capsules(registry_path: &Path) -> anyhow::Result<()> {
    let text = fs::read_to_string(registry_path)?;
    let entries: Vec<RegistryEntry> = serde_json::from_str(&text)?;

    println!(
        "Capsules registradas em \"{}\":",
        registry_path.to_string_lossy()
    );

    for e in entries {
        println!("- {:<30} ({}) -> {}", e.id, e.name, e.manifest);
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.list_capsules {
        // Só lista e sai
        list_capsules(&args.registry)?;
        return Ok(());
    }

    let manifest = if let Some(path) = args.manifest {
        // Caminho direto para o manifest
        CapsuleManifest::load(&path)?
    } else if let Some(id) = args.capsule_id {
        // Resolve via registry
        load_manifest_from_registry(&args.registry, &id)?
    } else {
        anyhow::bail!("Use --manifest <arquivo>, --capsule-id <id-da-capsula> ou --list-capsules");
    };

    runtime::run_capsule(&manifest)
}
