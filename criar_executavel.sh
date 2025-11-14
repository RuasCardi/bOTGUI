#!/bin/bash

echo "======================================================================"
echo "     🚀 Criando Pacote Executável do RoboTrading"
echo "======================================================================"
echo ""

# Criar pasta para distribuição
DIST_DIR="RoboTrading_Executavel"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

echo "✅ Copiando executável..."
cp target/release/robo-trading "$DIST_DIR/"

echo "✅ Copiando arquivo de configuração..."
cp .env "$DIST_DIR/.env.exemplo"

echo "✅ Copiando documentação..."
cp COMO_USAR.md "$DIST_DIR/"
cp GUIA_RAPIDO.md "$DIST_DIR/"
cp README.md "$DIST_DIR/"

echo "✅ Criando script de início rápido..."
cat > "$DIST_DIR/INICIAR.sh" << 'EOF'
#!/bin/bash

clear
echo "======================================================================"
echo "          🤖 RoboTrading - Início Rápido"
echo "======================================================================"
echo ""

if [ ! -f ".env" ]; then
    echo "⚠️  ATENÇÃO: Arquivo .env não encontrado!"
    echo ""
    echo "📝 Passos para começar:"
    echo "1. Renomeie '.env.exemplo' para '.env'"
    echo "2. Edite o .env e coloque suas chaves da Binance"
    echo "3. Execute este script novamente"
    echo ""
    read -p "Deseja criar .env agora? (s/n): " resposta
    if [ "$resposta" = "s" ]; then
        cp .env.exemplo .env
        echo ""
        echo "✅ Arquivo .env criado!"
        echo "Agora edite-o com suas chaves:"
        echo "  nano .env"
        echo ""
    fi
    exit 0
fi

echo "Escolha uma opção:"
echo ""
echo "1. 📊 Executar Backtest (teste em dados históricos)"
echo "2. 🎲 Monte Carlo (10.000 simulações)"
echo "3. 🔴 Operar AO VIVO no Testnet (dinheiro fake)"
echo "4. 💰 Operar AO VIVO REAL (CUIDADO!)"
echo "0. ❌ Sair"
echo ""
read -p "➤ Sua escolha: " choice

case $choice in
    1)
        echo ""
        echo "⏳ Executando backtest..."
        ./robo-trading backtest
        ;;
    2)
        echo ""
        echo "⏳ Executando Monte Carlo..."
        ./robo-trading monte-carlo
        ;;
    3)
        echo ""
        if grep -q "USE_TESTNET=true" .env; then
            echo "✅ Modo TESTNET ativado (seguro)"
            echo "🚀 Iniciando robô..."
            ./robo-trading live
        else
            echo "⚠️  ATENÇÃO: USE_TESTNET=false no .env"
            echo "Mude para 'true' antes de testar!"
        fi
        ;;
    4)
        echo ""
        echo "⚠️  ⚠️  ⚠️  MODO REAL - DINHEIRO DE VERDADE! ⚠️  ⚠️  ⚠️"
        echo ""
        read -p "TEM CERTEZA? (digite SIM em maiúsculas): " confirm
        if [ "$confirm" = "SIM" ]; then
            ./robo-trading live
        else
            echo "Operação cancelada."
        fi
        ;;
    0)
        echo ""
        echo "👋 Até logo!"
        exit 0
        ;;
    *)
        echo ""
        echo "❌ Opção inválida!"
        ;;
esac

echo ""
read -p "Pressione ENTER para sair..."
EOF

chmod +x "$DIST_DIR/INICIAR.sh"
chmod +x "$DIST_DIR/robo-trading"

echo "✅ Criando arquivo README..."
cat > "$DIST_DIR/LEIA-ME.txt" << 'EOF'
╔══════════════════════════════════════════════════════════╗
║         🤖 RoboTrading - Executável Standalone          ║
╚══════════════════════════════════════════════════════════╝

📦 CONTEÚDO DESTE PACOTE:

  robo-trading       - Executável principal
  INICIAR.sh         - Script de início rápido (USE ESTE!)
  .env.exemplo       - Arquivo de configuração modelo
  COMO_USAR.md       - Guia completo passo a passo
  GUIA_RAPIDO.md     - Referência rápida
  README.md          - Documentação técnica

🚀 INÍCIO RÁPIDO (3 passos):

  1. Renomeie ".env.exemplo" para ".env"
  2. Edite .env com suas chaves da Binance
  3. Execute: ./INICIAR.sh

📖 LEIA PRIMEIRO: COMO_USAR.md
   (tem tudo explicado, passo a passo)

⚠️  IMPORTANTE:

  • Comece SEMPRE no TESTNET (dinheiro fake)
  • Nunca ative "withdrawals" na API da Binance
  • Teste pelo menos 1 semana antes de usar real
  • Mercado de cripto é VOLÁTIL!

💡 SUPORTE:

  • Leia COMO_USAR.md para dúvidas
  • Leia logs em: logs/trading.log.*
  • Resultados em: backtest_results.json

📊 PRIMEIRO TESTE (sem risco):

  ./INICIAR.sh
  Escolha opção 1 (Backtest)

═══════════════════════════════════════════════════════════

Boa sorte! 🍀
EOF

echo ""
echo "======================================================================"
echo "✅ Pacote criado com sucesso em: $DIST_DIR/"
echo "======================================================================"
echo ""
echo "📦 Conteúdo:"
ls -lh "$DIST_DIR/"
echo ""
echo "🎯 Para usar:"
echo "   cd $DIST_DIR"
echo "   ./INICIAR.sh"
echo ""
echo "📚 Para distribuir:"
echo "   tar -czf RoboTrading.tar.gz $DIST_DIR/"
echo "   ou"
echo "   zip -r RoboTrading.zip $DIST_DIR/"
echo ""
