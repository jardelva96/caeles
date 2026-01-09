//! Sistema de build para cápsulas CAELES
//!
//! Este módulo implementa o pipeline completo de compilação de cápsulas:
//! 1. Detecção de projeto Rust válido
//! 2. Compilação para wasm32-unknown-unknown
//! 3. Validação do WASM gerado
//! 4. Geração/atualização de manifest
//! 5. Cálculo de checksums e metadata

mod project;
mod cargo;
mod validator;
mod manifest_gen;
mod artifacts;

pub use project::ProjectDetector;
pub use cargo::CargoBuilder;
pub use validator::WasmValidator;
pub use manifest_gen::ManifestGenerator;
pub use artifacts::{BuildArtifacts, BuildMetadata};

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Configuração do processo de build
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Diretório raiz do projeto (onde está o Cargo.toml)
    pub project_root: PathBuf,

    /// Modo de build (debug ou release)
    pub release: bool,

    /// Diretório de output customizado (opcional)
    pub output_dir: Option<PathBuf>,

    /// Gerar manifest automaticamente
    pub generate_manifest: bool,

    /// Calcular hash SHA-256 do WASM
    pub compute_hash: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            project_root: PathBuf::from("."),
            release: false,
            output_dir: None,
            generate_manifest: true,
            compute_hash: true,
        }
    }
}

/// Sistema central de build de cápsulas CAELES
pub struct BuildSystem {
    config: BuildConfig,
    detector: ProjectDetector,
    builder: CargoBuilder,
    validator: WasmValidator,
    manifest_gen: ManifestGenerator,
}

impl BuildSystem {
    /// Cria um novo sistema de build com configuração padrão
    pub fn new(config: BuildConfig) -> Result<Self> {
        let detector = ProjectDetector::new(&config.project_root)?;
        let builder = CargoBuilder::new(&config.project_root);
        let validator = WasmValidator::new();
        let manifest_gen = ManifestGenerator::new(&config.project_root);

        Ok(Self {
            config,
            detector,
            builder,
            validator,
            manifest_gen,
        })
    }

    /// Executa o build completo da cápsula
    pub fn build(&self) -> Result<BuildArtifacts> {
        println!("🔍 Detectando projeto Rust...");
        let project_info = self.detector.detect()?;
        println!("✅ Projeto detectado: {} v{}", project_info.name, project_info.version);

        println!("\n🔨 Compilando para wasm32-unknown-unknown...");
        let wasm_path = self.builder.build(self.config.release)?;
        println!("✅ WASM gerado: {}", wasm_path.display());

        println!("\n🔍 Validando WASM...");
        self.validator.validate(&wasm_path)?;
        println!("✅ WASM válido (exports: caeles_main, memory)");

        let mut artifacts = BuildArtifacts::new(wasm_path.clone());

        if self.config.compute_hash {
            println!("\n🔐 Calculando checksum...");
            let hash = artifacts.compute_wasm_hash()?;
            println!("✅ SHA-256: {}", hash);
        }

        if self.config.generate_manifest {
            println!("\n📝 Gerando manifest...");
            let manifest_path = self.manifest_gen.generate_or_update(
                &project_info,
                &wasm_path,
                &artifacts.metadata,
            )?;
            artifacts.set_manifest_path(manifest_path.clone());
            println!("✅ Manifest: {}", manifest_path.display());
        }

        // Copiar para diretório de output se especificado
        if let Some(output_dir) = &self.config.output_dir {
            println!("\n📦 Copiando artefatos para {}...", output_dir.display());
            artifacts.copy_to_output_dir(output_dir)?;
            println!("✅ Artefatos copiados");
        }

        println!("\n🎉 Build concluído com sucesso!");
        Ok(artifacts)
    }

    /// Executa apenas a compilação sem validação ou geração de manifest
    pub fn build_only(&self) -> Result<PathBuf> {
        self.builder.build(self.config.release)
    }

    /// Valida um WASM existente sem compilar
    pub fn validate_only(&self, wasm_path: &Path) -> Result<()> {
        self.validator.validate(wasm_path)
    }

    /// Gera/atualiza apenas o manifest sem compilar
    pub fn generate_manifest_only(&self) -> Result<PathBuf> {
        let project_info = self.detector.detect()?;
        let wasm_path = self.builder.expected_wasm_path(self.config.release);
        let metadata = BuildMetadata::default();
        self.manifest_gen.generate_or_update(&project_info, &wasm_path, &metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config_default() {
        let config = BuildConfig::default();
        assert!(!config.release);
        assert!(config.generate_manifest);
        assert!(config.compute_hash);
    }
}
