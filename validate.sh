#!/bin/bash
# Validação e estatísticas do projeto RoboTrading

echo "╔══════════════════════════════════════════════════════════╗"
echo "║       ROBOTRADING - VALIDAÇÃO DO PROJETO                ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Cores
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

check_passed=0
check_failed=0

check() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
        ((check_passed++))
    else
        echo -e "${RED}✗${NC} $2"
        ((check_failed++))
    fi
}

echo "📋 Verificando estrutura do projeto..."
echo ""

# Verifica arquivos principais
[ -f "Cargo.toml" ]
check $? "Cargo.toml existe"

[ -f ".env.example" ]
check $? ".env.example existe"

[ -f ".gitignore" ]
check $? ".gitignore existe"

[ -f "src/main.rs" ]
check $? "main.rs existe"

echo ""
echo "📚 Verificando documentação..."
echo ""

[ -f "README.md" ]
check $? "README.md existe"

[ -f "PRODUCTION.md" ]
check $? "PRODUCTION.md existe"

[ -f "OPTIMIZATION.md" ]
check $? "OPTIMIZATION.md existe"

[ -f "EXAMPLES.md" ]
check $? "EXAMPLES.md existe"

[ -f "SUMMARY.md" ]
check $? "SUMMARY.md existe"

echo ""
echo "🔧 Verificando módulos..."
echo ""

[ -d "src/api" ]
check $? "Módulo API existe"

[ -d "src/strategy" ]
check $? "Módulo Strategy existe"

[ -d "src/risk" ]
check $? "Módulo Risk existe"

[ -d "src/execution" ]
check $? "Módulo Execution existe"

[ -d "src/backtest" ]
check $? "Módulo Backtest existe"

[ -d "src/montecarlo" ]
check $? "Módulo Monte Carlo existe"

echo ""
echo "🚀 Verificando scripts..."
echo ""

[ -x "setup.sh" ]
check $? "setup.sh executável"

[ -x "run_backtest.sh" ]
check $? "run_backtest.sh executável"

[ -x "run_montecarlo.sh" ]
check $? "run_montecarlo.sh executável"

[ -x "run_live.sh" ]
check $? "run_live.sh executável"

echo ""
echo "🔍 Verificando compilação..."
echo ""

cargo check --quiet 2>&1 > /dev/null
check $? "Projeto compila sem erros"

echo ""
echo "📊 Estatísticas do Projeto"
echo "═══════════════════════════════════════════════════════════"

# Conta linhas de código
total_lines=$(find src -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
echo "  Linhas de código Rust: $total_lines"

# Conta arquivos
rust_files=$(find src -name "*.rs" | wc -l)
echo "  Arquivos Rust: $rust_files"

# Conta módulos
modules=$(find src -type d | wc -l)
echo "  Módulos: $((modules - 1))"

# Documentação
doc_files=$(ls -1 *.md 2>/dev/null | wc -l)
echo "  Arquivos de documentação: $doc_files"

doc_lines=$(cat *.md 2>/dev/null | wc -l)
echo "  Linhas de documentação: $doc_lines"

# Scripts
scripts=$(ls -1 *.sh 2>/dev/null | wc -l)
echo "  Scripts shell: $scripts"

echo ""
echo "═══════════════════════════════════════════════════════════"

# Calcula tamanho
size=$(du -sh . 2>/dev/null | awk '{print $1}')
echo "  Tamanho total do projeto: $size"

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║                  RESULTADO DA VALIDAÇÃO                  ║"
echo "╠══════════════════════════════════════════════════════════╣"
printf "║  %-55s  ║\n" "Checks passados: ${check_passed}"
printf "║  %-55s  ║\n" "Checks falhos: ${check_failed}"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

if [ $check_failed -eq 0 ]; then
    echo -e "${GREEN}✅ PROJETO 100% VÁLIDO E COMPLETO!${NC}"
    echo ""
    echo "🎉 Parabéns! O robô de trading está pronto para uso."
    echo ""
    echo "📋 Próximos passos:"
    echo "   1. Configure o arquivo .env"
    echo "   2. Execute: ./run_backtest.sh"
    echo "   3. Analise os resultados"
    echo "   4. Leia PRODUCTION.md antes de produção"
    echo ""
    exit 0
else
    echo -e "${RED}⚠️  ALGUNS CHECKS FALHARAM${NC}"
    echo ""
    echo "Execute o setup novamente:"
    echo "  ./setup.sh"
    echo ""
    exit 1
fi
