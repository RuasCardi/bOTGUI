use crate::config::RiskConfig;
use crate::types::{Candle, OrderSide};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRisk {
    pub symbol: String,
    pub side: OrderSide,
    pub entry_price: f64,
    pub quantity: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub trailing_stop: Option<f64>,
    pub max_price: f64, // Para trailing stop
    pub min_price: f64, // Para trailing stop
    pub atr_at_entry: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub total_exposure: f64,
    pub total_risk: f64,
    pub daily_pnl: f64,
    pub daily_loss_percent: f64,
    pub max_drawdown: f64,
    pub open_positions: usize,
    pub can_open_new: bool,
}

pub struct RiskManager {
    config: RiskConfig,
    positions: Arc<RwLock<HashMap<String, PositionRisk>>>,
    initial_capital: f64,
    current_capital: Arc<RwLock<f64>>,
    daily_start_capital: Arc<RwLock<f64>>,
    peak_capital: Arc<RwLock<f64>>,
    max_positions: usize,
}

impl RiskManager {
    pub fn new(config: RiskConfig, initial_capital: f64, max_positions: usize) -> Self {
        info!(
            "Initializing Risk Manager - Capital: ${:.2}, Max positions: {}",
            initial_capital, max_positions
        );

        Self {
            config,
            positions: Arc::new(RwLock::new(HashMap::new())),
            initial_capital,
            current_capital: Arc::new(RwLock::new(initial_capital)),
            daily_start_capital: Arc::new(RwLock::new(initial_capital)),
            peak_capital: Arc::new(RwLock::new(initial_capital)),
            max_positions,
        }
    }

    /// Calcula stop loss baseado em ATR
    pub fn calculate_stop_loss(
        &self,
        entry_price: f64,
        atr: f64,
        side: OrderSide,
    ) -> f64 {
        let stop_distance = atr * self.config.stop_loss_atr_multiplier;

        match side {
            OrderSide::Buy => entry_price - stop_distance,
            OrderSide::Sell => entry_price + stop_distance,
        }
    }

    /// Calcula take profit baseado em ATR
    pub fn calculate_take_profit(
        &self,
        entry_price: f64,
        atr: f64,
        side: OrderSide,
    ) -> f64 {
        let tp_distance = atr * self.config.take_profit_atr_multiplier;

        match side {
            OrderSide::Buy => entry_price + tp_distance,
            OrderSide::Sell => entry_price - tp_distance,
        }
    }

    /// Adiciona uma nova posição
    pub fn add_position(
        &self,
        symbol: String,
        side: OrderSide,
        entry_price: f64,
        quantity: f64,
        atr: f64,
    ) {
        let stop_loss = self.calculate_stop_loss(entry_price, atr, side);
        let take_profit = self.calculate_take_profit(entry_price, atr, side);

        let position = PositionRisk {
            symbol: symbol.clone(),
            side,
            entry_price,
            quantity,
            stop_loss,
            take_profit,
            trailing_stop: None,
            max_price: entry_price,
            min_price: entry_price,
            atr_at_entry: atr,
        };

        info!(
            "Adding position: {} {:?} @ {:.2} | SL: {:.2} | TP: {:.2}",
            symbol, side, entry_price, stop_loss, take_profit
        );

        self.positions.write().insert(symbol, position);
    }

    /// Remove uma posição
    pub fn remove_position(&self, symbol: &str) -> Option<PositionRisk> {
        info!("Removing position: {}", symbol);
        self.positions.write().remove(symbol)
    }

    /// Atualiza trailing stop
    pub fn update_trailing_stop(&self, symbol: &str, current_price: f64) -> Option<f64> {
        let mut positions = self.positions.write();
        
        if let Some(position) = positions.get_mut(symbol) {
            // Atualiza max/min price
            if current_price > position.max_price {
                position.max_price = current_price;
            }
            if current_price < position.min_price {
                position.min_price = current_price;
            }

            let price_change_pct = match position.side {
                OrderSide::Buy => (current_price - position.entry_price) / position.entry_price,
                OrderSide::Sell => (position.entry_price - current_price) / position.entry_price,
            };

            // Ativa trailing stop se atingir threshold
            if price_change_pct >= self.config.trailing_stop_activation {
                let trailing_distance = current_price * self.config.trailing_stop_distance;

                let new_trailing = match position.side {
                    OrderSide::Buy => position.max_price - trailing_distance,
                    OrderSide::Sell => position.min_price + trailing_distance,
                };

                // Só atualiza se for melhor que o anterior
                if position.trailing_stop.is_none() 
                    || (position.side == OrderSide::Buy && new_trailing > position.trailing_stop.unwrap())
                    || (position.side == OrderSide::Sell && new_trailing < position.trailing_stop.unwrap())
                {
                    position.trailing_stop = Some(new_trailing);
                    info!(
                        "Updated trailing stop for {}: {:.2} (profit: {:.2}%)",
                        symbol,
                        new_trailing,
                        price_change_pct * 100.0
                    );
                    return Some(new_trailing);
                }
            }
        }

        None
    }

    /// Verifica se stop loss foi atingido
    pub fn check_stop_loss(&self, symbol: &str, current_price: f64) -> bool {
        let positions = self.positions.read();
        
        if let Some(position) = positions.get(symbol) {
            // Verifica trailing stop primeiro
            if let Some(trailing) = position.trailing_stop {
                let hit = match position.side {
                    OrderSide::Buy => current_price <= trailing,
                    OrderSide::Sell => current_price >= trailing,
                };

                if hit {
                    warn!(
                        "Trailing stop hit for {}: price {:.2} <= trailing {:.2}",
                        symbol, current_price, trailing
                    );
                    return true;
                }
            }

            // Verifica stop loss fixo
            let hit = match position.side {
                OrderSide::Buy => current_price <= position.stop_loss,
                OrderSide::Sell => current_price >= position.stop_loss,
            };

            if hit {
                warn!(
                    "Stop loss hit for {}: price {:.2} <= SL {:.2}",
                    symbol, current_price, position.stop_loss
                );
            }

            return hit;
        }

        false
    }

    /// Verifica se take profit foi atingido
    pub fn check_take_profit(&self, symbol: &str, current_price: f64) -> bool {
        let positions = self.positions.read();
        
        if let Some(position) = positions.get(symbol) {
            let hit = match position.side {
                OrderSide::Buy => current_price >= position.take_profit,
                OrderSide::Sell => current_price <= position.take_profit,
            };

            if hit {
                info!(
                    "Take profit hit for {}: price {:.2} >= TP {:.2}",
                    symbol, current_price, position.take_profit
                );
            }

            return hit;
        }

        false
    }

    /// Calcula métricas de risco atuais
    pub fn get_risk_metrics(&self, current_prices: &HashMap<String, f64>) -> RiskMetrics {
        let positions = self.positions.read();
        let current_capital = *self.current_capital.read();
        let daily_start = *self.daily_start_capital.read();
        let peak = *self.peak_capital.read();

        let mut total_exposure = 0.0;
        let mut total_risk = 0.0;

        for (symbol, position) in positions.iter() {
            if let Some(&price) = current_prices.get(symbol) {
                let position_value = position.quantity * price;
                total_exposure += position_value;

                let risk = match position.side {
                    OrderSide::Buy => {
                        position.quantity * (position.entry_price - position.stop_loss)
                    }
                    OrderSide::Sell => {
                        position.quantity * (position.stop_loss - position.entry_price)
                    }
                };
                total_risk += risk;
            }
        }

        let daily_pnl = current_capital - daily_start;
        let daily_loss_percent = if daily_pnl < 0.0 {
            daily_pnl.abs() / daily_start
        } else {
            0.0
        };

        let max_drawdown = if current_capital < peak {
            (peak - current_capital) / peak
        } else {
            0.0
        };

        let can_open_new = positions.len() < self.max_positions
            && daily_loss_percent < self.config.max_daily_loss_percent;

        RiskMetrics {
            total_exposure,
            total_risk,
            daily_pnl,
            daily_loss_percent,
            max_drawdown,
            open_positions: positions.len(),
            can_open_new,
        }
    }

    /// Atualiza capital atual
    pub fn update_capital(&self, new_capital: f64) {
        let mut capital = self.current_capital.write();
        let mut peak = self.peak_capital.write();

        *capital = new_capital;

        if new_capital > *peak {
            *peak = new_capital;
        }
    }

    /// Reseta métricas diárias (chamar no início de cada dia)
    pub fn reset_daily_metrics(&self) {
        let current = *self.current_capital.read();
        *self.daily_start_capital.write() = current;
        info!("Daily metrics reset - Starting capital: ${:.2}", current);
    }

    /// Verifica se pode abrir nova posição
    pub fn can_open_position(&self, current_prices: &HashMap<String, f64>) -> bool {
        let metrics = self.get_risk_metrics(current_prices);

        if !metrics.can_open_new {
            if metrics.open_positions >= self.max_positions {
                warn!("Cannot open position: max positions reached ({})", self.max_positions);
            } else if metrics.daily_loss_percent >= self.config.max_daily_loss_percent {
                warn!(
                    "Cannot open position: daily loss limit reached ({:.2}%)",
                    metrics.daily_loss_percent * 100.0
                );
            }
        }

        metrics.can_open_new
    }

    /// Obtém posição por símbolo
    pub fn get_position(&self, symbol: &str) -> Option<PositionRisk> {
        self.positions.read().get(symbol).cloned()
    }

    /// Lista todas as posições
    pub fn list_positions(&self) -> Vec<PositionRisk> {
        self.positions.read().values().cloned().collect()
    }

    /// Calcula PnL de uma posição
    pub fn calculate_position_pnl(&self, symbol: &str, current_price: f64) -> Option<f64> {
        self.positions.read().get(symbol).map(|pos| {
            match pos.side {
                OrderSide::Buy => (current_price - pos.entry_price) * pos.quantity,
                OrderSide::Sell => (pos.entry_price - current_price) * pos.quantity,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_loss_calculation() {
        let config = RiskConfig {
            stop_loss_atr_multiplier: 1.5,
            take_profit_atr_multiplier: 2.5,
            trailing_stop_activation: 0.015,
            trailing_stop_distance: 0.008,
            max_daily_loss_percent: 0.05,
        };

        let risk_manager = RiskManager::new(config, 10000.0, 3);

        let sl_long = risk_manager.calculate_stop_loss(100.0, 2.0, OrderSide::Buy);
        assert_eq!(sl_long, 97.0); // 100 - (2.0 * 1.5)

        let sl_short = risk_manager.calculate_stop_loss(100.0, 2.0, OrderSide::Sell);
        assert_eq!(sl_short, 103.0); // 100 + (2.0 * 1.5)
    }

    #[test]
    fn test_position_management() {
        let config = RiskConfig {
            stop_loss_atr_multiplier: 1.5,
            take_profit_atr_multiplier: 2.5,
            trailing_stop_activation: 0.015,
            trailing_stop_distance: 0.008,
            max_daily_loss_percent: 0.05,
        };

        let risk_manager = RiskManager::new(config, 10000.0, 3);

        risk_manager.add_position(
            "BTCUSDT".to_string(),
            OrderSide::Buy,
            50000.0,
            0.1,
            500.0,
        );

        assert!(risk_manager.get_position("BTCUSDT").is_some());
        assert_eq!(risk_manager.list_positions().len(), 1);

        risk_manager.remove_position("BTCUSDT");
        assert!(risk_manager.get_position("BTCUSDT").is_none());
    }
}
