# 🎯 GUIA DE OTIMIZAÇÃO DE ESTRATÉGIAS

## 📊 Análise de Performance

### Como Interpretar os Resultados do Backtest

#### 1. Win Rate (Taxa de Acerto)
- **< 40%**: Estratégia ruim, revise a lógica
- **40-50%**: Aceitável se profit factor for bom
- **50-60%**: Bom
- **> 60%**: Excelente (mas cuidado com overfitting!)

#### 2. Profit Factor
```
Profit Factor = Ganhos Totais / Perdas Totais
```
- **< 1.0**: Perdendo dinheiro
- **1.0-1.5**: Margem muito estreita
- **1.5-2.0**: Bom
- **> 2.0**: Excelente

#### 3. Sharpe Ratio
```
Sharpe = (Retorno Médio - Taxa Livre de Risco) / Desvio Padrão
```
- **< 0**: Pior que deixar parado
- **0-1**: Ruim
- **1-2**: Bom
- **> 2**: Excelente

#### 4. Max Drawdown
Maior queda do pico até o vale:
- **< 10%**: Conservador
- **10-20%**: Aceitável
- **20-30%**: Arriscado
- **> 30%**: Muito arriscado

---

## 🔧 Otimização de Parâmetros

### Estratégia EMA + RSI + ATR

#### Parâmetros Padrão
```env
EMA_FAST=20
EMA_SLOW=50
RSI_PERIOD=14
ATR_PERIOD=14
ATR_THRESHOLD_MULTIPLIER=1.2
```

#### Teste Estas Variações

**Para Mercados de Alta Volatilidade (Crypto):**
```env
EMA_FAST=10
EMA_SLOW=30
RSI_PERIOD=7
ATR_THRESHOLD_MULTIPLIER=1.5
```

**Para Mercados Laterais:**
```env
EMA_FAST=15
EMA_SLOW=40
RSI_PERIOD=21
ATR_THRESHOLD_MULTIPLIER=0.8
```

**Para Scalping (trades rápidos):**
```env
EMA_FAST=8
EMA_SLOW=21
RSI_PERIOD=7
STOP_LOSS_ATR_MULTIPLIER=1.0
TAKE_PROFIT_ATR_MULTIPLIER=1.5
```

**Para Position Trading (trades longos):**
```env
EMA_FAST=50
EMA_SLOW=200
RSI_PERIOD=21
STOP_LOSS_ATR_MULTIPLIER=3.0
TAKE_PROFIT_ATR_MULTIPLIER=5.0
```

---

## 📈 Grid Search (Busca Sistemática)

### Script de Otimização

Crie um arquivo `optimize.sh`:

```bash
#!/bin/bash

echo "Otimizando parâmetros da estratégia..."

# Arrays de valores para testar
ema_fast=(10 15 20 25)
ema_slow=(30 40 50 60)
rsi_period=(7 14 21)

best_sharpe=0
best_params=""

for ef in "${ema_fast[@]}"; do
  for es in "${ema_slow[@]}"; do
    for rsi in "${rsi_period[@]}"; do
      if [ $ef -lt $es ]; then
        echo "Testando: EMA_FAST=$ef EMA_SLOW=$es RSI=$rsi"
        
        # Atualiza .env
        sed -i "s/^EMA_FAST=.*/EMA_FAST=$ef/" .env
        sed -i "s/^EMA_SLOW=.*/EMA_SLOW=$es/" .env
        sed -i "s/^RSI_PERIOD=.*/RSI_PERIOD=$rsi/" .env
        
        # Executa backtest e extrai Sharpe
        result=$(cargo run --release backtest 2>&1 | grep "Sharpe Ratio")
        sharpe=$(echo $result | grep -oP '\d+\.\d+')
        
        echo "  -> Sharpe: $sharpe"
        
        # Salva se for melhor
        if (( $(echo "$sharpe > $best_sharpe" | bc -l) )); then
          best_sharpe=$sharpe
          best_params="EMA_FAST=$ef EMA_SLOW=$es RSI=$rsi"
          echo "  ✅ NOVO MELHOR!"
        fi
      fi
    done
  done
done

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║         OTIMIZAÇÃO CONCLUÍDA                             ║"
echo "╠══════════════════════════════════════════════════════════╣"
echo "║ Melhor Sharpe: $best_sharpe                              ║"
echo "║ Parâmetros: $best_params                                 ║"
echo "╚══════════════════════════════════════════════════════════╝"
```

**⚠️ ATENÇÃO**: Grid search pode levar a **overfitting**! Sempre valide em dados out-of-sample.

---

## 🧮 Walk-Forward Analysis

### Processo Recomendado

1. **Divida os dados** em períodos:
   - Training: 70% (otimização)
   - Validation: 15% (validação)
   - Test: 15% (teste final)

2. **Otimize** nos dados de training

3. **Valide** nos dados de validation

4. **Teste** nos dados de test (apenas UMA VEZ!)

5. Se resultados forem consistentes, **vá para paper trading**

---

## 📊 Análise de Regime de Mercado

### Detectar Regime Atual

O robô já detecta 3 regimes:
- **Uptrend**: EMA rápida > EMA lenta
- **Downtrend**: EMA rápida < EMA lenta
- **Sideways**: EMAs próximas

### Ajustar Estratégia por Regime

**Ideia**: Use parâmetros diferentes para cada regime

```rust
// Em strategy.rs, adicione:
let params = match regime.trend.as_str() {
    "uptrend" => (20, 50, 1.5),   // (ema_fast, ema_slow, atr_mult)
    "downtrend" => (15, 40, 2.0),
    "sideways" => (10, 30, 0.8),
    _ => (20, 50, 1.2),
};
```

---

## 🎲 Validação com Monte Carlo

### Interpretação dos Resultados

**Exemplo de resultado bom:**
```
Probabilidade de Lucro: 75%
VaR 95%: $500 (de $10,000)
Max DD Esperado: 8%
```

**Exemplo de resultado ruim:**
```
Probabilidade de Lucro: 45%
VaR 95%: $3,000 (de $10,000)
Max DD Esperado: 35%
```

### Quando Confiar

✅ **Confie** se:
- Prob. lucro > 60%
- VaR < 20% do capital
- Max DD < 25%
- Resultados consistentes entre simulações

❌ **Não confie** se:
- Grande variação entre simulações
- VaR muito alto
- Drawdown esperado inaceitável

---

## 💡 Dicas de Otimização

### 1. Filtre por Volume
```rust
// Só opere em alta liquidez
if candle.volume < avg_volume * 1.5 {
    return None; // Ignora sinal
}
```

### 2. Adicione Filtro de Horário
```rust
// Evite horários de baixa liquidez
let hour = chrono::Utc::now().hour();
if hour < 6 || hour > 22 {
    return None; // Mercado quieto
}
```

### 3. Combine com Outros Indicadores
```rust
// Exemplo: adicione MACD
let macd = calculate_macd(&closes, 12, 26, 9);
if macd.histogram > 0.0 {
    // Confirma tendência de alta
}
```

### 4. Use Machine Learning (Avançado)
```rust
// Calcule probabilidade de sucesso
let win_prob = calculate_trade_probability(&signal);
if win_prob < 0.6 {
    return None; // Confiança baixa
}
```

---

## 🔄 Processo de Melhoria Contínua

### Workflow Recomendado

```
1. Coleta dados (3-6 meses) ───┐
                                │
2. Backtest inicial ────────────┤
                                │
3. Análise de resultados ───────┤
                                │
4. Ajuste parâmetros ───────────┤
                                │
5. Walk-forward test ───────────┤
                                │
6. Monte Carlo ─────────────────┤
                                │
7. Paper trading (2 semanas) ───┤
                                │
8. Live com capital mínimo ─────┤
                                │
9. Monitora 1 mês ──────────────┤
                                │
10. Escala gradualmente ────────┘
```

### Métricas para Acompanhar

Crie um arquivo `metrics.csv`:
```csv
Data,Trades,WinRate,ProfitFactor,Sharpe,MaxDD,PnL
2024-11-01,5,60%,1.8,1.4,3%,+150
2024-11-02,3,66%,2.1,1.6,2%,+200
...
```

---

## 🚨 Red Flags (Sinais de Alerta)

### Quando PARAR e Revisar

❌ **Win rate cai abruptamente** (ex: 60% → 40%)
- Mercado mudou de regime
- Estratégia não funciona mais
- **Ação**: Pause e analise

❌ **Drawdown > 15%**
- Risco muito alto
- **Ação**: Reduza posições imediatamente

❌ **Série de 5+ perdas consecutivas**
- Azar ou problema sistêmico?
- **Ação**: Pare por 24h, analise

❌ **Slippage alto** (> 0.5%)
- Liquidez baixa
- **Ação**: Troque de par ou reduza tamanho

---

## 📚 Recursos para Aprofundamento

### Livros Recomendados
- "Algorithmic Trading" - Ernest Chan
- "Advances in Financial Machine Learning" - Marcos López de Prado
- "Evidence-Based Technical Analysis" - David Aronson

### Indicadores Adicionais para Testar
- Bollinger Bands
- MACD
- Stochastic
- ADX (Average Directional Index)
- Volume Profile

### Websites Úteis
- QuantConnect (backtesting)
- TradingView (análise visual)
- Investopedia (conceitos)

---

## ✅ Checklist de Validação

Antes de ir para produção:

- [ ] Backtest em 12+ meses
- [ ] Win rate > 50%
- [ ] Profit factor > 1.5
- [ ] Sharpe > 1.0
- [ ] Max DD < 20%
- [ ] Walk-forward passou
- [ ] Monte Carlo confiável
- [ ] Paper trading 2+ semanas
- [ ] Slippage aceitável
- [ ] Sem bugs em 1 semana

---

**Lembre-se**: **A melhor otimização é SIMPLICIDADE + ROBUSTEZ**

Não complique demais. Uma estratégia simples e robusta vence uma complexa e frágil.

🚀 **Boa sorte na otimização!**
