#!/bin/bash
# Script de inicialização do RoboTrading

set -e

echo "🤖 RoboTrading - Setup Script"
echo "=============================="
echo ""

# Verifica se Rust está instalado
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust não encontrado!"
    echo "📦 Instalando Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    echo "✅ Rust instalado com sucesso!"
else
    echo "✅ Rust já está instalado"
    rustc --version
fi

echo ""

# Cria diretório de logs
echo "📁 Criando diretórios..."
mkdir -p logs
mkdir -p data/historical
echo "✅ Diretórios criados"

echo ""

# Copia .env.example se .env não existir
if [ ! -f .env ]; then
    echo "📝 Criando arquivo .env..."
    cp .env.example .env
    echo "⚠️  IMPORTANTE: Edite o arquivo .env com suas credenciais!"
    echo "   Use: nano .env"
else
    echo "✅ Arquivo .env já existe"
fi

echo ""

# Compila o projeto
echo "🔨 Compilando projeto..."
cargo build --release
echo "✅ Compilação concluída!"

echo ""

# Executa testes
echo "🧪 Executando testes..."
cargo test --quiet
echo "✅ Testes passaram!"

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║              SETUP CONCLUÍDO COM SUCESSO!                ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "📋 Próximos passos:"
echo "   1. Edite o arquivo .env com suas credenciais"
echo "   2. Execute um backtest: ./run_backtest.sh"
echo "   3. Execute Monte Carlo: ./run_montecarlo.sh"
echo "   4. Leia PRODUCTION.md antes de ir para produção"
echo ""
echo "🚀 Comandos disponíveis:"
echo "   ./run_backtest.sh      - Executa backtest"
echo "   ./run_montecarlo.sh    - Simula Monte Carlo"
echo "   ./run_live.sh          - Inicia trading ao vivo"
echo ""
