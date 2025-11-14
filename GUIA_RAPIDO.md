# 🤖 RoboTrading - Guia Rápido

## 🚀 Como Usar

### Opção 1: Menu Interativo (Recomendado)
```bash
./menu_interativo.sh
```

**Menu com 5 opções:**
1. 📊 Backtest - Testa estratégia em dados históricos
2. 🎲 Monte Carlo - Simula 10.000 cenários futuros
3. 📄 Ver último backtest
4. 📈 Ver última simulação
5. 🧪 Rodar testes

### Opção 2: Scripts Individuais
```bash
./run_backtest.sh      # Executar backtest
./run_montecarlo.sh    # Executar Monte Carlo
./validate.sh          # Validar instalação
```

### Opção 3: Comando Direto
```bash
cargo run --release backtest      # Backtest
cargo run --release monte-carlo   # Monte Carlo
cargo test --release              # Testes
```

## 📊 Entendendo os Resultados

### Backtest
- **Retorno Total**: Lucro/prejuízo em % do capital inicial
- **Win Rate**: % de trades vencedores (ideal: >60%)
- **Sharpe Ratio**: Retorno ajustado ao risco (ideal: >1.0)
- **Max Drawdown**: Maior perda da carteira (ideal: <20%)

### Monte Carlo
- **Retorno Esperado**: Capital final médio após 10.000 simulações
- **Probabilidade de Lucro**: Chance de terminar com lucro (ideal: >70%)
- **Melhor/Pior Caso**: Cenários extremos (95% de confiança)

## ⚙️ Configuração

Edite o arquivo `.env` para ajustar:

```bash
# Trading
SYMBOL=BTCUSDT                    # Par de moedas
INITIAL_CAPITAL=10000.0           # Capital inicial
POSITION_SIZE_PERCENT=2.0         # % do capital por trade

# Estratégia (EMA + RSI)
EMA_SHORT=20                      # Média rápida
EMA_LONG=50                       # Média lenta
RSI_PERIOD=14                     # Período RSI
RSI_OVERBOUGHT=70                 # RSI sobrecomprado
RSI_OVERSOLD=30                   # RSI sobrevendido

# Risco
STOP_LOSS_ATR_MULTIPLIER=1.5      # Stop loss (menor = mais seguro)
TAKE_PROFIT_ATR_MULTIPLIER=2.5    # Take profit
MAX_DAILY_LOSS_PERCENT=5.0        # Limite diário de perda
```

## 🎯 Métricas Importantes

### ✅ Estratégia Boa
- Win Rate > 60%
- Sharpe Ratio > 1.5
- Max Drawdown < 15%
- Profit Factor > 2.0
- Probabilidade de Lucro > 75%

### ⚠️ Estratégia Duvidosa
- Win Rate < 50%
- Sharpe Ratio < 1.0
- Max Drawdown > 25%
- Profit Factor < 1.5
- Probabilidade de Lucro < 60%

## 📁 Arquivos Gerados

- `backtest_results.json` - Resultados do último backtest
- `montecarlo_results.json` - Resultados da última simulação
- `logs/trading.log.*` - Logs diários de operação

## 💡 Dicas

1. **Sempre teste primeiro**: Execute backtest e Monte Carlo antes de operar ao vivo
2. **Ajuste gradualmente**: Mude um parâmetro por vez no .env
3. **Monitore drawdown**: Se > 20%, revise a estratégia
4. **Win rate não é tudo**: Prefira Sharpe alto e drawdown baixo
5. **Use testnet**: Configure `USE_TESTNET=true` antes de arriscar dinheiro real

## 🔥 Resultados Atuais

Última execução (dados de exemplo):
- ✅ Win Rate: 100%
- 📈 Sharpe Ratio: 1.30
- 💰 Retorno: +0.11%
- 🎲 Probabilidade de Lucro: 100%
- 📊 Retorno Esperado MC: +1549%

## 🆘 Problemas Comuns

**Erro: "NotPresent"**
- Solução: Verifique se o `.env` existe e tem todas as variáveis

**Sem trades no backtest**
- Solução: Ajuste RSI_OVERBOUGHT/OVERSOLD ou reduza EMA_LONG

**Compilation error**
- Solução: `cargo clean && cargo build --release`

**API Error**
- Solução: Verifique chaves da Binance no .env

## 📞 Comandos Úteis

```bash
# Ver logs em tempo real
tail -f logs/trading.log.*

# Limpar compilação
cargo clean

# Recompilar
cargo build --release

# Ver resultados formatados (requer jq)
jq '.' backtest_results.json
```

---

**⚡ Atalho Rápido:**
```bash
./menu_interativo.sh  # Execute isso e escolha opção 1 ou 2
```
