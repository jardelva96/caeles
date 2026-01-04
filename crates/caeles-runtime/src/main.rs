mod manifest;
mod runtime;

use crate::manifest::CapsuleManifest;
use clap::{Args as ClapArgs, Parser, Subcommand};
use serde::Deserialize;
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Duration;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub manifest: String,
}

#[derive(Debug, Parser)]
#[command(
    name = "caeles-runtime",
    about = "CAELES runtime: executa cápsulas a partir de manifest ou ID"
)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// Caminho para o arquivo de manifest JSON da cápsula
    #[arg(long, conflicts_with = "capsule_id")]
    manifest: Option<PathBuf>,

    /// ID da cápsula (procurado no registry JSON)
    #[arg(long, conflicts_with = "manifest")]
    capsule_id: Option<String>,

    /// Caminho para o arquivo de registry de cápsulas
    #[arg(long, default_value = "capsules/registry.json")]
    registry: PathBuf,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Interface inicial para criar um manifest de cápsula
    Init(InitArgs),

    /// Interface web para criar manifestos pelo navegador
    Web(WebArgs),
}

#[derive(Debug, ClapArgs)]
struct InitArgs {
    /// Caminho de saída para o arquivo de manifest gerado
    #[arg(long, default_value = "capsule.manifest.json")]
    output: PathBuf,

    /// ID da cápsula (ex.: com.caeles.examples.mycapsule)
    #[arg(long)]
    id: Option<String>,

    /// Nome amigável da cápsula
    #[arg(long)]
    name: Option<String>,

    /// Versão semântica
    #[arg(long, default_value = "0.1.0")]
    version: String,

    /// Caminho do wasm exportado pela cápsula (relativo ao manifest)
    #[arg(long, default_value = "capsule.wasm")]
    entry: String,

    /// Permitir notificações (não será perguntado se informado)
    #[arg(long)]
    allow_notifications: bool,

    /// Permitir rede (não será perguntado se informado)
    #[arg(long)]
    allow_network: bool,
}

#[derive(Debug, ClapArgs)]
struct WebArgs {
    /// Host de binding do servidor HTTP
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Porta de binding do servidor HTTP
    #[arg(long, default_value_t = 8080)]
    port: u16,
}

fn load_manifest_from_registry(registry_path: &Path, id: &str) -> anyhow::Result<CapsuleManifest> {
    let text = fs::read_to_string(registry_path)?;
    let entries: Vec<RegistryEntry> = serde_json::from_str(&text)?;

    let entry = entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| anyhow::anyhow!(format!("Capsule id '{id}' não encontrado no registry")))?;

    println!(
        "> Registro encontrado: {} (id: {})",
        entry.name,
        entry.id
    );

    let manifest_path = Path::new(&entry.manifest);
    CapsuleManifest::load(manifest_path)
}

fn prompt_with_default(label: &str, default: Option<&str>) -> io::Result<String> {
    if let Some(default) = default {
        print!("{label} [{default}]: ");
    } else {
        print!("{label}: ");
    }
    io::stdout().flush()?;

    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        Ok(default.unwrap_or("").to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn prompt_bool_with_default(label: &str, default: bool) -> io::Result<bool> {
    let default_hint = if default { "Y/n" } else { "y/N" };
    print!("{label} ({default_hint}): ");
    io::stdout().flush()?;

    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let trimmed = buf.trim().to_lowercase();

    match trimmed.as_str() {
        "" => Ok(default),
        "y" | "yes" | "s" | "sim" => Ok(true),
        "n" | "no" | "não" | "nao" => Ok(false),
        _ => Ok(default),
    }
}

fn run_init_wizard(args: InitArgs) -> anyhow::Result<()> {
    println!("=== CAELES – Criador inicial de manifest ===");

    let id = if let Some(id) = args.id {
        id
    } else {
        prompt_with_default("ID da cápsula (ex.: com.caeles.examples.mycapsule)", None)?
    };

    let name = if let Some(name) = args.name {
        name
    } else {
        prompt_with_default("Nome da cápsula", None)?
    };

    let version = prompt_with_default("Versão", Some(&args.version))?;
    let entry = prompt_with_default("Caminho do wasm (relativo ao manifest)", Some(&args.entry))?;

    let allow_notifications = if args.allow_notifications {
        true
    } else {
        prompt_bool_with_default("Permitir notificações (permissions.notifications)", false)?
    };

    let allow_network = if args.allow_network {
        true
    } else {
        prompt_bool_with_default("Permitir rede (permissions.network)", false)?
    };

    let manifest = CapsuleManifest::from_parts(
        id,
        name,
        version,
        entry,
        crate::manifest::Permissions {
            notifications: allow_notifications,
            network: allow_network,
        },
    );

    let json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&args.output, json)?;

    println!("Manifest criado em: {}", args.output.display());
    println!("Lembre-se de compilar sua cápsula para wasm32-unknown-unknown.");
    Ok(())
}

fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) -> io::Result<()> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())
}

fn render_form() -> String {
    r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8"/>
  <title>CAELES – Criar Manifesto de Cápsula</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 2rem auto; max-width: 640px; line-height: 1.5; }
    label { display: block; margin-top: 1rem; font-weight: 600; }
    input[type=text] { width: 100%; padding: 0.5rem; }
    .actions { margin-top: 1.5rem; }
    button { padding: 0.6rem 1rem; font-size: 1rem; }
    pre { background: #f6f8fa; padding: 1rem; overflow: auto; }
  </style>
</head>
<body>
  <h1>CAELES – Criar Manifesto de Cápsula</h1>
  <p>Preencha os campos abaixo e clique em <strong>Gerar manifest</strong>. O runtime atual espera cápsulas construídas para <code>wasm32-unknown-unknown</code> e expõe as funções de host (log, notify).</p>
  <form method="POST" action="/generate">
    <label>ID da cápsula</label>
    <input type="text" name="id" placeholder="com.caeles.examples.mycapsule" required />

    <label>Nome</label>
    <input type="text" name="name" placeholder="Minha Cápsula CAELES" required />

    <label>Versão</label>
    <input type="text" name="version" value="0.1.0" required />

    <label>Caminho do wasm (relativo ao manifest)</label>
    <input type="text" name="entry" value="capsule.wasm" required />

    <label><input type="checkbox" name="notifications" /> Permitir notificações</label>
    <label><input type="checkbox" name="network" /> Permitir rede</label>

    <div class="actions">
      <button type="submit">Gerar manifest</button>
    </div>
  </form>
</body>
</html>
"#
    .to_string()
}

fn parse_form(body: &str) -> CapsuleManifest {
    let mut id = String::new();
    let mut name = String::new();
    let mut version = "0.1.0".to_string();
    let mut entry = "capsule.wasm".to_string();
    let mut notifications = false;
    let mut network = false;

    for pair in body.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts
            .next()
            .map(|v| url_decode(v))
            .unwrap_or_default();

        match key {
            "id" => id = value,
            "name" => name = value,
            "version" => version = value,
            "entry" => entry = value,
            "notifications" => notifications = true,
            "network" => network = true,
            _ => {}
        }
    }

    CapsuleManifest::from_parts(
        id,
        name,
        version,
        entry,
        crate::manifest::Permissions {
            notifications,
            network,
        },
    )
}

fn url_decode(input: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = input.as_bytes().iter().copied().peekable();

    while let Some(b) = chars.next() {
        match b {
            b'+' => bytes.push(b' '),
            b'%' => {
                let hi = chars.next();
                let lo = chars.next();
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    if let (Some(hi_v), Some(lo_v)) = (from_hex(hi), from_hex(lo)) {
                        bytes.push((hi_v << 4) | lo_v);
                        continue;
                    }
                }
                bytes.push(b'%');
            }
            _ => bytes.push(b),
        }
    }

    String::from_utf8_lossy(&bytes).to_string()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn render_manifest_result(manifest: &CapsuleManifest) -> anyhow::Result<String> {
    let json = serde_json::to_string_pretty(manifest)?;
    let html = format!(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8"/>
  <title>Manifesto gerado</title>
  <style>
    body {{ font-family: system-ui, sans-serif; margin: 2rem auto; max-width: 720px; line-height: 1.5; }}
    pre {{ background: #f6f8fa; padding: 1rem; overflow: auto; }}
    a.button {{ display: inline-block; margin-top: 1rem; padding: 0.6rem 1rem; background: #0d6efd; color: #fff; text-decoration: none; border-radius: 4px; }}
  </style>
</head>
<body>
  <h1>Manifesto gerado</h1>
  <p>Copie o conteúdo abaixo para um arquivo <code>capsule.manifest.json</code> e compile sua cápsula para <code>wasm32-unknown-unknown</code>.</p>
  <pre>{json}</pre>
  <p><a class="button" href="/">Voltar</a></p>
</body>
</html>"#
    );
    Ok(html)
}

fn read_http_request(stream: &mut TcpStream) -> io::Result<(String, Vec<u8>)> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut buffer = Vec::new();
    let mut header_end: Option<usize> = None;
    let mut content_length: Option<usize> = None;
    let mut temp = [0u8; 4096];

    while buffer.len() < 64 * 1024 {
        let n = stream.read(&mut temp)?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..n]);

        if header_end.is_none() {
            if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(pos + 4);
                let headers = String::from_utf8_lossy(&buffer[..pos]);
                for line in headers.lines() {
                    let line = line.trim();
                    if let Some(rest) = line.strip_prefix("Content-Length:") {
                        if let Ok(len) = rest.trim().parse::<usize>() {
                            content_length = Some(len);
                        }
                    }
                }
            }
        }

        if let (Some(end), Some(len)) = (header_end, content_length) {
            if buffer.len() >= end + len {
                break;
            }
        }
    }

    let request = String::from_utf8_lossy(&buffer).to_string();
    Ok((request, buffer))
}

fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let (request, raw) = read_http_request(&mut stream)?;
    let mut lines = request.split("\r\n");
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    if method.eq_ignore_ascii_case("POST") && path == "/generate" {
        let header_end = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .map(|p| p + 4)
            .unwrap_or(raw.len());
        let body = &raw[header_end..];
        let body_str = String::from_utf8_lossy(body);
        let manifest = parse_form(&body_str);
        let html = render_manifest_result(&manifest)?;
        respond(&mut stream, "200 OK", "text/html; charset=utf-8", &html)?;
        return Ok(());
    }

    let html = render_form();
    respond(&mut stream, "200 OK", "text/html; charset=utf-8", &html)?;
    Ok(())
}

fn run_web_server(args: WebArgs) -> anyhow::Result<()> {
    let addr = format!("{}:{}", args.host, args.port);
    println!("> Servindo interface web em http://{addr} (Ctrl+C para sair)");
    let listener = TcpListener::bind(&addr)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) = handle_connection(stream) {
                    eprintln!("[web] erro atendendo requisição: {err}");
                }
            }
            Err(err) => {
                eprintln!("[web] erro de conexão: {err}");
            }
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(Command::Init(init_args)) = args.command {
        return run_init_wizard(init_args);
    }

    if let Some(Command::Web(web_args)) = args.command {
        return run_web_server(web_args);
    }

    let manifest = if let Some(path) = args.manifest {
        // Caminho direto para o manifest
        CapsuleManifest::load(&path)?
    } else if let Some(id) = args.capsule_id {
        // Resolve via registry
        load_manifest_from_registry(&args.registry, &id)?
    } else {
        anyhow::bail!("Use --manifest <arquivo> ou --capsule-id <id-da-capsula>");
    };

    println!(
        "> Manifest carregado: {} v{} (id: {})",
        manifest.name, manifest.version, manifest.id
    );
    println!(
        "> Permissões: notifications={}, network={}",
        manifest.permissions.notifications, manifest.permissions.network
    );

    runtime::run_capsule(&manifest)
}
