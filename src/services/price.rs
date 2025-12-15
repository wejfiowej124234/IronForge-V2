// Fiat Currency Price Service
// Uses backend proxy API to avoid CORS and rate limiting issues

use crate::shared::cache::CacheEntry;
use crate::shared::error::{ApiError, AppError};
use crate::shared::state::AppState;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;

const PRICE_CACHE_TTL_SECS: u64 = 300; // 5 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinPrice {
    pub symbol: String,
    pub usd: f64,
    pub usd_24h_change: f64,
    pub last_updated: u64,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoSimplePrice {
    usd: Option<f64>,
    usd_24h_change: Option<f64>,
}

#[derive(Clone, Copy)]
pub struct PriceService {
    app_state: AppState,
}

// Note: AppState is Copy, so cache operations work through Signal::write()

impl PriceService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Get price for a single cryptocurrency
    ///
    /// # Arguments
    /// * `symbol` - Coin symbol (e.g., "BTC", "ETH", "SOL")
    ///
    /// # Returns
    /// Current USD price and 24h change percentage
    pub async fn get_price(&self, symbol: &str) -> Result<CoinPrice, AppError> {
        let _coin_id = self.symbol_to_coingecko_id(symbol);
        let prices = self.get_prices(&[symbol]).await?;

        prices
            .get(symbol)
            .cloned()
            .ok_or_else(|| AppError::Api(ApiError::ResponseError("Price not found".into())))
    }

    /// Get prices for multiple cryptocurrencies in batch
    ///
    /// # Arguments
    /// * `symbols` - Array of coin symbols
    ///
    /// # Returns
    /// HashMap of symbol -> CoinPrice
    pub async fn get_prices(
        mut self,
        symbols: &[&str],
    ) -> Result<HashMap<String, CoinPrice>, AppError> {
        let coin_ids: Vec<String> = symbols
            .iter()
            .map(|s| self.symbol_to_coingecko_id(s))
            .collect();

        let ids_param = coin_ids.join(",");
        let cache_key = format!("price:batch:{}", ids_param);

        // Check cache first
        let cache = self.app_state.cache.read();
        if let Some(entry) = cache.get(&cache_key) {
            if !entry.is_expired(PRICE_CACHE_TTL_SECS) {
                if let Ok(prices) =
                    serde_json::from_value::<HashMap<String, CoinPrice>>(entry.value.clone())
                {
                    return Ok(prices);
                }
            }
        }
        drop(cache);

        // Fetch from backend API proxy (avoids CORS + rate limits)
        let api_client = self.app_state.get_api_client();
        let backend_url = format!("{}/api/v1/prices?symbols={}", api_client.base_url(), symbols.join(","));

        tracing::info!("Fetching prices from backend: {}", backend_url);

        // Use gloo-net for WASM-compatible HTTP requests
        use gloo_net::http::Request;
        let response_text = Request::get(&backend_url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Backend price fetch failed: {}", e);
                AppError::Api(ApiError::RequestFailed(format!("Backend unavailable: {}", e)))
            })?
            .text()
            .await
            .map_err(|e| AppError::Api(ApiError::ResponseError(e.to_string())))?;

        // Backend returns ApiResponse<PricesResponse> format
        #[derive(Deserialize)]
        struct BackendPriceData {
            symbol: String,
            price_usdt: f64,
            source: String,
        }

        #[derive(Deserialize)]
        struct BackendPricesResponse {
            prices: Vec<BackendPriceData>,
            last_updated: String,
        }

        #[derive(Deserialize)]
        struct BackendApiResponse {
            data: BackendPricesResponse,
        }

        let backend_resp: BackendApiResponse = serde_json::from_str(&response_text).map_err(|e| {
            tracing::error!("Failed to parse backend response: {} - Response: {}", e, response_text);
            AppError::Api(ApiError::ResponseError(format!("Invalid backend response: {}", e)))
        })?;

        let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
        let mut prices = HashMap::new();

        // Convert backend response to our CoinPrice format
        for price_data in backend_resp.data.prices {
            let symbol = price_data.symbol.to_uppercase();
            prices.insert(
                symbol.clone(),
                CoinPrice {
                    symbol,
                    usd: price_data.price_usdt,
                    usd_24h_change: 0.0, // Backend doesn't provide 24h change yet
                    last_updated: now,
                },
            );
        }

        // Update cache
        self.app_state.cache.write().insert(
            cache_key,
            CacheEntry {
                value: serde_json::to_value(&prices).unwrap(),
                stored_at: now,
            },
        );

        Ok(prices)
    }

    /// Get asset value in USD
    ///
    /// # Arguments
    /// * `symbol` - Coin symbol
    /// * `amount` - Token amount
    ///
    /// # Returns
    /// USD value
    #[allow(dead_code)] // 用于价格计算功能
    pub async fn get_usd_value(&self, symbol: &str, amount: f64) -> Result<f64, AppError> {
        let price = self.get_price(symbol).await?;
        Ok(price.usd * amount)
    }

    /// Map common symbol to CoinGecko ID
    fn symbol_to_coingecko_id(&self, symbol: &str) -> String {
        let id = match symbol.to_uppercase().as_str() {
            "BTC" => "bitcoin",
            "ETH" => "ethereum",
            "SOL" => "solana",
            "BNB" => "binancecoin",
            "MATIC" => "matic-network",
            "AVAX" => "avalanche-2",
            "DOT" => "polkadot",
            "USDC" => "usd-coin",
            "USDT" => "tether",
            "DAI" => "dai",
            _ => &symbol.to_lowercase(),
        };
        id.to_string()
    }
}

/// Hook for using price service in components
/// 获取价格服务实例
///
/// 注意：此函数当前未使用，但保留用于未来扩展
#[allow(dead_code)]
pub fn use_price_service() -> PriceService {
    let app_state = use_context::<AppState>();
    PriceService::new(app_state)
}

/// Hook for live price updates (polls every 30s)
/// 获取实时价格（响应式）
///
/// 注意：此函数当前未使用，但保留用于未来扩展
#[allow(dead_code)]
pub fn use_live_price(symbol: &str) -> Signal<Option<CoinPrice>> {
    let price = use_signal(|| None);
    let service = use_price_service();
    let symbol_owned = symbol.to_string();

    use_effect(move || {
        let mut price = price;
        let symbol_clone = symbol_owned.clone();
        spawn_local(async move {
            loop {
                match service.get_price(&symbol_clone).await {
                    Ok(p) => price.set(Some(p)),
                    Err(e) => {
                        tracing::error!("Failed to fetch price for {}: {:?}", symbol_clone, e)
                    }
                }
                gloo_timers::future::TimeoutFuture::new(30_000).await; // 30s
            }
        });
    });

    price
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_to_coingecko_id() {
        let app_state = AppState::new();
        let service = PriceService::new(app_state);

        assert_eq!(service.symbol_to_coingecko_id("BTC"), "bitcoin");
        assert_eq!(service.symbol_to_coingecko_id("eth"), "ethereum");
        assert_eq!(service.symbol_to_coingecko_id("SOL"), "solana");
    }

    #[test]
    fn test_usd_value_calculation() {
        let price = CoinPrice {
            symbol: "ETH".to_string(),
            usd: 2000.0,
            usd_24h_change: 5.0,
            last_updated: 0,
        };

        let value = price.usd * 1.5;
        assert_eq!(value, 3000.0);
    }
}
