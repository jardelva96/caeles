//! Detecção e análise de projetos Rust para build de cápsulas

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Informações extraídas do Cargo.toml
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// Nome do projeto (package.name)
    pub name: String,

    /// Versão do projeto (package.version)
    pub version: String,

    /// Caminho do Cargo.toml
    pub cargo_toml_path: PathBuf,

    /// Diretório raiz do projeto
    pub root_dir: PathBuf,

    /// Tipo de crate (lib ou bin)
    pub crate_type: CrateType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CrateType {
    Library,
    Binary,
}

/// Estrutura parcial do Cargo.toml para parsing
#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
    #[serde(default)]
    lib: Option<Library>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct Library {
    #[serde(rename = "crate-type")]
    crate_type: Option<Vec<String>>,
}

/// Detector de projetos Rust para cápsulas CAELES
pub struct ProjectDetector {
    root_dir: PathBuf,
}

impl ProjectDetector {
    /// Cria um novo detector para o diretório especificado
    pub fn new(root_dir: &Path) -> Result<Self> {
        let root_dir = root_dir
            .canonicalize()
            .context("Falha ao resolver caminho do diretório")?;

        Ok(Self { root_dir })
    }

    /// Detecta e valida um projeto Rust no diretório
    pub fn detect(&self) -> Result<ProjectInfo> {
        let cargo_toml_path = self.find_cargo_toml()?;

        let content = fs::read_to_string(&cargo_toml_path)
            .context("Falha ao ler Cargo.toml")?;

        let cargo_toml: CargoToml = toml::from_str(&content)
            .context("Falha ao parsear Cargo.toml (formato inválido)")?;

        let crate_type = self.detect_crate_type(&cargo_toml)?;

        // Validar que é um projeto adequado para cápsula
        self.validate_capsule_project(&cargo_toml_path)?;

        Ok(ProjectInfo {
            name: cargo_toml.package.name,
            version: cargo_toml.package.version,
            cargo_toml_path,
            root_dir: self.root_dir.clone(),
            crate_type,
        })
    }

    /// Encontra o Cargo.toml no diretório
    fn find_cargo_toml(&self) -> Result<PathBuf> {
        let cargo_toml = self.root_dir.join("Cargo.toml");

        if !cargo_toml.exists() {
            return Err(anyhow!(
                "Cargo.toml não encontrado em {}\n\
                 Certifique-se de executar o comando no diretório raiz do projeto Rust.",
                self.root_dir.display()
            ));
        }

        Ok(cargo_toml)
    }

    /// Detecta o tipo de crate
    fn detect_crate_type(&self, cargo_toml: &CargoToml) -> Result<CrateType> {
        // Verificar se há definição explícita de lib
        if let Some(lib) = &cargo_toml.lib {
            if let Some(types) = &lib.crate_type {
                if types.iter().any(|t| t == "cdylib") {
                    return Ok(CrateType::Library);
                }
            }
        }

        // Verificar estrutura de diretórios
        let src_dir = self.root_dir.join("src");
        if src_dir.join("lib.rs").exists() {
            return Ok(CrateType::Library);
        }

        if src_dir.join("main.rs").exists() {
            return Ok(CrateType::Binary);
        }

        Err(anyhow!(
            "Não foi possível detectar o tipo de crate.\n\
             Cápsulas devem ter src/lib.rs e [lib] com crate-type = [\"cdylib\"]"
        ))
    }

    /// Valida se o projeto é adequado para ser uma cápsula
    fn validate_capsule_project(&self, cargo_toml_path: &Path) -> Result<()> {
        let content = fs::read_to_string(cargo_toml_path)?;

        // Verificar se tem caeles-sdk como dependência (recomendado)
        if !content.contains("caeles-sdk") {
            eprintln!(
                "⚠️  AVISO: Projeto não tem 'caeles-sdk' como dependência.\n\
                 Para usar funções do host (log, notify), adicione:\n\n\
                 [dependencies]\n\
                 caeles-sdk = \"0.1\"\n"
            );
        }

        // Verificar se tem crate-type = cdylib
        if !content.contains("crate-type") || !content.contains("cdylib") {
            eprintln!(
                "⚠️  AVISO: Cápsula deve ter crate-type = [\"cdylib\"] no Cargo.toml.\n\
                 Adicione:\n\n\
                 [lib]\n\
                 crate-type = [\"cdylib\"]\n"
            );
        }

        // Verificar se existe src/lib.rs
        let lib_rs = cargo_toml_path.parent()
            .unwrap()
            .join("src")
            .join("lib.rs");

        if !lib_rs.exists() {
            return Err(anyhow!(
                "src/lib.rs não encontrado.\n\
                 Cápsulas devem ser library crates com src/lib.rs"
            ));
        }

        Ok(())
    }

    /// Verifica se o target wasm32-unknown-unknown está instalado
    pub fn check_wasm_target_installed() -> Result<()> {
        let output = std::process::Command::new("rustup")
            .args(["target", "list", "--installed"])
            .output()
            .context("Falha ao executar 'rustup target list'")?;

        let installed = String::from_utf8_lossy(&output.stdout);

        if !installed.contains("wasm32-unknown-unknown") {
            return Err(anyhow!(
                "Target wasm32-unknown-unknown não está instalado.\n\n\
                 Instale com:\n\
                 rustup target add wasm32-unknown-unknown"
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_type_equality() {
        assert_eq!(CrateType::Library, CrateType::Library);
        assert_ne!(CrateType::Library, CrateType::Binary);
    }
}
