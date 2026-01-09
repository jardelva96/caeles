//! Gerenciamento de artefatos de build

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Metadados do build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    /// Timestamp do build (Unix epoch)
    pub build_time: u64,

    /// Hash SHA-256 do WASM (opcional)
    pub wasm_hash: Option<String>,

    /// Tamanho do WASM em bytes
    pub wasm_size: Option<usize>,

    /// Modo de build (debug ou release)
    pub build_mode: String,
}

impl Default for BuildMetadata {
    fn default() -> Self {
        Self {
            build_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            wasm_hash: None,
            wasm_size: None,
            build_mode: "debug".to_string(),
        }
    }
}

/// Artefatos gerados pelo build
#[derive(Debug, Clone)]
pub struct BuildArtifacts {
    /// Caminho do WASM gerado
    pub wasm_path: PathBuf,

    /// Caminho do manifest gerado/atualizado
    pub manifest_path: Option<PathBuf>,

    /// Metadados do build
    pub metadata: BuildMetadata,
}

impl BuildArtifacts {
    /// Cria um novo conjunto de artefatos
    pub fn new(wasm_path: PathBuf) -> Self {
        let mut metadata = BuildMetadata::default();

        // Detectar modo pelo path
        if wasm_path.to_string_lossy().contains("release") {
            metadata.build_mode = "release".to_string();
        }

        // Obter tamanho do WASM
        if let Ok(file_metadata) = fs::metadata(&wasm_path) {
            metadata.wasm_size = Some(file_metadata.len() as usize);
        }

        Self {
            wasm_path,
            manifest_path: None,
            metadata,
        }
    }

    /// Define o caminho do manifest
    pub fn set_manifest_path(&mut self, path: PathBuf) {
        self.manifest_path = Some(path);
    }

    /// Calcula o hash SHA-256 do WASM
    pub fn compute_wasm_hash(&mut self) -> Result<String> {
        use std::io::Read;

        let mut file = fs::File::open(&self.wasm_path)
            .context("Falha ao abrir WASM para hash")?;

        let mut hasher = sha256::Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let hash = hasher.finalize_hex();
        self.metadata.wasm_hash = Some(hash.clone());

        Ok(hash)
    }

    /// Copia artefatos para um diretório de output
    pub fn copy_to_output_dir(&self, output_dir: &Path) -> Result<()> {
        // Criar diretório se não existir
        fs::create_dir_all(output_dir)
            .context("Falha ao criar diretório de output")?;

        // Copiar WASM
        let wasm_filename = self.wasm_path.file_name()
            .context("Falha ao obter nome do arquivo WASM")?;
        let output_wasm = output_dir.join(wasm_filename);
        fs::copy(&self.wasm_path, &output_wasm)
            .context("Falha ao copiar WASM")?;

        // Copiar manifest se existir
        if let Some(manifest_path) = &self.manifest_path {
            let manifest_filename = manifest_path.file_name()
                .context("Falha ao obter nome do manifest")?;
            let output_manifest = output_dir.join(manifest_filename);
            fs::copy(manifest_path, &output_manifest)
                .context("Falha ao copiar manifest")?;
        }

        // Salvar metadata como JSON
        let metadata_path = output_dir.join("build-metadata.json");
        let metadata_json = serde_json::to_string_pretty(&self.metadata)
            .context("Falha ao serializar metadata")?;
        fs::write(&metadata_path, metadata_json)
            .context("Falha ao salvar metadata")?;

        Ok(())
    }

    /// Exibe um resumo dos artefatos
    pub fn print_summary(&self) {
        println!("\n📦 Resumo do Build:");
        println!("─────────────────────────────────────");

        println!("WASM:     {}", self.wasm_path.display());

        if let Some(size) = self.metadata.wasm_size {
            let kb = size / 1024;
            let mb = kb as f64 / 1024.0;
            if kb < 1024 {
                println!("Tamanho:  {} KB", kb);
            } else {
                println!("Tamanho:  {:.2} MB", mb);
            }
        }

        if let Some(hash) = &self.metadata.wasm_hash {
            println!("SHA-256:  {}...{}", &hash[..8], &hash[hash.len()-8..]);
        }

        if let Some(manifest) = &self.manifest_path {
            println!("Manifest: {}", manifest.display());
        }

        println!("Modo:     {}", self.metadata.build_mode);

        println!("─────────────────────────────────────");
    }
}

/// Implementação simples de SHA-256
mod sha256 {
    pub struct Sha256 {
        state: [u32; 8],
        buffer: Vec<u8>,
        length: u64,
    }

    impl Sha256 {
        pub fn new() -> Self {
            Self {
                state: [
                    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
                ],
                buffer: Vec::new(),
                length: 0,
            }
        }

        pub fn update(&mut self, data: &[u8]) {
            self.length += data.len() as u64;
            self.buffer.extend_from_slice(data);

            while self.buffer.len() >= 64 {
                let chunk: [u8; 64] = self.buffer[..64].try_into().unwrap();
                self.process_chunk(&chunk);
                self.buffer.drain(..64);
            }
        }

        pub fn finalize_hex(mut self) -> String {
            let bit_len = self.length * 8;
            self.buffer.push(0x80);

            while (self.buffer.len() + 8) % 64 != 0 {
                self.buffer.push(0);
            }

            self.buffer.extend_from_slice(&bit_len.to_be_bytes());

            while !self.buffer.is_empty() {
                let chunk: [u8; 64] = self.buffer[..64].try_into().unwrap();
                self.process_chunk(&chunk);
                self.buffer.drain(..64);
            }

            self.state
                .iter()
                .map(|v| format!("{:08x}", v))
                .collect::<String>()
        }

        fn process_chunk(&mut self, chunk: &[u8; 64]) {
            const K: [u32; 64] = [
                0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
                0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
                0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
                0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
                0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
                0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
                0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
                0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
            ];

            let mut w = [0u32; 64];
            for (i, chunk_slice) in chunk.chunks_exact(4).enumerate().take(16) {
                w[i] = u32::from_be_bytes(chunk_slice.try_into().unwrap());
            }

            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
            }

            let mut state = self.state;

            for i in 0..64 {
                let s1 = state[4].rotate_right(6) ^ state[4].rotate_right(11) ^ state[4].rotate_right(25);
                let ch = (state[4] & state[5]) ^ ((!state[4]) & state[6]);
                let temp1 = state[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
                let s0 = state[0].rotate_right(2) ^ state[0].rotate_right(13) ^ state[0].rotate_right(22);
                let maj = (state[0] & state[1]) ^ (state[0] & state[2]) ^ (state[1] & state[2]);
                let temp2 = s0.wrapping_add(maj);

                state[7] = state[6];
                state[6] = state[5];
                state[5] = state[4];
                state[4] = state[3].wrapping_add(temp1);
                state[3] = state[2];
                state[2] = state[1];
                state[1] = state[0];
                state[0] = temp1.wrapping_add(temp2);
            }

            for i in 0..8 {
                self.state[i] = self.state[i].wrapping_add(state[i]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_metadata_default() {
        let metadata = BuildMetadata::default();
        assert_eq!(metadata.build_mode, "debug");
        assert!(metadata.build_time > 0);
    }

    #[test]
    fn test_sha256_empty() {
        let mut hasher = sha256::Sha256::new();
        hasher.update(b"");
        let hash = hasher.finalize_hex();
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hello() {
        let mut hasher = sha256::Sha256::new();
        hasher.update(b"hello");
        let hash = hasher.finalize_hex();
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
