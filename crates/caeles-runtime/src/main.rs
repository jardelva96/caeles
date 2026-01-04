mod manifest;
mod runtime;

use crate::manifest::CapsuleManifest;
use clap::{Args as ClapArgs, Parser, Subcommand};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

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
    /// Lista as cápsulas disponíveis no registry.
    List(ListArgs),
    /// Interface interativa em terminal para listar e executar cápsulas.
    Ui(UiArgs),
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

#[derive(Debug, ClapArgs)]
struct ListArgs {
    /// Caminho para o arquivo de registry de cápsulas
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,

    /// Formato de saída (texto ou json)
    #[arg(long, value_parser = ["text", "json"], default_value = "text")]
    format: String,

    /// Filtro (trecho do ID ou nome)
    #[arg(long)]
    filter: Option<String>,
}

#[derive(Debug, ClapArgs)]
struct UiArgs {
    /// Caminho para o arquivo de registry de cápsulas
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,

    /// Filtro inicial (trecho do ID ou nome)
    #[arg(long)]
    filter: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub manifest: String,
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

fn list_registry(registry_path: &Path) -> anyhow::Result<Vec<RegistryEntry>> {
    let text = fs::read_to_string(registry_path)?;
    let entries: Vec<RegistryEntry> = serde_json::from_str(&text)?;
    Ok(entries)
}

fn filter_entries<'a>(
    entries: &'a [RegistryEntry],
    filter: &Option<String>,
) -> Vec<&'a RegistryEntry> {
    if let Some(f) = filter {
        let needle = f.to_lowercase();
        entries
            .iter()
            .filter(|e| {
                e.id.to_lowercase().contains(&needle) || e.name.to_lowercase().contains(&needle)
            })
            .collect()
    } else {
        entries.iter().collect()
    }
}

fn ui_loop(registry_path: &Path, initial_filter: Option<String>) -> anyhow::Result<()> {
    use std::io::{self, Write};

    let mut filter: Option<String> = initial_filter;

    loop {
        let entries = list_registry(registry_path)?;
        let visible_entries = filter_entries(&entries, &filter);

        if entries.is_empty() {
            println!("Nenhuma cápsula encontrada em {}.", registry_path.display());
        } else {
            println!("Capsules disponíveis:");
            if let Some(f) = &filter {
                println!("(filtro ativo: \"{}\")", f);
            }
            for (idx, entry) in visible_entries.iter().enumerate() {
                println!(
                    "  [{}] {} ({}) -> {}",
                    idx + 1,
                    entry.name,
                    entry.id,
                    entry.manifest
                );
            }
        }

        println!("\nComandos: número para executar, 'r' recarrega, 'q' sai, 's <texto>' filtra, 'id <capsule-id>' executa por ID, 'json' alterna modo detalhado.");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();

        if trimmed.eq_ignore_ascii_case("q") {
            println!("Saindo da UI.");
            break;
        } else if trimmed.eq_ignore_ascii_case("r") {
            filter = None;
            continue;
        } else if let Some(rest) = trimmed.strip_prefix("s ") {
            let value = rest.trim();
            if value.is_empty() {
                filter = None;
                println!("Filtro removido.\n");
            } else {
                filter = Some(value.to_string());
                println!("Filtro atualizado para \"{value}\".\n");
            }
            continue;
        } else if let Some(rest) = trimmed.strip_prefix("id ") {
            let id = rest.trim();
            if id.is_empty() {
                println!("Informe um ID após 'id'.");
                continue;
            }
            println!("Executando cápsula com ID {id}...");
            match load_manifest_from_registry(registry_path, id)
                .and_then(|manifest| runtime::run_capsule(&manifest))
            {
                Ok(_) => println!("Execução concluída.\n"),
                Err(err) => println!("Falha ao executar cápsula: {err}\n"),
            }
            continue;
        }

        if trimmed.eq_ignore_ascii_case("json") {
            if visible_entries.is_empty() {
                println!("Nenhuma cápsula para mostrar.\n");
            } else {
                let as_json = serde_json::to_string_pretty(&visible_entries)?;
                println!("{as_json}\n");
            }
            continue;
        }

        let idx: usize = match trimmed.parse() {
            Ok(n) => n,
            Err(_) => {
                println!("Entrada inválida: {trimmed}");
                continue;
            }
        };

        if idx == 0 || idx > visible_entries.len() {
            println!("Índice fora do intervalo.");
            continue;
        }

        let selected = visible_entries[idx - 1];
        println!("Executando cápsula {} ({})...", selected.name, selected.id);

        match load_manifest_from_registry(registry_path, &selected.id)
            .and_then(|manifest| runtime::run_capsule(&manifest))
        {
            Ok(_) => println!("Execução concluída.\n"),
            Err(err) => println!("Falha ao executar cápsula: {err}\n"),
        }
    }

    Ok(())
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
        Commands::List(args) => {
            let entries = list_registry(&args.registry)?;
            let visible = filter_entries(&entries, &args.filter);
            if args.format == "json" {
                println!("{}", serde_json::to_string_pretty(&visible)?);
            } else {
                println!("Capsules em {}:", args.registry.display());
                for entry in visible {
                    println!("- {} ({}) -> {}", entry.name, entry.id, entry.manifest);
                }
            }
            Ok(())
        }
        Commands::Ui(args) => ui_loop(&args.registry, args.filter),
    }
}
