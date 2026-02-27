mod manifest;
mod runtime;

use crate::manifest::CapsuleManifest;
use clap::{Args, Parser, Subcommand};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub manifest: String,
}

#[derive(Debug, Parser)]
#[command(name = "caeles", about = "CAELES CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Executa uma cápsula a partir de um manifest ou ID do registry.
    Run(RunArgs),
    /// Lista as cápsulas disponíveis no registry.
    List(ListArgs),
    /// Compila uma cápsula para WebAssembly (wasm32-unknown-unknown).
    Build(BuildArgs),
}

#[derive(Debug, Args)]
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

#[derive(Debug, Args)]
struct ListArgs {
    /// Caminho para o arquivo de registry de cápsulas
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,
}

#[derive(Debug, Args)]
struct BuildArgs {
    /// Caminho para o Cargo.toml da cápsula (diretório ou arquivo).
    path: PathBuf,

    /// Compila no perfil release.
    #[arg(long, default_value_t = false)]
    release: bool,

    /// Triple de compilação (padrão CAELES).
    #[arg(long, default_value = "wasm32-unknown-unknown")]
    target: String,
}

fn load_registry_entries(registry_path: &Path) -> anyhow::Result<Vec<RegistryEntry>> {
    let text = fs::read_to_string(registry_path)?;
    let entries: Vec<RegistryEntry> = serde_json::from_str(&text)?;

    let mut seen_ids = HashSet::new();
    for entry in &entries {
        if !seen_ids.insert(&entry.id) {
            anyhow::bail!("ID duplicado no registry: '{}'", entry.id);
        }
    }

    Ok(entries)
}

fn resolve_manifest_path(registry_path: &Path, manifest: &str) -> PathBuf {
    let manifest_path = Path::new(manifest);

    if manifest_path.is_absolute() {
        return manifest_path.to_path_buf();
    }

    let registry_dir = registry_path.parent().unwrap_or_else(|| Path::new("."));

    if manifest_path.exists() {
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
    if !manifest_path.exists() {
        anyhow::bail!(
            "Manifest da cápsula '{}' não encontrado em '{}'",
            id,
            manifest_path.display()
        );
    }

    CapsuleManifest::load(&manifest_path)
}

fn run_command(args: RunArgs) -> anyhow::Result<()> {
    let manifest = if let Some(path) = args.manifest {
        CapsuleManifest::load(&path)?
    } else if let Some(id) = args.capsule_id {
        load_manifest_from_registry(&args.registry, &id)?
    } else {
        anyhow::bail!("Use --manifest <arquivo> ou --capsule-id <id-da-capsula>");
    };

    runtime::run_capsule(&manifest)
}

fn list_command(args: ListArgs) -> anyhow::Result<()> {
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
        if manifest_path.exists() {
            println!("  manifest: {}", manifest_path.display());
        } else {
            println!("  manifest: {} [não encontrado]", manifest_path.display());
        }
    }

    Ok(())
}

fn build_command(args: BuildArgs) -> anyhow::Result<()> {
    let manifest_path = if args.path.is_dir() {
        args.path.join("Cargo.toml")
    } else {
        args.path.clone()
    };

    if !manifest_path.exists() {
        anyhow::bail!("Cargo.toml não encontrado em '{}'", manifest_path.display());
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--target")
        .arg(&args.target);

    if args.release {
        cmd.arg("--release");
    }

    println!("> Executando: {:?}", cmd);
    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Falha ao compilar cápsula com cargo build");
    }

    println!("> Build concluído: {}", manifest_path.display());
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => run_command(args),
        Commands::List(args) => list_command(args),
        Commands::Build(args) => build_command(args),
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_manifest_path, Cli, Commands};
    use clap::Parser;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("caeles-{prefix}-{suffix}"));
        fs::create_dir_all(&dir).expect("temp directory should be created");
        dir
    }

    #[test]
    fn parse_run_subcommand() {
        let cli =
            Cli::try_parse_from(["caeles", "run", "--capsule-id", "com.caeles.example.hello"])
                .expect("run command should parse");
        assert!(matches!(cli.command, Commands::Run(_)));
    }

    #[test]
    fn parse_build_subcommand() {
        let cli = Cli::try_parse_from(["caeles", "build", "capsules/hello-capsule"])
            .expect("build command should parse");
        assert!(matches!(cli.command, Commands::Build(_)));
    }

    #[test]
    fn resolve_manifest_path_keeps_existing_relative_manifest() {
        let root = temp_dir("existing-relative");
        let existing_manifest = root.join("capsules/hello-capsule/manifest.json");

        fs::create_dir_all(
            existing_manifest
                .parent()
                .expect("manifest should have parent"),
        )
        .expect("manifest parent should be created");
        fs::write(&existing_manifest, "{}")
            .expect("manifest file should be created for test setup");

        let previous_dir = std::env::current_dir().expect("current dir should be readable");
        std::env::set_current_dir(&root).expect("current dir should be changed for test");

        let resolved = resolve_manifest_path(
            Path::new("capsules/registry.json"),
            "capsules/hello-capsule/manifest.json",
        );

        std::env::set_current_dir(previous_dir).expect("current dir should be restored");

        assert_eq!(resolved, Path::new("capsules/hello-capsule/manifest.json"));

        fs::remove_dir_all(root).expect("temp directory should be removed");
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
