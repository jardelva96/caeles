# CAELES

**CAELES** é um motor de **cápsulas WebAssembly** focado em **Android**.  

<p align="center">
  <img src="./caeleslogo.png" alt="Logo CAELES" width="320" />
</p>

## 🔍 O que é o CAELES?

O **CAELES** é uma plataforma para executar **cápsulas** – pequenos módulos compilados para **WASM/WASI** – de forma:

- 🔒 isolada (sandbox WebAssembly)  
- 📱 pensada primeiro para **Android**  
- ⚡ leve e portátil (o mesmo `.wasm` pode rodar em vários hosts)  

Você escreve a lógica da cápsula (por exemplo em Rust), gera um `.wasm`, descreve tudo em um **manifesto CAELES**, e o **núcleo CAELES** cuida de carregar e executar.

---

## 🧩 Conceitos principais

### Cápsula

Uma **cápsula CAELES** é a unidade básica do sistema.  
Ela é composta por:

- `capsule.wasm` – binário WebAssembly (`wasm32-wasi`)  
- `capsule.manifest.json` – arquivo declarando como e com quais permissões ela roda

Exemplo **simplificado** de manifesto (formato ainda em evolução):

```json
{
  "id": "com.caeles.example.demo",
  "name": "Cápsula Demo",
  "version": "0.1.0",

  "entry": "capsule.wasm",

  "permissions": {
    "notifications": false,
    "network": false
  },

  "lifecycle": {
    "kind": "on_demand"
  }
}
```
Núcleo CAELES (runtime)
O núcleo CAELES é o “motor” que:

lê e valida o manifesto

localiza e carrega o .wasm

prepara o ambiente WASI (args, env, I/O, filesystem sandbox)

aplica permissões conforme o manifesto

faz a ponte com o sistema host (Android, desktop, etc.)

A implementação é em Rust, usando WebAssembly/WASI como base.

🏗️ Arquitetura (alto nível)

[ Android / Desktop / Outro host ]
               │
               ▼
        [ Núcleo CAELES ]
           (Rust + WASM)
               │
      carrega e executa
               │
               ▼
        [ Cápsula WASM ]
          (wasm32-wasi)
No Android, o CAELES deve ser embutido em um app host, que chama o núcleo nativo.

Em desktop, o núcleo pode ser usado para desenvolvimento, debug e testes de cápsulas.

🚦 Estado atual
🚧 Projeto em fase inicial (experimento).

Objetivos desta fase:

definir o conceito de cápsula CAELES v0

experimentar o núcleo em Rust executando uma cápsula simples (wasm32-wasi)

preparar o caminho para uma futura integração com Android

A API, o formato de manifesto e a estrutura do código ainda podem mudar bastante.

🧪 Visão de uso (futuro)
Fluxo esperado para desenvolvedores:

Escrever a cápsula em Rust (ou outra linguagem que compile para WASM):

rustup target add wasm32-wasi
cargo build --target wasm32-wasi
Isso gera algo como:

target/wasm32-wasi/debug/minha-capsula.wasm
Criar um manifesto CAELES apontando para o .wasm:

```json
{
  "id": "com.caeles.examples.mycapsule",
  "name": "Minha Cápsula CAELES",
  "version": "0.1.0",
  "entry": "minha-capsula.wasm",
  "permissions": {
    "notifications": false
  },
  "lifecycle": {
    "kind": "on_demand"
  }
}
```
Executar com o núcleo CAELES (quando disponível):


caeles run path/para/capsule.manifest.json

Também é possível usar a CLI no estilo Docker (`caeles <comando>`):

```bash
# após instalar/gerar o binário
caeles list
caeles list --json
caeles run --capsule-id com.caeles.example.hello
caeles run --manifest capsules/hello-capsule/manifest.json

# build da cápsula (este subcomando exige Rust/Cargo no ambiente)
caeles build capsules/hello-capsule

# empacota uma cápsula local (manifest + wasm)
caeles package --capsule-id com.caeles.example.hello

# “pull” local para diretório de artefatos
caeles pull com.caeles.example.hello

# lista imagens locais (pacotes/pulls)
caeles images
caeles images --json

# execuções recentes (estilo docker ps)
caeles ps --limit 10
caeles ps --limit 10 --json
caeles ps --status failed --capsule-id com.caeles.example.hello

# inspeciona uma cápsula do registry
caeles inspect com.caeles.example.hello
caeles inspect com.caeles.example.hello --json

# inspeciona uma execução específica
caeles inspect-run run-<id>
caeles inspect-run run-<id> --json

# logs de uma execução específica
caeles logs run-<id>
caeles logs run-<id> --json

# remove uma execução específica do histórico
caeles rm run-<id>

# remove em lote por filtros
caeles rm --status failed --capsule-id com.caeles.example.hello

# limpa todo histórico/logs locais
caeles rm --all
```

Durante desenvolvimento do próprio projeto, você também pode rodar via Cargo:

```bash
cargo run -p caeles-runtime -- list
cargo run -p caeles-runtime -- run --capsule-id com.caeles.example.hello
```
Ou, no Android, via um app host que lista e executa cápsulas.

🤝 Contribuição
No momento, o foco é:

consolidar os conceitos (cápsula, manifesto, núcleo)

evoluir o código inicial em Rust

documentar decisões e ideias neste repositório

Sugestões de arquitetura, formato de manifesto, nomes de conceitos e ideias de cápsulas são bem-vindas via issues.
