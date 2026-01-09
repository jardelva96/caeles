mod manifest;
mod runtime;
mod backend;
mod build;

use crate::manifest::CapsuleManifest;
use clap::{Args as ClapArgs, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write as IoWrite};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use once_cell::sync::OnceCell;

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub manifest: String,
}

#[derive(Debug, Parser)]
#[command(
    name = "caeles-runtime",
    about = "CAELES runtime: gerenciamento completo de cápsulas WebAssembly"
)]
struct Args {
    /// Comando a executar
    #[command(subcommand)]
    command: Option<Command>,

    /// Caminho para o arquivo de manifest JSON da cápsula (modo compatibilidade)
    #[arg(long)]
    manifest: Option<PathBuf>,

    /// ID da cápsula (procurado no registry JSON) (modo compatibilidade)
    #[arg(long)]
    capsule_id: Option<String>,

    /// Caminho para o arquivo de registry de cápsulas (modo compatibilidade)
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,

    /// Lista as cápsulas cadastradas no registry e sai (modo compatibilidade)
    #[arg(long)]
    list_capsules: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Compila a cápsula para WebAssembly
    Build(BuildArgs),

    /// Instala uma cápsula no sistema
    Install(InstallArgs),

    /// Lista cápsulas instaladas
    List(ListArgs),

    /// Remove uma cápsula instalada
    Remove(RemoveArgs),

    /// Inicia uma cápsula em background
    Start(StartArgs),

    /// Para uma cápsula rodando
    Stop(StopArgs),

    /// Mostra status de cápsulas
    Status(StatusArgs),

    /// Exibe logs de uma cápsula
    Logs(LogsArgs),

    /// Mostra informações detalhadas de uma cápsula
    Info(InfoArgs),

    /// Inspeciona profundamente uma cápsula
    Inspect(InspectArgs),

    /// Interface inicial para criar um manifest de cápsula
    Init(InitArgs),

    /// Interface web para criar manifestos pelo navegador
    Web(WebArgs),
}

#[derive(Debug, ClapArgs)]
struct InitArgs {
    /// Caminho de saída para o arquivo de manifest gerado
    #[arg(long, default_value = "capsule.manifest.json")]
    output: PathBuf,

    /// ID da cápsula (ex.: com.caeles.examples.mycapsule)
    #[arg(long)]
    id: Option<String>,

    /// Nome da cápsula
    #[arg(long)]
    name: Option<String>,

    /// Versão
    #[arg(long, default_value = "0.1.0")]
    version: String,

    /// Caminho do wasm
    #[arg(long, default_value = "capsule.wasm")]
    entry: String,

    /// Permitir notificações
    #[arg(long)]
    allow_notifications: bool,

    /// Permitir rede
    #[arg(long)]
    allow_network: bool,
}

#[derive(Debug, ClapArgs)]
struct WebArgs {
    /// Host de binding do servidor HTTP
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Porta de binding do servidor HTTP
    #[arg(long, default_value_t = 8080)]
    port: u16,
}

#[derive(Debug, ClapArgs)]
struct BuildArgs {
    /// Diretório do projeto (padrão: diretório atual)
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Compilar em modo release (otimizado)
    #[arg(long, short = 'r')]
    release: bool,

    /// Diretório de output para artefatos (opcional)
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    /// Não gerar/atualizar manifest automaticamente
    #[arg(long)]
    no_manifest: bool,

    /// Não calcular hash SHA-256 do WASM
    #[arg(long)]
    no_hash: bool,
}

#[derive(Debug, ClapArgs)]
struct InstallArgs {
    /// Caminho para o manifest da cápsula (opcional se --path especificado)
    #[arg(long)]
    manifest: Option<PathBuf>,

    /// Caminho do projeto (usa manifest gerado pelo build)
    #[arg(long)]
    path: Option<PathBuf>,

    /// Forçar reinstalação se já instalada
    #[arg(long, short = 'f')]
    force: bool,
}

#[derive(Debug, ClapArgs)]
struct ListArgs {
    /// Mostrar detalhes completos de cada cápsula
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Formato de saída (table, json)
    #[arg(long, default_value = "table")]
    format: String,
}

#[derive(Debug, ClapArgs)]
struct RemoveArgs {
    /// ID da cápsula a remover
    capsule_id: String,

    /// Não pedir confirmação
    #[arg(long, short = 'y')]
    yes: bool,
}

#[derive(Debug, ClapArgs)]
struct StartArgs {
    /// ID da cápsula a iniciar
    capsule_id: String,
}

#[derive(Debug, ClapArgs)]
struct StopArgs {
    /// ID da cápsula a parar
    capsule_id: String,

    /// Forçar parada (kill)
    #[arg(long, short = 'f')]
    force: bool,
}

#[derive(Debug, ClapArgs)]
struct StatusArgs {
    /// Mostrar apenas cápsulas rodando
    #[arg(long)]
    running: bool,

    /// Formato de saída (table, json)
    #[arg(long, default_value = "table")]
    format: String,
}

#[derive(Debug, ClapArgs)]
struct LogsArgs {
    /// ID da cápsula
    capsule_id: String,

    /// Número de linhas a exibir (padrão: todas)
    #[arg(long, short = 'n')]
    lines: Option<usize>,

    /// Seguir logs em tempo real (streaming)
    #[arg(long, short = 'f')]
    follow: bool,

    /// Mostrar apenas logs desde timestamp (Unix epoch)
    #[arg(long)]
    since: Option<u64>,

    /// Mostrar logs de erro ao invés de stdout
    #[arg(long)]
    errors: bool,

    /// Limpar todos os logs da cápsula
    #[arg(long)]
    clear: bool,
}

#[derive(Debug, ClapArgs)]
struct InfoArgs {
    /// ID da cápsula
    capsule_id: String,

    /// Formato de saída (table, json)
    #[arg(long, default_value = "table")]
    format: String,
}

#[derive(Debug, ClapArgs)]
struct InspectArgs {
    /// ID da cápsula
    capsule_id: String,

    /// Formato de saída (json, yaml)
    #[arg(long, default_value = "json")]
    format: String,

    /// Comparar com outra cápsula
    #[arg(long)]
    compare: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CapsuleState {
    Stopped,
    Running,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManagedCapsule {
    manifest: CapsuleManifest,
    state: CapsuleState,
}

static REGISTRY: OnceCell<Mutex<Vec<ManagedCapsule>>> = OnceCell::new();

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

fn prompt_with_default(label: &str, default: Option<&str>) -> io::Result<String> {
    if let Some(d) = default {
        print!("{label} [{d}]: ");
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

fn run_build(args: BuildArgs) -> anyhow::Result<()> {
    use crate::build::{BuildConfig, BuildSystem, ProjectDetector};

    println!("🚀 CAELES Build System\n");

    // Verificar se o target wasm32-unknown-unknown está instalado
    ProjectDetector::check_wasm_target_installed()?;

    // Configurar build
    let config = BuildConfig {
        project_root: args.path.clone(),
        release: args.release,
        output_dir: args.output.clone(),
        generate_manifest: !args.no_manifest,
        compute_hash: !args.no_hash,
    };

    // Criar sistema de build
    let build_system = BuildSystem::new(config)?;

    // Executar build
    let artifacts = build_system.build()?;

    // Exibir resumo
    artifacts.print_summary();

    println!("\n✅ Build concluído com sucesso!");

    // Dica de próximos passos
    if let Some(manifest_path) = &artifacts.manifest_path {
        println!("\n💡 Próximos passos:");
        println!("   1. Executar: caeles-runtime --manifest {}", manifest_path.display());
        println!("   2. Ou instalar no registry para execução rápida");
    }

    Ok(())
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

    manifest.save(&args.output)?;
    println!("\n✅ Manifest salvo em: {}", args.output.display());

    Ok(())
}

fn run_install(args: InstallArgs) -> anyhow::Result<()> {
    use crate::backend::storage::CapsuleStorage;

    println!("📦 CAELES Install\n");

    let storage = CapsuleStorage::new()?;

    // Determinar manifest a usar
    let manifest_path = if let Some(manifest) = args.manifest {
        manifest
    } else if let Some(path) = args.path {
        path.join("capsule.manifest.json")
    } else {
        PathBuf::from("capsule.manifest.json")
    };

    // Carregar manifest
    let manifest = CapsuleManifest::load(&manifest_path)
        .context(format!("Falha ao carregar manifest: {}", manifest_path.display()))?;

    println!("📄 Manifest: {}", manifest.name);
    println!("🆔 ID: {}", manifest.id);
    println!("📌 Versão: {}", manifest.version);

    // Verificar se já está instalada
    if storage.is_installed(&manifest.id) && !args.force {
        anyhow::bail!(
            "Cápsula '{}' já está instalada!\n\
             Use --force para forçar reinstalação ou 'caeles remove {}' primeiro.",
            manifest.id,
            manifest.id
        );
    }

    // Remover se force e já instalada
    if storage.is_installed(&manifest.id) && args.force {
        println!("\n🗑️  Removendo instalação anterior...");
        storage.remove_capsule(&manifest.id)?;
    }

    // Obter caminho do WASM
    let wasm_path = manifest.wasm_path();

    if !wasm_path.exists() {
        anyhow::bail!(
            "Arquivo WASM não encontrado: {}\n\
             Execute 'caeles build' primeiro para compilar a cápsula.",
            wasm_path.display()
        );
    }

    // Instalar
    println!("\n📥 Instalando cápsula...");
    storage.install_capsule(&manifest.id, &wasm_path, &manifest_path)?;

    println!("✅ Cápsula '{}' instalada com sucesso!", manifest.id);
    println!("\n💡 Próximos passos:");
    println!("   caeles list              # Ver cápsulas instaladas");
    println!("   caeles start {}  # Iniciar cápsula", manifest.id);

    Ok(())
}

fn run_list(args: ListArgs) -> anyhow::Result<()> {
    use crate::backend::storage::CapsuleStorage;

    let storage = CapsuleStorage::new()?;
    let installed = storage.list_installed()?;

    if installed.is_empty() {
        println!("📦 Nenhuma cápsula instalada.");
        println!("\n💡 Instale uma cápsula com:");
        println!("   caeles build && caeles install");
        return Ok(());
    }

    if args.format == "json" {
        // Output JSON
        let mut capsules_json = Vec::new();
        for id in &installed {
            let manifest_path = storage.get_manifest_path(id)?;
            let manifest = CapsuleManifest::load(&manifest_path)?;
            let metadata = storage.get_metadata(id)?;

            let capsule_info = serde_json::json!({
                "id": manifest.id,
                "name": manifest.name,
                "version": manifest.version,
                "installed_at": metadata.installed_at,
                "run_count": metadata.run_count,
            });
            capsules_json.push(capsule_info);
        }

        println!("{}", serde_json::to_string_pretty(&capsules_json)?);
        return Ok(());
    }

    // Output tabular
    println!("📦 Cápsulas Instaladas ({}):\n", installed.len());

    if args.verbose {
        // Modo verbose
        for id in &installed {
            let manifest_path = storage.get_manifest_path(id)?;
            let manifest = CapsuleManifest::load(&manifest_path)?;
            let metadata = storage.get_metadata(id)?;
            let wasm_path = storage.get_wasm_path(id)?;
            let wasm_size = fs::metadata(&wasm_path)?.len();

            println!("─────────────────────────────────────");
            println!("ID:       {}", manifest.id);
            println!("Nome:     {}", manifest.name);
            println!("Versão:   {}", manifest.version);
            println!("WASM:     {} KB", wasm_size / 1024);
            println!("Execuções: {}", metadata.run_count);
            println!("Instalado: {}", format_timestamp(metadata.installed_at));
        }
        println!("─────────────────────────────────────");
    } else {
        // Modo compacto
        println!("{:<40} {:<25} {:<10}", "ID", "NOME", "VERSÃO");
        println!("{}", "─".repeat(77));

        for id in &installed {
            let manifest_path = storage.get_manifest_path(id)?;
            let manifest = CapsuleManifest::load(&manifest_path)?;

            println!(
                "{:<40} {:<25} {:<10}",
                truncate(&manifest.id, 40),
                truncate(&manifest.name, 25),
                manifest.version
            );
        }
    }

    // Estatísticas
    let stats = storage.stats()?;
    println!("\n📊 Storage: {} instaladas, {:.2} MB em {}",
        stats.total_capsules,
        stats.total_size_mb(),
        stats.storage_path.display()
    );

    Ok(())
}

fn run_remove(args: RemoveArgs) -> anyhow::Result<()> {
    use crate::backend::storage::CapsuleStorage;

    let storage = CapsuleStorage::new()?;

    // Verificar se está instalada
    if !storage.is_installed(&args.capsule_id) {
        anyhow::bail!("Cápsula '{}' não está instalada", args.capsule_id);
    }

    // Carregar info para exibir
    let manifest_path = storage.get_manifest_path(&args.capsule_id)?;
    let manifest = CapsuleManifest::load(&manifest_path)?;

    println!("🗑️  Remover cápsula:");
    println!("   ID:      {}", manifest.id);
    println!("   Nome:    {}", manifest.name);
    println!("   Versão:  {}", manifest.version);

    // Confirmação
    if !args.yes {
        print!("\nTem certeza? (s/N): ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().to_lowercase().starts_with('s') {
            println!("❌ Cancelado");
            return Ok(());
        }
    }

    // Remover
    println!("\n🗑️  Removendo...");
    storage.remove_capsule(&args.capsule_id)?;

    println!("✅ Cápsula '{}' removida com sucesso!", args.capsule_id);

    Ok(())
}

fn run_start(args: StartArgs) -> anyhow::Result<()> {
    use crate::backend::{lifecycle::InstanceManager, logs::LogManager, storage::CapsuleStorage, runner};

    println!("🚀 CAELES Start\n");

    let storage = CapsuleStorage::new()?;
    let state_dir = storage.root().join("state");
    let manager = InstanceManager::new(state_dir)?;
    let log_manager = LogManager::new(storage.root().to_path_buf())?;

    // Verificar se cápsula está instalada
    if !storage.is_installed(&args.capsule_id) {
        anyhow::bail!(
            "Cápsula '{}' não está instalada.\n\
             Instale com: caeles install",
            args.capsule_id
        );
    }

    // Verificar se já está rodando
    if manager.is_running(&args.capsule_id) {
        anyhow::bail!("Cápsula '{}' já está rodando!", args.capsule_id);
    }

    // Carregar manifest
    let manifest_path = storage.get_manifest_path(&args.capsule_id)?;
    let manifest = CapsuleManifest::load(&manifest_path)?;

    println!("📄 Cápsula: {}", manifest.name);
    println!("🆔 ID: {}", manifest.id);

    // Registrar instância
    manager.register(args.capsule_id.clone())?;

    // Iniciar processo em background
    println!("\n🔄 Iniciando cápsula em background...");
    let process = runner::start_capsule_background(&args.capsule_id, &manifest_path)?;
    let pid = process.child.id();

    // Configurar captura de logs
    let capsule_id_clone_stdout = args.capsule_id.clone();
    let capsule_id_clone_stderr = args.capsule_id.clone();

    let log_manager_clone_stdout = LogManager::new(storage.root().to_path_buf())?;
    let log_manager_clone_stderr = LogManager::new(storage.root().to_path_buf())?;

    let process_with_logs = runner::start_log_capture(
        process,
        move |line| {
            let _ = log_manager_clone_stdout.write_log(&capsule_id_clone_stdout, &line);
        },
        move |line| {
            let _ = log_manager_clone_stderr.write_error_log(&capsule_id_clone_stderr, &line);
        },
    );

    // Marcar como iniciada
    manager.mark_started(&args.capsule_id, pid)?;

    // Atualizar metadata de execução
    let mut metadata = storage.get_metadata(&args.capsule_id)?;
    metadata.mark_run();

    println!("✅ Cápsula iniciada com PID {}", pid);
    println!("\n💡 Próximos passos:");
    println!("   caeles status         # Ver status");
    println!("   caeles logs {}   # Ver logs", args.capsule_id);
    println!("   caeles stop {}  # Parar cápsula", args.capsule_id);

    Ok(())
}

fn run_stop(args: StopArgs) -> anyhow::Result<()> {
    use crate::backend::{lifecycle::InstanceManager, storage::CapsuleStorage};

    println!("🛑 CAELES Stop\n");

    let storage = CapsuleStorage::new()?;
    let state_dir = storage.root().join("state");
    let manager = InstanceManager::new(state_dir)?;

    // Verificar se está rodando
    if !manager.is_running(&args.capsule_id) {
        anyhow::bail!("Cápsula '{}' não está rodando", args.capsule_id);
    }

    let info = manager.get(&args.capsule_id)
        .ok_or_else(|| anyhow::anyhow!("Instância não encontrada"))?;

    println!("📄 Cápsula: {}", args.capsule_id);
    println!("🆔 PID: {}", info.pid.unwrap_or(0));
    println!("⏱️  Uptime: {}", info.uptime_human());

    println!("\n🛑 Parando cápsula...");

    // TODO: Implementar stop real do processo
    // Por enquanto, apenas marcar como parada
    manager.mark_stopped(&args.capsule_id)?;

    println!("✅ Cápsula '{}' parada com sucesso!", args.capsule_id);

    Ok(())
}

fn run_status(args: StatusArgs) -> anyhow::Result<()> {
    use crate::backend::{lifecycle::InstanceManager, storage::CapsuleStorage};

    let storage = CapsuleStorage::new()?;
    let state_dir = storage.root().join("state");
    let manager = InstanceManager::new(state_dir)?;

    let instances = if args.running {
        manager.list_running()
    } else {
        manager.list()
    };

    if instances.is_empty() {
        if args.running {
            println!("🔍 Nenhuma cápsula rodando.");
        } else {
            println!("🔍 Nenhuma instância registrada.");
        }
        println!("\n💡 Inicie uma cápsula com:");
        println!("   caeles start <capsule-id>");
        return Ok(());
    }

    if args.format == "json" {
        let json = serde_json::to_string_pretty(&instances)?;
        println!("{}", json);
        return Ok(());
    }

    // Output tabular
    let title = if args.running {
        format!("🚀 Cápsulas Rodando ({})", instances.len())
    } else {
        format!("📊 Status de Cápsulas ({})", instances.len())
    };

    println!("{}\n", title);
    println!("{:<40} {:<12} {:<10} {:<15}", "ID", "STATUS", "PID", "UPTIME");
    println!("{}", "─".repeat(80));

    for info in &instances {
        let pid_str = info.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string());
        let uptime = if info.status.to_string() == "running" {
            info.uptime_human()
        } else {
            "-".to_string()
        };

        // Colorir status (se terminal suportar)
        let status_display = match info.status.to_string().as_str() {
            "running" => format!("\x1b[32m{}\x1b[0m", info.status),  // Verde
            "failed" => format!("\x1b[31m{}\x1b[0m", info.status),   // Vermelho
            _ => info.status.to_string(),
        };

        println!(
            "{:<40} {:<12} {:<10} {:<15}",
            truncate(&info.capsule_id, 40),
            status_display,
            pid_str,
            uptime
        );
    }

    println!("\n💡 Comandos:");
    println!("   caeles stop <id>      # Parar cápsula");
    println!("   caeles status --running  # Apenas rodando");

    Ok(())
}

fn run_logs(args: LogsArgs) -> anyhow::Result<()> {
    use crate::backend::{logs::LogManager, storage::CapsuleStorage};

    let storage = CapsuleStorage::new()?;
    let log_manager = LogManager::new(storage.root().to_path_buf())?;

    // Verificar se cápsula está instalada
    if !storage.is_installed(&args.capsule_id) {
        anyhow::bail!("Cápsula '{}' não está instalada", args.capsule_id);
    }

    // Limpar logs se solicitado
    if args.clear {
        println!("🗑️  Limpando logs de '{}'...", args.capsule_id);
        log_manager.clear_all_logs(&args.capsule_id)?;
        println!("✅ Logs removidos com sucesso!");
        return Ok(());
    }

    // Ler logs
    let logs = if args.errors {
        log_manager.read_error_logs(&args.capsule_id, args.lines)?
    } else {
        log_manager.read_logs(&args.capsule_id, args.lines, args.follow, args.since)?
    };

    if logs.is_empty() {
        println!("📝 Nenhum log disponível para '{}'", args.capsule_id);
        println!("\n💡 Dica: Logs são capturados quando a cápsula é iniciada com:");
        println!("   caeles start {}", args.capsule_id);
        return Ok(());
    }

    // Exibir logs
    let log_type = if args.errors { "STDERR" } else { "STDOUT" };
    println!("📝 Logs de '{}' ({})\n", args.capsule_id, log_type);

    for line in &logs {
        println!("{}", line);
    }

    // Exibir estatísticas
    let stats = log_manager.get_stats(&args.capsule_id)?;
    println!("\n📊 Estatísticas:");
    println!("   Arquivos:     {}", stats.total_files);
    println!("   Tamanho:      {:.2} MB", stats.total_size_mb());
    println!("   Linhas atual: {}", stats.current_lines);

    // Follow mode (streaming)
    if args.follow {
        println!("\n🔄 Seguindo logs... (Ctrl+C para sair)\n");

        // TODO: Implementar streaming real de logs
        // Por enquanto, apenas mostrar mensagem
        println!("⚠️  Modo follow ainda não implementado completamente");
        println!("💡 Use 'caeles logs {}' para ver logs mais recentes", args.capsule_id);
    }

    Ok(())
}

fn run_info(args: InfoArgs) -> anyhow::Result<()> {
    use crate::backend::inspector::{CapsuleInspector, format_timestamp};

    let inspector = CapsuleInspector::new()?;
    let info = inspector.inspect(&args.capsule_id)?;

    if args.format == "json" {
        println!("{}", serde_json::to_string_pretty(&info)?);
        return Ok(());
    }

    // Output formatado
    println!("\n📦 Informações da Cápsula\n");
    println!("═══════════════════════════════════════════════════════════════");

    // Informações básicas
    println!("\n🏷️  BÁSICO");
    println!("─────────────────────────────────────────────────────────────");
    println!("ID:           {}", info.manifest.id);
    println!("Nome:         {}", info.manifest.name);
    println!("Versão:       {}", info.manifest.version);
    println!("Estado:       {}", info.current_state.status);

    if let Some(pid) = info.current_state.pid {
        println!("PID:          {}", pid);
    }

    if let Some(uptime) = info.current_state.uptime_secs {
        let minutes = uptime / 60;
        let hours = minutes / 60;
        if hours > 0 {
            println!("Uptime:       {}h {}m", hours, minutes % 60);
        } else if minutes > 0 {
            println!("Uptime:       {}m {}s", minutes, uptime % 60);
        } else {
            println!("Uptime:       {}s", uptime);
        }
    }

    // Instalação
    println!("\n📥 INSTALAÇÃO");
    println!("─────────────────────────────────────────────────────────────");
    println!("Instalado:    {}", format_timestamp(info.installation.installed_at));
    println!("Reinstalações: {}", info.installation.install_count);
    println!("WASM:         {:.2} MB", info.resources.wasm_size_mb);
    println!("Caminho:      {}", info.installation.wasm_path.display());

    // Execução
    println!("\n▶️  EXECUÇÃO");
    println!("─────────────────────────────────────────────────────────────");
    println!("Total runs:   {}", info.execution_history.total_runs);

    if let Some(last_run) = info.execution_history.last_run {
        println!("Última exec:  {}", format_timestamp(last_run));
    } else {
        println!("Última exec:  Nunca executada");
    }

    // Logs
    println!("\n📝 LOGS");
    println!("─────────────────────────────────────────────────────────────");
    println!("Arquivos:     {}", info.logs.total_log_files);
    println!("Tamanho:      {:.2} MB", info.logs.total_log_size_mb);
    println!("Linhas (out): {}", info.logs.current_log_lines);
    println!("Linhas (err): {}", info.logs.error_log_lines);

    // Recursos
    println!("\n💾 RECURSOS");
    println!("─────────────────────────────────────────────────────────────");
    println!("Disco total:  {:.2} MB", info.resources.total_disk_usage_mb);
    println!("  • WASM:     {:.2} MB", info.resources.wasm_size_mb);
    println!("  • Logs:     {:.2} MB", info.resources.logs_size_mb);
    println!("  • State:    {:.2} MB", info.resources.state_size_mb);
    println!("Mem estimada: {:.2} MB", info.resources.estimated_memory_mb);

    // Permissões
    println!("\n🔒 PERMISSÕES");
    println!("─────────────────────────────────────────────────────────────");
    println!("Notificações: {}", if info.manifest.permissions.notifications { "✓" } else { "✗" });
    println!("Rede:         {}", if info.manifest.permissions.network { "✓" } else { "✗" });

    println!("\n═══════════════════════════════════════════════════════════════\n");

    Ok(())
}

fn run_inspect(args: InspectArgs) -> anyhow::Result<()> {
    use crate::backend::inspector::CapsuleInspector;

    let inspector = CapsuleInspector::new()?;

    // Modo comparação
    if let Some(compare_id) = args.compare {
        println!("🔍 Comparando cápsulas...\n");

        let comparison = inspector.compare(&args.capsule_id, &compare_id)?;

        if args.format == "json" {
            println!("{}", serde_json::to_string_pretty(&comparison)?);
            return Ok(());
        }

        println!("📊 Diferenças encontradas:\n");
        if comparison.differences.is_empty() {
            println!("✓ As cápsulas são idênticas em estrutura básica");
        } else {
            for diff in comparison.differences {
                println!("  • {}", diff);
            }
        }

        println!("\n💡 Use --format json para detalhes completos");
        return Ok(());
    }

    // Inspeção normal
    let info = inspector.inspect(&args.capsule_id)?;

    // Output JSON completo
    println!("{}", serde_json::to_string_pretty(&info)?);

    Ok(())
}

fn format_timestamp(ts: u64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let datetime = UNIX_EPOCH + Duration::from_secs(ts);
    let now = SystemTime::now();

    if let Ok(duration) = now.duration_since(datetime) {
        let secs = duration.as_secs();

        if secs < 60 {
            return format!("{} segundos atrás", secs);
        } else if secs < 3600 {
            return format!("{} minutos atrás", secs / 60);
        } else if secs < 86400 {
            return format!("{} horas atrás", secs / 3600);
        } else {
            return format!("{} dias atrás", secs / 86400);
        }
    }

    "data desconhecida".to_string()
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) -> io::Result<()> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())
}

fn render_form() -> String {
    r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8"/>
  <title>CAELES – Gerenciar Cápsulas (preview)</title>
  <style>
    :root {
      --bg: #0b1021;
      --card: #11162b;
      --accent: #4cc2ff;
      --text: #f4f6fb;
      --muted: #9ba3b5;
      --border: #1f2b4d;
      --input: #0f1428;
      --success: #6be7b5;
    }
    * { box-sizing: border-box; }
    body {
      background: radial-gradient(120% 120% at 10% 20%, #10204d, #0b1021 60%);
      color: var(--text);
      font-family: "Inter", system-ui, sans-serif;
      margin: 0;
      min-height: 100vh;
      padding: 2rem 1rem 3rem;
      display: flex;
      justify-content: center;
    }
    .shell {
      width: min(960px, 100%);
    }
    h1 { margin: 0 0 0.5rem; letter-spacing: -0.02em; }
    p { color: var(--muted); margin: 0.2rem 0 1rem; }
    .card {
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 14px;
      padding: 1.25rem;
      box-shadow: 0 20px 80px rgba(0,0,0,0.45);
    }
    label { display: block; margin-top: 0.9rem; font-weight: 600; font-size: 0.95rem; }
    input[type=text] {
      width: 100%; padding: 0.65rem 0.75rem;
      background: var(--input); color: var(--text);
      border: 1px solid var(--border); border-radius: 10px;
      font-size: 0.95rem;
    }
    input[type=text]:focus { outline: 1px solid var(--accent); border-color: var(--accent); }
    .checkbox { display: flex; align-items: center; gap: 0.5rem; margin-top: 0.7rem; color: var(--muted); }
    .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 1rem; }
    .actions { margin-top: 1.25rem; display: flex; gap: 0.75rem; flex-wrap: wrap; align-items: center; }
    button {
      padding: 0.75rem 1.2rem;
      font-size: 1rem;
      background: linear-gradient(135deg, var(--accent), #7de0ff);
      border: none; color: #04122a; font-weight: 700;
      border-radius: 12px; cursor: pointer;
      box-shadow: 0 12px 30px rgba(76,194,255,0.25);
    }
    button.secondary {
      background: transparent;
      color: var(--text);
      border: 1px solid var(--border);
      box-shadow: none;
    }
    pre {
      background: #0c1329;
      border: 1px solid var(--border);
      padding: 1rem;
      overflow: auto;
      border-radius: 10px;
      color: #e8f0ff;
    }
    .hint { color: var(--muted); font-size: 0.9rem; }
    .badge { display: inline-flex; align-items: center; gap: 0.35rem; padding: 0.35rem 0.65rem; border-radius: 999px; background: #10203d; color: var(--accent); font-weight: 600; font-size: 0.9rem; }
    .row { display: flex; gap: 0.8rem; flex-wrap: wrap; align-items: center; }
    .muted-card { background: #0c1329; border: 1px dashed var(--border); border-radius: 12px; padding: 0.75rem 1rem; color: var(--muted); font-size: 0.95rem; }
    a { color: var(--accent); }
  </style>
</head>
<body>
  <div class="shell">
    <div class="row" style="margin-bottom:0.6rem;">
      <div class="badge">CAELES Runtime · Preview UI</div>
    </div>
    <h1>Gerenciar cápsulas (preview)</h1>
    <p>Crie, inicie, pare ou remova cápsulas. Construir sempre para <code>wasm32-unknown-unknown</code> e aponte o <code>entry</code> para o .wasm gerado.</p>

    <div class="card">
      <form method="POST" action="/generate">
        <div class="grid">
          <div>
            <label>ID da cápsula</label>
            <input type="text" name="id" placeholder="com.caeles.examples.mycapsule" required />
            <div class="hint">Use um namespace reverso (ex.: com.empresa.app). </div>
          </div>
          <div>
            <label>Nome</label>
            <input type="text" name="name" placeholder="Minha Cápsula CAELES" required />
            <div class="hint">Nome amigável exibido para o usuário.</div>
          </div>
        </div>

        <div class="grid" style="margin-top:0.4rem;">
          <div>
            <label>Versão</label>
            <input type="text" name="version" value="0.1.0" required />
            <div class="hint">Versão semântica (ex.: 0.1.0).</div>
          </div>
          <div>
            <label>Caminho do wasm (relativo ao manifest)</label>
            <input type="text" name="entry" value="capsule.wasm" required />
            <div class="hint">Aponte para o .wasm gerado (ex.: target/wasm32-unknown-unknown/debug/minha.wasm).</div>
          </div>
        </div>

        <div class="row" style="margin-top:0.8rem;">
          <label class="checkbox">
            <input type="checkbox" name="notifications" />
            Permitir notificações
          </label>
          <label class="checkbox">
            <input type="checkbox" name="network" />
            Permitir rede
          </label>
        </div>

        <div class="actions">
          <button type="submit">Gerar manifest</button>
          <div class="muted-card">Dica: compile a cápsula com <code>cargo build --target wasm32-unknown-unknown</code> antes de executar no runtime.</div>
        </div>
      </form>
    </div>

    <div class="card" style="margin-top:1rem;">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <div>
          <h2 style="margin:0;">Cápsulas cadastradas (sessão atual)</h2>
          <p class="muted" style="margin:0.1rem 0 0;">Lista mantida apenas em memória enquanto o runtime estiver rodando.</p>
        </div>
        <button class="secondary" type="button" onclick="loadCapsules()">Atualizar</button>
      </div>
      <div id="capsule-table" style="margin-top:1rem;" class="muted-card">Carregando...</div>
    </div>
  </div>
<script>
async function loadCapsules() {
  const table = document.getElementById('capsule-table');
  table.textContent = 'Carregando...';
  try {
    const res = await fetch('/api/manifests');
    if (!res.ok) throw new Error('Falha ao carregar');
    const data = await res.json();
    if (data.length === 0) {
      table.textContent = 'Nenhuma cápsula cadastrada.';
      return;
    }
    const rows = data.map(c => `
      <div style="display:grid;grid-template-columns:1fr 1fr 1fr 200px;gap:0.5rem;align-items:center;padding:0.65rem 0;border-bottom:1px solid var(--border);">
        <div>
          <div><strong>${c.manifest.name}</strong></div>
          <div class="muted">${c.manifest.id}</div>
        </div>
        <div>Versão: ${c.manifest.version}</div>
        <div>Status: <span style="color:${c.state === 'Running' ? '#6be7b5' : '#9ba3b5'}">${c.state}</span></div>
        <div class="row" style="gap:0.35rem; justify-content:flex-end;">
          <button class="secondary" type="button" onclick="startCapsule('${c.manifest.id}')">Iniciar</button>
          <button class="secondary" type="button" onclick="stopCapsule('${c.manifest.id}')">Parar</button>
          <button class="secondary" type="button" onclick="deleteCapsule('${c.manifest.id}')">Excluir</button>
        </div>
      </div>
    `).join('');
    table.innerHTML = rows;
  } catch (err) {
    table.textContent = 'Erro ao carregar cápsulas.';
  }
}
async function apiPost(url, body) {
  const res = await fetch(url, {method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(body)});
  if (!res.ok) throw new Error('erro');
}
async function startCapsule(id){ try { await apiPost('/api/manifests/start',{id}); loadCapsules(); } catch {} }
async function stopCapsule(id){ try { await apiPost('/api/manifests/stop',{id}); loadCapsules(); } catch {} }
async function deleteCapsule(id){
  try {
    const res = await fetch('/api/manifests?id='+encodeURIComponent(id), {method:'DELETE'});
    if (!res.ok) throw new Error('erro');
    loadCapsules();
  } catch {}
}
window.addEventListener('load', loadCapsules);
</script>
</body>
</html>
"#
    .to_string()
}

fn parse_form(body: &str) -> CapsuleManifest {
    let mut id = String::new();
    let mut name = String::new();
    let mut version = "0.1.0".to_string();
    let mut entry = "capsule.wasm".to_string();
    let mut notifications = false;
    let mut network = false;

    for pair in body.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts
            .next()
            .map(|v| url_decode(v))
            .unwrap_or_default();

        match key {
            "id" => id = value,
            "name" => name = value,
            "version" => version = value,
            "entry" => entry = value,
            "notifications" => notifications = true,
            "network" => network = true,
            _ => {}
        }
    }

    CapsuleManifest::from_parts(
        id,
        name,
        version,
        entry,
        crate::manifest::Permissions {
            notifications,
            network,
        },
    )
}

fn url_decode(input: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = input.as_bytes().iter().copied().peekable();

    while let Some(b) = chars.next() {
        match b {
            b'+' => bytes.push(b' '),
            b'%' => {
                let hi = chars.next();
                let lo = chars.next();
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    if let (Some(hi_v), Some(lo_v)) = (from_hex(hi), from_hex(lo)) {
                        bytes.push((hi_v << 4) | lo_v);
                        continue;
                    }
                }
                bytes.push(b'%');
            }
            _ => bytes.push(b),
        }
    }

    String::from_utf8_lossy(&bytes).to_string()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn render_manifest_result(manifest: &CapsuleManifest) -> anyhow::Result<String> {
    let json = serde_json::to_string_pretty(manifest)?;
    let json_escaped = html_escape(&json);
    let suggested_file = "capsule.manifest.json";

    let cli_example = html_escape(&format!(
        "cargo run -p caeles-runtime -- --manifest {}",
        suggested_file
    ));

    let html = format!(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8"/>
  <title>Manifesto gerado</title>
  <style>
    body {{ font-family: "Inter", system-ui, sans-serif; margin: 0; padding: 2rem 1rem; display: flex; justify-content: center; background: #0b1021; color: #f4f6fb; }}
    .card {{ background: #11162b; border: 1px solid #1f2b4d; border-radius: 14px; padding: 1.5rem; width: min(880px, 100%); box-shadow: 0 20px 80px rgba(0,0,0,0.45); }}
    h1 {{ margin-top: 0; letter-spacing: -0.02em; }}
    pre {{ background: #0c1329; padding: 1rem; overflow: auto; border-radius: 10px; border: 1px solid #1f2b4d; color: #e8f0ff; }}
    .button {{ display: inline-block; margin-top: 1rem; padding: 0.6rem 1rem; background: linear-gradient(135deg, #4cc2ff, #7de0ff); color: #04122a; text-decoration: none; border-radius: 10px; font-weight: 700; }}
    .row {{ display: flex; gap: 0.6rem; flex-wrap: wrap; align-items: center; }}
    .muted {{ color: #9ba3b5; }}
    code {{ background: #0c1329; padding: 0.2rem 0.4rem; border-radius: 6px; }}
  </style>
</head>
<body>
  <div class="card">
    <h1>Manifesto gerado</h1>
    <p class="muted">Salve como <code>{suggested_file}</code> e rode o runtime apontando para ele.</p>
    <div class="row">
      <a class="button" href="/" aria-label="Voltar ao formulário">Voltar</a>
      <a class="button" href="data:application/json;charset=utf-8,{json_escaped}" download="{suggested_file}" aria-label="Baixar manifest JSON">Baixar JSON</a>
    </div>
    <div style="margin-top:1rem;">
      <div class="muted">Exemplo de uso:</div>
      <pre>{cli_example}</pre>
    </div>
    <div style="margin-top:1rem;">
      <div class="muted">Conteúdo do manifest:</div>
      <pre>{json_escaped}</pre>
    </div>
  </div>
</body>
</html>"#,
    );
    Ok(html)
}

fn read_http_request(stream: &mut TcpStream) -> io::Result<(String, Vec<u8>)> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut buffer = Vec::new();
    let mut header_end: Option<usize> = None;
    let mut content_length: Option<usize> = None;
    let mut temp = [0u8; 4096];

    while buffer.len() < 64 * 1024 {
        let n = stream.read(&mut temp)?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..n]);

        if header_end.is_none() {
            if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(pos + 4);
                let headers = String::from_utf8_lossy(&buffer[..pos]);
                for line in headers.lines() {
                    let line = line.trim();
                    if let Some(rest) = line.strip_prefix("Content-Length:") {
                        if let Ok(len) = rest.trim().parse::<usize>() {
                            content_length = Some(len);
                        }
                    }
                }

                // Se não houver Content-Length, estamos tratando um GET/HEAD simples:
                // podemos parar de ler após o fim do cabeçalho.
                if content_length.is_none() {
                    break;
                }
            }
        }

        if let (Some(end), Some(len)) = (header_end, content_length) {
            if buffer.len() >= end + len {
                break;
            }
        }
    }

    let request = String::from_utf8_lossy(&buffer).to_string();
    Ok((request, buffer))
}

fn get_registry() -> &'static Mutex<Vec<ManagedCapsule>> {
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

fn parse_query(path: &str) -> (&str, Vec<(String, String)>) {
    if let Some((p, q)) = path.split_once('?') {
        let params = q
            .split('&')
            .filter(|s| !s.is_empty())
            .filter_map(|pair| {
                let mut it = pair.splitn(2, '=');
                let k = it.next()?.to_string();
                let v = it.next().unwrap_or_default().to_string();
                Some((k, v))
            })
            .collect();
        (p, params)
    } else {
        (path, Vec::new())
    }
}

fn respond_json(stream: &mut TcpStream, status: &str, value: &serde_json::Value) -> io::Result<()> {
    let body = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    respond(stream, status, "application/json; charset=utf-8", &body)
}

fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let (request, raw) = read_http_request(&mut stream)?;
    let mut lines = request.split("\r\n");
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let raw_path = parts.next().unwrap_or("/");
    let (path, query) = parse_query(raw_path);

    if method.eq_ignore_ascii_case("GET") && path == "/health" {
        respond(&mut stream, "200 OK", "text/plain; charset=utf-8", "ok")?;
        return Ok(());
    }

    // API: listar
    if method.eq_ignore_ascii_case("GET") && path == "/api/manifests" {
        let registry = get_registry().lock().unwrap();
        let value = serde_json::json!(*registry);
        respond_json(&mut stream, "200 OK", &value)?;
        return Ok(());
    }

    // API: criar
    if method.eq_ignore_ascii_case("POST") && path == "/api/manifests" {
        let header_end = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .map(|p| p + 4)
            .unwrap_or(raw.len());
        let body = &raw[header_end..];
        let manifest: CapsuleManifest = serde_json::from_slice(body)?;
        let mut registry = get_registry().lock().unwrap();
        if registry.iter().any(|c| c.manifest.id == manifest.id) {
            respond(
                &mut stream,
                "409 Conflict",
                "text/plain; charset=utf-8",
                "ID já existe",
            )?;
            return Ok(());
        }
        registry.push(ManagedCapsule {
            manifest,
            state: CapsuleState::Stopped,
        });
        respond(&mut stream, "201 Created", "text/plain; charset=utf-8", "created")?;
        return Ok(());
    }

    // API: start/stop
    if method.eq_ignore_ascii_case("POST") && (path == "/api/manifests/start" || path == "/api/manifests/stop") {
        let header_end = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .map(|p| p + 4)
            .unwrap_or(raw.len());
        let body = &raw[header_end..];
        let payload: serde_json::Value = serde_json::from_slice(body)?;
        let id = payload
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut registry = get_registry().lock().unwrap();
        if let Some(item) = registry.iter_mut().find(|c| c.manifest.id == id) {
            item.state = if path.ends_with("start") {
                CapsuleState::Running
            } else {
                CapsuleState::Stopped
            };
            respond(&mut stream, "200 OK", "text/plain; charset=utf-8", "ok")?;
        } else {
            respond(
                &mut stream,
                "404 Not Found",
                "text/plain; charset=utf-8",
                "ID não encontrado",
            )?;
        }
        return Ok(());
    }

    // API: delete
    if method.eq_ignore_ascii_case("DELETE") && path == "/api/manifests" {
        let id = query
            .iter()
            .find(|(k, _)| k == "id")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let mut registry = get_registry().lock().unwrap();
        let before = registry.len();
        registry.retain(|c| c.manifest.id != id);
        let status = if registry.len() < before { "200 OK" } else { "404 Not Found" };
        respond(&mut stream, status, "text/plain; charset=utf-8", "ok")?;
        return Ok(());
    }

    if method.eq_ignore_ascii_case("POST") && path == "/generate" {
        let header_end = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .map(|p| p + 4)
            .unwrap_or(raw.len());
        let body = &raw[header_end..];
        let body_str = String::from_utf8_lossy(body);
        let manifest = parse_form(&body_str);
        let html = render_manifest_result(&manifest)?;
        respond(&mut stream, "200 OK", "text/html; charset=utf-8", &html)?;
        return Ok(());
    }

    if !method.eq_ignore_ascii_case("GET") || path != "/" {
        respond(
            &mut stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "Rota não encontrada. Use GET / ou POST /generate.",
        )?;
        return Ok(());
    }

    let html = render_form();
    respond(&mut stream, "200 OK", "text/html; charset=utf-8", &html)?;
    Ok(())
}

fn run_web_server(args: WebArgs) -> anyhow::Result<()> {
    let addr = format!("{}:{}", args.host, args.port);
    println!("> Servindo interface web em http://{addr} (Ctrl+C para sair)");
    let listener = TcpListener::bind(&addr)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) = handle_connection(stream) {
                    eprintln!("[web] erro atendendo requisição: {err}");
                }
            }
            Err(err) => {
                eprintln!("[web] erro de conexão: {err}");
            }
        }
    }
    Ok(())
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Modo compatibilidade: --list-capsules
    if args.list_capsules {
        list_capsules(&args.registry)?;
        return Ok(());
    }

    // Novos comandos via subcomando
    if let Some(command) = args.command {
        match command {
            Command::Build(build_args) => return run_build(build_args),
            Command::Install(install_args) => return run_install(install_args),
            Command::List(list_args) => return run_list(list_args),
            Command::Remove(remove_args) => return run_remove(remove_args),
            Command::Start(start_args) => return run_start(start_args),
            Command::Stop(stop_args) => return run_stop(stop_args),
            Command::Status(status_args) => return run_status(status_args),
            Command::Logs(logs_args) => return run_logs(logs_args),
            Command::Init(init_args) => return run_init_wizard(init_args),
            Command::Web(web_args) => return run_web_server(web_args),
        }
    }

    // Modo compatibilidade: execução direta via --manifest ou --capsule-id
    let manifest = if let Some(path) = args.manifest {
        CapsuleManifest::load(&path)?
    } else if let Some(id) = args.capsule_id {
        load_manifest_from_registry(&args.registry, &id)?
    } else {
        anyhow::bail!(
            "Use um dos modos:\n\
             1. Comando: caeles-runtime <build|install|list|remove|start|stop|status>\n\
             2. Modo compatibilidade: --manifest <arquivo> ou --capsule-id <id> ou --list-capsules"
        );
    };

    runtime::run_capsule(&manifest)
}
