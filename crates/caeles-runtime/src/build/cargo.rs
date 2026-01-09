//! Executor de cargo build para compilação de cápsulas

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Executor de cargo build para WASM
pub struct CargoBuilder {
    project_root: PathBuf,
}

impl CargoBuilder {
    /// Cria um novo builder para o projeto
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
        }
    }

    /// Executa cargo build --target wasm32-unknown-unknown
    pub fn build(&self, release: bool) -> Result<PathBuf> {
        // Verificar se cargo está disponível
        self.check_cargo_available()?;

        // Construir argumentos do comando
        let mut args = vec![
            "build",
            "--target",
            "wasm32-unknown-unknown",
        ];

        if release {
            args.push("--release");
        }

        // Executar cargo build
        let output = Command::new("cargo")
            .current_dir(&self.project_root)
            .args(&args)
            .output()
            .context("Falha ao executar 'cargo build'")?;

        // Processar resultado
        self.handle_build_output(&output)?;

        // Retornar caminho do WASM gerado
        let wasm_path = self.expected_wasm_path(release);

        if !wasm_path.exists() {
            return Err(anyhow!(
                "WASM não foi gerado no caminho esperado: {}\n\
                 Verifique se o Cargo.toml tem [lib] com crate-type = [\"cdylib\"]",
                wasm_path.display()
            ));
        }

        Ok(wasm_path)
    }

    /// Retorna o caminho esperado do WASM gerado
    pub fn expected_wasm_path(&self, release: bool) -> PathBuf {
        let build_type = if release { "release" } else { "debug" };

        // Obter o nome do pacote do Cargo.toml
        let package_name = self.get_package_name()
            .unwrap_or_else(|_| "unknown".to_string());

        // Converter nome para snake_case (cargo faz isso automaticamente)
        let wasm_name = package_name.replace('-', "_");

        self.project_root
            .join("target")
            .join("wasm32-unknown-unknown")
            .join(build_type)
            .join(format!("{}.wasm", wasm_name))
    }

    /// Verifica se cargo está disponível
    fn check_cargo_available(&self) -> Result<()> {
        Command::new("cargo")
            .arg("--version")
            .output()
            .context("'cargo' não encontrado. Certifique-se de que Rust está instalado.")?;

        Ok(())
    }

    /// Processa a saída do cargo build
    fn handle_build_output(&self, output: &Output) -> Result<()> {
        // Imprimir stdout (progresso do build)
        if !output.stdout.is_empty() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            print!("{}", stdout);
        }

        // Verificar se houve erro
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Falha na compilação:\n\n{}",
                stderr
            ));
        }

        Ok(())
    }

    /// Obtém o nome do pacote do Cargo.toml
    fn get_package_name(&self) -> Result<String> {
        let cargo_toml_path = self.project_root.join("Cargo.toml");
        let content = std::fs::read_to_string(&cargo_toml_path)
            .context("Falha ao ler Cargo.toml")?;

        // Parse simples para extrair o nome
        for line in content.lines() {
            if line.trim().starts_with("name") {
                if let Some(name) = line.split('=').nth(1) {
                    let name = name.trim().trim_matches('"').trim_matches('\'');
                    return Ok(name.to_string());
                }
            }
        }

        Err(anyhow!("Campo 'name' não encontrado no Cargo.toml"))
    }

    /// Limpa artefatos de build anteriores
    pub fn clean(&self) -> Result<()> {
        let output = Command::new("cargo")
            .current_dir(&self.project_root)
            .arg("clean")
            .output()
            .context("Falha ao executar 'cargo clean'")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Falha ao limpar build:\n{}", stderr));
        }

        println!("✅ Build artifacts limpos");
        Ok(())
    }

    /// Verifica se há updates de dependências disponíveis
    pub fn check_updates(&self) -> Result<()> {
        // Tentar usar cargo-outdated se disponível
        let output = Command::new("cargo")
            .current_dir(&self.project_root)
            .args(["outdated", "--exit-code", "0"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if !stdout.trim().is_empty() {
                    println!("\n📦 Atualizações de dependências disponíveis:\n{}", stdout);
                }
            }
            _ => {
                // cargo-outdated não instalado, ignorar
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_name_conversion() {
        // Nome com hífen deve ser convertido para underscore
        let name = "hello-capsule";
        let wasm_name = name.replace('-', "_");
        assert_eq!(wasm_name, "hello_capsule");
    }

    #[test]
    fn test_expected_wasm_path_structure() {
        let builder = CargoBuilder::new(Path::new("/fake/path"));
        let path = builder.expected_wasm_path(false);

        assert!(path.to_string_lossy().contains("wasm32-unknown-unknown"));
        assert!(path.to_string_lossy().contains("debug"));
        assert!(path.extension().unwrap() == "wasm");
    }
}
