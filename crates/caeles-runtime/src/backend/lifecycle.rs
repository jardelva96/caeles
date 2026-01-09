//! Sistema de gerenciamento de ciclo de vida de cápsulas

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Estado de uma instância de cápsula
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InstanceStatus {
    /// Cápsula está rodando
    Running,
    /// Cápsula foi parada
    Stopped,
    /// Cápsula finalizou com sucesso
    Exited,
    /// Cápsula falhou/crashed
    Failed,
}

impl std::fmt::Display for InstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceStatus::Running => write!(f, "running"),
            InstanceStatus::Stopped => write!(f, "stopped"),
            InstanceStatus::Exited => write!(f, "exited"),
            InstanceStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Informações de uma instância rodando
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    /// ID da cápsula
    pub capsule_id: String,

    /// Status atual
    pub status: InstanceStatus,

    /// PID do processo (se rodando)
    pub pid: Option<u32>,

    /// Timestamp de quando foi iniciada
    pub started_at: Option<u64>,

    /// Timestamp de quando foi parada
    pub stopped_at: Option<u64>,

    /// Código de saída (se finalizou)
    pub exit_code: Option<i32>,

    /// Número de restarts
    pub restart_count: u32,

    /// Última verificação de health
    pub last_health_check: Option<u64>,
}

impl InstanceInfo {
    pub fn new(capsule_id: String) -> Self {
        Self {
            capsule_id,
            status: InstanceStatus::Stopped,
            pid: None,
            started_at: None,
            stopped_at: None,
            exit_code: None,
            restart_count: 0,
            last_health_check: None,
        }
    }

    /// Marca como iniciada
    pub fn mark_started(&mut self, pid: u32) {
        self.status = InstanceStatus::Running;
        self.pid = Some(pid);
        self.started_at = Some(current_timestamp());
        self.stopped_at = None;
        self.exit_code = None;
    }

    /// Marca como parada
    pub fn mark_stopped(&mut self) {
        self.status = InstanceStatus::Stopped;
        self.pid = None;
        self.stopped_at = Some(current_timestamp());
    }

    /// Marca como finalizada
    pub fn mark_exited(&mut self, exit_code: i32) {
        self.status = if exit_code == 0 {
            InstanceStatus::Exited
        } else {
            InstanceStatus::Failed
        };
        self.pid = None;
        self.stopped_at = Some(current_timestamp());
        self.exit_code = Some(exit_code);
    }

    /// Retorna uptime em segundos (se rodando)
    pub fn uptime_secs(&self) -> Option<u64> {
        if let Some(started) = self.started_at {
            if self.status == InstanceStatus::Running {
                let now = current_timestamp();
                return Some(now - started);
            }
        }
        None
    }

    /// Formata uptime de forma legível
    pub fn uptime_human(&self) -> String {
        match self.uptime_secs() {
            Some(secs) => format_duration(secs),
            None => "não rodando".to_string(),
        }
    }
}

/// Gerenciador de ciclo de vida de instâncias
pub struct InstanceManager {
    state_dir: PathBuf,
    instances: Arc<Mutex<HashMap<String, InstanceInfo>>>,
}

impl InstanceManager {
    /// Cria um novo gerenciador
    pub fn new(state_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&state_dir)
            .context("Falha ao criar diretório de estado")?;

        let manager = Self {
            state_dir,
            instances: Arc::new(Mutex::new(HashMap::new())),
        };

        // Carregar estado persistido
        manager.load_state()?;

        Ok(manager)
    }

    /// Registra uma nova instância
    pub fn register(&self, capsule_id: String) -> Result<()> {
        let mut instances = self.instances.lock().unwrap();

        if !instances.contains_key(&capsule_id) {
            instances.insert(capsule_id.clone(), InstanceInfo::new(capsule_id));
            drop(instances);
            self.save_state()?;
        }

        Ok(())
    }

    /// Marca instância como iniciada
    pub fn mark_started(&self, capsule_id: &str, pid: u32) -> Result<()> {
        let mut instances = self.instances.lock().unwrap();

        if let Some(info) = instances.get_mut(capsule_id) {
            info.mark_started(pid);
            drop(instances);
            self.save_state()?;
            Ok(())
        } else {
            anyhow::bail!("Instância '{}' não registrada", capsule_id)
        }
    }

    /// Marca instância como parada
    pub fn mark_stopped(&self, capsule_id: &str) -> Result<()> {
        let mut instances = self.instances.lock().unwrap();

        if let Some(info) = instances.get_mut(capsule_id) {
            info.mark_stopped();
            drop(instances);
            self.save_state()?;
            Ok(())
        } else {
            anyhow::bail!("Instância '{}' não registrada", capsule_id)
        }
    }

    /// Marca instância como finalizada
    pub fn mark_exited(&self, capsule_id: &str, exit_code: i32) -> Result<()> {
        let mut instances = self.instances.lock().unwrap();

        if let Some(info) = instances.get_mut(capsule_id) {
            info.mark_exited(exit_code);
            drop(instances);
            self.save_state()?;
            Ok(())
        } else {
            anyhow::bail!("Instância '{}' não registrada", capsule_id)
        }
    }

    /// Obtém informações de uma instância
    pub fn get(&self, capsule_id: &str) -> Option<InstanceInfo> {
        let instances = self.instances.lock().unwrap();
        instances.get(capsule_id).cloned()
    }

    /// Lista todas as instâncias
    pub fn list(&self) -> Vec<InstanceInfo> {
        let instances = self.instances.lock().unwrap();
        instances.values().cloned().collect()
    }

    /// Lista apenas instâncias rodando
    pub fn list_running(&self) -> Vec<InstanceInfo> {
        let instances = self.instances.lock().unwrap();
        instances
            .values()
            .filter(|i| i.status == InstanceStatus::Running)
            .cloned()
            .collect()
    }

    /// Verifica se uma instância está rodando
    pub fn is_running(&self, capsule_id: &str) -> bool {
        let instances = self.instances.lock().unwrap();
        instances
            .get(capsule_id)
            .map(|i| i.status == InstanceStatus::Running)
            .unwrap_or(false)
    }

    /// Atualiza health check
    pub fn update_health(&self, capsule_id: &str) -> Result<()> {
        let mut instances = self.instances.lock().unwrap();

        if let Some(info) = instances.get_mut(capsule_id) {
            info.last_health_check = Some(current_timestamp());
            Ok(())
        } else {
            anyhow::bail!("Instância '{}' não registrada", capsule_id)
        }
    }

    /// Salva estado em disco
    fn save_state(&self) -> Result<()> {
        let instances = self.instances.lock().unwrap();
        let state_file = self.state_dir.join("instances.json");

        let json = serde_json::to_string_pretty(&*instances)
            .context("Falha ao serializar estado")?;

        fs::write(&state_file, json)
            .context("Falha ao salvar estado")?;

        Ok(())
    }

    /// Carrega estado do disco
    fn load_state(&self) -> Result<()> {
        let state_file = self.state_dir.join("instances.json");

        if !state_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&state_file)
            .context("Falha ao ler arquivo de estado")?;

        let loaded: HashMap<String, InstanceInfo> = serde_json::from_str(&content)
            .context("Falha ao parsear estado")?;

        let mut instances = self.instances.lock().unwrap();
        *instances = loaded;

        // Atualizar status de processos que podem ter morrido
        for info in instances.values_mut() {
            if info.status == InstanceStatus::Running {
                if let Some(pid) = info.pid {
                    if !is_process_running(pid) {
                        info.mark_exited(1); // Assumir que crashed
                    }
                }
            }
        }

        Ok(())
    }

    /// Limpa instâncias finalizadas
    pub fn cleanup_exited(&self) -> Result<usize> {
        let mut instances = self.instances.lock().unwrap();

        let before = instances.len();
        instances.retain(|_, info| {
            info.status == InstanceStatus::Running ||
            info.status == InstanceStatus::Stopped
        });
        let removed = before - instances.len();

        if removed > 0 {
            drop(instances);
            self.save_state()?;
        }

        Ok(removed)
    }
}

/// Verifica se um processo está rodando (cross-platform)
fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                output.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }

    #[cfg(not(any(unix, windows)))]
    {
        false // Assume que não está rodando em plataformas desconhecidas
    }
}

/// Retorna timestamp atual
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Formata duração em formato legível
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{}h {}m", hours, mins)
    } else {
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn test_manager() -> InstanceManager {
        let test_dir = env::temp_dir().join("caeles-lifecycle-test");
        if test_dir.exists() {
            let _ = fs::remove_dir_all(&test_dir);
        }
        InstanceManager::new(test_dir).unwrap()
    }

    #[test]
    fn test_instance_lifecycle() {
        let mut info = InstanceInfo::new("test".to_string());
        assert_eq!(info.status, InstanceStatus::Stopped);

        info.mark_started(1234);
        assert_eq!(info.status, InstanceStatus::Running);
        assert_eq!(info.pid, Some(1234));

        info.mark_stopped();
        assert_eq!(info.status, InstanceStatus::Stopped);
        assert_eq!(info.pid, None);
    }

    #[test]
    fn test_manager_register() {
        let manager = test_manager();
        manager.register("test.capsule".to_string()).unwrap();

        let info = manager.get("test.capsule").unwrap();
        assert_eq!(info.capsule_id, "test.capsule");
        assert_eq!(info.status, InstanceStatus::Stopped);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3665), "1h 1m");
        assert_eq!(format_duration(90000), "1d 1h");
    }
}
