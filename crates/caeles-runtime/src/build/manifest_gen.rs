//! Gerador e atualizador de manifests para cápsulas CAELES

use crate::build::project::ProjectInfo;
use crate::build::artifacts::BuildMetadata;
use crate::manifest::{CapsuleManifest, Permissions};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Gerador de manifests
pub struct ManifestGenerator {
    project_root: PathBuf,
}

impl ManifestGenerator {
    /// Cria um novo gerador para o projeto
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
        }
    }

    /// Gera ou atualiza o manifest da cápsula
    pub fn generate_or_update(
        &self,
        project_info: &ProjectInfo,
        wasm_path: &Path,
        metadata: &BuildMetadata,
    ) -> Result<PathBuf> {
        let manifest_path = self.project_root.join("capsule.manifest.json");

        // Se já existe, atualizar; senão, criar novo
        let manifest = if manifest_path.exists() {
            self.update_existing(&manifest_path, project_info, wasm_path, metadata)?
        } else {
            self.generate_new(project_info, wasm_path, metadata)?
        };

        // Salvar no disco
        self.save_manifest(&manifest, &manifest_path)?;

        Ok(manifest_path)
    }

    /// Gera um novo manifest do zero
    fn generate_new(
        &self,
        project_info: &ProjectInfo,
        wasm_path: &Path,
        _metadata: &BuildMetadata,
    ) -> Result<CapsuleManifest> {
        // Gerar ID no formato com.caeles.<package-name>
        let id = self.generate_capsule_id(&project_info.name);

        // Caminho relativo do WASM
        let entry = self.make_relative_path(wasm_path)?;

        // Permissões padrão (todas desabilitadas)
        let permissions = Permissions {
            notifications: false,
            network: false,
        };

        Ok(CapsuleManifest::from_parts(
            id,
            project_info.name.clone(),
            project_info.version.clone(),
            entry,
            permissions,
        ))
    }

    /// Atualiza um manifest existente
    fn update_existing(
        &self,
        manifest_path: &Path,
        project_info: &ProjectInfo,
        wasm_path: &Path,
        _metadata: &BuildMetadata,
    ) -> Result<CapsuleManifest> {
        let mut manifest = CapsuleManifest::load(manifest_path)
            .context("Falha ao carregar manifest existente")?;

        // Atualizar campos que podem ter mudado
        manifest.version = project_info.version.clone();
        manifest.entry = self.make_relative_path(wasm_path)?;

        // Preservar ID, name e permissions originais

        Ok(manifest)
    }

    /// Gera um ID de cápsula no formato padrão
    fn generate_capsule_id(&self, package_name: &str) -> String {
        // Formato: com.caeles.<package-name>
        // Substituir underscores por pontos para seguir convenção de IDs
        let normalized = package_name.replace('_', ".");
        format!("com.caeles.{}", normalized)
    }

    /// Converte caminho absoluto em relativo ao diretório do projeto
    fn make_relative_path(&self, wasm_path: &Path) -> Result<String> {
        // Tentar fazer path relativo ao projeto
        if let Ok(relative) = wasm_path.strip_prefix(&self.project_root) {
            Ok(relative.to_string_lossy().replace('\\', "/"))
        } else {
            // Se não conseguir, usar path absoluto
            Ok(wasm_path.to_string_lossy().replace('\\', "/"))
        }
    }

    /// Salva o manifest no disco com formatação bonita
    fn save_manifest(&self, manifest: &CapsuleManifest, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(manifest)
            .context("Falha ao serializar manifest")?;

        fs::write(path, json)
            .context("Falha ao escrever manifest no disco")?;

        Ok(())
    }

    /// Gera um manifest interativamente (para comando init)
    pub fn generate_interactive(&self) -> Result<CapsuleManifest> {
        use std::io::{self, Write};

        println!("📝 Criando manifest CAELES interativamente\n");

        // ID
        print!("ID da cápsula (ex: com.caeles.example.hello): ");
        io::stdout().flush()?;
        let mut id = String::new();
        io::stdin().read_line(&mut id)?;
        let id = id.trim().to_string();

        // Name
        print!("Nome amigável: ");
        io::stdout().flush()?;
        let mut name = String::new();
        io::stdin().read_line(&mut name)?;
        let name = name.trim().to_string();

        // Version
        print!("Versão (padrão: 0.1.0): ");
        io::stdout().flush()?;
        let mut version = String::new();
        io::stdin().read_line(&mut version)?;
        let version = version.trim();
        let version = if version.is_empty() {
            "0.1.0".to_string()
        } else {
            version.to_string()
        };

        // Entry
        print!("Caminho do WASM (padrão: target/wasm32-unknown-unknown/debug/<name>.wasm): ");
        io::stdout().flush()?;
        let mut entry = String::new();
        io::stdin().read_line(&mut entry)?;
        let entry = entry.trim();
        let entry = if entry.is_empty() {
            format!(
                "target/wasm32-unknown-unknown/debug/{}.wasm",
                name.replace('-', "_")
            )
        } else {
            entry.to_string()
        };

        // Permissions - Notifications
        print!("Permitir notificações? (s/N): ");
        io::stdout().flush()?;
        let mut notif = String::new();
        io::stdin().read_line(&mut notif)?;
        let notifications = notif.trim().to_lowercase() == "s";

        // Permissions - Network
        print!("Permitir acesso à rede? (s/N): ");
        io::stdout().flush()?;
        let mut net = String::new();
        io::stdin().read_line(&mut net)?;
        let network = net.trim().to_lowercase() == "s";

        let permissions = Permissions {
            notifications,
            network,
        };

        Ok(CapsuleManifest::from_parts(
            id,
            name,
            version,
            entry,
            permissions,
        ))
    }

    /// Valida um manifest existente
    pub fn validate_manifest(manifest_path: &Path) -> Result<()> {
        let manifest = CapsuleManifest::load(manifest_path)
            .context("Falha ao carregar manifest")?;

        // Validar campos obrigatórios
        if manifest.id.is_empty() {
            anyhow::bail!("Campo 'id' não pode estar vazio");
        }

        if manifest.name.is_empty() {
            anyhow::bail!("Campo 'name' não pode estar vazio");
        }

        if manifest.version.is_empty() {
            anyhow::bail!("Campo 'version' não pode estar vazio");
        }

        if manifest.entry.is_empty() {
            anyhow::bail!("Campo 'entry' não pode estar vazio");
        }

        // Validar formato do ID (deve ser estilo reverse domain)
        if !manifest.id.contains('.') {
            eprintln!(
                "⚠️  AVISO: ID '{}' não segue o formato recomendado.\n\
                 Use formato reverse-domain: com.caeles.example.mycapsule",
                manifest.id
            );
        }

        // Validar versão (semver básico)
        if !manifest.version.chars().any(|c| c.is_numeric()) {
            eprintln!(
                "⚠️  AVISO: Versão '{}' não parece ser semver válida.\n\
                 Use formato: MAJOR.MINOR.PATCH (ex: 1.0.0)",
                manifest.version
            );
        }

        println!("✅ Manifest válido");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_capsule_id() {
        let gen = ManifestGenerator::new(Path::new("."));

        assert_eq!(
            gen.generate_capsule_id("hello-capsule"),
            "com.caeles.hello-capsule"
        );

        assert_eq!(
            gen.generate_capsule_id("my_capsule"),
            "com.caeles.my.capsule"
        );
    }

    #[test]
    fn test_make_relative_path() {
        let gen = ManifestGenerator::new(Path::new("/project"));

        // Caminho dentro do projeto
        let wasm = Path::new("/project/target/wasm32-unknown-unknown/debug/app.wasm");
        let relative = gen.make_relative_path(wasm).unwrap();

        assert!(relative.contains("target"));
        assert!(relative.contains("wasm32-unknown-unknown"));
    }
}
