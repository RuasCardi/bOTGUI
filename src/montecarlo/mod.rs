use crate::backtest::{BacktestMetrics, Trade};
use rand::prelude::*;
use rand_distr::Normal;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloResult {
    pub simulations: usize,
    pub confidence_level: f64,
    pub expected_return: f64,
    pub best_case: f64,
    pub worst_case: f64,
    pub percentile_5: f64,
    pub percentile_25: f64,
    pub percentile_50: f64,
    pub percentile_75: f64,
    pub percentile_95: f64,
    pub probability_of_profit: f64,
    pub expected_max_drawdown: f64,
    pub var_95: f64, // Value at Risk
}

pub struct MonteCarloSimulator {
    simulations: usize,
}

impl MonteCarloSimulator {
    pub fn new(simulations: usize) -> Self {
        info!("Initializing Monte Carlo Simulator with {} simulations", simulations);
        Self { simulations }
    }

    /// Executa simulação de Monte Carlo baseada em trades históricos
    pub fn simulate(&self, trades: &[Trade], initial_capital: f64, periods: usize) -> MonteCarloResult {
        info!("Running Monte Carlo simulation for {} periods", periods);

        if trades.is_empty() {
            panic!("Cannot run Monte Carlo with no historical trades");
        }

        // Calcula estatísticas dos trades
        let returns: Vec<f64> = trades.iter().map(|t| t.pnl_percent / 100.0).collect();
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let std_return = self.calculate_std(&returns, mean_return);

        // Distribuição normal dos retornos
        let normal = Normal::new(mean_return, std_return).unwrap();
        let mut rng = thread_rng();

        // Executa simulações
        let mut final_capitals = Vec::new();
        let mut max_drawdowns = Vec::new();

        for _ in 0..self.simulations {
            let mut capital = initial_capital;
            let mut peak = initial_capital;
            let mut max_dd = 0.0;

            for _ in 0..periods {
                // Gera retorno aleatório
                let return_sample: f64 = normal.sample(&mut rng);
                capital *= 1.0 + return_sample;

                // Atualiza drawdown
                if capital > peak {
                    peak = capital;
                }
                let dd = (peak - capital) / peak;
                if dd > max_dd {
                    max_dd = dd;
                }
            }

            final_capitals.push(capital);
            max_drawdowns.push(max_dd);
        }

        // Ordena resultados
        final_capitals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        max_drawdowns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Calcula percentis
        let percentile = |values: &[f64], p: f64| -> f64 {
            let idx = ((values.len() as f64) * p) as usize;
            values[idx.min(values.len() - 1)]
        };

        let expected_return = final_capitals.iter().sum::<f64>() / final_capitals.len() as f64;
        let best_case = *final_capitals.last().unwrap();
        let worst_case = *final_capitals.first().unwrap();
        let p5 = percentile(&final_capitals, 0.05);
        let p25 = percentile(&final_capitals, 0.25);
        let p50 = percentile(&final_capitals, 0.50);
        let p75 = percentile(&final_capitals, 0.75);
        let p95 = percentile(&final_capitals, 0.95);

        let probability_of_profit = final_capitals.iter().filter(|&&c| c > initial_capital).count() as f64
            / final_capitals.len() as f64;

        let expected_max_dd = max_drawdowns.iter().sum::<f64>() / max_drawdowns.len() as f64;
        let var_95 = initial_capital - p5; // Value at Risk

        info!("Monte Carlo completed - Expected return: ${:.2}", expected_return);

        MonteCarloResult {
            simulations: self.simulations,
            confidence_level: 0.95,
            expected_return,
            best_case,
            worst_case,
            percentile_5: p5,
            percentile_25: p25,
            percentile_50: p50,
            percentile_75: p75,
            percentile_95: p95,
            probability_of_profit,
            expected_max_drawdown: expected_max_dd,
            var_95,
        }
    }

    /// Simula usando bootstrap (reamostragem com reposição)
    pub fn bootstrap_simulate(
        &self,
        trades: &[Trade],
        initial_capital: f64,
        periods: usize,
    ) -> MonteCarloResult {
        info!("Running Bootstrap Monte Carlo simulation");

        let mut rng = thread_rng();
        let mut final_capitals = Vec::new();
        let mut max_drawdowns = Vec::new();

        for _ in 0..self.simulations {
            let mut capital = initial_capital;
            let mut peak = initial_capital;
            let mut max_dd = 0.0;

            for _ in 0..periods {
                // Seleciona trade aleatório com reposição
                let trade = trades.choose(&mut rng).unwrap();
                let return_pct = trade.pnl_percent / 100.0;
                capital *= 1.0 + return_pct;

                // Atualiza drawdown
                if capital > peak {
                    peak = capital;
                }
                let dd = (peak - capital) / peak;
                if dd > max_dd {
                    max_dd = dd;
                }
            }

            final_capitals.push(capital);
            max_drawdowns.push(max_dd);
        }

        // Processa resultados (igual ao método anterior)
        final_capitals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        max_drawdowns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let percentile = |values: &[f64], p: f64| -> f64 {
            let idx = ((values.len() as f64) * p) as usize;
            values[idx.min(values.len() - 1)]
        };

        let expected_return = final_capitals.iter().sum::<f64>() / final_capitals.len() as f64;
        let probability_of_profit = final_capitals.iter().filter(|&&c| c > initial_capital).count() as f64
            / final_capitals.len() as f64;
        let expected_max_dd = max_drawdowns.iter().sum::<f64>() / max_drawdowns.len() as f64;

        MonteCarloResult {
            simulations: self.simulations,
            confidence_level: 0.95,
            expected_return,
            best_case: *final_capitals.last().unwrap(),
            worst_case: *final_capitals.first().unwrap(),
            percentile_5: percentile(&final_capitals, 0.05),
            percentile_25: percentile(&final_capitals, 0.25),
            percentile_50: percentile(&final_capitals, 0.50),
            percentile_75: percentile(&final_capitals, 0.75),
            percentile_95: percentile(&final_capitals, 0.95),
            probability_of_profit,
            expected_max_drawdown: expected_max_dd,
            var_95: initial_capital - percentile(&final_capitals, 0.05),
        }
    }

    fn calculate_std(&self, values: &[f64], mean: f64) -> f64 {
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }
}

impl MonteCarloResult {
    pub fn print_summary(&self, initial_capital: f64) {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║           MONTE CARLO SIMULATION RESULTS                 ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Simulações:          {:>6}                             ║", self.simulations);
        println!("║ Intervalo Confiança: {:>5.0}%                             ║", self.confidence_level * 100.0);
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Capital Inicial:     ${:>12.2}                     ║", initial_capital);
        println!("║ Retorno Esperado:    ${:>12.2} ({:>5.1}%)           ║", 
            self.expected_return, ((self.expected_return / initial_capital) - 1.0) * 100.0);
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Melhor Caso:         ${:>12.2}                     ║", self.best_case);
        println!("║ Pior Caso:           ${:>12.2}                     ║", self.worst_case);
        println!("║ Percentil 95%:       ${:>12.2}                     ║", self.percentile_95);
        println!("║ Percentil 75%:       ${:>12.2}                     ║", self.percentile_75);
        println!("║ Mediana (50%):       ${:>12.2}                     ║", self.percentile_50);
        println!("║ Percentil 25%:       ${:>12.2}                     ║", self.percentile_25);
        println!("║ Percentil 5%:        ${:>12.2}                     ║", self.percentile_5);
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Prob. de Lucro:      {:>12.1}%                     ║", self.probability_of_profit * 100.0);
        println!("║ Max DD Esperado:     {:>12.2}%                     ║", self.expected_max_drawdown * 100.0);
        println!("║ VaR 95%:             ${:>12.2}                     ║", self.var_95);
        println!("╚══════════════════════════════════════════════════════════╝\n");
    }
}
