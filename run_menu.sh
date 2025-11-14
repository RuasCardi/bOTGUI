#!/bin/bash

clear

echo "======================================================================"
echo "          🤖 RoboTrading - Menu Interativo"
echo "======================================================================"
echo ""
echo "1. 📊 Executar Backtest"
echo "2. 🎲 Simulação Monte Carlo"
echo "3. 📄 Ver último backtest"
echo "4. 📈 Ver última simulação"
echo "0. ❌ Sair"
echo ""
read -p "➤ Escolha uma opção: " choice

case $choice in
    1)
        echo ""
        echo "⏳ Executando backtest..."
        cargo run --release backtest
        ;;
    2)
        echo ""
        echo "⏳ Executando Monte Carlo..."
        cargo run --release monte-carlo
        ;;
    3)
        echo ""
        if [ -f backtest_results.json ]; then
            echo "📊 ÚLTIMO BACKTEST"
            echo "======================================================================"
            jq -r '.metrics | "💰 Capital Final: \(.total_return_percent)%\n📈 Win Rate: \(.win_rate * 100)%\n🎯 Total Trades: \(.total_trades)\n✅ Vencedores: \(.winning_trades)\n❌ Perdedores: \(.losing_trades)\n💵 P&L Total: $\(.total_pnl)\n📊 Sharpe Ratio: \(.sharpe_ratio)\n⚠️  Max Drawdown: \(.max_drawdown_percent)%"' backtest_results.json
            echo ""
            echo "======================================================================"
        else
            echo "❌ Nenhum backtest anterior encontrado."
            echo "Execute a opção 1 primeiro."
        fi
        ;;
    4)
        echo ""
        if [ -f montecarlo_results.json ]; then
            echo "🎲 ÚLTIMA SIMULAÇÃO MONTE CARLO"
            echo "======================================================================"
            jq -r '"🎲 Simulações: \(.simulations)\n💰 Retorno Esperado: $\(.expected_return)\n🎯 Probabilidade de Lucro: \(.profit_probability * 100)%\n📈 Melhor Caso: $\(.best_case)\n📉 Pior Caso: $\(.worst_case)\n⚠️  Max Drawdown: \(.max_drawdown * 100)%"' montecarlo_results.json
            echo ""
            echo "======================================================================"
        else
            echo "❌ Nenhuma simulação anterior encontrada."
            echo "Execute a opção 2 primeiro."
        fi
        ;;
    0)
        echo ""
        echo "✅ Até logo! 👋"
        exit 0
        ;;
    *)
        echo ""
        echo "❌ Opção inválida!"
        ;;
esac

echo ""
read -p "Pressione ENTER para continuar..."
exec "$0"
