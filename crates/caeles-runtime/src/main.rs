mod manifest;
mod runtime;

#[cfg(test)]
mod manifest_test;

use crate::manifest::CapsuleManifest;
use anyhow::Context;
use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    pub id: String,
    /// Human-readable name for future UI display
    #[allow(dead_code)]
    pub name: String,
    pub manifest: String,
}

#[derive(Debug, Parser)]
#[command(
    name = "caeles-runtime",
    about = "CAELES runtime: executa cápsulas a partir de manifest ou ID",
    long_about = "CAELES é um motor de cápsulas WebAssembly focado em Android.\nExecuta módulos WASM com isolamento e controle de permissões.",
    version = "0.1.0"
)]
struct Args {
    /// Caminho para o arquivo de manifest JSON da cápsula
    #[arg(long, conflicts_with = "capsule_id", conflicts_with = "list")]
    manifest: Option<PathBuf>,

    /// ID da cápsula (procurado no registry JSON)
    #[arg(long, conflicts_with = "manifest", conflicts_with = "list")]
    capsule_id: Option<String>,

    /// Caminho para o arquivo de registry de cápsulas
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,

    /// Lista todas as cápsulas disponíveis no registry
    #[arg(long, conflicts_with = "manifest", conflicts_with = "capsule_id")]
    list: bool,
}

fn load_manifest_from_registry(registry_path: &Path, id: &str) -> anyhow::Result<CapsuleManifest> {
    let text = fs::read_to_string(registry_path)
        .with_context(|| format!("Não foi possível ler o registry: {}", registry_path.display()))?;
    
    let entries: Vec<RegistryEntry> = serde_json::from_str(&text)
        .context("Registry JSON inválido")?;

    let entry = entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| anyhow::anyhow!(
            "Capsule id '{}' não encontrado no registry.\nUse --list para ver cápsulas disponíveis.", 
            id
        ))?;

    let manifest_path = Path::new(&entry.manifest);
    CapsuleManifest::load(manifest_path)
}

fn list_capsules(registry_path: &Path) -> anyhow::Result<()> {
    let text = fs::read_to_string(registry_path)
        .with_context(|| format!("Não foi possível ler o registry: {}", registry_path.display()))?;
    
    let entries: Vec<RegistryEntry> = serde_json::from_str(&text)
        .context("Registry JSON inválido")?;

    if entries.is_empty() {
        println!("Nenhuma cápsula registrada em {}", registry_path.display());
        return Ok(());
    }

    println!("Cápsulas disponíveis:");
    println!();
    for entry in entries {
        println!("  🔹 {}", entry.name);
        println!("     ID: {}", entry.id);
        println!("     Manifest: {}", entry.manifest);
        println!();
    }
    
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.list {
        return list_capsules(&args.registry);
    }

    let manifest = if let Some(path) = args.manifest {
        // Caminho direto para o manifest
        CapsuleManifest::load(&path)?
    } else if let Some(id) = args.capsule_id {
        // Resolve via registry
        load_manifest_from_registry(&args.registry, &id)?
    } else {
        anyhow::bail!("Use --manifest <arquivo>, --capsule-id <id-da-capsula>, ou --list para listar cápsulas");
    };

    runtime::run_capsule(&manifest)
}
