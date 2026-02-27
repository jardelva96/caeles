mod manifest;
mod runtime;

use crate::manifest::CapsuleManifest;
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const STATE_DIR: &str = ".caeles/state";

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub manifest: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RunRecord {
    run_id: String,
    capsule_id: String,
    capsule_name: String,
    manifest_path: String,
    status: String,
    started_at_unix_ms: u128,
    finished_at_unix_ms: u128,
}

#[derive(Debug, Parser)]
#[command(name = "caeles", about = "CAELES CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run(RunArgs),
    List(ListArgs),
    Build(BuildArgs),
    Ps(PsArgs),
    Inspect(InspectArgs),
    Logs(LogsArgs),
    Rm(RmArgs),
}

#[derive(Debug, Args)]
struct RunArgs {
    #[arg(long, conflicts_with = "capsule_id")]
    manifest: Option<PathBuf>,
    #[arg(long, conflicts_with = "manifest")]
    capsule_id: Option<String>,
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,
}

#[derive(Debug, Args)]
struct ListArgs {
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,
}

#[derive(Debug, Args)]
struct BuildArgs {
    path: PathBuf,
    #[arg(long, default_value_t = false)]
    release: bool,
    #[arg(long, default_value = "wasm32-unknown-unknown")]
    target: String,
}

#[derive(Debug, Args)]
struct PsArgs {
    #[arg(long, default_value_t = 10)]
    limit: usize,
}

#[derive(Debug, Args)]
struct InspectArgs {
    capsule_id: String,
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,
}

#[derive(Debug, Args)]
struct LogsArgs {
    run_id: String,
    #[arg(long)]
    tail: Option<usize>,
}

#[derive(Debug, Args)]
struct RmArgs {
    /// Remove uma execução específica registrada em `caeles ps`.
    run_id: Option<String>,
    /// Remove todo o histórico de execução e logs.
    #[arg(long, default_value_t = false, conflicts_with = "run_id")]
    all: bool,
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis()
}

fn ensure_state_dirs() -> anyhow::Result<PathBuf> {
    let base = PathBuf::from(STATE_DIR);
    fs::create_dir_all(base.join("logs"))?;
    Ok(base)
}

fn runs_file_path(base: &Path) -> PathBuf {
    base.join("runs.jsonl")
}

fn append_run_record(base: &Path, record: &RunRecord) -> anyhow::Result<()> {
    let line = serde_json::to_string(record)?;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(runs_file_path(base))?;
    writeln!(f, "{line}")?;
    Ok(())
}

fn load_run_records(base: &Path) -> anyhow::Result<Vec<RunRecord>> {
    let runs_path = runs_file_path(base);
    if !runs_path.exists() {
        return Ok(vec![]);
    }

    let file = fs::File::open(runs_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: RunRecord = serde_json::from_str(&line)?;
        records.push(record);
    }

    Ok(records)
}

fn persist_run_records(base: &Path, records: &[RunRecord]) -> anyhow::Result<()> {
    let mut text = String::new();
    for r in records {
        text.push_str(&serde_json::to_string(r)?);
        text.push('\n');
    }
    fs::write(runs_file_path(base), text)?;
    Ok(())
}

fn log_file_path(base: &Path, run_id: &str) -> PathBuf {
    base.join("logs").join(format!("{run_id}.log"))
}

fn write_log_line(base: &Path, run_id: &str, message: &str) -> anyhow::Result<()> {
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path(base, run_id))?;
    writeln!(f, "{message}")?;
    Ok(())
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

fn resolve_manifest_by_args(args: &RunArgs) -> anyhow::Result<(CapsuleManifest, PathBuf)> {
    if let Some(path) = &args.manifest {
        return Ok((CapsuleManifest::load(path)?, path.clone()));
    }

    if let Some(id) = &args.capsule_id {
        let entries = load_registry_entries(&args.registry)?;
        let entry = entries.iter().find(|e| e.id == *id).ok_or_else(|| {
            anyhow::anyhow!(format!("Capsule id '{id}' não encontrado no registry"))
        })?;

        let manifest_path = resolve_manifest_path(&args.registry, &entry.manifest);
        if !manifest_path.exists() {
            anyhow::bail!(
                "Manifest da cápsula '{}' não encontrado em '{}'",
                id,
                manifest_path.display()
            );
        }

        return Ok((CapsuleManifest::load(&manifest_path)?, manifest_path));
    }

    anyhow::bail!("Use --manifest <arquivo> ou --capsule-id <id-da-capsula>")
}

fn run_command(args: RunArgs) -> anyhow::Result<()> {
    let state_dir = ensure_state_dirs()?;
    let (manifest, manifest_path) = resolve_manifest_by_args(&args)?;

    let started = now_unix_ms();
    let run_id = format!("run-{started}");

    write_log_line(
        &state_dir,
        &run_id,
        &format!(
            "starting capsule id={} name={} manifest={}",
            manifest.id,
            manifest.name,
            manifest_path.display()
        ),
    )?;

    let result = runtime::run_capsule(&manifest);

    let finished = now_unix_ms();
    let status = if result.is_ok() { "exited" } else { "failed" };

    if let Err(err) = &result {
        write_log_line(&state_dir, &run_id, &format!("runtime_error: {err}"))?;
    } else {
        write_log_line(&state_dir, &run_id, "runtime_exit: success")?;
    }

    append_run_record(
        &state_dir,
        &RunRecord {
            run_id: run_id.clone(),
            capsule_id: manifest.id.clone(),
            capsule_name: manifest.name.clone(),
            manifest_path: manifest_path.display().to_string(),
            status: status.to_string(),
            started_at_unix_ms: started,
            finished_at_unix_ms: finished,
        },
    )?;

    println!("> run id: {run_id}");
    result
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
    let status = cmd.status().map_err(|e| {
        anyhow::anyhow!(
            "Não foi possível executar o comando cargo. Instale Rust/Cargo para usar `caeles build`: {e}"
        )
    })?;
    if !status.success() {
        anyhow::bail!("Falha ao compilar cápsula com cargo build");
    }

    println!("> Build concluído: {}", manifest_path.display());
    Ok(())
}

fn ps_command(args: PsArgs) -> anyhow::Result<()> {
    let state_dir = ensure_state_dirs()?;
    let mut runs = load_run_records(&state_dir)?;

    if runs.is_empty() {
        println!("Nenhuma execução registrada ainda.");
        return Ok(());
    }

    runs.sort_by_key(|r| r.started_at_unix_ms);
    runs.reverse();

    println!("RUN ID | CAPSULE | STATUS | STARTED(ms) | DURATION(ms)");
    for record in runs.into_iter().take(args.limit) {
        let duration = record
            .finished_at_unix_ms
            .saturating_sub(record.started_at_unix_ms);
        println!(
            "{} | {} ({}) | {} | {} | {}",
            record.run_id,
            record.capsule_name,
            record.capsule_id,
            record.status,
            record.started_at_unix_ms,
            duration
        );
    }

    Ok(())
}

fn inspect_command(args: InspectArgs) -> anyhow::Result<()> {
    let entries = load_registry_entries(&args.registry)?;
    let entry = entries
        .iter()
        .find(|e| e.id == args.capsule_id)
        .ok_or_else(|| anyhow::anyhow!("Capsule id '{}' não encontrado", args.capsule_id))?;

    let manifest_path = resolve_manifest_path(&args.registry, &entry.manifest);

    println!("id: {}", entry.id);
    println!("name: {}", entry.name);
    println!("registry: {}", args.registry.display());
    println!("manifest: {}", manifest_path.display());
    println!("manifest_exists: {}", manifest_path.exists());

    let state_dir = ensure_state_dirs()?;
    let mut runs: Vec<RunRecord> = load_run_records(&state_dir)?
        .into_iter()
        .filter(|r| r.capsule_id == entry.id)
        .collect();
    runs.sort_by_key(|r| r.started_at_unix_ms);
    runs.reverse();

    if runs.is_empty() {
        println!("last_runs: []");
    } else {
        println!("last_runs:");
        for r in runs.into_iter().take(5) {
            println!(
                "- run_id={} status={} started_ms={} finished_ms={} manifest={}",
                r.run_id, r.status, r.started_at_unix_ms, r.finished_at_unix_ms, r.manifest_path
            );
        }
    }

    Ok(())
}

fn logs_command(args: LogsArgs) -> anyhow::Result<()> {
    let state_dir = ensure_state_dirs()?;
    let path = log_file_path(&state_dir, &args.run_id);
    if !path.exists() {
        anyhow::bail!("Logs da execução '{}' não encontrados", args.run_id);
    }

    let text = fs::read_to_string(path)?;
    let mut lines: Vec<&str> = text.lines().collect();
    if let Some(tail) = args.tail {
        if tail < lines.len() {
            lines = lines.split_off(lines.len() - tail);
        }
    }

    for line in lines {
        println!("{line}");
    }

    Ok(())
}

fn rm_command(args: RmArgs) -> anyhow::Result<()> {
    let state_dir = ensure_state_dirs()?;

    if args.all {
        let runs_path = runs_file_path(&state_dir);
        if runs_path.exists() {
            fs::remove_file(&runs_path)?;
        }
        let logs_dir = state_dir.join("logs");
        if logs_dir.exists() {
            fs::remove_dir_all(&logs_dir)?;
        }
        fs::create_dir_all(state_dir.join("logs"))?;
        println!("Histórico e logs removidos.");
        return Ok(());
    }

    let run_id = args
        .run_id
        .ok_or_else(|| anyhow::anyhow!("Informe <run_id> ou use --all"))?;

    let mut runs = load_run_records(&state_dir)?;
    let before = runs.len();
    runs.retain(|r| r.run_id != run_id);
    if runs.len() == before {
        anyhow::bail!("Run id '{}' não encontrado", run_id);
    }
    persist_run_records(&state_dir, &runs)?;

    let log_path = log_file_path(&state_dir, &run_id);
    if log_path.exists() {
        fs::remove_file(log_path)?;
    }

    println!("Run '{}' removido.", run_id);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => run_command(args),
        Commands::List(args) => list_command(args),
        Commands::Build(args) => build_command(args),
        Commands::Ps(args) => ps_command(args),
        Commands::Inspect(args) => inspect_command(args),
        Commands::Logs(args) => logs_command(args),
        Commands::Rm(args) => rm_command(args),
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
    fn parse_ps_subcommand() {
        let cli = Cli::try_parse_from(["caeles", "ps", "--limit", "3"]).expect("ps should parse");
        assert!(matches!(cli.command, Commands::Ps(_)));
    }

    #[test]
    fn parse_rm_all_subcommand() {
        let cli = Cli::try_parse_from(["caeles", "rm", "--all"]).expect("rm should parse");
        assert!(matches!(cli.command, Commands::Rm(_)));
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
