use crate::types::Candle;

/// Calcula EMA (Exponential Moving Average)
pub fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema_values = Vec::with_capacity(prices.len());

    // Primeiro valor é SMA
    let sma: f64 = prices.iter().take(period).sum::<f64>() / period as f64;
    ema_values.push(sma);

    // Calcula EMA para os valores restantes
    for &price in &prices[period..] {
        let last_ema = *ema_values.last().unwrap();
        let ema = (price - last_ema) * multiplier + last_ema;
        ema_values.push(ema);
    }

    ema_values
}

/// Calcula RSI (Relative Strength Index)
pub fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    // Calcula ganhos e perdas
    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(change.abs());
        }
    }

    let mut rsi_values = Vec::new();

    // Primeiro RSI usa média simples
    let avg_gain: f64 = gains.iter().take(period).sum::<f64>() / period as f64;
    let avg_loss: f64 = losses.iter().take(period).sum::<f64>() / period as f64;

    let rs = if avg_loss == 0.0 {
        100.0
    } else {
        avg_gain / avg_loss
    };
    rsi_values.push(100.0 - (100.0 / (1.0 + rs)));

    // Usa EMA para valores subsequentes
    let mut avg_gain = avg_gain;
    let mut avg_loss = avg_loss;

    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;

        let rs = if avg_loss == 0.0 {
            100.0
        } else {
            avg_gain / avg_loss
        };
        rsi_values.push(100.0 - (100.0 / (1.0 + rs)));
    }

    rsi_values
}

/// Calcula ATR (Average True Range)
pub fn calculate_atr(candles: &[Candle], period: usize) -> Vec<f64> {
    if candles.len() < period + 1 {
        return vec![];
    }

    let mut true_ranges = Vec::new();

    for i in 1..candles.len() {
        let high = candles[i].high;
        let low = candles[i].low;
        let prev_close = candles[i - 1].close;

        let tr = (high - low)
            .max((high - prev_close).abs())
            .max((low - prev_close).abs());

        true_ranges.push(tr);
    }

    let mut atr_values = Vec::new();

    // Primeiro ATR é média simples
    let first_atr: f64 = true_ranges.iter().take(period).sum::<f64>() / period as f64;
    atr_values.push(first_atr);

    // Usa EMA para valores subsequentes
    for i in period..true_ranges.len() {
        let prev_atr = atr_values.last().unwrap();
        let atr = (prev_atr * (period - 1) as f64 + true_ranges[i]) / period as f64;
        atr_values.push(atr);
    }

    atr_values
}

/// Calcula Bollinger Bands
pub fn calculate_bollinger_bands(
    prices: &[f64],
    period: usize,
    std_dev: f64,
) -> Vec<(f64, f64, f64)> {
    if prices.len() < period {
        return vec![];
    }

    let mut bands = Vec::new();

    for i in period - 1..prices.len() {
        let window = &prices[i - period + 1..=i];
        let sma: f64 = window.iter().sum::<f64>() / period as f64;

        let variance: f64 = window.iter().map(|&x| (x - sma).powi(2)).sum::<f64>() / period as f64;
        let std = variance.sqrt();

        let upper = sma + (std * std_dev);
        let lower = sma - (std * std_dev);

        bands.push((upper, sma, lower));
    }

    bands
}

/// Calcula MACD (Moving Average Convergence Divergence)
pub fn calculate_macd(
    prices: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Vec<(f64, f64, f64)> {
    let ema_fast = calculate_ema(prices, fast_period);
    let ema_slow = calculate_ema(prices, slow_period);

    if ema_fast.is_empty() || ema_slow.is_empty() {
        return vec![];
    }

    // MACD line = EMA_fast - EMA_slow
    let offset = slow_period - fast_period;
    let macd_line: Vec<f64> = ema_fast[offset..]
        .iter()
        .zip(ema_slow.iter())
        .map(|(fast, slow)| fast - slow)
        .collect();

    // Signal line = EMA of MACD line
    let signal_line = calculate_ema(&macd_line, signal_period);

    // Histogram = MACD - Signal
    let offset2 = signal_period - 1;
    let histogram: Vec<f64> = macd_line[offset2..]
        .iter()
        .zip(signal_line.iter())
        .map(|(macd, signal)| macd - signal)
        .collect();

    macd_line[offset2..]
        .iter()
        .zip(signal_line.iter())
        .zip(histogram.iter())
        .map(|((&m, &s), &h)| (m, s, h))
        .collect()
}

/// Calcula volume médio
pub fn calculate_average_volume(candles: &[Candle], period: usize) -> Vec<f64> {
    if candles.len() < period {
        return vec![];
    }

    let mut avg_volumes = Vec::new();

    for i in period - 1..candles.len() {
        let avg: f64 = candles[i - period + 1..=i]
            .iter()
            .map(|c| c.volume)
            .sum::<f64>()
            / period as f64;
        avg_volumes.push(avg);
    }

    avg_volumes
}

/// Detecta padrão de volatilidade
pub fn detect_volatility_regime(atr_values: &[f64], lookback: usize) -> String {
    if atr_values.len() < lookback {
        return "unknown".to_string();
    }

    let recent_atr: f64 = atr_values.iter().rev().take(lookback).sum::<f64>() / lookback as f64;
    let historical_atr: f64 = atr_values.iter().sum::<f64>() / atr_values.len() as f64;

    let ratio = recent_atr / historical_atr;

    if ratio > 1.5 {
        "high".to_string()
    } else if ratio < 0.7 {
        "low".to_string()
    } else {
        "normal".to_string()
    }
}

/// Calcula desvio padrão dos retornos
pub fn calculate_returns_std(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let mut returns = Vec::new();
    for i in 1..prices.len() {
        returns.push((prices[i] - prices[i - 1]) / prices[i - 1]);
    }

    let mut std_values = Vec::new();

    for i in period - 1..returns.len() {
        let window = &returns[i - period + 1..=i];
        let mean: f64 = window.iter().sum::<f64>() / period as f64;
        let variance: f64 = window.iter().map(|&r| (r - mean).powi(2)).sum::<f64>() / period as f64;
        std_values.push(variance.sqrt());
    }

    std_values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_calculation() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
        let ema = calculate_ema(&prices, 5);
        assert!(!ema.is_empty());
        assert!(ema.last().unwrap() > &10.0);
    }

    #[test]
    fn test_rsi_calculation() {
        let prices = vec![
            44.0, 44.25, 44.5, 43.75, 44.0, 44.25, 44.5, 44.75, 45.0, 45.25, 45.5, 45.25, 45.0,
            44.75, 44.5,
        ];
        let rsi = calculate_rsi(&prices, 14);
        assert!(!rsi.is_empty());
        assert!(rsi.last().unwrap() >= &0.0 && rsi.last().unwrap() <= &100.0);
    }

    #[test]
    fn test_atr_calculation() {
        let candles: Vec<Candle> = (0..20)
            .map(|i| Candle {
                timestamp: i,
                open: 100.0 + i as f64,
                high: 102.0 + i as f64,
                low: 99.0 + i as f64,
                close: 101.0 + i as f64,
                volume: 1000.0,
            })
            .collect();

        let atr = calculate_atr(&candles, 14);
        assert!(!atr.is_empty());
        assert!(atr.last().unwrap() > &0.0);
    }
}
