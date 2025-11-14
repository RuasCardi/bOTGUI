# 📖 EXEMPLOS PRÁTICOS DE USO

## 🎬 Tutorial Passo-a-Passo

### 1. Instalação Inicial

```bash
# Clone ou navegue até o diretório
cd ~/Área\ de\ trabalho/RoboTrading

# Execute o script de setup
./setup.sh

# Isso irá:
# - Verificar/instalar Rust
# - Criar diretórios necessários
# - Compilar o projeto
# - Executar testes
```

---

### 2. Configuração Básica

```bash
# Edite o arquivo .env
nano .env
```

**Configuração mínima para começar:**
```env
# API (use testnet para testes!)
BINANCE_API_KEY=sua_api_key_aqui
BINANCE_SECRET_KEY=sua_secret_key_aqui
USE_TESTNET=true

# Trading
TRADING_PAIR=BTCUSDT
INITIAL_CAPITAL=1000.0
POSITION_SIZE_PERCENT=0.01    # 1% por trade
MAX_POSITIONS=2

# Risk
MAX_DAILY_LOSS_PERCENT=0.03   # Para em 3% de perda diária
```

---

### 3. Primeiro Backtest

```bash
# Execute o backtest
./run_backtest.sh

# Ou manualmente:
cargo run --release backtest
```

**O que você verá:**
```
📊 Executando Backtest...

[INFO] Fetching historical data for BTCUSDT...
[INFO] Loaded 1000 candles
[INFO] Running backtest on 1000 candles

╔══════════════════════════════════════════════════════════╗
║              BACKTEST RESULTS SUMMARY                    ║
╠══════════════════════════════════════════════════════════╣
║ Capital Inicial:     $    1000.00                        ║
║ Capital Final:       $    1245.00                        ║
║ PnL Total:           $     245.00                        ║
║ Retorno:                   24.50%                        ║
...
```

**Analise:**
- Win Rate > 50%? ✅
- Profit Factor > 1.5? ✅
- Max Drawdown < 20%? ✅

Se todos SIM → Continue para Monte Carlo

---

### 4. Simulação Monte Carlo

```bash
./run_montecarlo.sh
```

**Interpretação:**
```
Probabilidade de Lucro:      78.5%  ← Quanto maior, melhor
VaR 95%:             $    150.00    ← Máxima perda esperada
Max DD Esperado:             8.2%   ← Drawdown médio
```

**Decisão:**
- Prob > 60%? ✅ → Bom
- VaR < 20% capital? ✅ → Aceitável
- DD < 15%? ✅ → Ótimo

---

### 5. Teste em Testnet

```bash
# Certifique-se que USE_TESTNET=true
nano .env

# Inicie o bot
cargo run --release live
```

**Monitore por pelo menos 1 semana:**
```bash
# Em outro terminal, veja os logs
tail -f logs/trading.log
```

**O que observar:**
- Bot inicia sem erros ✅
- Conecta à API ✅
- Identifica sinais corretamente ✅
- Executa ordens ✅
- Stop loss funciona ✅

---

## 🎯 Casos de Uso Específicos

### Caso 1: Scalping Agressivo

**Objetivo:** Muitos trades pequenos

```env
# .env para scalping
EMA_FAST=8
EMA_SLOW=21
RSI_PERIOD=7
POSITION_SIZE_PERCENT=0.005   # 0.5% por trade
STOP_LOSS_ATR_MULTIPLIER=0.8
TAKE_PROFIT_ATR_MULTIPLIER=1.2
```

**Timeframe:** Use M5 ou M15

---

### Caso 2: Swing Trading Conservador

**Objetivo:** Poucos trades, maior duração

```env
# .env para swing trading
EMA_FAST=50
EMA_SLOW=200
RSI_PERIOD=21
POSITION_SIZE_PERCENT=0.03    # 3% por trade
STOP_LOSS_ATR_MULTIPLIER=3.0
TAKE_PROFIT_ATR_MULTIPLIER=5.0
```

**Timeframe:** Use H4 ou D1

---

### Caso 3: Trading em Horários Específicos

**Edite `src/main.rs`:**

```rust
// Adicione na função trading_cycle():
let hour = chrono::Utc::now().hour();

// Só opera das 8h às 20h UTC
if hour < 8 || hour > 20 {
    info!("Fora do horário de trading");
    return Ok(());
}
```

Recompile:
```bash
cargo build --release
```

---

### Caso 4: Múltiplos Pares

**1. Duplique a configuração:**
```bash
cp .env .env.btc
cp .env .env.eth
```

**2. Edite cada arquivo:**
```env
# .env.btc
TRADING_PAIR=BTCUSDT

# .env.eth
TRADING_PAIR=ETHUSDT
```

**3. Execute múltiplas instâncias:**
```bash
# Terminal 1
env $(cat .env.btc) cargo run --release live

# Terminal 2
env $(cat .env.eth) cargo run --release live
```

---

## 🔧 Troubleshooting Comum

### Problema 1: Erro de Autenticação

```
Error: API error: Authentication failed
```

**Solução:**
```bash
# 1. Verifique as credenciais
cat .env | grep API_KEY

# 2. Confirme que API key tem permissões corretas
# Login Binance > API Management > Permissões

# 3. Se estiver em testnet, use keys da testnet
# https://testnet.binance.vision/
```

---

### Problema 2: Saldo Insuficiente

```
Error: Insufficient balance
```

**Solução:**
```bash
# Reduza o tamanho da posição
nano .env
# Altere:
POSITION_SIZE_PERCENT=0.005  # Era 0.02, agora 0.5%
```

---

### Problema 3: Nenhum Trade Executado

```
INFO: Signal detected but conditions not met
```

**Causa:** Estratégia muito restritiva

**Solução:**
```bash
nano .env
# Relaxe os parâmetros:
ATR_THRESHOLD_MULTIPLIER=0.8  # Era 1.2
```

---

### Problema 4: Muitas Perdas Consecutivas

**Análise:**
```bash
# Veja os últimos trades
grep "Trade closed" logs/trading.log | tail -20
```

**Ação:**
```bash
# PARE o bot
# Analise:
# - Mercado mudou de regime?
# - Volatilidade anormal?
# - Notícias impactantes?

# Se necessário, ajuste parâmetros ou espere
```

---

## 📊 Análise de Logs

### Ver Trades Executados

```bash
grep "Opening" logs/trading.log
```

**Saída:**
```
[INFO] Opening LONG position: 0.001 BTCUSDT @ 50000.00
[INFO] Stop loss: 49250.00 | Take profit: 51250.00
```

### Ver Sinais Detectados

```bash
grep "signal detected" logs/trading.log
```

### Ver Métricas de Risco

```bash
grep "Risk metrics" logs/trading.log
```

---

## 🎓 Exercícios Práticos

### Exercício 1: Otimize a Estratégia

**Objetivo:** Melhorar o Sharpe Ratio

**Passos:**
1. Execute backtest base
2. Anote Sharpe atual
3. Teste EMA_FAST = 15, 20, 25
4. Para cada, execute backtest
5. Compare Sharpes
6. Use o melhor

**Comando:**
```bash
# Teste 1
sed -i 's/EMA_FAST=.*/EMA_FAST=15/' .env
./run_backtest.sh > result_15.txt

# Teste 2
sed -i 's/EMA_FAST=.*/EMA_FAST=20/' .env
./run_backtest.sh > result_20.txt

# Compare
grep "Sharpe" result_*.txt
```

---

### Exercício 2: Valide em Diferentes Períodos

**Objetivo:** Testar robustez temporal

**Código em Python:**
```python
# validate_periods.py
import subprocess
import json

periods = [
    ("2024-01-01", "2024-03-31", "Q1"),
    ("2024-04-01", "2024-06-30", "Q2"),
    ("2024-07-01", "2024-09-30", "Q3"),
    ("2024-10-01", "2024-12-31", "Q4"),
]

for start, end, name in periods:
    print(f"\nTesting {name}...")
    # Aqui você modificaria o backtest para aceitar datas
    # Por enquanto, é um exemplo conceitual
```

---

### Exercício 3: Compare com Buy & Hold

**Objetivo:** Validar se estratégia é melhor que simplesmente comprar e segurar

**Em `src/backtest/mod.rs`, adicione:**
```rust
pub fn calculate_buy_and_hold(candles: &[Candle], capital: f64) -> f64 {
    let first_price = candles.first().unwrap().close;
    let last_price = candles.last().unwrap().close;
    let quantity = capital / first_price;
    quantity * last_price
}
```

**Compare:**
```
Estratégia: +24.5%
Buy & Hold: +18.2%
Vantagem:   +6.3%  ✅ Estratégia vence!
```

---

## 🎨 Visualização de Resultados

### Criar Gráfico de Equity Curve

**Use Python + matplotlib:**

```python
# plot_equity.py
import json
import matplotlib.pyplot as plt

with open('backtest_results.json') as f:
    data = json.load(f)

equity = data['equity_curve']
plt.plot(equity)
plt.title('Equity Curve')
plt.xlabel('Trade')
plt.ylabel('Capital ($)')
plt.grid(True)
plt.savefig('equity_curve.png')
print("Gráfico salvo em equity_curve.png")
```

**Execute:**
```bash
python3 plot_equity.py
```

---

## 🚀 Deploy em Produção

### Opção 1: VPS (Recomendado)

```bash
# 1. No servidor
ssh usuario@seu-servidor.com

# 2. Clone o projeto
git clone https://github.com/seu-repo/RoboTrading.git
cd RoboTrading

# 3. Configure
cp .env.example .env
nano .env  # Preencha credenciais

# 4. Compile
cargo build --release

# 5. Crie serviço systemd
sudo nano /etc/systemd/system/robotrading.service

# 6. Ative
sudo systemctl enable robotrading
sudo systemctl start robotrading

# 7. Monitore
sudo journalctl -u robotrading -f
```

---

### Opção 2: Docker (Avançado)

**Crie `Dockerfile`:**
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/robo-trading /usr/local/bin/
CMD ["robo-trading", "live"]
```

**Execute:**
```bash
docker build -t robotrading .
docker run -d --name bot --env-file .env robotrading
```

---

## 📞 Suporte e Comunidade

### Logs de Debug

```bash
# Execute com debug ativado
RUST_LOG=debug cargo run --release live
```

### Relatar Problemas

Ao relatar um bug, inclua:
1. Versão do Rust (`rustc --version`)
2. Sistema operacional
3. Conteúdo do `.env` (SEM credenciais!)
4. Logs relevantes
5. Passos para reproduzir

---

## ✅ Checklist Final

Antes de considerar o sistema pronto:

- [ ] Compilou sem erros
- [ ] Todos os testes passaram
- [ ] Backtest com resultados positivos
- [ ] Monte Carlo confiável
- [ ] Testado em testnet por 1+ semana
- [ ] Logs funcionando corretamente
- [ ] Graceful shutdown funcionando (Ctrl+C)
- [ ] Risk management validado
- [ ] Documentação lida e compreendida

---

**🎉 Parabéns! Você tem um robô de trading profissional!**

**Próximos passos:**
1. Otimize a estratégia (OPTIMIZATION.md)
2. Prepare para produção (PRODUCTION.md)
3. Monitore constantemente
4. Ajuste conforme necessário

**Lembre-se:** Trading envolve risco. Use sempre capital que pode perder!

🚀 **Happy Trading!**
