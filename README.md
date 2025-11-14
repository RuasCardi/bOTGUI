# 🤖 RoboTrading - Sistema Profissional de Trading Algorítmico

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Sistema completo de trading automatizado em Rust** com estratégias baseadas em análise técnica, machine learning probabilístico, backtesting avançado e simulação Monte Carlo.

---

## 🎯 Características Principais

### ✅ Funcionalidades Implementadas

- ✅ **Conexão API Binance** (REST + autenticação HMAC SHA256)
- ✅ **Execução de Ordens** (Market, Limit, Stop Loss, Take Profit)
- ✅ **Estratégia EMA + RSI + ATR** com detecção de regime de mercado
- ✅ **Risk Management Avançado**
  - Stop Loss dinâmico baseado em ATR
  - Take Profit adaptativo
  - Trailing Stop inteligente
  - Limite de perda diária
  - Gestão de múltiplas posições
- ✅ **Backtesting Completo** com métricas profissionais
  - Win Rate, Profit Factor, Sharpe Ratio
  - Sortino Ratio, Calmar Ratio
  - Max Drawdown, Expectancy
- ✅ **Simulação Monte Carlo** (paramétrica e bootstrap)
- ✅ **Logging Estruturado** com tracing
- ✅ **Retry Logic** e tratamento de erros robusto
- ✅ **Simulação de Latência** para testes
- ✅ **Configuração via .env**

---

## 📁 Estrutura do Projeto

```
RoboTrading/
├── src/
│   ├── main.rs                 # Orquestração e modos de operação
│   ├── config.rs               # Configuração do sistema
│   ├── error.rs                # Tipos de erro customizados
│   ├── types.rs                # Tipos de dados (Order, Candle, etc.)
│   ├── api/
│   │   ├── mod.rs
│   │   └── binance.rs          # Cliente Binance (REST API)
│   ├── strategy/
│   │   ├── mod.rs
│   │   ├── indicators.rs       # Indicadores técnicos (EMA, RSI, ATR, etc.)
│   │   └── strategy.rs         # Lógica da estratégia de trading
│   ├── risk/
│   │   └── mod.rs              # Gerenciamento de risco
│   ├── execution/
│   │   └── mod.rs              # Executor de ordens com retry
│   ├── backtest/
│   │   └── mod.rs              # Engine de backtesting
│   └── montecarlo/
│       └── mod.rs              # Simulador Monte Carlo
├── Cargo.toml                  # Dependências Rust
├── .env.example                # Exemplo de configuração
├── README.md                   # Este arquivo
└── PRODUCTION.md               # Checklist para produção
```

---

## 🚀 Instalação

### Pré-requisitos

- **Rust 1.70+** (instale via [rustup](https://rustup.rs/))
- **Conta Binance** com API keys

### 1. Clone o repositório

```bash
cd "~/Área de trabalho/RoboTrading"
```

### 2. Configure as variáveis de ambiente

```bash
cp .env.example .env
nano .env
```

Preencha com suas credenciais:

```env
BINANCE_API_KEY=sua_api_key_aqui
BINANCE_SECRET_KEY=sua_secret_key_aqui
USE_TESTNET=true
TRADING_PAIR=BTCUSDT
INITIAL_CAPITAL=10000.0
```

### 3. Compile o projeto

```bash
cargo build --release
```

---

## 💻 Modos de Operação

### 1️⃣ Backtest (Teste Histórico)

Testa a estratégia em dados históricos:

```bash
cargo run --release backtest
```

**Saída:**
```
╔══════════════════════════════════════════════════════════╗
║              BACKTEST RESULTS SUMMARY                    ║
╠══════════════════════════════════════════════════════════╣
║ Capital Inicial:     $   10000.00                        ║
║ Capital Final:       $   12450.00                        ║
║ PnL Total:           $    2450.00                        ║
║ Retorno:                   24.50%                        ║
╠══════════════════════════════════════════════════════════╣
║ Total de Trades:         45                              ║
║ Trades Vencedores:       28 ( 62.2%)                     ║
║ Trades Perdedores:       17 ( 37.8%)                     ║
╠══════════════════════════════════════════════════════════╣
║ Win Rate:                  62.22%                        ║
║ Profit Factor:              1.85                         ║
║ Sharpe Ratio:               1.42                         ║
║ Max Drawdown:        $    350.00 ( 3.50%)                ║
╚══════════════════════════════════════════════════════════╝
```

### 2️⃣ Monte Carlo (Simulação Probabilística)

Simula milhares de cenários futuros:

```bash
cargo run --release monte-carlo
```

**Saída:**
```
╔══════════════════════════════════════════════════════════╗
║           MONTE CARLO SIMULATION RESULTS                 ║
╠══════════════════════════════════════════════════════════╣
║ Simulações:          10000                               ║
║ Retorno Esperado:    $   11250.00 ( 12.5%)              ║
║ Prob. de Lucro:             78.5%                        ║
║ VaR 95%:             $    1200.00                        ║
╚══════════════════════════════════════════════════════════╝
```

### 3️⃣ Live Trading (Produção)

**⚠️ ATENÇÃO: Usa dinheiro real!**

```bash
cargo run --release live
```

Inicia o robô em modo produção. Aguarda 10 segundos antes de começar.

### 4️⃣ Paper Trading (Simulação em Tempo Real)

```bash
cargo run --release paper
```

*(Em desenvolvimento)*

---

## 📊 Estratégia de Trading

### 🔍 Lógica

A estratégia implementada combina múltiplos indicadores:

#### **Sinal de COMPRA (LONG)**
```
✅ EMA(20) > EMA(50)        → Tendência de alta
✅ RSI > 50                  → Momentum positivo
✅ ATR > ATR_médio × 1.2    → Volatilidade adequada
```

#### **Sinal de VENDA (SHORT)**
```
✅ EMA(20) < EMA(50)        → Tendência de baixa
✅ RSI < 50                  → Momentum negativo
✅ ATR > ATR_médio × 1.2    → Volatilidade adequada
```

### 🛡️ Risk Management

- **Stop Loss**: `Preço de entrada ± (ATR × 1.5)`
- **Take Profit**: `Preço de entrada ± (ATR × 2.5)`
- **Trailing Stop**: Ativa após 1.5% de lucro, ajusta em 0.8%
- **Limite diário**: Fecha se perder 5% do capital
- **Max posições**: 3 simultâneas

### 📈 Indicadores Técnicos

| Indicador | Período | Uso |
|-----------|---------|-----|
| EMA Fast  | 20      | Tendência de curto prazo |
| EMA Slow  | 50      | Tendência de longo prazo |
| RSI       | 14      | Força relativa / momentum |
| ATR       | 14      | Volatilidade / stops dinâmicos |

---

## 🧪 Testes

### Executar todos os testes

```bash
cargo test
```

### Testar módulo específico

```bash
cargo test --lib strategy
cargo test --lib backtest
```

### Com logs detalhados

```bash
RUST_LOG=debug cargo test -- --nocapture
```

---

## 📝 Logs

Logs são salvos em:
- **Console**: Saída padrão colorida
- **Arquivo**: `./logs/trading.log` (rotação diária)

Níveis de log:
```bash
export LOG_LEVEL=debug   # debug, info, warn, error
cargo run
```

---

## ⚙️ Configuração Avançada

### Ajustar parâmetros da estratégia

Edite `.env`:

```env
# Estratégia
EMA_FAST=20
EMA_SLOW=50
RSI_PERIOD=14
RSI_OVERBOUGHT=70
RSI_OVERSOLD=30
ATR_PERIOD=14
ATR_THRESHOLD_MULTIPLIER=1.2

# Risk Management
STOP_LOSS_ATR_MULTIPLIER=1.5
TAKE_PROFIT_ATR_MULTIPLIER=2.5
TRAILING_STOP_ACTIVATION=0.015
TRAILING_STOP_DISTANCE=0.008
MAX_DAILY_LOSS_PERCENT=0.05

# Posição
POSITION_SIZE_PERCENT=0.02
MAX_POSITIONS=3
```

### Simular latência

Para testar robustez:

```env
LATENCY_SIMULATION_MS=100  # 100ms de atraso artificial
```

---

## 📊 Métricas de Performance

### Interpretação

| Métrica | Bom | Ótimo | Descrição |
|---------|-----|-------|-----------|
| **Win Rate** | > 50% | > 60% | % de trades vencedores |
| **Profit Factor** | > 1.5 | > 2.0 | Lucro total / Perda total |
| **Sharpe Ratio** | > 1.0 | > 2.0 | Retorno ajustado por risco |
| **Max Drawdown** | < 20% | < 10% | Maior queda do pico |
| **Expectancy** | > $0 | > $50 | Lucro esperado por trade |

---

## 🔒 Segurança

### ✅ Boas Práticas Implementadas

- ✅ API keys via variáveis de ambiente
- ✅ HMAC SHA256 para autenticação
- ✅ Timeout em requisições (30s)
- ✅ Retry com exponential backoff
- ✅ Validação de saldo antes de operar
- ✅ Limite de perda diária
- ✅ Graceful shutdown (Ctrl+C)

### ⚠️ NUNCA

- ❌ Commite arquivos `.env` no git
- ❌ Compartilhe suas API keys
- ❌ Rode em produção sem testar
- ❌ Use todo seu capital de uma vez

---

## 🐛 Troubleshooting

### Erro: "Authentication failed"

```bash
# Verifique suas credenciais
cat .env | grep API_KEY

# Teste a conexão
cargo run -- backtest
```

### Erro: "Insufficient balance"

```bash
# Reduza o tamanho da posição
POSITION_SIZE_PERCENT=0.01  # 1% ao invés de 2%
```

### Erro: "Network timeout"

```bash
# Aumente o timeout
ORDER_TIMEOUT_SECONDS=60
```

---

## 📚 Recursos Adicionais

- [Documentação Binance API](https://binance-docs.github.io/apidocs/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Trading Strategies Guide](https://www.investopedia.com/trading/)

---

## 🔮 Roadmap

- [ ] Suporte para múltiplas exchanges (Bybit, OKX)
- [ ] WebSocket para dados em tempo real
- [ ] Dashboard web com métricas ao vivo
- [ ] Machine learning para otimização de parâmetros
- [ ] Notificações via Telegram
- [ ] Database para histórico de trades
- [ ] Análise de sentimento (Twitter, news)

---

## 📄 Licença

MIT License - veja [LICENSE](LICENSE) para detalhes.

---

## ⚠️ Disclaimer

**Este software é fornecido "como está" para fins educacionais.**

- ⚠️ Trading envolve risco de perda de capital
- ⚠️ Desempenho passado não garante resultados futuros
- ⚠️ Teste extensivamente antes de usar dinheiro real
- ⚠️ O desenvolvedor não se responsabiliza por perdas

**USE POR SUA CONTA E RISCO!**

---

## 👨‍💻 Desenvolvedor

Desenvolvido por um Quant Trader profissional especializado em:
- Rust para sistemas de alta performance
- Análise estatística e machine learning
- Trading algorítmico e risk management
- Otimização de estratégias

---

## 🙏 Contribuições

Contribuições são bem-vindas! Abra uma issue ou pull request.

---

**Happy Trading! 🚀📈**
