# CAELES Makefile
# Facilita a compilação e execução do projeto

.PHONY: help build build-release build-capsules clean test run-hello run-logger install-target

# Detecta o sistema operacional
UNAME_S := $(shell uname -s)

help: ## Mostra esta mensagem de ajuda
	@echo "CAELES - Motor de cápsulas WebAssembly"
	@echo ""
	@echo "Comandos disponíveis:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

install-target: ## Instala o target wasm32-unknown-unknown
	@echo "📦 Instalando target wasm32-unknown-unknown..."
	rustup target add wasm32-unknown-unknown

build: ## Compila o runtime em modo debug
	@echo "🔨 Compilando runtime..."
	cargo build --package caeles-runtime

build-release: ## Compila o runtime em modo release (otimizado)
	@echo "🔨 Compilando runtime (release)..."
	cargo build --release --package caeles-runtime

build-capsules: install-target ## Compila as cápsulas de exemplo como WASM
	@echo "🔨 Compilando cápsulas WASM..."
	cargo build --target wasm32-unknown-unknown --package hello-capsule --package logger-capsule

build-all: build build-capsules ## Compila tudo (runtime + cápsulas)
	@echo "✅ Build completo!"

test: ## Executa todos os testes
	@echo "🧪 Executando testes..."
	cargo test

run-hello: build build-capsules ## Executa a cápsula hello-capsule
	@echo "🚀 Executando hello-capsule..."
	./target/debug/caeles-runtime --capsule-id com.caeles.example.hello

run-logger: build build-capsules ## Executa a cápsula logger-capsule
	@echo "🚀 Executando logger-capsule..."
	./target/debug/caeles-runtime --capsule-id com.caeles.example.logger

clean: ## Remove arquivos de build
	@echo "🧹 Limpando arquivos de build..."
	cargo clean

check: ## Verifica o código sem compilar
	@echo "🔍 Verificando código..."
	cargo check --workspace

fmt: ## Formata o código
	@echo "✨ Formatando código..."
	cargo fmt --all

lint: ## Executa o linter (clippy)
	@echo "🔍 Executando linter..."
	cargo clippy --all-targets --all-features -- -D warnings

dev: build-all test ## Build completo + testes (para desenvolvimento)
	@echo "✅ Ambiente de desenvolvimento pronto!"

# Alias para facilitar
all: build-all ## Alias para build-all
