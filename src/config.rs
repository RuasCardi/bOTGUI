use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub exchange: ExchangeConfig,
    pub trading: TradingConfig,
    pub risk: RiskConfig,
    pub strategy: StrategyConfig,
    pub execution: ExecutionConfig,
    pub logging: LoggingConfig,
    pub backtest: BacktestConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeConfig {
    pub api_key: String,
    pub secret_key: String,
    pub use_testnet: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradingConfig {
    pub trading_pair: String,
    pub initial_capital: f64,
    pub position_size_percent: f64,
    pub max_positions: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RiskConfig {
    pub stop_loss_atr_multiplier: f64,
    pub take_profit_atr_multiplier: f64,
    pub trailing_stop_activation: f64,
    pub trailing_stop_distance: f64,
    pub max_daily_loss_percent: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StrategyConfig {
    pub ema_fast: usize,
    pub ema_slow: usize,
    pub rsi_period: usize,
    pub rsi_overbought: f64,
    pub rsi_oversold: f64,
    pub atr_period: usize,
    pub atr_threshold_multiplier: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExecutionConfig {
    pub order_retry_attempts: u32,
    pub order_timeout_seconds: u64,
    pub latency_simulation_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub log_level: String,
    pub log_file: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BacktestConfig {
    pub start_date: String,
    pub end_date: String,
    pub data_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        Ok(Config {
            exchange: ExchangeConfig {
                api_key: env::var("BINANCE_API_KEY")?,
                secret_key: env::var("BINANCE_SECRET_KEY")?,
                use_testnet: env::var("USE_TESTNET")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
            },
            trading: TradingConfig {
                trading_pair: env::var("TRADING_PAIR")
                    .unwrap_or_else(|_| "BTCUSDT".to_string()),
                initial_capital: env::var("INITIAL_CAPITAL")
                    .unwrap_or_else(|_| "10000.0".to_string())
                    .parse()?,
                position_size_percent: env::var("POSITION_SIZE_PERCENT")
                    .unwrap_or_else(|_| "0.02".to_string())
                    .parse()?,
                max_positions: env::var("MAX_POSITIONS")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
            },
            risk: RiskConfig {
                stop_loss_atr_multiplier: env::var("STOP_LOSS_ATR_MULTIPLIER")
                    .unwrap_or_else(|_| "1.5".to_string())
                    .parse()?,
                take_profit_atr_multiplier: env::var("TAKE_PROFIT_ATR_MULTIPLIER")
                    .unwrap_or_else(|_| "2.5".to_string())
                    .parse()?,
                trailing_stop_activation: env::var("TRAILING_STOP_ACTIVATION")
                    .unwrap_or_else(|_| "0.015".to_string())
                    .parse()?,
                trailing_stop_distance: env::var("TRAILING_STOP_DISTANCE")
                    .unwrap_or_else(|_| "0.008".to_string())
                    .parse()?,
                max_daily_loss_percent: env::var("MAX_DAILY_LOSS_PERCENT")
                    .unwrap_or_else(|_| "0.05".to_string())
                    .parse()?,
            },
            strategy: StrategyConfig {
                ema_fast: env::var("EMA_FAST")
                    .unwrap_or_else(|_| "20".to_string())
                    .parse()?,
                ema_slow: env::var("EMA_SLOW")
                    .unwrap_or_else(|_| "50".to_string())
                    .parse()?,
                rsi_period: env::var("RSI_PERIOD")
                    .unwrap_or_else(|_| "14".to_string())
                    .parse()?,
                rsi_overbought: env::var("RSI_OVERBOUGHT")
                    .unwrap_or_else(|_| "70".to_string())
                    .parse()?,
                rsi_oversold: env::var("RSI_OVERSOLD")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                atr_period: env::var("ATR_PERIOD")
                    .unwrap_or_else(|_| "14".to_string())
                    .parse()?,
                atr_threshold_multiplier: env::var("ATR_THRESHOLD_MULTIPLIER")
                    .unwrap_or_else(|_| "1.2".to_string())
                    .parse()?,
            },
            execution: ExecutionConfig {
                order_retry_attempts: env::var("ORDER_RETRY_ATTEMPTS")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
                order_timeout_seconds: env::var("ORDER_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                latency_simulation_ms: env::var("LATENCY_SIMULATION_MS")
                    .unwrap_or_else(|_| "0".to_string())
                    .parse()?,
            },
            logging: LoggingConfig {
                log_level: env::var("LOG_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
                log_file: env::var("LOG_FILE")
                    .unwrap_or_else(|_| "./logs/trading.log".to_string()),
            },
            backtest: BacktestConfig {
                start_date: env::var("BACKTEST_START_DATE")
                    .unwrap_or_else(|_| "2024-01-01".to_string()),
                end_date: env::var("BACKTEST_END_DATE")
                    .unwrap_or_else(|_| "2024-12-31".to_string()),
                data_path: env::var("BACKTEST_DATA_PATH")
                    .unwrap_or_else(|_| "./data/historical/".to_string()),
            },
        })
    }
}
