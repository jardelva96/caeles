//! Sistema de inspeção e análise de cápsulas

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::backend::{lifecycle::InstanceManager, logs::LogManager, storage::CapsuleStorage};
use crate::manifest::CapsuleManifest;

/// Informações completas de uma cápsula
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleInfo {
    /// Informações básicas do manifest
    pub manifest: CapsuleManifest,

    /// Informações de instalação
    pub installation: InstallationInfo,

    /// Histórico de execuções
    pub execution_history: ExecutionHistory,

    /// Métricas de performance
    pub performance: PerformanceMetrics,

    /// Informações de logs
    pub logs: LogsInfo,

    /// Estado atual
    pub current_state: CurrentState,

    /// Análise de recursos
    pub resources: ResourceAnalysis,
}

/// Informações de instalação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInfo {
    pub installed_at: u64,
    pub install_count: u32,
    pub wasm_size_bytes: u64,
    pub wasm_path: PathBuf,
    pub manifest_path: PathBuf,
    pub checksum: Option<String>,
}

/// Histórico de execuções
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHistory {
    pub total_runs: u32,
    pub successful_runs: u32,
    pub failed_runs: u32,
    pub last_run: Option<u64>,
    pub last_exit_code: Option<i32>,
    pub average_runtime_secs: Option<f64>,
    pub recent_runs: Vec<RunRecord>,
}

/// Registro de uma execução
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub duration_secs: Option<u64>,
    pub exit_code: Option<i32>,
    pub status: String,
}

/// Métricas de performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_cpu_time_secs: Option<f64>,
    pub peak_memory_mb: Option<f64>,
    pub average_memory_mb: Option<f64>,
    pub disk_reads_mb: Option<f64>,
    pub disk_writes_mb: Option<f64>,
    pub network_sent_mb: Option<f64>,
    pub network_received_mb: Option<f64>,
}

/// Informações de logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsInfo {
    pub total_log_files: usize,
    pub total_log_size_mb: f64,
    pub current_log_lines: usize,
    pub error_log_lines: usize,
    pub oldest_log: Option<u64>,
    pub newest_log: Option<u64>,
}

/// Estado atual da cápsula
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentState {
    pub status: String,
    pub pid: Option<u32>,
    pub started_at: Option<u64>,
    pub uptime_secs: Option<u64>,
    pub is_running: bool,
}

/// Análise de recursos utilizados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnalysis {
    pub total_disk_usage_mb: f64,
    pub wasm_size_mb: f64,
    pub logs_size_mb: f64,
    pub state_size_mb: f64,
    pub estimated_memory_mb: f64,
}

/// Inspector de cápsulas
pub struct CapsuleInspector {
    storage: CapsuleStorage,
    log_manager: LogManager,
}

impl CapsuleInspector {
    /// Cria um novo inspector
    pub fn new() -> Result<Self> {
        let storage = CapsuleStorage::new()?;
        let log_manager = LogManager::new(storage.root().to_path_buf())?;

        Ok(Self {
            storage,
            log_manager,
        })
    }

    /// Obtém informações completas de uma cápsula
    pub fn inspect(&self, capsule_id: &str) -> Result<CapsuleInfo> {
        // Verificar se está instalada
        if !self.storage.is_installed(capsule_id) {
            anyhow::bail!("Cápsula '{}' não está instalada", capsule_id);
        }

        // Carregar manifest
        let manifest_path = self.storage.get_manifest_path(capsule_id)?;
        let manifest = CapsuleManifest::load(&manifest_path)?;

        // Obter informações de instalação
        let installation = self.get_installation_info(capsule_id)?;

        // Obter histórico de execuções
        let execution_history = self.get_execution_history(capsule_id)?;

        // Obter métricas de performance
        let performance = self.get_performance_metrics(capsule_id)?;

        // Obter informações de logs
        let logs = self.get_logs_info(capsule_id)?;

        // Obter estado atual
        let current_state = self.get_current_state(capsule_id)?;

        // Analisar recursos
        let resources = self.analyze_resources(capsule_id)?;

        Ok(CapsuleInfo {
            manifest,
            installation,
            execution_history,
            performance,
            logs,
            current_state,
            resources,
        })
    }

    /// Obtém informações de instalação
    fn get_installation_info(&self, capsule_id: &str) -> Result<InstallationInfo> {
        let metadata = self.storage.get_metadata(capsule_id)?;
        let wasm_path = self.storage.get_wasm_path(capsule_id)?;
        let manifest_path = self.storage.get_manifest_path(capsule_id)?;

        let wasm_size = fs::metadata(&wasm_path)?.len();

        // Calcular checksum do WASM (simplificado - usar hash real em produção)
        let checksum = Some(format!("{:x}", wasm_size)); // Placeholder

        Ok(InstallationInfo {
            installed_at: metadata.installed_at,
            install_count: metadata.install_count,
            wasm_size_bytes: wasm_size,
            wasm_path,
            manifest_path,
            checksum,
        })
    }

    /// Obtém histórico de execuções
    fn get_execution_history(&self, capsule_id: &str) -> Result<ExecutionHistory> {
        let metadata = self.storage.get_metadata(capsule_id)?;

        // Carregar histórico (simplificado - expandir com storage real)
        let recent_runs = Vec::new(); // TODO: Implementar storage de histórico

        Ok(ExecutionHistory {
            total_runs: metadata.run_count,
            successful_runs: 0, // TODO: Tracking de sucesso/falha
            failed_runs: 0,
            last_run: metadata.last_run,
            last_exit_code: None,
            average_runtime_secs: None,
            recent_runs,
        })
    }

    /// Obtém métricas de performance
    fn get_performance_metrics(&self, _capsule_id: &str) -> Result<PerformanceMetrics> {
        // TODO: Implementar coleta real de métricas
        Ok(PerformanceMetrics {
            total_cpu_time_secs: None,
            peak_memory_mb: None,
            average_memory_mb: None,
            disk_reads_mb: None,
            disk_writes_mb: None,
            network_sent_mb: None,
            network_received_mb: None,
        })
    }

    /// Obtém informações de logs
    fn get_logs_info(&self, capsule_id: &str) -> Result<LogsInfo> {
        let stats = self.log_manager.get_stats(capsule_id)?;
        let log_files = self.log_manager.list_log_files(capsule_id)?;

        let oldest_log = log_files.last().map(|f| f.modified);
        let newest_log = log_files.first().map(|f| f.modified);

        // Contar linhas de erro
        let error_logs = self.log_manager.read_error_logs(capsule_id, None)?;
        let error_log_lines = error_logs.len();

        Ok(LogsInfo {
            total_log_files: stats.total_files,
            total_log_size_mb: stats.total_size_mb(),
            current_log_lines: stats.current_lines,
            error_log_lines,
            oldest_log,
            newest_log,
        })
    }

    /// Obtém estado atual
    fn get_current_state(&self, capsule_id: &str) -> Result<CurrentState> {
        let state_dir = self.storage.root().join("state");

        if let Ok(manager) = InstanceManager::new(state_dir) {
            if let Some(info) = manager.get(capsule_id) {
                return Ok(CurrentState {
                    status: info.status.to_string(),
                    pid: info.pid,
                    started_at: info.started_at,
                    uptime_secs: info.uptime_secs(),
                    is_running: manager.is_running(capsule_id),
                });
            }
        }

        Ok(CurrentState {
            status: "stopped".to_string(),
            pid: None,
            started_at: None,
            uptime_secs: None,
            is_running: false,
        })
    }

    /// Analisa recursos utilizados
    fn analyze_resources(&self, capsule_id: &str) -> Result<ResourceAnalysis> {
        let wasm_path = self.storage.get_wasm_path(capsule_id)?;
        let wasm_size = fs::metadata(&wasm_path)?.len();

        let logs_stats = self.log_manager.get_stats(capsule_id)?;
        let logs_size = logs_stats.total_size_bytes;

        // Tamanho do state (simplificado)
        let state_size = 0; // TODO: Calcular tamanho real do state

        let total_disk = wasm_size + logs_size + state_size;

        Ok(ResourceAnalysis {
            total_disk_usage_mb: total_disk as f64 / (1024.0 * 1024.0),
            wasm_size_mb: wasm_size as f64 / (1024.0 * 1024.0),
            logs_size_mb: logs_stats.total_size_mb(),
            state_size_mb: state_size as f64 / (1024.0 * 1024.0),
            estimated_memory_mb: wasm_size as f64 / (1024.0 * 1024.0) * 1.5, // Estimativa: 1.5x WASM
        })
    }

    /// Lista informações resumidas de todas as cápsulas
    pub fn list_all(&self) -> Result<Vec<CapsuleSummary>> {
        let installed = self.storage.list_installed()?;
        let mut summaries = Vec::new();

        for capsule_id in installed {
            if let Ok(summary) = self.get_summary(&capsule_id) {
                summaries.push(summary);
            }
        }

        Ok(summaries)
    }

    /// Obtém resumo de uma cápsula
    pub fn get_summary(&self, capsule_id: &str) -> Result<CapsuleSummary> {
        let manifest_path = self.storage.get_manifest_path(capsule_id)?;
        let manifest = CapsuleManifest::load(&manifest_path)?;
        let metadata = self.storage.get_metadata(capsule_id)?;

        let state_dir = self.storage.root().join("state");
        let is_running = if let Ok(manager) = InstanceManager::new(state_dir) {
            manager.is_running(capsule_id)
        } else {
            false
        };

        let wasm_path = self.storage.get_wasm_path(capsule_id)?;
        let wasm_size = fs::metadata(&wasm_path)?.len();

        Ok(CapsuleSummary {
            id: capsule_id.to_string(),
            name: manifest.name,
            version: manifest.version,
            installed_at: metadata.installed_at,
            run_count: metadata.run_count,
            is_running,
            wasm_size_mb: wasm_size as f64 / (1024.0 * 1024.0),
        })
    }

    /// Compara duas cápsulas
    pub fn compare(&self, id1: &str, id2: &str) -> Result<CapsuleComparison> {
        let info1 = self.inspect(id1)?;
        let info2 = self.inspect(id2)?;

        Ok(CapsuleComparison {
            capsule1: info1,
            capsule2: info2,
            differences: self.find_differences(&info1, &info2),
        })
    }

    /// Encontra diferenças entre duas cápsulas
    fn find_differences(&self, info1: &CapsuleInfo, info2: &CapsuleInfo) -> Vec<String> {
        let mut diffs = Vec::new();

        if info1.manifest.version != info2.manifest.version {
            diffs.push(format!(
                "Versão: {} vs {}",
                info1.manifest.version, info2.manifest.version
            ));
        }

        if info1.installation.wasm_size_bytes != info2.installation.wasm_size_bytes {
            diffs.push(format!(
                "Tamanho WASM: {:.2} MB vs {:.2} MB",
                info1.resources.wasm_size_mb,
                info2.resources.wasm_size_mb
            ));
        }

        if info1.execution_history.total_runs != info2.execution_history.total_runs {
            diffs.push(format!(
                "Execuções: {} vs {}",
                info1.execution_history.total_runs,
                info2.execution_history.total_runs
            ));
        }

        diffs
    }
}

/// Resumo compacto de uma cápsula
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub installed_at: u64,
    pub run_count: u32,
    pub is_running: bool,
    pub wasm_size_mb: f64,
}

/// Comparação entre duas cápsulas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleComparison {
    pub capsule1: CapsuleInfo,
    pub capsule2: CapsuleInfo,
    pub differences: Vec<String>,
}

/// Helper para formatação de timestamps
pub fn format_timestamp(ts: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

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
        } else if secs < 2592000 {
            return format!("{} dias atrás", secs / 86400);
        } else {
            return format!("{} meses atrás", secs / 2592000);
        }
    }

    "data desconhecida".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 30 segundos atrás
        let result = format_timestamp(now - 30);
        assert!(result.contains("segundos"));

        // 5 minutos atrás
        let result = format_timestamp(now - 300);
        assert!(result.contains("minutos"));
    }
}
