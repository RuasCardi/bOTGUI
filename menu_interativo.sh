#!/bin/bash

# Cores
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

show_menu() {
    clear
    echo "======================================================================"
    echo "          🤖 RoboTrading - Sistema de Trading Automatizado"
    echo "======================================================================"
    echo ""
    echo "1. 📊 Executar Backtest (teste histórico)"
    echo "2. 🎲 Simulação Monte Carlo (10.000 cenários)"
    echo "3. 📄 Ver resultados do último backtest"
    echo "4. 📈 Ver resultados da última simulação"
    echo "5. 🧪 Rodar todos os testes unitários"
    echo "0. ❌ Sair"
    echo ""
}

run_backtest() {
    echo ""
    echo -e "${BLUE}⏳ Executando backtest...${NC}"
    echo ""
    cargo run --release --bin robo-trading backtest
    local exit_code=$?
    echo ""
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✅ Backtest concluído! Resultados salvos em backtest_results.json${NC}"
    else
        echo -e "${RED}❌ Erro ao executar backtest${NC}"
    fi
}

run_montecarlo() {
    echo ""
    echo -e "${BLUE}⏳ Executando Monte Carlo (pode demorar alguns segundos)...${NC}"
    echo ""
    cargo run --release --bin robo-trading monte-carlo
    local exit_code=$?
    echo ""
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✅ Simulação concluída! Resultados salvos em montecarlo_results.json${NC}"
    else
        echo -e "${RED}❌ Erro ao executar simulação${NC}"
    fi
}

show_backtest() {
    echo ""
    if [ ! -f backtest_results.json ]; then
        echo -e "${RED}❌ Nenhum backtest encontrado.${NC}"
        echo "Execute a opção 1 primeiro para gerar resultados."
        return
    fi

    echo "======================================================================"
    echo "📊 RESULTADOS DO ÚLTIMO BACKTEST"
    echo "======================================================================"
    echo ""
    
    if command -v jq &> /dev/null; then
        # Se tem jq instalado, usa formatação bonita
        jq -r '.metrics | 
            "💰 Retorno Total:        \(.total_return_percent)%",
            "💵 P&L Total:            $\(.total_pnl)",
            "",
            "📈 Total de Trades:      \(.total_trades)",
            "✅ Trades Vencedores:    \(.winning_trades) (\(.win_rate * 100 | floor)%)",
            "❌ Trades Perdedores:    \(.losing_trades)",
            "",
            "🎯 Performance:",
            "   • Profit Factor:      \(.profit_factor)",
            "   • Expectativa:        $\(.expectancy)",
            "   • Ganho Médio:        $\(.avg_win)",
            "   • Perda Média:        $\(.avg_loss | fabs)",
            "",
            "⚠️  Risco:",
            "   • Max Drawdown:       \(.max_drawdown_percent)%",
            "   • Sharpe Ratio:       \(.sharpe_ratio)",
            "   • Sortino Ratio:      \(.sortino_ratio)",
            "   • Calmar Ratio:       \(.calmar_ratio)"
        ' backtest_results.json
    else
        # Se não tem jq, mostra o JSON bruto (mas legível)
        cat backtest_results.json | grep -E '"total_return_percent"|"total_pnl"|"total_trades"|"win_rate"|"sharpe_ratio"' | sed 's/,$//' | sed 's/^[ \t]*//'
    fi
    
    echo ""
    echo "======================================================================"
}

show_montecarlo() {
    echo ""
    if [ ! -f montecarlo_results.json ]; then
        echo -e "${RED}❌ Nenhuma simulação encontrada.${NC}"
        echo "Execute a opção 2 primeiro para gerar resultados."
        return
    fi

    echo "======================================================================"
    echo "🎲 RESULTADOS DA SIMULAÇÃO MONTE CARLO"
    echo "======================================================================"
    echo ""
    
    if command -v jq &> /dev/null; then
        jq -r '
            "🎲 Simulações:           \(.simulations)",
            "💰 Capital Inicial:      $\(.initial_capital)",
            "",
            "💎 Projeções:",
            "   • Retorno Esperado:   $\(.expected_return | floor)",
            "   • Melhor Cenário:     $\(.best_case | floor)",
            "   • Pior Cenário:       $\(.worst_case | floor)",
            "",
            "🎯 Probabilidades:",
            "   • Chance de Lucro:    \(.profit_probability * 100 | floor)%",
            "   • Max Drawdown:       \(.max_drawdown * 100 | floor * 10 / 10)%"
        ' montecarlo_results.json
    else
        cat montecarlo_results.json | grep -E '"expected_return"|"profit_probability"|"best_case"|"worst_case"' | sed 's/,$//' | sed 's/^[ \t]*//'
    fi
    
    echo ""
    echo "======================================================================"
}

run_tests() {
    echo ""
    echo -e "${BLUE}⏳ Executando testes unitários...${NC}"
    echo ""
    cargo test --release
    local exit_code=$?
    echo ""
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✅ Todos os testes passaram!${NC}"
    else
        echo -e "${RED}❌ Alguns testes falharam${NC}"
    fi
}

# Loop principal
while true; do
    show_menu
    read -p "➤ Escolha uma opção: " choice
    
    case $choice in
        1)
            run_backtest
            ;;
        2)
            run_montecarlo
            ;;
        3)
            show_backtest
            ;;
        4)
            show_montecarlo
            ;;
        5)
            run_tests
            ;;
        0)
            echo ""
            echo -e "${GREEN}✅ Até logo! 👋${NC}"
            echo ""
            exit 0
            ;;
        *)
            echo ""
            echo -e "${RED}❌ Opção inválida! Escolha um número de 0 a 5.${NC}"
            ;;
    esac
    
    echo ""
    read -p "Pressione ENTER para voltar ao menu..."
done
