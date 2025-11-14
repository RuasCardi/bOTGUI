use crate::backtest::{BacktestMetrics, Trade};
use crate::types::OrderSide;
use std::io::{self, Write};

pub struct CLI;

impl CLI {
    pub fn print_header() {
        println!("\n{}", "=".repeat(70));
        println!("{}🤖 RoboTrading - Sistema de Trading Automatizado{}", 
            " ".repeat(10), " ".repeat(10));
        println!("{}", "=".repeat(70));
    }

    pub fn print_backtest_summary(metrics: &BacktestMetrics, initial_capital: f64) {
        println!("\n📊 RESUMO DO BACKTEST\n");
        println!("{}", "-".repeat(70));
        
        // Capital
        let final_capital = initial_capital * (1.0 + metrics.total_return_percent / 100.0);
        let profit = metrics.total_pnl;
        let profit_pct = metrics.total_return_percent;
        let profit_color = if profit >= 0.0 { "✅" } else { "❌" };
        
        println!("💰 Capital Inicial:      ${:.2}", initial_capital);
        println!("💎 Capital Final:        ${:.2}", final_capital);
        println!("{} Lucro/Prejuízo:      ${:.2} ({:+.2}%)\n", 
            profit_color, profit, profit_pct);
        
        // Trades
        let total_trades = metrics.total_trades;
        let winning_trades = metrics.winning_trades;
        let losing_trades = metrics.losing_trades;
        
        println!("📈 Total de Trades:      {}", total_trades);
        println!("✅ Trades Vencedores:    {} ({:.1}%)", 
            winning_trades, metrics.win_rate * 100.0);
        println!("❌ Trades Perdedores:    {} ({:.1}%)\n", 
            losing_trades, (1.0 - metrics.win_rate) * 100.0);
        
        // Performance
        println!("🎯 Métricas de Performance:");
        println!("   • Profit Factor:      {:.2}", metrics.profit_factor);
        println!("   • Expectativa:        ${:.2}", metrics.expectancy);
        println!("   • Ganho Médio:        ${:.2}", metrics.avg_win);
        println!("   • Perda Média:        ${:.2}", metrics.avg_loss.abs());
        println!();
        
        // Risco
        println!("⚠️  Métricas de Risco:");
        println!("   • Max Drawdown:       ${:.2} ({:.2}%)", 
            metrics.max_drawdown, metrics.max_drawdown_percent);
        println!("   • Sharpe Ratio:       {:.2}", metrics.sharpe_ratio);
        println!("   • Sortino Ratio:      {:.2}", metrics.sortino_ratio);
        println!("   • Calmar Ratio:       {:.2}", metrics.calmar_ratio);
        println!();
        
        // Tempo
        println!("⏱️  Período:");
        println!("   • Tempo Médio:        {:.1}h", metrics.avg_holding_period_hours);
        println!("   • Wins Consecutivos:  {}", metrics.max_consecutive_wins);
        println!("   • Losses Consecutivos: {}", metrics.max_consecutive_losses);
        
        println!("{}", "-".repeat(70));
    }

    pub fn print_trade_list(trades: &[Trade]) {
        if trades.is_empty() {
            println!("\n⚠️  Nenhum trade executado ainda.\n");
            return;
        }

        println!("\n📋 HISTÓRICO DE TRADES\n");
        println!("{}", "-".repeat(110));
        println!("{:<5} {:<20} {:<8} {:<12} {:<12} {:<15} {:<10}",
            "Nº", "Data/Hora", "Lado", "Entrada", "Saída", "P&L", "Status");
        println!("{}", "-".repeat(110));

        for (i, trade) in trades.iter().enumerate() {
            let side_emoji = match trade.side {
                OrderSide::Buy => "🟢 BUY ",
                OrderSide::Sell => "🔴 SELL",
            };
            
            // Converter timestamp unix para datetime legível (simples)
            let time_str = format!("Trade #{}", i + 1);
            
            let pnl_emoji = if trade.pnl >= 0.0 { "✅" } else { "❌" };
            let pnl_str = format!("${:+.2} ({:+.1}%)", trade.pnl, trade.pnl_percent);
            
            println!("{:<5} {:<20} {:<8} {:<12.2} {:<12.2} {:<15} {:<10}",
                i + 1,
                time_str,
                side_emoji,
                trade.entry_price,
                trade.exit_price,
                pnl_str,
                pnl_emoji
            );
        }
        
        println!("{}", "-".repeat(110));
        
        // Resumo rápido
        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let winning_trades = trades.iter().filter(|t| t.pnl >= 0.0).count();
        let avg_pnl = total_pnl / trades.len() as f64;
        
        println!("\n💼 Resumo: {} trades | Vencedores: {} | P&L Total: ${:+.2} | P&L Médio: ${:+.2}",
            trades.len(), winning_trades, total_pnl, avg_pnl);
        println!();
    }

    pub fn print_monte_carlo_summary(
        simulations: usize,
        expected_return: f64,
        initial_capital: f64,
        best_case: f64,
        worst_case: f64,
        profit_probability: f64,
        max_drawdown: f64,
    ) {
        println!("\n🎲 SIMULAÇÃO MONTE CARLO\n");
        println!("{}", "-".repeat(70));
        
        println!("📊 Configuração:");
        println!("   • Simulações:         {:>10}", simulations);
        println!("   • Capital Inicial:    ${:>10.2}\n", initial_capital);
        
        let expected_profit = expected_return - initial_capital;
        let expected_pct = (expected_profit / initial_capital) * 100.0;
        
        println!("💰 Projeções:");
        println!("   • Retorno Esperado:   ${:>10.2} ({:+.1}%)", 
            expected_return, expected_pct);
        println!("   • Melhor Cenário:     ${:>10.2} ({:+.1}%)", 
            best_case, ((best_case - initial_capital) / initial_capital) * 100.0);
        println!("   • Pior Cenário:       ${:>10.2} ({:+.1}%)\n", 
            worst_case, ((worst_case - initial_capital) / initial_capital) * 100.0);
        
        let prob_emoji = if profit_probability >= 0.7 { "🟢" } else if profit_probability >= 0.5 { "🟡" } else { "🔴" };
        println!("🎯 Probabilidades:");
        println!("   {} Chance de Lucro:    {:>10.1}%", prob_emoji, profit_probability * 100.0);
        println!("   ⚠️  Max Drawdown:      {:>10.2}%\n", max_drawdown * 100.0);
        
        println!("{}", "-".repeat(70));
        
        // Recomendação
        if profit_probability >= 0.7 && max_drawdown < 0.2 {
            println!("\n✅ AVALIAÇÃO: Estratégia promissora! Alta probabilidade de lucro com risco controlado.\n");
        } else if profit_probability >= 0.5 {
            println!("\n⚠️  AVALIAÇÃO: Estratégia moderada. Considere ajustar parâmetros de risco.\n");
        } else {
            println!("\n❌ AVALIAÇÃO: Estratégia arriscada. Revisar estratégia antes de operar ao vivo.\n");
        }
    }

    pub fn print_live_status(
        capital: f64,
        daily_pnl: f64,
        win_rate: f64,
        open_positions: usize,
        total_trades: usize,
    ) {
        // Limpar tela (funciona no Linux)
        print!("\x1B[2J\x1B[1;1H");
        
        Self::print_header();
        
        println!("\n🔴 MODO AO VIVO - Operando...\n");
        println!("{}", "-".repeat(70));
        
        let pnl_emoji = if daily_pnl >= 0.0 { "📈" } else { "📉" };
        let pnl_pct = (daily_pnl / (capital - daily_pnl)) * 100.0;
        
        println!("💰 Capital Atual:        ${:.2}", capital);
        println!("{} P&L Hoje:             ${:+.2} ({:+.2}%)\n", pnl_emoji, daily_pnl, pnl_pct);
        
        println!("📊 Performance:");
        println!("   • Posições Abertas:   {}", open_positions);
        println!("   • Total de Trades:    {}", total_trades);
        println!("   • Win Rate:           {:.1}%\n", win_rate * 100.0);
        
        println!("{}", "-".repeat(70));
        println!("\n💡 Dica: Pressione Ctrl+C para parar o bot de forma segura.\n");
    }

    pub fn print_menu() {
        Self::print_header();
        println!("\n📋 MENU DE OPÇÕES\n");
        println!("  1. 📊 Executar Backtest (teste histórico)");
        println!("  2. 🎲 Simulação Monte Carlo (projeções futuras)");
        println!("  3. 🔴 Operar ao Vivo (testnet)");
        println!("  4. 💼 Operar ao Vivo (REAL - cuidado!)");
        println!("  5. 📄 Ver último backtest");
        println!("  6. 📈 Ver última simulação");
        println!("  0. ❌ Sair\n");
        print!("➤ Escolha uma opção: ");
        io::stdout().flush().unwrap();
    }

    pub fn read_input() -> String {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }

    pub fn print_separator() {
        println!("\n{}", "=".repeat(70));
    }

    pub fn print_warning(message: &str) {
        println!("\n⚠️  ATENÇÃO: {}\n", message);
    }

    pub fn print_success(message: &str) {
        println!("\n✅ {}\n", message);
    }

    pub fn print_error(message: &str) {
        println!("\n❌ ERRO: {}\n", message);
    }

    pub fn confirm_action(message: &str) -> bool {
        print!("\n⚠️  {} (s/n): ", message);
        io::stdout().flush().unwrap();
        let input = Self::read_input().to_lowercase();
        input == "s" || input == "sim" || input == "y" || input == "yes"
    }
}
