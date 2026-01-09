//! Sistema de gerenciamento de logs de cápsulas

use anyhow::{Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Gerenciador de logs de cápsulas
pub struct LogManager {
    logs_dir: PathBuf,
}

impl LogManager {
    /// Cria um novo gerenciador de logs
    pub fn new(root_dir: PathBuf) -> Result<Self> {
        let logs_dir = root_dir.join("logs");
        fs::create_dir_all(&logs_dir)
            .context("Falha ao criar diretório de logs")?;

        Ok(Self { logs_dir })
    }

    /// Obtém o diretório de logs de uma cápsula
    fn capsule_log_dir(&self, capsule_id: &str) -> PathBuf {
        self.logs_dir.join(Self::sanitize_id(capsule_id))
    }

    /// Obtém o caminho do arquivo de log atual
    pub fn get_current_log_path(&self, capsule_id: &str) -> Result<PathBuf> {
        let log_dir = self.capsule_log_dir(capsule_id);
        fs::create_dir_all(&log_dir)
            .context("Falha ao criar diretório de logs da cápsula")?;

        Ok(log_dir.join("current.log"))
    }

    /// Obtém o caminho do arquivo de log de erro atual
    pub fn get_current_error_log_path(&self, capsule_id: &str) -> Result<PathBuf> {
        let log_dir = self.capsule_log_dir(capsule_id);
        fs::create_dir_all(&log_dir)
            .context("Falha ao criar diretório de logs da cápsula")?;

        Ok(log_dir.join("error.log"))
    }

    /// Escreve uma linha no log
    pub fn write_log(&self, capsule_id: &str, line: &str) -> Result<()> {
        let log_path = self.get_current_log_path(capsule_id)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .context("Falha ao abrir arquivo de log")?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        writeln!(file, "[{}] {}", Self::format_timestamp(timestamp), line)
            .context("Falha ao escrever no log")?;

        Ok(())
    }

    /// Escreve uma linha no log de erro
    pub fn write_error_log(&self, capsule_id: &str, line: &str) -> Result<()> {
        let log_path = self.get_current_error_log_path(capsule_id)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .context("Falha ao abrir arquivo de log de erro")?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        writeln!(file, "[{}] {}", Self::format_timestamp(timestamp), line)
            .context("Falha ao escrever no log de erro")?;

        Ok(())
    }

    /// Lê logs de uma cápsula
    pub fn read_logs(
        &self,
        capsule_id: &str,
        lines: Option<usize>,
        follow: bool,
        since: Option<u64>,
    ) -> Result<Vec<String>> {
        let log_path = self.get_current_log_path(capsule_id)?;

        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&log_path)
            .context("Falha ao abrir arquivo de log")?;
        let reader = BufReader::new(file);

        let mut all_lines: Vec<String> = reader
            .lines()
            .filter_map(|line| line.ok())
            .collect();

        // Filtrar por timestamp se especificado
        if let Some(since_ts) = since {
            all_lines.retain(|line| {
                if let Some(ts) = Self::extract_timestamp(line) {
                    ts >= since_ts
                } else {
                    true
                }
            });
        }

        // Limitar número de linhas se especificado
        if let Some(n) = lines {
            let start = if all_lines.len() > n {
                all_lines.len() - n
            } else {
                0
            };
            all_lines = all_lines[start..].to_vec();
        }

        Ok(all_lines)
    }

    /// Lê logs de erro de uma cápsula
    pub fn read_error_logs(
        &self,
        capsule_id: &str,
        lines: Option<usize>,
    ) -> Result<Vec<String>> {
        let log_path = self.get_current_error_log_path(capsule_id)?;

        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&log_path)
            .context("Falha ao abrir arquivo de log de erro")?;
        let reader = BufReader::new(file);

        let mut all_lines: Vec<String> = reader
            .lines()
            .filter_map(|line| line.ok())
            .collect();

        // Limitar número de linhas se especificado
        if let Some(n) = lines {
            let start = if all_lines.len() > n {
                all_lines.len() - n
            } else {
                0
            };
            all_lines = all_lines[start..].to_vec();
        }

        Ok(all_lines)
    }

    /// Rotaciona logs de uma cápsula
    pub fn rotate_logs(&self, capsule_id: &str) -> Result<()> {
        let log_dir = self.capsule_log_dir(capsule_id);

        if !log_dir.exists() {
            return Ok(());
        }

        let current_log = log_dir.join("current.log");
        let error_log = log_dir.join("error.log");

        // Rotacionar log principal
        if current_log.exists() {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let rotated_name = format!("current.log.{}", timestamp);
            let rotated_path = log_dir.join(rotated_name);

            fs::rename(&current_log, &rotated_path)
                .context("Falha ao rotacionar log principal")?;
        }

        // Rotacionar log de erro
        if error_log.exists() {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let rotated_name = format!("error.log.{}", timestamp);
            let rotated_path = log_dir.join(rotated_name);

            fs::rename(&error_log, &rotated_path)
                .context("Falha ao rotacionar log de erro")?;
        }

        Ok(())
    }

    /// Lista todos os arquivos de log de uma cápsula
    pub fn list_log_files(&self, capsule_id: &str) -> Result<Vec<LogFile>> {
        let log_dir = self.capsule_log_dir(capsule_id);

        if !log_dir.exists() {
            return Ok(Vec::new());
        }

        let mut log_files = Vec::new();

        for entry in fs::read_dir(&log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let metadata = fs::metadata(&path)?;
                let size = metadata.len();
                let modified = metadata.modified()?
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                log_files.push(LogFile {
                    name: path.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    path: path.clone(),
                    size,
                    modified,
                });
            }
        }

        // Ordenar por data de modificação (mais recente primeiro)
        log_files.sort_by(|a, b| b.modified.cmp(&a.modified));

        Ok(log_files)
    }

    /// Limpa logs antigos de uma cápsula
    pub fn cleanup_old_logs(&self, capsule_id: &str, keep_count: usize) -> Result<usize> {
        let log_files = self.list_log_files(capsule_id)?;

        // Manter apenas os N arquivos mais recentes
        let mut removed = 0;
        for file in log_files.iter().skip(keep_count) {
            // Não remover current.log e error.log
            if file.name == "current.log" || file.name == "error.log" {
                continue;
            }

            fs::remove_file(&file.path)?;
            removed += 1;
        }

        Ok(removed)
    }

    /// Verifica se logs devem ser rotacionados (baseado em tamanho)
    pub fn should_rotate(&self, capsule_id: &str, max_size_mb: u64) -> Result<bool> {
        let log_path = self.get_current_log_path(capsule_id)?;

        if !log_path.exists() {
            return Ok(false);
        }

        let metadata = fs::metadata(&log_path)?;
        let size_mb = metadata.len() / (1024 * 1024);

        Ok(size_mb >= max_size_mb)
    }

    /// Obtém estatísticas de logs
    pub fn get_stats(&self, capsule_id: &str) -> Result<LogStats> {
        let log_files = self.list_log_files(capsule_id)?;

        let total_files = log_files.len();
        let total_size = log_files.iter().map(|f| f.size).sum();

        let current_log_path = self.get_current_log_path(capsule_id)?;
        let current_size = if current_log_path.exists() {
            fs::metadata(&current_log_path)?.len()
        } else {
            0
        };

        let current_lines = if current_log_path.exists() {
            let file = File::open(&current_log_path)?;
            let reader = BufReader::new(file);
            reader.lines().count()
        } else {
            0
        };

        Ok(LogStats {
            total_files,
            total_size_bytes: total_size,
            current_size_bytes: current_size,
            current_lines,
        })
    }

    /// Limpa todos os logs de uma cápsula
    pub fn clear_all_logs(&self, capsule_id: &str) -> Result<()> {
        let log_dir = self.capsule_log_dir(capsule_id);

        if log_dir.exists() {
            fs::remove_dir_all(&log_dir)?;
        }

        Ok(())
    }

    // Helpers

    fn sanitize_id(id: &str) -> String {
        id.replace('.', "_")
    }

    fn format_timestamp(ts: u64) -> String {
        use std::time::{Duration, UNIX_EPOCH};

        let datetime = UNIX_EPOCH + Duration::from_secs(ts);

        // Formato simples: YYYY-MM-DD HH:MM:SS
        // Em produção, usar chrono para formatação adequada
        format!("{:?}", datetime)
    }

    fn extract_timestamp(line: &str) -> Option<u64> {
        // Extrair timestamp do formato [timestamp] message
        if let Some(start) = line.find('[') {
            if let Some(end) = line.find(']') {
                if let Ok(ts) = line[start + 1..end].parse::<u64>() {
                    return Some(ts);
                }
            }
        }
        None
    }
}

/// Informações de um arquivo de log
#[derive(Debug, Clone)]
pub struct LogFile {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64,
}

/// Estatísticas de logs de uma cápsula
#[derive(Debug, Clone)]
pub struct LogStats {
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub current_size_bytes: u64,
    pub current_lines: usize,
}

impl LogStats {
    pub fn total_size_mb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn current_size_kb(&self) -> u64 {
        self.current_size_bytes / 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_log_manager_creation() {
        let dir = tempdir().unwrap();
        let manager = LogManager::new(dir.path().to_path_buf()).unwrap();

        assert!(manager.logs_dir.exists());
    }

    #[test]
    fn test_write_and_read_logs() {
        let dir = tempdir().unwrap();
        let manager = LogManager::new(dir.path().to_path_buf()).unwrap();

        manager.write_log("test.capsule", "Hello, world!").unwrap();
        manager.write_log("test.capsule", "Second line").unwrap();

        let logs = manager.read_logs("test.capsule", None, false, None).unwrap();
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_log_rotation() {
        let dir = tempdir().unwrap();
        let manager = LogManager::new(dir.path().to_path_buf()).unwrap();

        manager.write_log("test.capsule", "Before rotation").unwrap();
        manager.rotate_logs("test.capsule").unwrap();
        manager.write_log("test.capsule", "After rotation").unwrap();

        let files = manager.list_log_files("test.capsule").unwrap();
        assert!(files.len() >= 2); // current.log + rotated file
    }

    #[test]
    fn test_cleanup_old_logs() {
        let dir = tempdir().unwrap();
        let manager = LogManager::new(dir.path().to_path_buf()).unwrap();

        // Criar múltiplos logs rotacionados
        for i in 0..5 {
            manager.write_log("test.capsule", &format!("Log {}", i)).unwrap();
            manager.rotate_logs("test.capsule").unwrap();
        }

        let files_before = manager.list_log_files("test.capsule").unwrap();
        let removed = manager.cleanup_old_logs("test.capsule", 3).unwrap();

        assert!(removed > 0);
        let files_after = manager.list_log_files("test.capsule").unwrap();
        assert!(files_after.len() <= files_before.len());
    }

    #[test]
    fn test_log_stats() {
        let dir = tempdir().unwrap();
        let manager = LogManager::new(dir.path().to_path_buf()).unwrap();

        manager.write_log("test.capsule", "Line 1").unwrap();
        manager.write_log("test.capsule", "Line 2").unwrap();
        manager.write_log("test.capsule", "Line 3").unwrap();

        let stats = manager.get_stats("test.capsule").unwrap();
        assert!(stats.current_lines >= 3);
        assert!(stats.current_size_bytes > 0);
    }
}
