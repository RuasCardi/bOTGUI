use crate::config::StrategyConfig;
use crate::strategy::indicators::*;
use crate::types::{Candle, OrderSide};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Signal {
    Long,
    Short,
    Neutral,
    CloseLong,
    CloseShort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRegime {
    pub trend: String,      // "uptrend", "downtrend", "sideways"
    pub volatility: String, // "low", "normal", "high"
    pub strength: f64,      // 0.0 a 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySignal {
    pub signal: Signal,
    pub confidence: f64,
    pub entry_price: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub regime: MarketRegime,
    pub indicators: IndicatorValues,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorValues {
    pub ema_fast: f64,
    pub ema_slow: f64,
    pub rsi: f64,
    pub atr: f64,
    pub atr_avg: f64,
}

pub struct TradingStrategy {
    config: StrategyConfig,
}

impl TradingStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        info!("Initializing trading strategy with config: {:?}", config);
        Self { config }
    }

    /// Analisa o mercado e gera sinal de trading
    pub fn analyze(&self, candles: &[Candle]) -> Option<StrategySignal> {
        if candles.len() < self.config.ema_slow + 10 {
            debug!("Insufficient candles for analysis");
            return None;
        }

        // Extrai preços de fechamento
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

        // Calcula indicadores
        let ema_fast = calculate_ema(&closes, self.config.ema_fast);
        let ema_slow = calculate_ema(&closes, self.config.ema_slow);
        let rsi = calculate_rsi(&closes, self.config.rsi_period);
        let atr = calculate_atr(candles, self.config.atr_period);

        if ema_fast.is_empty() || ema_slow.is_empty() || rsi.is_empty() || atr.is_empty() {
            return None;
        }

        // Pega valores mais recentes
        let current_ema_fast = *ema_fast.last().unwrap();
        let current_ema_slow = *ema_slow.last().unwrap();
        let current_rsi = *rsi.last().unwrap();
        let current_atr = *atr.last().unwrap();
        let current_price = closes.last().unwrap();

        // Calcula ATR médio
        let atr_avg: f64 = atr.iter().sum::<f64>() / atr.len() as f64;
        let atr_threshold = atr_avg * self.config.atr_threshold_multiplier;

        // Detecta regime de mercado
        let regime = self.detect_market_regime(
            current_ema_fast,
            current_ema_slow,
            current_rsi,
            &atr,
        );

        // Valores dos indicadores
        let indicator_values = IndicatorValues {
            ema_fast: current_ema_fast,
            ema_slow: current_ema_slow,
            rsi: current_rsi,
            atr: current_atr,
            atr_avg,
        };

        // ========== LÓGICA DE SINAL ==========

        let mut signal = Signal::Neutral;
        let mut confidence = 0.0;

        // LONG: EMA20 > EMA50, RSI > 50, ATR > threshold
        if current_ema_fast > current_ema_slow
            && current_rsi > 50.0
            && current_atr > atr_threshold
        {
            signal = Signal::Long;

            // Calcula confiança baseada em múltiplos fatores
            let ema_spread = (current_ema_fast - current_ema_slow) / current_ema_slow;
            let rsi_strength = (current_rsi - 50.0) / 50.0; // 0 a 1
            let atr_strength = (current_atr / atr_avg).min(2.0) / 2.0; // 0 a 1

            confidence = (ema_spread * 10.0 + rsi_strength + atr_strength) / 3.0;
            confidence = confidence.clamp(0.0, 1.0);

            info!(
                "LONG signal detected - Confidence: {:.2}%, EMA spread: {:.2}%, RSI: {:.2}, ATR: {:.2}",
                confidence * 100.0,
                ema_spread * 100.0,
                current_rsi,
                current_atr
            );
        }
        // SHORT: EMA20 < EMA50, RSI < 50, ATR > threshold
        else if current_ema_fast < current_ema_slow
            && current_rsi < 50.0
            && current_atr > atr_threshold
        {
            signal = Signal::Short;

            let ema_spread = (current_ema_slow - current_ema_fast) / current_ema_slow;
            let rsi_strength = (50.0 - current_rsi) / 50.0;
            let atr_strength = (current_atr / atr_avg).min(2.0) / 2.0;

            confidence = (ema_spread * 10.0 + rsi_strength + atr_strength) / 3.0;
            confidence = confidence.clamp(0.0, 1.0);

            info!(
                "SHORT signal detected - Confidence: {:.2}%, EMA spread: {:.2}%, RSI: {:.2}, ATR: {:.2}",
                confidence * 100.0,
                ema_spread * 100.0,
                current_rsi,
                current_atr
            );
        }
        // Condições de saída
        else if current_rsi > self.config.rsi_overbought {
            signal = Signal::CloseLong;
            confidence = 0.8;
            info!("Close LONG signal - RSI overbought: {:.2}", current_rsi);
        } else if current_rsi < self.config.rsi_oversold {
            signal = Signal::CloseShort;
            confidence = 0.8;
            info!("Close SHORT signal - RSI oversold: {:.2}", current_rsi);
        }

        // Só retorna sinal se tiver confiança mínima
        if signal != Signal::Neutral && confidence < 0.3 {
            debug!("Signal confidence too low: {:.2}%", confidence * 100.0);
            return None;
        }

        Some(StrategySignal {
            signal,
            confidence,
            entry_price: *current_price,
            stop_loss: None,  // Será calculado pelo risk manager
            take_profit: None, // Será calculado pelo risk manager
            regime,
            indicators: indicator_values,
        })
    }

    /// Detecta regime de mercado atual
    fn detect_market_regime(
        &self,
        ema_fast: f64,
        ema_slow: f64,
        rsi: f64,
        atr_values: &[f64],
    ) -> MarketRegime {
        // Detecta tendência
        let trend = if ema_fast > ema_slow * 1.02 {
            "uptrend".to_string()
        } else if ema_fast < ema_slow * 0.98 {
            "downtrend".to_string()
        } else {
            "sideways".to_string()
        };

        // Detecta volatilidade
        let volatility = detect_volatility_regime(atr_values, 10);

        // Calcula força da tendência
        let ema_diff = ((ema_fast - ema_slow) / ema_slow).abs();
        let rsi_deviation = ((rsi - 50.0) / 50.0).abs();
        let strength = (ema_diff * 10.0 + rsi_deviation).min(1.0);

        MarketRegime {
            trend,
            volatility,
            strength,
        }
    }

    /// Verifica se deve fechar posição existente
    pub fn should_close_position(
        &self,
        candles: &[Candle],
        position_side: OrderSide,
        entry_price: f64,
    ) -> bool {
        if let Some(signal) = self.analyze(candles) {
            match (position_side, signal.signal) {
                (OrderSide::Buy, Signal::Short) => true,
                (OrderSide::Buy, Signal::CloseLong) => true,
                (OrderSide::Sell, Signal::Long) => true,
                (OrderSide::Sell, Signal::CloseShort) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Calcula tamanho da posição baseado em volatilidade
    pub fn calculate_position_size(
        &self,
        capital: f64,
        price: f64,
        atr: f64,
        risk_percent: f64,
    ) -> f64 {
        // Kelly Criterion adaptado
        // Position size = (edge / odds) * capital
        // Simplificado: baseado em risco e volatilidade

        let risk_amount = capital * risk_percent;
        let atr_stop = atr * 1.5; // Stop loss baseado em ATR

        let position_size = risk_amount / atr_stop;

        // Limita a 20% do capital por posição
        let max_position = capital * 0.2 / price;
        position_size.min(max_position)
    }

    /// Calcula probabilidade de sucesso do trade
    pub fn calculate_trade_probability(&self, signal: &StrategySignal) -> f64 {
        let mut probability = 0.5; // Base 50%

        // Ajusta baseado na força do regime
        probability += signal.regime.strength * 0.2;

        // Ajusta baseado na confiança
        probability += signal.confidence * 0.2;

        // Ajusta baseado na volatilidade
        match signal.regime.volatility.as_str() {
            "high" => probability -= 0.1,
            "low" => probability -= 0.05,
            _ => {}
        }

        // Ajusta baseado no RSI
        let rsi_optimal = (signal.indicators.rsi - 50.0).abs() < 20.0;
        if rsi_optimal {
            probability += 0.1;
        }

        probability.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candles(count: usize) -> Vec<Candle> {
        (0..count)
            .map(|i| Candle {
                timestamp: i as i64 * 60000,
                open: 100.0 + i as f64,
                high: 102.0 + i as f64,
                low: 99.0 + i as f64,
                close: 101.0 + i as f64,
                volume: 1000.0,
            })
            .collect()
    }

    #[test]
    fn test_strategy_analysis() {
        let config = StrategyConfig {
            ema_fast: 20,
            ema_slow: 50,
            rsi_period: 14,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
            atr_period: 14,
            atr_threshold_multiplier: 1.2,
        };

        let strategy = TradingStrategy::new(config);
        let candles = create_test_candles(100);

        let result = strategy.analyze(&candles);
        assert!(result.is_some());
    }
}
