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

> Dica rápida: o placeholder `<manifest>` deve ser substituído por um caminho real,
> por exemplo: `cargo run -p caeles-runtime -- --manifest capsules/hello-capsule/capsule.manifest.json`.

### Interface inicial para criar manifest

Use o assistente embutido para gerar rapidamente um manifesto compatível com o runtime:

```
cargo run -p caeles-runtime -- init --output capsule.manifest.json
```

O comando abre um passo a passo interativo pedindo ID, nome, versão, caminho do wasm (alvo `wasm32-unknown-unknown`) e permissões de notificações/rede. Você também pode preencher flags diretamente, por exemplo:

```
cargo run -p caeles-runtime -- init --id com.caeles.demo --name "Demo" --allow-notifications
```

### Interface web para criar manifest

Também é possível criar manifestos via navegador com a interface web mínima embutida no runtime:

```
cargo run -p caeles-runtime -- web --host 127.0.0.1 --port 8080
```

Abra o endereço informado (por padrão http://127.0.0.1:8080), preencha os campos e copie o JSON gerado. Ele já segue o formato esperado pelo runtime (alvo `wasm32-unknown-unknown` e permissões de host).

### Compilando cápsulas no Windows

Os erros de link envolvendo `host_log` e `host_notify` acontecem quando a cápsula é
compilada como DLL nativa (alvo padrão). A cápsula precisa ser gerada como WASM:

```
rustup target add wasm32-unknown-unknown
cargo build -p hello-capsule --target wasm32-unknown-unknown
cargo build -p logger-capsule --target wasm32-unknown-unknown
```

Depois de gerar o `.wasm`, aponte o campo `entry` do manifest para o caminho correto,
por exemplo:

```
capsules/hello-capsule/target/wasm32-unknown-unknown/debug/hello_capsule.wasm
```

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
