# Contribuindo para o CAELES

Obrigado por seu interesse em contribuir para o CAELES! 🎉

## Como Contribuir

### 1. Configurando o Ambiente

```bash
# Clone o repositório
git clone https://github.com/jardelva96/caeles.git
cd caeles

# Instale o Rust (se ainda não tiver)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Instale o target WASM
rustup target add wasm32-unknown-unknown

# Compile o projeto
make build-all

# Execute os testes
make test
```

### 2. Estrutura do Projeto

```
caeles/
├── crates/
│   ├── caeles-runtime/    # Runtime que executa cápsulas
│   └── caeles-sdk/        # SDK para criar cápsulas
├── capsules/              # Cápsulas de exemplo
│   ├── hello-capsule/
│   ├── logger-capsule/
│   └── registry.json      # Registry de cápsulas
├── Makefile               # Comandos de build
└── README.md
```

### 3. Fluxo de Trabalho

1. **Fork** o repositório
2. Crie uma **branch** para sua feature:
   ```bash
   git checkout -b feature/minha-funcionalidade
   ```
3. Faça suas alterações e **commite**:
   ```bash
   git commit -am 'Adiciona nova funcionalidade'
   ```
4. **Teste** suas mudanças:
   ```bash
   make test
   make run-hello
   ```
5. **Push** para sua branch:
   ```bash
   git push origin feature/minha-funcionalidade
   ```
6. Abra um **Pull Request**

### 4. Diretrizes de Código

#### Formatação

```bash
# Formate o código antes de commitar
make fmt
```

#### Linting

```bash
# Execute o linter
make lint
```

#### Testes

- Adicione testes para novas funcionalidades
- Certifique-se de que todos os testes passam:
  ```bash
  cargo test
  ```

#### Documentação

- Documente funções públicas com doc comments (`///`)
- Atualize README.md se necessário
- Adicione exemplos quando apropriado

### 5. Áreas para Contribuição

#### 🎯 Funcionalidades Prioritárias

- **Mais Host Functions**
  - Filesystem (leitura/escrita sandboxed)
  - Network (HTTP requests com permissões)
  - Time/Date functions
  - Crypto functions

- **Integração Android**
  - JNI bindings
  - App host de exemplo
  - Documentação de integração

- **Melhorias no Runtime**
  - Timeout configurável
  - Limits de memória ajustáveis
  - Métricas de performance
  - Logs estruturados

#### 🧪 Testes

- Testes de integração
- Testes de performance
- Testes de segurança
- Exemplos de cápsulas

#### 📚 Documentação

- Tutoriais
- Guias de uso
- Exemplos práticos
- Tradução de docs

### 6. Convenções de Commit

Use mensagens de commit claras e descritivas:

- `feat: adiciona suporte a filesystem`
- `fix: corrige leak de memória no runtime`
- `docs: atualiza README com novos exemplos`
- `test: adiciona testes para permissões`
- `refactor: reorganiza código do runtime`
- `perf: melhora performance de carregamento`

### 7. Reportando Bugs

Ao reportar bugs, inclua:

- Descrição clara do problema
- Passos para reproduzir
- Comportamento esperado vs. atual
- Versão do Rust e do CAELES
- Sistema operacional
- Logs relevantes

### 8. Sugerindo Features

Ao sugerir features, descreva:

- O problema que resolve
- Como deveria funcionar
- Exemplos de uso
- Por que seria útil

### 9. Revisão de Código

- Seja construtivo e respeitoso
- Foque no código, não na pessoa
- Explique o "porquê" das sugestões
- Reconheça boas práticas

### 10. Comunicação

- Use português ou inglês nas issues/PRs
- Seja claro e objetivo
- Peça ajuda quando necessário

## Código de Conduta

- Seja respeitoso com todos
- Aceite críticas construtivas
- Foque no que é melhor para o projeto
- Mostre empatia com outros contribuidores

## Dúvidas?

Abra uma issue com sua dúvida ou entre em contato com os mantenedores.

Obrigado por contribuir! 🚀
