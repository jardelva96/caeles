//! Validador de módulos WASM para cápsulas CAELES

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::Path;
use wasmtime::*;

/// Validador de módulos WASM
pub struct WasmValidator {
    engine: Engine,
}

impl WasmValidator {
    /// Cria um novo validador
    pub fn new() -> Self {
        let engine = Engine::default();
        Self { engine }
    }

    /// Valida um módulo WASM para uso como cápsula CAELES
    pub fn validate(&self, wasm_path: &Path) -> Result<()> {
        // 1. Verificar que o arquivo existe
        if !wasm_path.exists() {
            return Err(anyhow!(
                "Arquivo WASM não encontrado: {}",
                wasm_path.display()
            ));
        }

        // 2. Ler o arquivo WASM
        let wasm_bytes = fs::read(wasm_path)
            .context("Falha ao ler arquivo WASM")?;

        // 3. Validar que é um módulo WASM válido
        let module = Module::new(&self.engine, &wasm_bytes)
            .context("Arquivo não é um módulo WASM válido")?;

        // 4. Validar exports obrigatórios
        self.validate_exports(&module)?;

        // 5. Verificar imports (avisar sobre WASI)
        self.check_imports(&module)?;

        // 6. Verificar tamanho razoável
        self.check_size(&wasm_bytes)?;

        Ok(())
    }

    /// Valida que o módulo exporta as funções necessárias
    fn validate_exports(&self, module: &Module) -> Result<()> {
        let exports: Vec<String> = module
            .exports()
            .map(|e| e.name().to_string())
            .collect();

        // Verificar se tem caeles_main
        if !exports.iter().any(|e| e == "caeles_main") {
            return Err(anyhow!(
                "Módulo WASM não exporta 'caeles_main'.\n\n\
                 Sua cápsula deve ter a função de entrada:\n\n\
                 #[no_mangle]\n\
                 pub extern \"C\" fn caeles_main() {{\n\
                     // código da cápsula\n\
                 }}\n"
            ));
        }

        // Verificar se tem memory (necessário para comunicação com host)
        if !exports.iter().any(|e| e == "memory") {
            return Err(anyhow!(
                "Módulo WASM não exporta 'memory'.\n\n\
                 Isso geralmente acontece quando:\n\
                 1. O target não é wasm32-unknown-unknown\n\
                 2. O crate-type não é [\"cdylib\"]\n\n\
                 Verifique seu Cargo.toml."
            ));
        }

        Ok(())
    }

    /// Verifica imports do módulo (avisar sobre WASI)
    fn check_imports(&self, module: &Module) -> Result<()> {
        let imports: Vec<(String, String)> = module
            .imports()
            .map(|i| (i.module().to_string(), i.name().to_string()))
            .collect();

        // Verificar se tem imports WASI
        let has_wasi = imports.iter().any(|(module, _)| {
            module.starts_with("wasi_")
        });

        if has_wasi {
            return Err(anyhow!(
                "Módulo WASM contém imports WASI.\n\n\
                 O runtime CAELES atual não suporta WASI.\n\
                 Compile para wasm32-unknown-unknown (não wasm32-wasi).\n\n\
                 Use apenas as funções do caeles-sdk para comunicação com o host."
            ));
        }

        // Verificar imports esperados do CAELES
        let caeles_imports: Vec<_> = imports.iter()
            .filter(|(module, _)| module == "caeles")
            .collect();

        if !caeles_imports.is_empty() {
            println!("📦 Imports do CAELES detectados:");
            for (_, name) in caeles_imports {
                println!("   - {}", name);
            }
        }

        Ok(())
    }

    /// Verifica se o tamanho do WASM é razoável
    fn check_size(&self, wasm_bytes: &[u8]) -> Result<()> {
        let size_kb = wasm_bytes.len() / 1024;
        let size_mb = size_kb as f64 / 1024.0;

        println!("📦 Tamanho do WASM: {:.2} MB ({} KB)", size_mb, size_kb);

        // Avisar se for muito grande (>10MB)
        if size_kb > 10 * 1024 {
            eprintln!(
                "\n⚠️  AVISO: WASM muito grande ({:.2} MB)\n\
                 Considere:\n\
                 1. Compilar em release mode (--release)\n\
                 2. Usar wasm-opt para otimização\n\
                 3. Remover dependências desnecessárias\n",
                size_mb
            );
        }

        // Avisar se for muito pequeno (<1KB) - provavelmente vazio
        if size_kb < 1 {
            eprintln!(
                "\n⚠️  AVISO: WASM muito pequeno ({} bytes)\n\
                 Isso pode indicar um problema na compilação.\n",
                wasm_bytes.len()
            );
        }

        Ok(())
    }

    /// Extrai informações detalhadas do módulo WASM
    pub fn inspect(&self, wasm_path: &Path) -> Result<WasmInfo> {
        let wasm_bytes = fs::read(wasm_path)?;
        let module = Module::new(&self.engine, &wasm_bytes)?;

        let exports: Vec<String> = module
            .exports()
            .map(|e| e.name().to_string())
            .collect();

        let imports: Vec<(String, String)> = module
            .imports()
            .map(|i| (i.module().to_string(), i.name().to_string()))
            .collect();

        Ok(WasmInfo {
            size_bytes: wasm_bytes.len(),
            exports,
            imports,
        })
    }
}

impl Default for WasmValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Informações sobre um módulo WASM
#[derive(Debug, Clone)]
pub struct WasmInfo {
    pub size_bytes: usize,
    pub exports: Vec<String>,
    pub imports: Vec<(String, String)>,
}

impl WasmInfo {
    /// Formata o tamanho em formato legível
    pub fn size_human(&self) -> String {
        let kb = self.size_bytes / 1024;
        if kb < 1024 {
            format!("{} KB", kb)
        } else {
            let mb = kb as f64 / 1024.0;
            format!("{:.2} MB", mb)
        }
    }

    /// Verifica se tem um export específico
    pub fn has_export(&self, name: &str) -> bool {
        self.exports.iter().any(|e| e == name)
    }

    /// Verifica se tem imports WASI
    pub fn has_wasi_imports(&self) -> bool {
        self.imports.iter().any(|(module, _)| module.starts_with("wasi_"))
    }

    /// Retorna imports do módulo CAELES
    pub fn caeles_imports(&self) -> Vec<&str> {
        self.imports
            .iter()
            .filter(|(module, _)| module == "caeles")
            .map(|(_, name)| name.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_info_size_human() {
        let info = WasmInfo {
            size_bytes: 2048,
            exports: vec![],
            imports: vec![],
        };
        assert_eq!(info.size_human(), "2 KB");

        let info_mb = WasmInfo {
            size_bytes: 2 * 1024 * 1024,
            exports: vec![],
            imports: vec![],
        };
        assert_eq!(info_mb.size_human(), "2.00 MB");
    }

    #[test]
    fn test_has_export() {
        let info = WasmInfo {
            size_bytes: 0,
            exports: vec!["caeles_main".to_string(), "memory".to_string()],
            imports: vec![],
        };

        assert!(info.has_export("caeles_main"));
        assert!(info.has_export("memory"));
        assert!(!info.has_export("other_function"));
    }

    #[test]
    fn test_has_wasi_imports() {
        let info = WasmInfo {
            size_bytes: 0,
            exports: vec![],
            imports: vec![
                ("wasi_snapshot_preview1".to_string(), "fd_write".to_string()),
            ],
        };

        assert!(info.has_wasi_imports());
    }

    #[test]
    fn test_caeles_imports() {
        let info = WasmInfo {
            size_bytes: 0,
            exports: vec![],
            imports: vec![
                ("caeles".to_string(), "host_log".to_string()),
                ("caeles".to_string(), "host_notify".to_string()),
                ("other".to_string(), "something".to_string()),
            ],
        };

        let caeles_imports = info.caeles_imports();
        assert_eq!(caeles_imports.len(), 2);
        assert!(caeles_imports.contains(&"host_log"));
        assert!(caeles_imports.contains(&"host_notify"));
    }
}
