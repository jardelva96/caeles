mod manifest;
mod runtime;

use crate::manifest::CapsuleManifest;
use clap::{Args as ClapArgs, Parser, Subcommand};
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};
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
    #[command(subcommand)]
    command: Option<Command>,

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

#[derive(Debug, Subcommand)]
enum Command {
    /// Interface inicial para criar um manifest de cápsula
    Init(InitArgs),
}

#[derive(Debug, ClapArgs)]
struct InitArgs {
    /// Caminho de saída para o arquivo de manifest gerado
    #[arg(long, default_value = "capsule.manifest.json")]
    output: PathBuf,

    /// ID da cápsula (ex.: com.caeles.examples.mycapsule)
    #[arg(long)]
    id: Option<String>,

    /// Nome amigável da cápsula
    #[arg(long)]
    name: Option<String>,

    /// Versão semântica
    #[arg(long, default_value = "0.1.0")]
    version: String,

    /// Caminho do wasm exportado pela cápsula (relativo ao manifest)
    #[arg(long, default_value = "capsule.wasm")]
    entry: String,

    /// Permitir notificações (não será perguntado se informado)
    #[arg(long)]
    allow_notifications: bool,

    /// Permitir rede (não será perguntado se informado)
    #[arg(long)]
    allow_network: bool,
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

fn prompt_with_default(label: &str, default: Option<&str>) -> io::Result<String> {
    if let Some(default) = default {
        print!("{label} [{default}]: ");
    } else {
        print!("{label}: ");
    }
    io::stdout().flush()?;

    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        Ok(default.unwrap_or("").to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn prompt_bool_with_default(label: &str, default: bool) -> io::Result<bool> {
    let default_hint = if default { "Y/n" } else { "y/N" };
    print!("{label} ({default_hint}): ");
    io::stdout().flush()?;

    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let trimmed = buf.trim().to_lowercase();

    match trimmed.as_str() {
        "" => Ok(default),
        "y" | "yes" | "s" | "sim" => Ok(true),
        "n" | "no" | "não" | "nao" => Ok(false),
        _ => Ok(default),
    }
}

fn run_init_wizard(args: InitArgs) -> anyhow::Result<()> {
    println!("=== CAELES – Criador inicial de manifest ===");

    let id = if let Some(id) = args.id {
        id
    } else {
        prompt_with_default("ID da cápsula (ex.: com.caeles.examples.mycapsule)", None)?
    };

    let name = if let Some(name) = args.name {
        name
    } else {
        prompt_with_default("Nome da cápsula", None)?
    };

    let version = prompt_with_default("Versão", Some(&args.version))?;
    let entry = prompt_with_default("Caminho do wasm (relativo ao manifest)", Some(&args.entry))?;

    let allow_notifications = if args.allow_notifications {
        true
    } else {
        prompt_bool_with_default("Permitir notificações (permissions.notifications)", false)?
    };

    let allow_network = if args.allow_network {
        true
    } else {
        prompt_bool_with_default("Permitir rede (permissions.network)", false)?
    };

    let manifest = CapsuleManifest::from_parts(
        id,
        name,
        version,
        entry,
        crate::manifest::Permissions {
            notifications: allow_notifications,
            network: allow_network,
        },
    );

    let json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&args.output, json)?;

    println!("Manifest criado em: {}", args.output.display());
    println!("Lembre-se de compilar sua cápsula para wasm32-unknown-unknown.");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(Command::Init(init_args)) = args.command {
        return run_init_wizard(init_args);
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
