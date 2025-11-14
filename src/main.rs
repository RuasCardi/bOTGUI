mod api;
mod backtest;
mod cli;
mod config;
mod error;
mod execution;
mod montecarlo;
mod risk;
mod strategy;
mod types;

use api::BinanceClient;
use backtest::Backtester;
use config::Config;
use error::Result;
use execution::OrderExecutor;
use montecarlo::MonteCarloSimulator;
use risk::RiskManager;
use strategy::TradingStrategy;
use types::{Candle, OrderSide, Timeframe};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::signal;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializa logging
    setup_logging();

    info!("🚀 Starting RoboTrading System");

    // Carrega configuração
    let config = Config::from_env().expect("Failed to load configuration");
    info!("Configuration loaded successfully");

    // Decide modo de operação
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("live");

    match mode {
        "backtest" => run_backtest(config).await?,
        "monte-carlo" => run_monte_carlo(config).await?,
        "live" => run_live_trading(config).await?,
        "paper" => run_paper_trading(config).await?,
        _ => {
            eprintln!("Unknown mode: {}", mode);
            eprintln!("Usage: cargo run [backtest|monte-carlo|live|paper]");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Configura sistema de logging
fn setup_logging() {
    let file_appender = tracing_appender::rolling::daily("./logs", "trading.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();
}

/// Executa backtest histórico
async fn run_backtest(config: Config) -> Result<()> {
    info!("╔════════════════════════════════════════╗");
    info!("║      BACKTEST MODE                     ║");
    info!("╚════════════════════════════════════════╝");

    // Cria cliente para buscar dados históricos
    let client = BinanceClient::new(
        config.exchange.api_key.clone(),
        config.exchange.secret_key.clone(),
        config.exchange.use_testnet,
    );

    // Busca dados históricos
    info!("Fetching historical data for {}...", config.trading.trading_pair);
    let candles = client
        .get_klines(&config.trading.trading_pair, Timeframe::H1, 1000)
        .await?;

    info!("Loaded {} candles", candles.len());

    // Cria backtester
    let backtester = Backtester::new(
        config.strategy.clone(),
        config.trading.initial_capital,
        config.trading.position_size_percent,
    );

    // Executa backtest
    let result = backtester.run(&candles);
    result.print_summary();

    // Salva resultados
    let json = serde_json::to_string_pretty(&result)?;
    std::fs::write("backtest_results.json", json)?;
    info!("Results saved to backtest_results.json");

    Ok(())
}

/// Executa simulação de Monte Carlo
async fn run_monte_carlo(config: Config) -> Result<()> {
    info!("╔════════════════════════════════════════╗");
    info!("║    MONTE CARLO SIMULATION MODE         ║");
    info!("╚════════════════════════════════════════╝");

    // Primeiro executa backtest para obter trades históricos
    let client = BinanceClient::new(
        config.exchange.api_key.clone(),
        config.exchange.secret_key.clone(),
        config.exchange.use_testnet,
    );

    info!("Fetching historical data...");
    let candles = client
        .get_klines(&config.trading.trading_pair, Timeframe::H1, 1000)
        .await?;

    let backtester = Backtester::new(
        config.strategy.clone(),
        config.trading.initial_capital,
        config.trading.position_size_percent,
    );

    let backtest_result = backtester.run(&candles);
    backtest_result.print_summary();

    // Executa Monte Carlo
    let monte_carlo = MonteCarloSimulator::new(10000);
    
    info!("Running parametric Monte Carlo simulation...");
    let mc_result = monte_carlo.simulate(
        &backtest_result.trades,
        config.trading.initial_capital,
        100,
    );
    mc_result.print_summary(config.trading.initial_capital);

    info!("Running bootstrap Monte Carlo simulation...");
    let mc_bootstrap = monte_carlo.bootstrap_simulate(
        &backtest_result.trades,
        config.trading.initial_capital,
        100,
    );
    mc_bootstrap.print_summary(config.trading.initial_capital);

    // Salva resultados
    let json = serde_json::to_string_pretty(&mc_result)?;
    std::fs::write("montecarlo_results.json", json)?;
    info!("Results saved to montecarlo_results.json");

    Ok(())
}

/// Executa trading ao vivo (PRODUÇÃO)
async fn run_live_trading(config: Config) -> Result<()> {
    info!("╔════════════════════════════════════════╗");
    info!("║         LIVE TRADING MODE              ║");
    info!("║         ⚠️  REAL MONEY ⚠️              ║");
    info!("╚════════════════════════════════════════╝");

    if config.exchange.use_testnet {
        warn!("Running in testnet mode - no real money");
    } else {
        warn!("⚠️  LIVE TRADING WITH REAL MONEY - BE CAREFUL! ⚠️");
        info!("Starting in 10 seconds... Press Ctrl+C to cancel");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    // Inicializa componentes
    let client = Arc::new(BinanceClient::new(
        config.exchange.api_key.clone(),
        config.exchange.secret_key.clone(),
        config.exchange.use_testnet,
    ));

    let strategy = TradingStrategy::new(config.strategy.clone());
    let risk_manager = Arc::new(RiskManager::new(
        config.risk.clone(),
        config.trading.initial_capital,
        config.trading.max_positions,
    ));
    let executor = OrderExecutor::new(client.clone(), config.execution.clone());

    // Testa conexão
    client.ping().await?;
    info!("✓ API connection established");

    // Verifica conta
    let account = client.get_account().await?;
    info!("✓ Account verified - Can trade: {}", account.can_trade);

    // Loop principal de trading
    let mut ticker = interval(Duration::from_secs(60)); // Analisa a cada 1 minuto
    let symbol = config.trading.trading_pair.clone();

    info!("🤖 Trading bot started - monitoring {}...", symbol);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Err(e) = trading_cycle(
                    &client,
                    &strategy,
                    &risk_manager,
                    &executor,
                    &symbol,
                ).await {
                    error!("Trading cycle error: {}", e);
                }
            }
            _ = signal::ctrl_c() => {
                info!("Shutdown signal received");
                break;
            }
        }
    }

    // Graceful shutdown
    info!("Closing all positions...");
    // TODO: Implementar fechamento de posições
    info!("✓ Shutdown complete");

    Ok(())
}

/// Ciclo de trading
async fn trading_cycle(
    client: &Arc<BinanceClient>,
    strategy: &TradingStrategy,
    risk_manager: &Arc<RiskManager>,
    executor: &OrderExecutor,
    symbol: &str,
) -> Result<()> {
    // Busca dados recentes
    let candles = client.get_klines(symbol, Timeframe::H1, 100).await?;
    let current_price = candles.last().unwrap().close;

    // Atualiza trailing stops
    let positions = risk_manager.list_positions();
    for position in &positions {
        risk_manager.update_trailing_stop(&position.symbol, current_price);

        // Verifica stop loss
        if risk_manager.check_stop_loss(&position.symbol, current_price) {
            info!("Stop loss triggered for {}", position.symbol);
            let close_side = match position.side {
                OrderSide::Buy => OrderSide::Sell,
                OrderSide::Sell => OrderSide::Buy,
            };
            executor
                .execute_market_order(&position.symbol, close_side, position.quantity)
                .await?;
            risk_manager.remove_position(&position.symbol);
        }

        // Verifica take profit
        if risk_manager.check_take_profit(&position.symbol, current_price) {
            info!("Take profit triggered for {}", position.symbol);
            let close_side = match position.side {
                OrderSide::Buy => OrderSide::Sell,
                OrderSide::Sell => OrderSide::Buy,
            };
            executor
                .execute_market_order(&position.symbol, close_side, position.quantity)
                .await?;
            risk_manager.remove_position(&position.symbol);
        }
    }

    // Verifica novos sinais
    if let Some(signal) = strategy.analyze(&candles) {
        let mut prices = HashMap::new();
        prices.insert(symbol.to_string(), current_price);

        if risk_manager.can_open_position(&prices) {
            match signal.signal {
                strategy::Signal::Long | strategy::Signal::Short => {
                    let side = if matches!(signal.signal, strategy::Signal::Long) {
                        OrderSide::Buy
                    } else {
                        OrderSide::Sell
                    };

                    // Calcula tamanho da posição
                    let capital = 10000.0; // TODO: Obter do saldo real
                    let position_value = capital * 0.02;
                    let quantity = position_value / current_price;

                    info!(
                        "Opening {} position: {} {} @ {:.2}",
                        match side {
                            OrderSide::Buy => "LONG",
                            OrderSide::Sell => "SHORT",
                        },
                        quantity,
                        symbol,
                        current_price
                    );

                    // Executa ordem
                    let stop_loss = risk_manager.calculate_stop_loss(
                        current_price,
                        signal.indicators.atr,
                        side,
                    );
                    let take_profit = risk_manager.calculate_take_profit(
                        current_price,
                        signal.indicators.atr,
                        side,
                    );

                    executor
                        .open_position_with_stops(symbol, side, quantity, stop_loss, take_profit)
                        .await?;

                    risk_manager.add_position(
                        symbol.to_string(),
                        side,
                        current_price,
                        quantity,
                        signal.indicators.atr,
                    );
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Executa paper trading (simulação sem dinheiro real)
async fn run_paper_trading(config: Config) -> Result<()> {
    info!("╔════════════════════════════════════════╗");
    info!("║       PAPER TRADING MODE               ║");
    info!("║       (Simulation - No Real Money)     ║");
    info!("╚════════════════════════════════════════╝");

    // TODO: Implementar paper trading
    info!("Paper trading mode coming soon...");

    Ok(())
}
