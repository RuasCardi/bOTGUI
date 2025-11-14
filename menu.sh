#!/bin/bash

# Script interativo simples para RoboTrading

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
cd "$SCRIPT_DIR"

# Cores para o terminal
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # Sem cor

mostrar_menu() {
    clear
    echo "======================================================================"
    echo "         🤖 RoboTrading - Sistema de Trading Automatizado"
    echo "======================================================================"
    echo ""
    echo "📋 MENU DE OPÇÕES"
    echo ""
    echo "  1. 📊 Executar Backtest (teste histórico)"
    echo "  2. 🎲 Simulação Monte Carlo (projeções futuras)"
    echo "  3. 📄 Ver último backtest"
    echo "  4. 📈 Ver última simulação"
    echo "  5. 🧪 Rodar testes unit��rios"
    echo "  0. ❌ Sair"
    echo ""
    echo -n "➤ Escolha uma opção: "
}

ver_backtest() {
    echo ""
    if [ -f "backtest_results.json" ]; then
        echo -e "${GREEN}📊 ÚLTIMO BACKTEST${NC}"
        echo "======================================================================"
        echo ""
        
        # Extrair métricas principais (requires jq)
        if command -v jq &> /dev/null; then
            TOTAL_RETURN=$(jq '.metrics.total_return_percent' backtest_results.json 2>/dev/null || echo "N/A")
            TOTAL_TRADES=$(jq '.metrics.total_trades' backtest_results.json 2>/dev/null || echo "N/A")
            WIN_RATE=$(jq '.metrics.win_rate * 100' backtest_results.json 2>/dev/null || echo "N/A")
            SHARPE=$(jq '.metrics.sharpe_ratio' backtest_results.json 2>/dev/null || echo "N/A")
            
            echo "💰 Retorno Total: ${TOTAL_RETURN}%"
            echo "📈 Total de Trades: ${TOTAL_TRADES}"
            echo "✅ Win Rate: ${WIN_RATE}%"
            echo "📊 Sharpe Ratio: ${SHARPE}"
        else
            echo "⚠️  Instale 'jq' para visualizar os detalhes: sudo apt-get install jq"
            echo ""
            echo "Resumo bruto:"
            head -30 backtest_results.json
        fi
        
        echo ""
        echo "Arquivo completo: backtest_results.json"
    else
        echo -e "${RED}❌ Nenhum backtest encontrado${NC}"
        echo "Execute a opção 1 primeiro."
    fi
    echo ""
}

ver_montecarlo() {
    echo ""
    if [ -f "montecarlo_results.json" ]; then
        echo -e "${GREEN}🎲 ÚLTIMA SIMULAÇÃO MONTE CARLO${NC}"
        echo "======================================================================"
        echo ""
        
        if command -v jq &> /dev/null; then
            EXPECTED=$(jq '.expected_return' montecarlo_results.json 2>/dev/null || echo "N/A")
            BEST=$(jq '.best_case' montecarlo_results.json 2>/dev/null || echo "N/A")
            WORST=$(jq '.worst_case' montecarlo_results.json 2>/dev/null || echo "N/A")
            
            echo "💰 Retorno Esperado: \$${EXPECTED}"
            echo "🎯 Melhor Cenário: \$${BEST}"
            echo "⚠️  Pior Cenário: \$${WORST}"
        else
            echo "⚠️  Instale 'jq' para visualizar os detalhes: sudo apt-get install jq"
            echo ""
            echo "Resumo bruto:"
            head -20 montecarlo_results.json
        fi
        
        echo ""
        echo "Arquivo completo: montecarlo_results.json"
    else
        echo -e "${RED}❌ Nenhuma simulação encontrada${NC}"
        echo "Execute a opção 2 primeiro."
    fi
    echo ""
}

while true; do
    mostrar_menu
    read opcao
    
    case $opcao in
        1)
            clear
            echo -e "${BLUE}🔄 Executando Backtest...${NC}"
            echo "======================================================================"
            cargo run --release backtest
            echo ""
            echo "======================================================================"
            ;;
        2)
            clear
            echo -e "${BLUE}🔄 Executando Monte Carlo...${NC}"
            echo "======================================================================"
            cargo run --release monte-carlo
            echo ""
            echo "======================================================================"
            ;;
        3)
            clear
            ver_backtest
            ;;
        4)
            clear
            ver_montecarlo
            ;;
        5)
            clear
            echo -e "${BLUE}🧪 Executando Testes Unitários...${NC}"
            echo "======================================================================"
            cargo test --release
            echo ""
            echo "======================================================================"
            ;;
        0)
            clear
            echo -e "${GREEN}✅ Até logo! 👋${NC}"
            echo ""
            exit 0
            ;;
        *)
            clear
            echo -e "${RED}❌ Opção inválida!${NC}"
            sleep 1
            ;;
    esac
    
    if [ "$opcao" != "0" ]; then
        echo ""
        echo -n "Pressione ENTER para voltar ao menu..."
        read
    fi
done
