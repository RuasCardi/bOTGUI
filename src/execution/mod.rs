use crate::api::BinanceClient;
use crate::config::ExecutionConfig;
use crate::error::{Result, TradingError};
use crate::types::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

pub struct OrderExecutor {
    client: Arc<BinanceClient>,
    config: ExecutionConfig,
}

impl OrderExecutor {
    pub fn new(client: Arc<BinanceClient>, config: ExecutionConfig) -> Self {
        info!("Initializing Order Executor");
        Self { client, config }
    }

    /// Executa ordem market com retry
    pub async fn execute_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
    ) -> Result<OrderResponse> {
        let order = Order {
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Market,
            quantity,
            price: None,
            stop_price: None,
            client_order_id: Some(self.generate_client_order_id()),
        };

        info!(
            "Executing MARKET order: {:?} {} {}",
            side, quantity, symbol
        );

        self.execute_with_retry(order).await
    }

    /// Executa ordem limit com retry
    pub async fn execute_limit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
    ) -> Result<OrderResponse> {
        let order = Order {
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Limit,
            quantity,
            price: Some(price),
            stop_price: None,
            client_order_id: Some(self.generate_client_order_id()),
        };

        info!(
            "Executing LIMIT order: {:?} {} {} @ {:.2}",
            side, quantity, symbol, price
        );

        self.execute_with_retry(order).await
    }

    /// Executa ordem stop loss
    pub async fn execute_stop_loss(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        stop_price: f64,
    ) -> Result<OrderResponse> {
        let order = Order {
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::StopLoss,
            quantity,
            price: None,
            stop_price: Some(stop_price),
            client_order_id: Some(self.generate_client_order_id()),
        };

        info!(
            "Executing STOP LOSS: {:?} {} {} @ {:.2}",
            side, quantity, symbol, stop_price
        );

        self.execute_with_retry(order).await
    }

    /// Executa ordem com retry logic
    async fn execute_with_retry(&self, order: Order) -> Result<OrderResponse> {
        let mut attempts = 0;
        let max_attempts = self.config.order_retry_attempts;

        loop {
            attempts += 1;

            // Simula latência se configurado
            if self.config.latency_simulation_ms > 0 {
                sleep(Duration::from_millis(self.config.latency_simulation_ms)).await;
            }

            match self.client.create_order(&order).await {
                Ok(response) => {
                    info!(
                        "Order executed successfully on attempt {}: Order ID {}",
                        attempts, response.order_id
                    );
                    return Ok(response);
                }
                Err(e) => {
                    error!("Order execution failed (attempt {}): {}", attempts, e);

                    if attempts >= max_attempts {
                        error!("Max retry attempts reached for order");
                        return Err(e);
                    }

                    // Aguarda antes de tentar novamente (exponential backoff)
                    let wait_time = 2_u64.pow(attempts - 1);
                    warn!("Retrying order in {} seconds...", wait_time);
                    sleep(Duration::from_secs(wait_time)).await;
                }
            }
        }
    }

    /// Cancela ordem com retry
    pub async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<()> {
        let mut attempts = 0;
        let max_attempts = self.config.order_retry_attempts;

        loop {
            attempts += 1;

            match self.client.cancel_order(symbol, order_id).await {
                Ok(_) => {
                    info!("Order {} cancelled successfully", order_id);
                    return Ok(());
                }
                Err(e) => {
                    error!("Order cancellation failed (attempt {}): {}", attempts, e);

                    if attempts >= max_attempts {
                        return Err(e);
                    }

                    let wait_time = 2_u64.pow(attempts - 1);
                    sleep(Duration::from_secs(wait_time)).await;
                }
            }
        }
    }

    /// Aguarda ordem ser executada
    pub async fn wait_for_fill(
        &self,
        symbol: &str,
        order_id: &str,
        timeout_secs: u64,
    ) -> Result<OrderResponse> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            if start.elapsed() > timeout {
                error!("Timeout waiting for order {} to fill", order_id);
                return Err(TradingError::OrderError(
                    "Order fill timeout".to_string(),
                ));
            }

            let order = self.client.get_order(symbol, order_id).await?;

            match order.status {
                OrderStatus::Filled => {
                    info!("Order {} filled successfully", order_id);
                    return Ok(order);
                }
                OrderStatus::PartiallyFilled => {
                    debug!("Order {} partially filled, waiting...", order_id);
                }
                OrderStatus::Canceled | OrderStatus::Rejected | OrderStatus::Expired => {
                    error!("Order {} failed with status: {:?}", order_id, order.status);
                    return Err(TradingError::OrderError(format!(
                        "Order failed: {:?}",
                        order.status
                    )));
                }
                _ => {
                    debug!("Order {} status: {:?}", order_id, order.status);
                }
            }

            sleep(Duration::from_millis(500)).await;
        }
    }

    /// Abre posição com stop loss e take profit
    pub async fn open_position_with_stops(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        stop_loss: f64,
        take_profit: f64,
    ) -> Result<(OrderResponse, Option<OrderResponse>, Option<OrderResponse>)> {
        // 1. Executa ordem principal
        let main_order = self.execute_market_order(symbol, side, quantity).await?;

        // Aguarda confirmação
        sleep(Duration::from_secs(1)).await;

        // 2. Coloca stop loss
        let sl_side = match side {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        };

        let stop_order = match self
            .execute_stop_loss(symbol, sl_side, quantity, stop_loss)
            .await
        {
            Ok(order) => Some(order),
            Err(e) => {
                warn!("Failed to place stop loss: {}", e);
                None
            }
        };

        // 3. Coloca take profit (limit order)
        let tp_order = match self
            .execute_limit_order(symbol, sl_side, quantity, take_profit)
            .await
        {
            Ok(order) => Some(order),
            Err(e) => {
                warn!("Failed to place take profit: {}", e);
                None
            }
        };

        info!(
            "Position opened: {} {:?} {} | SL: {:.2} | TP: {:.2}",
            symbol, side, quantity, stop_loss, take_profit
        );

        Ok((main_order, stop_order, tp_order))
    }

    /// Fecha posição completamente
    pub async fn close_position(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        stop_order_id: Option<&str>,
        tp_order_id: Option<&str>,
    ) -> Result<OrderResponse> {
        // Cancela stop loss e take profit pendentes
        if let Some(order_id) = stop_order_id {
            let _ = self.cancel_order(symbol, order_id).await;
        }

        if let Some(order_id) = tp_order_id {
            let _ = self.cancel_order(symbol, order_id).await;
        }

        // Fecha posição com ordem market
        let close_side = match side {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        };

        info!("Closing position: {} {:?} {}", symbol, side, quantity);

        self.execute_market_order(symbol, close_side, quantity)
            .await
    }

    /// Ajusta stop loss de uma ordem existente
    pub async fn modify_stop_loss(
        &self,
        symbol: &str,
        old_order_id: &str,
        side: OrderSide,
        quantity: f64,
        new_stop_price: f64,
    ) -> Result<OrderResponse> {
        // Cancela stop loss antigo
        self.cancel_order(symbol, old_order_id).await?;

        // Coloca novo stop loss
        sleep(Duration::from_millis(200)).await;

        info!(
            "Modifying stop loss for {}: new price {:.2}",
            symbol, new_stop_price
        );

        self.execute_stop_loss(symbol, side, quantity, new_stop_price)
            .await
    }

    /// Gera ID único para ordem
    fn generate_client_order_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("robo_{}", timestamp)
    }

    /// Valida quantidade antes de executar
    pub fn validate_quantity(&self, quantity: f64, min_qty: f64, step_size: f64) -> f64 {
        // Arredonda para o step size mais próximo
        let qty = (quantity / step_size).floor() * step_size;

        // Garante mínimo
        if qty < min_qty {
            warn!(
                "Quantity {:.8} below minimum {:.8}, using minimum",
                qty, min_qty
            );
            return min_qty;
        }

        qty
    }

    /// Obtém preço atual do mercado
    pub async fn get_current_price(&self, symbol: &str) -> Result<f64> {
        self.client.get_price(symbol).await
    }

    /// Verifica saldo disponível
    pub async fn check_available_balance(&self, asset: &str) -> Result<f64> {
        let account = self.client.get_account().await?;

        for balance in account.balances {
            if balance.asset == asset {
                return Ok(balance.free);
            }
        }

        Err(TradingError::ApiError(format!(
            "Asset {} not found in account",
            asset
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_order_id_generation() {
        let config = ExecutionConfig {
            order_retry_attempts: 3,
            order_timeout_seconds: 30,
            latency_simulation_ms: 0,
        };

        let client = Arc::new(BinanceClient::new(
            "test".to_string(),
            "test".to_string(),
            true,
        ));

        let executor = OrderExecutor::new(client, config);
        let id1 = executor.generate_client_order_id();
        std::thread::sleep(std::time::Duration::from_millis(10)); // Garante IDs diferentes
        let id2 = executor.generate_client_order_id();

        assert_ne!(id1, id2);
        assert!(id1.starts_with("robo_"));
    }

    #[test]
    fn test_quantity_validation() {
        let config = ExecutionConfig {
            order_retry_attempts: 3,
            order_timeout_seconds: 30,
            latency_simulation_ms: 0,
        };

        let client = Arc::new(BinanceClient::new(
            "test".to_string(),
            "test".to_string(),
            true,
        ));

        let executor = OrderExecutor::new(client, config);

        let qty = executor.validate_quantity(0.12345, 0.001, 0.001);
        assert_eq!(qty, 0.123);

        let qty2 = executor.validate_quantity(0.0005, 0.001, 0.001);
        assert_eq!(qty2, 0.001);
    }
}
