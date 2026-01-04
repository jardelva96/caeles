# CAELES

**CAELES** é um motor de **cápsulas WebAssembly** focado em **Android**.  

<p align="center">
  <img src="./caeleslogo.png" alt="Logo CAELES" width="320" />
</p>

## 🔍 O que é o CAELES?

O **CAELES** é uma plataforma para executar **cápsulas** – pequenos módulos compilados para **WASM** – de forma:

- 🔒 isolada (sandbox WebAssembly)  
- 📱 pensada primeiro para **Android**  
- ⚡ leve e portátil (o mesmo `.wasm` pode rodar em vários hosts)  

Você escreve a lógica da cápsula (por exemplo em Rust), gera um `.wasm`, descreve tudo em um **manifesto CAELES**, e o **núcleo CAELES** cuida de carregar e executar.

---

## 🧩 Conceitos principais

### Cápsula

Uma **cápsula CAELES** é a unidade básica do sistema.  
Ela é composta por:

- `capsule.wasm` – binário WebAssembly (`wasm32-unknown-unknown`)  
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

fornece as funções de host do CAELES (log, notify, etc.)

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
   (wasm32-unknown-unknown)
No Android, o CAELES deve ser embutido em um app host, que chama o núcleo nativo.

Em desktop, o núcleo pode ser usado para desenvolvimento, debug e testes de cápsulas.

🚦 Estado atual
🚧 Projeto em fase inicial (experimento).

Objetivos desta fase:

definir o conceito de cápsula CAELES v0

experimentar o núcleo em Rust executando uma cápsula simples (wasm32-unknown-unknown)

preparar o caminho para uma futura integração com Android

A API, o formato de manifesto e a estrutura do código ainda podem mudar bastante.

🧪 Visão de uso (futuro)
Fluxo esperado para desenvolvedores:

Escrever a cápsula em Rust (ou outra linguagem que compile para WASM):

rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown
Isso gera algo como:

target/wasm32-unknown-unknown/debug/minha-capsula.wasm
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

caeles-runtime --manifest path/para/capsule.manifest.json
Ou, no Android, via um app host que lista e executa cápsulas.

🤝 Contribuição
No momento, o foco é:

consolidar os conceitos (cápsula, manifesto, núcleo)

evoluir o código inicial em Rust

documentar decisões e ideias neste repositório

> ℹ️ **Estado atual:** o runtime CAELES ainda **não** embute WASI. As cápsulas devem ser
> compiladas para `wasm32-unknown-unknown` e usar apenas as funções de host expostas
> pelo runtime (ex.: `host_log`, `host_notify`). Caso precise de WASI, será necessário
> estender o runtime com o suporte adequado.

Sugestões de arquitetura, formato de manifesto, nomes de conceitos e ideias de cápsulas são bem-vindas via issues.
