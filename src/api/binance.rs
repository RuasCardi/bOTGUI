use crate::error::{Result, TradingError};
use crate::types::*;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

type HmacSha256 = Hmac<Sha256>;

const BINANCE_API_URL: &str = "https://api.binance.com";
const BINANCE_TESTNET_URL: &str = "https://testnet.binance.vision";

pub struct BinanceClient {
    client: Client,
    api_key: String,
    secret_key: String,
    base_url: String,
}

impl BinanceClient {
    pub fn new(api_key: String, secret_key: String, use_testnet: bool) -> Self {
        let base_url = if use_testnet {
            BINANCE_TESTNET_URL.to_string()
        } else {
            BINANCE_API_URL.to_string()
        };

        info!("Initializing Binance client (testnet: {})", use_testnet);

        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
            secret_key,
            base_url,
        }
    }

    /// Gera timestamp atual em milissegundos
    fn get_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Cria assinatura HMAC SHA256
    fn sign(&self, query_string: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    /// Monta query string com timestamp e assinatura
    fn build_signed_query(&self, mut params: HashMap<String, String>) -> String {
        params.insert("timestamp".to_string(), self.get_timestamp().to_string());
        
        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        
        let signature = self.sign(&query_string);
        format!("{}&signature={}", query_string, signature)
    }

    /// GET request público
    async fn get_public(&self, endpoint: &str, params: HashMap<String, String>) -> Result<Value> {
        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        
        let url = format!("{}/api/v3/{}?{}", self.base_url, endpoint, query_string);
        
        debug!("GET request: {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        self.handle_response(response).await
    }

    /// GET request autenticado
    async fn get_signed(&self, endpoint: &str, params: HashMap<String, String>) -> Result<Value> {
        let query_string = self.build_signed_query(params);
        let url = format!("{}/api/v3/{}?{}", self.base_url, endpoint, query_string);
        
        debug!("GET signed request: {}", endpoint);
        
        let response = self.client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;
        
        self.handle_response(response).await
    }

    /// POST request autenticado
    async fn post_signed(&self, endpoint: &str, params: HashMap<String, String>) -> Result<Value> {
        let query_string = self.build_signed_query(params);
        let url = format!("{}/api/v3/{}?{}", self.base_url, endpoint, query_string);
        
        debug!("POST signed request: {}", endpoint);
        
        let response = self.client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;
        
        self.handle_response(response).await
    }

    /// DELETE request autenticado
    async fn delete_signed(&self, endpoint: &str, params: HashMap<String, String>) -> Result<Value> {
        let query_string = self.build_signed_query(params);
        let url = format!("{}/api/v3/{}?{}", self.base_url, endpoint, query_string);
        
        debug!("DELETE signed request: {}", endpoint);
        
        let response = self.client
            .delete(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;
        
        self.handle_response(response).await
    }

    /// Processa resposta da API
    async fn handle_response(&self, response: reqwest::Response) -> Result<Value> {
        let status = response.status();
        let body = response.text().await?;
        
        if !status.is_success() {
            error!("API error - Status: {}, Body: {}", status, body);
            return Err(TradingError::ApiError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }
        
        let json: Value = serde_json::from_str(&body)?;
        
        // Verifica se há erro na resposta JSON
        if let Some(code) = json.get("code") {
            if code.as_i64().unwrap_or(0) != 0 {
                let msg = json.get("msg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                error!("Binance API error: {} (code: {})", msg, code);
                return Err(TradingError::ApiError(format!("{} (code: {})", msg, code)));
            }
        }
        
        Ok(json)
    }

    // ==================== PUBLIC API ====================

    /// Obtém preço atual de um símbolo
    pub async fn get_price(&self, symbol: &str) -> Result<f64> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        
        let response = self.get_public("ticker/price", params).await?;
        
        let price = response["price"]
            .as_str()
            .ok_or_else(|| TradingError::ApiError("Invalid price format".to_string()))?
            .parse::<f64>()
            .map_err(|e| TradingError::ApiError(format!("Failed to parse price: {}", e)))?;
        
        debug!("Price for {}: {}", symbol, price);
        Ok(price)
    }

    /// Obtém dados de mercado 24h
    pub async fn get_24h_ticker(&self, symbol: &str) -> Result<MarketData> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        
        let response = self.get_public("ticker/24hr", params).await?;
        
        Ok(MarketData {
            symbol: symbol.to_string(),
            price: response["lastPrice"].as_str().unwrap().parse().unwrap(),
            bid: response["bidPrice"].as_str().unwrap().parse().unwrap(),
            ask: response["askPrice"].as_str().unwrap().parse().unwrap(),
            volume_24h: response["volume"].as_str().unwrap().parse().unwrap(),
            timestamp: self.get_timestamp() as i64,
        })
    }

    /// Obtém histórico de candles
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: Timeframe,
        limit: u32,
    ) -> Result<Vec<Candle>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("interval".to_string(), interval.to_string());
        params.insert("limit".to_string(), limit.to_string());
        
        let response = self.get_public("klines", params).await?;
        
        let candles: Result<Vec<Candle>> = response
            .as_array()
            .ok_or_else(|| TradingError::ApiError("Invalid klines format".to_string()))?
            .iter()
            .map(|kline| {
                let arr = kline.as_array()
                    .ok_or_else(|| TradingError::ApiError("Invalid kline format".to_string()))?;
                
                Ok(Candle {
                    timestamp: arr[0].as_i64().unwrap(),
                    open: arr[1].as_str().unwrap().parse().unwrap(),
                    high: arr[2].as_str().unwrap().parse().unwrap(),
                    low: arr[3].as_str().unwrap().parse().unwrap(),
                    close: arr[4].as_str().unwrap().parse().unwrap(),
                    volume: arr[5].as_str().unwrap().parse().unwrap(),
                })
            })
            .collect();
        
        let candles = candles?;
        debug!("Fetched {} candles for {}", candles.len(), symbol);
        Ok(candles)
    }

    // ==================== AUTHENTICATED API ====================

    /// Obtém informações da conta
    pub async fn get_account(&self) -> Result<AccountInfo> {
        let params = HashMap::new();
        let response = self.get_signed("account", params).await?;
        
        let balances: Vec<AccountBalance> = response["balances"]
            .as_array()
            .ok_or_else(|| TradingError::ApiError("Invalid balances format".to_string()))?
            .iter()
            .map(|b| AccountBalance {
                asset: b["asset"].as_str().unwrap().to_string(),
                free: b["free"].as_str().unwrap().parse().unwrap(),
                locked: b["locked"].as_str().unwrap().parse().unwrap(),
            })
            .collect();
        
        Ok(AccountInfo {
            balances,
            can_trade: response["canTrade"].as_bool().unwrap_or(false),
            can_withdraw: response["canWithdraw"].as_bool().unwrap_or(false),
            can_deposit: response["canDeposit"].as_bool().unwrap_or(false),
            update_time: response["updateTime"].as_i64().unwrap_or(0),
        })
    }

    /// Cria uma ordem
    pub async fn create_order(&self, order: &Order) -> Result<OrderResponse> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), order.symbol.clone());
        params.insert("side".to_string(), format!("{:?}", order.side).to_uppercase());
        params.insert("type".to_string(), format!("{:?}", order.order_type).to_uppercase());
        params.insert("quantity".to_string(), format!("{:.8}", order.quantity));
        
        if let Some(price) = order.price {
            params.insert("price".to_string(), format!("{:.8}", price));
            params.insert("timeInForce".to_string(), "GTC".to_string());
        }
        
        if let Some(stop_price) = order.stop_price {
            params.insert("stopPrice".to_string(), format!("{:.8}", stop_price));
        }
        
        if let Some(client_order_id) = &order.client_order_id {
            params.insert("newClientOrderId".to_string(), client_order_id.clone());
        }
        
        info!("Creating order: {:?} {} {} @ {:?}", 
            order.side, order.quantity, order.symbol, order.price);
        
        let response = self.post_signed("order", params).await?;
        
        Ok(OrderResponse {
            order_id: response["orderId"].to_string(),
            client_order_id: response["clientOrderId"].as_str().unwrap().to_string(),
            symbol: response["symbol"].as_str().unwrap().to_string(),
            status: serde_json::from_value(response["status"].clone())?,
            side: serde_json::from_value(response["side"].clone())?,
            order_type: serde_json::from_value(response["type"].clone())?,
            price: response["price"].as_str().unwrap_or("0").parse().unwrap(),
            quantity: response["origQty"].as_str().unwrap().parse().unwrap(),
            executed_qty: response["executedQty"].as_str().unwrap().parse().unwrap(),
            timestamp: response["transactTime"].as_i64().unwrap(),
        })
    }

    /// Cancela uma ordem
    pub async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<()> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("orderId".to_string(), order_id.to_string());
        
        info!("Canceling order: {} on {}", order_id, symbol);
        
        self.delete_signed("order", params).await?;
        Ok(())
    }

    /// Obtém status de uma ordem
    pub async fn get_order(&self, symbol: &str, order_id: &str) -> Result<OrderResponse> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("orderId".to_string(), order_id.to_string());
        
        let response = self.get_signed("order", params).await?;
        
        Ok(OrderResponse {
            order_id: response["orderId"].to_string(),
            client_order_id: response["clientOrderId"].as_str().unwrap().to_string(),
            symbol: response["symbol"].as_str().unwrap().to_string(),
            status: serde_json::from_value(response["status"].clone())?,
            side: serde_json::from_value(response["side"].clone())?,
            order_type: serde_json::from_value(response["type"].clone())?,
            price: response["price"].as_str().unwrap_or("0").parse().unwrap(),
            quantity: response["origQty"].as_str().unwrap().parse().unwrap(),
            executed_qty: response["executedQty"].as_str().unwrap().parse().unwrap(),
            timestamp: response["time"].as_i64().unwrap(),
        })
    }

    /// Testa conectividade com a API
    pub async fn ping(&self) -> Result<()> {
        let params = HashMap::new();
        self.get_public("ping", params).await?;
        info!("Ping successful");
        Ok(())
    }

    /// Obtém horário do servidor
    pub async fn get_server_time(&self) -> Result<i64> {
        let params = HashMap::new();
        let response = self.get_public("time", params).await?;
        Ok(response["serverTime"].as_i64().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() {
        let client = BinanceClient::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            true,
        );
        
        // Este teste só funciona se houver conexão com internet
        // Em produção, use mocks para testes unitários
        // assert!(client.ping().await.is_ok());
    }
}
