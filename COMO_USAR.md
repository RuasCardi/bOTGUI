# 🚀 COMO USAR O ROBOTRADING - GUIA COMPLETO

## 📦 EXECUTÁVEL JÁ ESTÁ PRONTO!

O executável está em: `target/release/robo-trading`

---

## 🎯 PASSO A PASSO COMPLETO

### **1️⃣ TESTAR SEM RISCO (COMECE AQUI!)**

Não precisa de conta Binance ainda. Apenas teste:

```bash
./menu_interativo.sh
```

Escolha **opção 1** (Backtest) para ver como o robô se comporta.

---

### **2️⃣ CRIAR CONTA NA BINANCE**

#### **Opção A: Conta Real (para operar de verdade)**
1. Acesse: https://www.binance.com/register
2. Crie sua conta
3. Faça verificação KYC (documentos)
4. Deposite dinheiro (comece com pouco, tipo R$500)

#### **Opção B: Testnet (para treinar - RECOMENDADO)**
1. Acesse: https://testnet.binance.vision/
2. Faça login com GitHub/Google
3. Receba dinheiro fake automático
4. **ZERO RISCO** - use para aprender!

---

### **3️⃣ PEGAR CHAVES API DA BINANCE**

#### **Se escolheu TESTNET (fake money):**
1. Vá em https://testnet.binance.vision/
2. Clique em "API Key"
3. Copie:
   - **API Key** (começa com algo como `abc123...`)
   - **Secret Key** (maior, tipo `xyz789...`)

#### **Se escolheu REAL (dinheiro de verdade):**
1. Binance.com → Perfil → API Management
2. Crie nova API Key
3. **IMPORTANTE - Configurações de segurança:**
   - ✅ Ative: "Enable Reading"
   - ✅ Ative: "Enable Spot & Margin Trading"
   - ❌ **NÃO ATIVE**: "Enable Withdrawals" (NUNCA!)
4. Configure IP whitelist (seu IP de casa)
5. Copie API Key e Secret Key

---

### **4️⃣ CONFIGURAR O ROBÔ**

Edite o arquivo `.env` na pasta do projeto:

```bash
nano .env
# ou
gedit .env
# ou
code .env
```

**Cole isso (ajuste suas chaves):**

```bash
# ============================================
# CHAVES DA BINANCE (COPIE DO PASSO 3)
# ============================================
BINANCE_API_KEY=sua_api_key_aqui
BINANCE_SECRET_KEY=sua_secret_key_aqui

# ============================================
# MODO DE OPERAÇÃO
# ============================================
# true = TESTNET (dinheiro fake - SEGURO)
# false = REAL (dinheiro de verdade - CUIDADO!)
USE_TESTNET=true

# ============================================
# CONFIGURAÇÕES DE TRADING
# ============================================
SYMBOL=BTCUSDT              # Par que vai operar (Bitcoin/USDT)
TIMEFRAME=1h                # Intervalo dos candles
INITIAL_CAPITAL=10000.0     # Capital inicial ($10.000)
POSITION_SIZE_PERCENT=2.0   # Usa 2% do capital por trade

# ============================================
# ESTRATÉGIA (EMA + RSI + ATR)
# ============================================
EMA_SHORT=20                # Média móvel rápida
EMA_LONG=50                 # Média móvel lenta
RSI_PERIOD=14               # Período do RSI
RSI_OVERBOUGHT=70           # RSI sobrecomprado (vender)
RSI_OVERSOLD=30             # RSI sobrevendido (comprar)
ATR_PERIOD=14               # Período do ATR
ATR_THRESHOLD_MULTIPLIER=1.2

# ============================================
# CONTROLE DE RISCO
# ============================================
STOP_LOSS_ATR_MULTIPLIER=1.5      # Stop loss = 1.5 x ATR
TAKE_PROFIT_ATR_MULTIPLIER=2.5    # Take profit = 2.5 x ATR
TRAILING_STOP_PERCENT=1.0         # Trailing stop 1%
MAX_DAILY_LOSS_PERCENT=5.0        # Para se perder 5% no dia
MAX_POSITIONS=3                   # Máximo 3 posições abertas

# ============================================
# MONTE CARLO
# ============================================
NUM_SIMULATIONS=10000       # Número de simulações
```

**Salve o arquivo!**

---

### **5️⃣ COMO O ROBÔ FUNCIONA**

#### **O robô NÃO precisa que você faça login manualmente!**

Ele usa as **chaves API** para se conectar automaticamente:

```
Você → Arquivo .env (tem as chaves) → Robô → Binance API → Trades
```

**Você só precisa:**
1. Colocar as chaves no `.env`
2. Rodar o robô
3. Pronto! Ele opera sozinho

---

## 🎮 MODOS DE USO

### **Modo 1: Menu Interativo (Mais Fácil)**
```bash
./menu_interativo.sh
```

**Opções do menu:**
- `1` - Backtest (testa em dados históricos)
- `2` - Monte Carlo (simula 10.000 cenários)
- `3` - Ver resultados do backtest
- `4` - Ver resultados do Monte Carlo
- `5` - Rodar testes unitários

### **Modo 2: Executável Direto**
```bash
# Backtest
./target/release/robo-trading backtest

# Monte Carlo
./target/release/robo-trading monte-carlo

# Operar ao vivo (CUIDADO!)
./target/release/robo-trading live
```

### **Modo 3: Scripts Prontos**
```bash
./run_backtest.sh       # Executa backtest
./run_montecarlo.sh     # Executa Monte Carlo
./run_live.sh           # Executa ao vivo
```

---

## 💰 OPERAR AO VIVO

### **Antes de operar com dinheiro real:**

1. ✅ Rode backtest: `./menu_interativo.sh` → opção 1
   - Win rate deve ser > 60%
   - Sharpe ratio > 1.5
   - Max drawdown < 20%

2. ✅ Rode Monte Carlo: opção 2
   - Probabilidade de lucro > 70%
   - Retorno esperado positivo

3. ✅ Teste no TESTNET por 1 semana:
   ```bash
   # No .env: USE_TESTNET=true
   ./target/release/robo-trading live
   ```

4. ✅ Só então mude para real:
   ```bash
   # No .env: USE_TESTNET=false
   ./target/release/robo-trading live
   ```

### **Comando para rodar ao vivo:**
```bash
./target/release/robo-trading live
```

**O robô vai:**
- ✅ Conectar na Binance automaticamente
- ✅ Analisar mercado a cada 1 minuto
- ✅ Abrir trades quando detectar sinal
- ✅ Fechar com stop loss ou take profit
- ✅ Logar tudo em `logs/trading.log`

---

## 📊 MONITORAR EM TEMPO REAL

Enquanto o robô roda, abra outro terminal:

```bash
# Ver todos os logs
tail -f logs/trading.log.*

# Ver apenas trades (compras/vendas)
tail -f logs/trading.log.* | grep -E "BUY|SELL|Trade"

# Ver apenas lucros/perdas
tail -f logs/trading.log.* | grep -E "PnL|Profit|Loss"
```

---

## 🔒 SEGURANÇA

### **Proteções do Robô:**
- ✅ Stop loss automático (limita perdas)
- ✅ Take profit automático (garante lucros)
- ✅ Max loss diário (para se perder 5%)
- ✅ Max posições simultâneas
- ✅ Trailing stop (protege lucros)

### **Proteções suas:**
- ✅ **NUNCA** ative "Enable Withdrawals" na API
- ✅ Use IP whitelist na Binance
- ✅ Comece com capital pequeno ($100-500)
- ✅ Teste primeiro no TESTNET
- ✅ Monitore diariamente

---

## 📱 RESUMO RÁPIDO

### **Para testar (sem risco):**
```bash
./menu_interativo.sh
# Escolha opção 1
```

### **Para operar de verdade:**

1. Crie conta na Binance
2. Pegue API Key + Secret Key
3. Cole no arquivo `.env`
4. Configure `USE_TESTNET=true`
5. Rode: `./target/release/robo-trading live`
6. Teste 1 semana
7. Mude `USE_TESTNET=false`
8. Rode novamente!

---

## ⚠️ AVISO LEGAL

- Este robô pode **ganhar** ou **perder** dinheiro
- Mercado de cripto é **volátil**
- Nunca invista mais do que pode perder
- Resultados passados **não garantem** resultados futuros
- Use por sua conta e risco

---

## 🆘 PROBLEMAS?

### **Erro: "API key not found"**
→ Verifique se copiou as chaves corretas no `.env`

### **Erro: "Invalid API-key"**
→ Verifique se está usando testnet com chaves testnet (ou real com real)

### **Não abre trades**
→ Ajuste RSI_OVERBOUGHT/OVERSOLD no `.env`

### **Muitos trades perdendo**
→ Aumente STOP_LOSS_ATR_MULTIPLIER

---

## 📞 COMANDOS ÚTEIS

```bash
# Ver capital disponível
grep "Capital" logs/trading.log.* | tail -5

# Contar trades do dia
grep "Trade" logs/trading.log.$(date +%Y-%m-%d) | wc -l

# Ver último resultado
cat backtest_results.json | jq '.metrics'

# Parar o robô
Ctrl + C
```

---

🎉 **Pronto! Agora você sabe tudo!**

**Comece testando com:**
```bash
./menu_interativo.sh
```
