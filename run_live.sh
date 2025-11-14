#!/bin/bash
# Inicia trading ao vivo

echo "⚠️  ════════════════════════════════════════════════════════"
echo "⚠️  ATENÇÃO: VOCÊ ESTÁ INICIANDO O ROBÔ EM MODO PRODUÇÃO!"
echo "⚠️  ════════════════════════════════════════════════════════"
echo ""
echo "📋 Checklist pré-inicialização:"
echo "   [ ] Li e segui o PRODUCTION.md?"
echo "   [ ] Testei extensivamente em testnet?"
echo "   [ ] Rodei paper trading por 2+ semanas?"
echo "   [ ] Configurei capital e risk management corretamente?"
echo "   [ ] Estou preparado para monitorar o bot?"
echo ""
read -p "Confirma que todas as verificações foram feitas? (sim/NÃO): " confirm

if [ "$confirm" != "sim" ]; then
    echo ""
    echo "❌ Operação cancelada. Faça o checklist antes de continuar."
    exit 1
fi

echo ""
read -p "Digite 'INICIAR' para confirmar novamente: " confirm2

if [ "$confirm2" != "INICIAR" ]; then
    echo ""
    echo "❌ Operação cancelada."
    exit 1
fi

echo ""
echo "🚀 Iniciando RoboTrading em modo LIVE..."
echo "⚠️  Use Ctrl+C para parar"
echo ""

cargo run --release live
