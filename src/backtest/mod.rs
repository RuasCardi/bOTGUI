use crate::config::StrategyConfig;
use crate::strategy::{Signal, TradingStrategy};
use crate::types::{Candle, OrderSide};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub entry_time: i64,
    pub exit_time: i64,
    pub side: OrderSide,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub pnl_percent: f64,
    pub holding_period: i64,
    pub exit_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub total_return_percent: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub max_drawdown: f64,
    pub max_drawdown_percent: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub avg_holding_period_hours: f64,
    pub max_consecutive_wins: usize,
    pub max_consecutive_losses: usize,
    pub expectancy: f64,
}

#[derive(Debug, Clone)]
struct Position {
    side: OrderSide,
    entry_price: f64,
    entry_time: i64,
    quantity: f64,
    stop_loss: f64,
    take_profit: f64,
}

pub struct Backtester {
    strategy: TradingStrategy,
    initial_capital: f64,
    position_size_percent: f64,
    commission_percent: f64,
}

impl Backtester {
    pub fn new(
        strategy_config: StrategyConfig,
        initial_capital: f64,
        position_size_percent: f64,
    ) -> Self {
        info!("Initializing Backtester with ${:.2} capital", initial_capital);

        Self {
            strategy: TradingStrategy::new(strategy_config),
            initial_capital,
            position_size_percent,
            commission_percent: 0.001, // 0.1% de comissão
        }
    }

    /// Executa backtest completo
    pub fn run(&self, candles: &[Candle]) -> BacktestResult {
        info!("Running backtest on {} candles", candles.len());

        let mut capital = self.initial_capital;
        let mut peak_capital = self.initial_capital;
        let mut current_position: Option<Position> = None;
        let mut trades: Vec<Trade> = Vec::new();
        let mut equity_curve: Vec<f64> = vec![self.initial_capital];

        // Precisa de histórico suficiente para indicadores
        let min_history = 100;

        for i in min_history..candles.len() {
            let history = &candles[0..=i];
            let current_candle = &candles[i];
            let current_price = current_candle.close;

            // Verifica se tem posição aberta
            if let Some(ref position) = current_position {
                // Verifica stop loss
                if self.check_stop_loss(position, current_price) {
                    let trade = self.close_position(
                        position,
                        current_price,
                        current_candle.timestamp,
                        "Stop Loss",
                    );
                    capital += trade.pnl;
                    trades.push(trade);
                    current_position = None;
                }
                // Verifica take profit
                else if self.check_take_profit(position, current_price) {
                    let trade = self.close_position(
                        position,
                        current_price,
                        current_candle.timestamp,
                        "Take Profit",
                    );
                    capital += trade.pnl;
                    trades.push(trade);
                    current_position = None;
                }
                // Verifica sinal de saída da estratégia
                else if self
                    .strategy
                    .should_close_position(history, position.side, position.entry_price)
                {
                    let trade = self.close_position(
                        position,
                        current_price,
                        current_candle.timestamp,
                        "Strategy Signal",
                    );
                    capital += trade.pnl;
                    trades.push(trade);
                    current_position = None;
                }
            }

            // Se não tem posição, busca sinal de entrada
            if current_position.is_none() {
                if let Some(signal) = self.strategy.analyze(history) {
                    match signal.signal {
                        Signal::Long | Signal::Short => {
                            let side = if signal.signal == Signal::Long {
                                OrderSide::Buy
                            } else {
                                OrderSide::Sell
                            };

                            // Calcula tamanho da posição
                            let position_value = capital * self.position_size_percent;
                            let quantity = position_value / current_price;

                            // Calcula stop loss e take profit
                            let atr = signal.indicators.atr;
                            let stop_loss = match side {
                                OrderSide::Buy => current_price - (atr * 1.5),
                                OrderSide::Sell => current_price + (atr * 1.5),
                            };
                            let take_profit = match side {
                                OrderSide::Buy => current_price + (atr * 2.5),
                                OrderSide::Sell => current_price - (atr * 2.5),
                            };

                            // Desconta comissão de entrada
                            capital -= position_value * self.commission_percent;

                            current_position = Some(Position {
                                side,
                                entry_price: current_price,
                                entry_time: current_candle.timestamp,
                                quantity,
                                stop_loss,
                                take_profit,
                            });
                        }
                        _ => {}
                    }
                }
            }

            // Atualiza peak capital
            if capital > peak_capital {
                peak_capital = capital;
            }

            equity_curve.push(capital);
        }

        // Fecha posição se ainda estiver aberta
        if let Some(position) = current_position {
            let last_candle = candles.last().unwrap();
            let trade = self.close_position(
                &position,
                last_candle.close,
                last_candle.timestamp,
                "End of Backtest",
            );
            capital += trade.pnl;
            trades.push(trade);
        }

        let metrics = self.calculate_metrics(&trades, &equity_curve);

        info!("Backtest completed: {} trades, {:.2}% return", 
            trades.len(), metrics.total_return_percent);

        BacktestResult {
            trades,
            metrics,
            equity_curve,
            initial_capital: self.initial_capital,
            final_capital: capital,
        }
    }

    fn check_stop_loss(&self, position: &Position, current_price: f64) -> bool {
        match position.side {
            OrderSide::Buy => current_price <= position.stop_loss,
            OrderSide::Sell => current_price >= position.stop_loss,
        }
    }

    fn check_take_profit(&self, position: &Position, current_price: f64) -> bool {
        match position.side {
            OrderSide::Buy => current_price >= position.take_profit,
            OrderSide::Sell => current_price <= position.take_profit,
        }
    }

    fn close_position(
        &self,
        position: &Position,
        exit_price: f64,
        exit_time: i64,
        exit_reason: &str,
    ) -> Trade {
        let pnl = match position.side {
            OrderSide::Buy => (exit_price - position.entry_price) * position.quantity,
            OrderSide::Sell => (position.entry_price - exit_price) * position.quantity,
        };

        // Desconta comissão de saída
        let position_value = exit_price * position.quantity;
        let pnl_after_commission = pnl - (position_value * self.commission_percent);

        let pnl_percent = (pnl_after_commission / (position.entry_price * position.quantity)) * 100.0;
        let holding_period = exit_time - position.entry_time;

        Trade {
            entry_time: position.entry_time,
            exit_time,
            side: position.side,
            entry_price: position.entry_price,
            exit_price,
            quantity: position.quantity,
            pnl: pnl_after_commission,
            pnl_percent,
            holding_period,
            exit_reason: exit_reason.to_string(),
        }
    }

    fn calculate_metrics(&self, trades: &[Trade], equity_curve: &[f64]) -> BacktestMetrics {
        if trades.is_empty() {
            warn!("No trades to calculate metrics");
            return BacktestMetrics::default();
        }

        let total_trades = trades.len();
        let winning_trades = trades.iter().filter(|t| t.pnl > 0.0).count();
        let losing_trades = trades.iter().filter(|t| t.pnl < 0.0).count();
        let win_rate = winning_trades as f64 / total_trades as f64;

        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let total_return_percent = (total_pnl / self.initial_capital) * 100.0;

        let wins: Vec<f64> = trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).collect();
        let losses: Vec<f64> = trades.iter().filter(|t| t.pnl < 0.0).map(|t| t.pnl.abs()).collect();

        let avg_win = if !wins.is_empty() {
            wins.iter().sum::<f64>() / wins.len() as f64
        } else {
            0.0
        };

        let avg_loss = if !losses.is_empty() {
            losses.iter().sum::<f64>() / losses.len() as f64
        } else {
            0.0
        };

        let profit_factor = if avg_loss > 0.0 {
            (avg_win * wins.len() as f64) / (avg_loss * losses.len() as f64)
        } else {
            0.0
        };

        // Calcula drawdown máximo
        let (max_dd, max_dd_pct) = self.calculate_max_drawdown(equity_curve);

        // Calcula Sharpe Ratio
        let returns = self.calculate_returns(equity_curve);
        let sharpe = self.calculate_sharpe_ratio(&returns);
        let sortino = self.calculate_sortino_ratio(&returns);
        let calmar = if max_dd_pct > 0.0 {
            total_return_percent / max_dd_pct
        } else {
            0.0
        };

        // Períodos de holding
        let avg_holding = trades.iter().map(|t| t.holding_period).sum::<i64>() as f64
            / trades.len() as f64
            / 3600000.0; // Converte para horas

        // Sequências
        let (max_wins, max_losses) = self.calculate_consecutive_streaks(trades);

        // Expectancy
        let expectancy = (win_rate * avg_win) - ((1.0 - win_rate) * avg_loss);

        BacktestMetrics {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            total_pnl,
            total_return_percent,
            avg_win,
            avg_loss,
            profit_factor,
            max_drawdown: max_dd,
            max_drawdown_percent: max_dd_pct,
            sharpe_ratio: sharpe,
            sortino_ratio: sortino,
            calmar_ratio: calmar,
            avg_holding_period_hours: avg_holding,
            max_consecutive_wins: max_wins,
            max_consecutive_losses: max_losses,
            expectancy,
        }
    }

    fn calculate_max_drawdown(&self, equity: &[f64]) -> (f64, f64) {
        let mut max_dd = 0.0;
        let mut max_dd_pct = 0.0;
        let mut peak = equity[0];

        for &value in equity {
            if value > peak {
                peak = value;
            }

            let dd = peak - value;
            let dd_pct = (dd / peak) * 100.0;

            if dd > max_dd {
                max_dd = dd;
                max_dd_pct = dd_pct;
            }
        }

        (max_dd, max_dd_pct)
    }

    fn calculate_returns(&self, equity: &[f64]) -> Vec<f64> {
        let mut returns = Vec::new();
        for i in 1..equity.len() {
            let ret = (equity[i] - equity[i - 1]) / equity[i - 1];
            returns.push(ret);
        }
        returns
    }

    fn calculate_sharpe_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            // Anualiza (assumindo retornos diários)
            (mean_return / std_dev) * (252.0_f64).sqrt()
        } else {
            0.0
        }
    }

    fn calculate_sortino_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        
        let downside_returns: Vec<f64> = returns
            .iter()
            .filter(|&&r| r < 0.0)
            .copied()
            .collect();

        if downside_returns.is_empty() {
            return 0.0;
        }

        let downside_variance = downside_returns
            .iter()
            .map(|r| r.powi(2))
            .sum::<f64>()
            / downside_returns.len() as f64;
        let downside_std = downside_variance.sqrt();

        if downside_std > 0.0 {
            (mean_return / downside_std) * (252.0_f64).sqrt()
        } else {
            0.0
        }
    }

    fn calculate_consecutive_streaks(&self, trades: &[Trade]) -> (usize, usize) {
        let mut max_wins = 0;
        let mut max_losses = 0;
        let mut current_wins = 0;
        let mut current_losses = 0;

        for trade in trades {
            if trade.pnl > 0.0 {
                current_wins += 1;
                current_losses = 0;
                if current_wins > max_wins {
                    max_wins = current_wins;
                }
            } else {
                current_losses += 1;
                current_wins = 0;
                if current_losses > max_losses {
                    max_losses = current_losses;
                }
            }
        }

        (max_wins, max_losses)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub trades: Vec<Trade>,
    pub metrics: BacktestMetrics,
    pub equity_curve: Vec<f64>,
    pub initial_capital: f64,
    pub final_capital: f64,
}

impl Default for BacktestMetrics {
    fn default() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate: 0.0,
            total_pnl: 0.0,
            total_return_percent: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            profit_factor: 0.0,
            max_drawdown: 0.0,
            max_drawdown_percent: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            avg_holding_period_hours: 0.0,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            expectancy: 0.0,
        }
    }
}

impl BacktestResult {
    pub fn print_summary(&self) {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║              BACKTEST RESULTS SUMMARY                    ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Capital Inicial:     ${:>12.2}                     ║", self.initial_capital);
        println!("║ Capital Final:       ${:>12.2}                     ║", self.final_capital);
        println!("║ PnL Total:           ${:>12.2}                     ║", self.metrics.total_pnl);
        println!("║ Retorno:             {:>12.2}%                     ║", self.metrics.total_return_percent);
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Total de Trades:     {:>6}                             ║", self.metrics.total_trades);
        println!("║ Trades Vencedores:   {:>6} ({:>5.1}%)                    ║", 
            self.metrics.winning_trades, self.metrics.win_rate * 100.0);
        println!("║ Trades Perdedores:   {:>6} ({:>5.1}%)                    ║", 
            self.metrics.losing_trades, (1.0 - self.metrics.win_rate) * 100.0);
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Win Rate:            {:>12.2}%                     ║", self.metrics.win_rate * 100.0);
        println!("║ Ganho Médio:         ${:>12.2}                     ║", self.metrics.avg_win);
        println!("║ Perda Média:         ${:>12.2}                     ║", self.metrics.avg_loss);
        println!("║ Profit Factor:       {:>12.2}                      ║", self.metrics.profit_factor);
        println!("║ Expectancy:          ${:>12.2}                     ║", self.metrics.expectancy);
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Max Drawdown:        ${:>12.2} ({:>5.2}%)           ║", 
            self.metrics.max_drawdown, self.metrics.max_drawdown_percent);
        println!("║ Sharpe Ratio:        {:>12.2}                      ║", self.metrics.sharpe_ratio);
        println!("║ Sortino Ratio:       {:>12.2}                      ║", self.metrics.sortino_ratio);
        println!("║ Calmar Ratio:        {:>12.2}                      ║", self.metrics.calmar_ratio);
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Holding Médio:       {:>12.2} horas                ║", self.metrics.avg_holding_period_hours);
        println!("║ Sequência Vitórias:  {:>6}                             ║", self.metrics.max_consecutive_wins);
        println!("║ Sequência Derrotas:  {:>6}                             ║", self.metrics.max_consecutive_losses);
        println!("╚══════════════════════════════════════════════════════════╝\n");
    }
}
