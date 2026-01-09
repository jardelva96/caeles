//! Executor de cápsulas em background

use crate::manifest::CapsuleManifest;
use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Informações de um processo rodando
pub struct RunningProcess {
    pub child: Child,
    pub capsule_id: String,
    pub stdout: Option<ChildStdout>,
    pub stderr: Option<ChildStderr>,
}

/// Inicia uma cápsula em background com captura de logs
pub fn start_capsule_background(
    capsule_id: &str,
    manifest_path: &Path,
) -> Result<RunningProcess> {
    // Obter caminho do executável atual
    let exe = std::env::current_exe()
        .context("Falha ao obter caminho do executável")?;

    // Iniciar processo filho que executa a cápsula
    let mut child = Command::new(exe)
        .arg("--manifest")
        .arg(manifest_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Falha ao iniciar processo da cápsula")?;

    // Extrair stdout e stderr para captura
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    Ok(RunningProcess {
        child,
        capsule_id: capsule_id.to_string(),
        stdout,
        stderr,
    })
}

/// Inicia threads para capturar stdout e stderr de um processo
pub fn start_log_capture<F>(
    mut process: RunningProcess,
    on_stdout: F,
    on_stderr: F,
) -> RunningProcess
where
    F: Fn(String) + Send + 'static + Clone,
{
    // Capturar stdout
    if let Some(stdout) = process.stdout.take() {
        let on_stdout = on_stdout.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    on_stdout(line);
                }
            }
        });
    }

    // Capturar stderr
    if let Some(stderr) = process.stderr.take() {
        let on_stderr = on_stderr.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    on_stderr(line);
                }
            }
        });
    }

    process
}

/// Verifica se um processo ainda está rodando
pub fn check_process_status(child: &mut Child) -> Option<i32> {
    match child.try_wait() {
        Ok(Some(status)) => status.code(),
        Ok(None) => None,  // Ainda rodando
        Err(_) => Some(1), // Erro, assumir que terminou
    }
}

/// Mata um processo de forma graciosa
pub fn stop_process(child: &mut Child) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Enviar SIGTERM
        unsafe {
            libc::kill(child.id() as i32, libc::SIGTERM);
        }

        // Aguardar um pouco
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Verificar se terminou
        if child.try_wait()?.is_none() {
            // Ainda rodando, forçar com SIGKILL
            child.kill()?;
        }
    }

    #[cfg(windows)]
    {
        // No Windows, usar kill direto (não há SIGTERM equivalente simples)
        child.kill()?;
    }

    #[cfg(not(any(unix, windows)))]
    {
        child.kill()?;
    }

    child.wait()?;
    Ok(())
}
