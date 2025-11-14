use robo_trading::*;
use anyhow::Result;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar logging simples
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    loop {
        cli::CLI::print_menu();
        let choice = cli::CLI::read_input();

        match choice.as_str() {
            "1" => {
                cli::CLI::print_separator();
                if let Err(e) = run_backtest_interactive().await {
                    cli::CLI::print_error(&format!("Erro no backtest: {}", e));
                }
            }
            "2" => {
                cli::CLI::print_separator();
                if let Err(e) = run_montecarlo_interactive().await {
                    cli::CLI::print_error(&format!("Erro na simulação: {}", e));
                }
            }
            "3" => {
                cli::CLI::print_separator();
                cli::CLI::print_warning("Modo TESTNET - Usando dinheiro fictício");
                if cli::CLI::confirm_action("Confirma operação em TESTNET?") {
                    if let Err(e) = run_live_trading_interactive(true).await {
                        cli::CLI::print_error(&format!("Erro no modo ao vivo: {}", e));
                    }
                }
            }
            "4" => {
                cli::CLI::print_separator();
                cli::CLI::print_warning("MODO REAL - Você irá operar com DINHEIRO REAL!");
                println!("⚠️  Certifique-se de ter configurado as chaves API corretamente no .env");
                println!("⚠️  Recomenda-se testar primeiro no modo TESTNET (opção 3)");
                
                if cli::CLI::confirm_action("TEM CERTEZA que deseja operar com dinheiro REAL?") {
                    if cli::CLI::confirm_action("Confirma NOVAMENTE? Esta ação não pode ser desfeita!") {
                        if let Err(e) = run_live_trading_interactive(false).await {
                            cli::CLI::print_error(&format!("Erro no modo ao vivo: {}", e));
                        }
                    }
                }
            }
            "5" => {
                cli::CLI::print_separator();
                show_last_backtest();
            }
            "6" => {
                cli::CLI::print_separator();
                show_last_montecarlo();
            }
            "0" => {
                cli::CLI::print_success("Até logo! 👋");
                break;
            }
            _ => {
                cli::CLI::print_error("Opção inválida! Escolha um número de 0 a 6.");
            }
        }

        if choice != "0" {
            println!("\nPressione ENTER para voltar ao menu...");
            cli::CLI::read_input();
        }
    }

    Ok(())
}

async fn run_backtest_interactive() -> Result<()> {
    use robo_trading::{api::BinanceClient, backtest::Backtester, config::Config, cli::CLI};

    let config = Config::from_env().map_err(|e| anyhow::anyhow!("Erro ao carregar config: {}", e))?;
    
    println!("⏳ Buscando dados históricos da Binance...");
    
    let client = BinanceClient::new(
        config.exchange.api_key.clone(),
        config.exchange.secret_key.clone(),
        config.exchange.use_testnet,
    );

    let candles = client
        .get_klines(&config.trading.trading_pair, "1h", 500)
        .await?;

    println!("✅ {} candles carregados", candles.len());
    println!("⚙️  Executando estratégia...\n");

    let mut backtester = Backtester::new(config.strategy, config.risk, config.execution);
    let (metrics, trades) = backtester.run(candles).await?;

    // Salvar resultados
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let results = serde_json::json!({
        "metrics": metrics,
        "trades": trades,
        "initial_capital": config.trading.initial_capital,
        "timestamp": timestamp
    });
    fs::write("backtest_results.json", serde_json::to_string_pretty(&results)?)?;

    // Mostrar resultados
    CLI::print_backtest_summary(&metrics, config.trading.initial_capital);
    CLI::print_trade_list(&trades);

    CLI::print_success("Resultados salvos em backtest_results.json");

    Ok(())
}

async fn run_montecarlo_interactive() -> Result<()> {
    use robo_trading::{api::BinanceClient, backtest::Backtester, montecarlo::MonteCarloSimulator, config::Config, cli::CLI};

    let config = Config::from_env().map_err(|e| anyhow::anyhow!("Erro ao carregar config: {}", e))?;
    
    println!("⏳ Buscando dados históricos...");
    
    let client = BinanceClient::new(
        config.exchange.api_key.clone(),
        config.exchange.secret_key.clone(),
        config.exchange.use_testnet,
    );

    let candles = client
        .get_klines(&config.trading.trading_pair, "1h", 500)
        .await?;

    println!("✅ {} candles carregados", candles.len());
    println!("⚙️  Rodando backtest inicial...\n");

    let mut backtester = Backtester::new(
        config.strategy.clone(),
        config.risk.clone(),
        config.execution.clone(),
    );
    let (_, trades) = backtester.run(candles).await?;

    if trades.is_empty() {
        CLI::print_error("Nenhum trade foi executado. Ajuste os parâmetros da estratégia.");
        return Ok(());
    }

    println!("🎲 Executando 10.000 simulações Monte Carlo...");
    
    let simulator = MonteCarloSimulator::new(config.montecarlo.num_simulations);
    let results = simulator.simulate(&trades, config.trading.initial_capital);

    // Salvar resultados
    fs::write("montecarlo_results.json", serde_json::to_string_pretty(&results)?)?;

    // Mostrar resultados
    CLI::print_monte_carlo_summary(
        config.montecarlo.num_simulations,
        results.expected_return,
        config.trading.initial_capital,
        results.best_case,
        results.worst_case,
        results.profit_probability,
        results.max_drawdown,
    );

    CLI::print_success("Resultados salvos em montecarlo_results.json");

    Ok(())
}

async fn run_live_trading_interactive(testnet: bool) -> Result<()> {
    use robo_trading::{api::BinanceClient, config::Config, cli::CLI};

    let mut config = Config::from_env().map_err(|e| anyhow::anyhow!("Erro ao carregar config: {}", e))?;
    config.exchange.use_testnet = testnet;

    if !testnet {
        CLI::print_warning("OPERANDO COM DINHEIRO REAL!");
    }

    println!("🚀 Iniciando robô de trading...");
    println!("⏳ Conectando à Binance...");

    let client = BinanceClient::new(
        config.exchange.api_key.clone(),
        config.exchange.secret_key.clone(),
        config.exchange.use_testnet,
    );

    // Testar conexão
    match client.ping().await {
        Ok(_) => println!("✅ Conectado à Binance!"),
        Err(e) => {
            CLI::print_error(&format!("Falha ao conectar: {}", e));
            return Ok(());
        }
    }

    CLI::print_warning("Modo ao vivo ainda não implementado completamente.");
    println!("⚙️  Em breve: monitoramento em tempo real, execução automática de trades.");
    println!("💡 Por enquanto, use o backtest e Monte Carlo para validar sua estratégia.");

    Ok(())
}

fn show_last_backtest() {
    use robo_trading::cli::CLI;

    match fs::read_to_string("backtest_results.json") {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => {
                    if let Some(metrics) = json.get("metrics") {
                        match serde_json::from_value(metrics.clone()) {
                            Ok(metrics) => {
                                let initial_capital = json.get("initial_capital")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(10000.0);
                                
                                CLI::print_backtest_summary(&metrics, initial_capital);
                                
                                if let Some(trades) = json.get("trades") {
                                    if let Ok(trades) = serde_json::from_value::<Vec<robo_trading::backtest::Trade>>(trades.clone()) {
                                        CLI::print_trade_list(&trades);
                                    }
                                }
                            }
                            Err(e) => CLI::print_error(&format!("Erro ao parsear métricas: {}", e)),
                        }
                    }
                }
                Err(e) => CLI::print_error(&format!("Erro ao ler JSON: {}", e)),
            }
        }
        Err(_) => {
            CLI::print_error("Nenhum backtest anterior encontrado.");
            println!("Execute a opção 1 primeiro para gerar resultados.");
        }
    }
}

fn show_last_montecarlo() {
    use robo_trading::cli::CLI;

    match fs::read_to_string("montecarlo_results.json") {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => {
                    let simulations = json.get("simulations").and_then(|v| v.as_u64()).unwrap_or(10000) as usize;
                    let expected_return = json.get("expected_return").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let best_case = json.get("best_case").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let worst_case = json.get("worst_case").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let profit_probability = json.get("profit_probability").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let max_drawdown = json.get("max_drawdown").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let initial_capital = json.get("initial_capital").and_then(|v| v.as_f64()).unwrap_or(10000.0);

                    CLI::print_monte_carlo_summary(
                        simulations,
                        expected_return,
                        initial_capital,
                        best_case,
                        worst_case,
                        profit_probability,
                        max_drawdown,
                    );
                }
                Err(e) => CLI::print_error(&format!("Erro ao ler JSON: {}", e)),
            }
        }
        Err(_) => {
            CLI::print_error("Nenhuma simulação anterior encontrada.");
            println!("Execute a opção 2 primeiro para gerar resultados.");
        }
    }
}
