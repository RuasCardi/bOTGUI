#!/bin/bash
# Executa simulação Monte Carlo

echo "🎲 Executando Simulação Monte Carlo..."
echo ""

cargo run --release monte-carlo

echo ""
echo "✅ Simulação concluída!"
echo "📄 Resultados salvos em: montecarlo_results.json"
