mod manifest;
mod runtime;

use crate::manifest::CapsuleManifest;
use clap::{Args as ClapArgs, Parser, Subcommand};
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
    name = "caeles",
    about = "CAELES runtime: executa cápsulas a partir de manifest ou ID"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Executa uma cápsula a partir de um manifest ou ID.
    Run(RunArgs),
}

#[derive(Debug, ClapArgs)]
struct RunArgs {
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

    let manifest_path = Path::new(&entry.manifest);
    CapsuleManifest::load(manifest_path)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => {
            let manifest = if let Some(path) = args.manifest {
                // Caminho direto para o manifest
                CapsuleManifest::load(&path)?
            } else if let Some(id) = args.capsule_id {
                // Resolve via registry
                load_manifest_from_registry(&args.registry, &id)?
            } else {
                anyhow::bail!("Use caeles run --manifest <arquivo> ou caeles run --capsule-id <id-da-capsula>");
            };

            runtime::run_capsule(&manifest)
        }
    }
}
