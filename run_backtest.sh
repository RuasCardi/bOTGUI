#!/bin/bash
# Executa backtest

echo "📊 Executando Backtest..."
echo ""

cargo run --release backtest

echo ""
echo "✅ Backtest concluído!"
echo "📄 Resultados salvos em: backtest_results.json"
