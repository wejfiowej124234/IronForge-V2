use anyhow::Result;
use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// RPC客户端
/// 为未来扩展准备的RPC客户端实现
#[allow(dead_code)] // 为未来扩展准备
#[derive(Clone)]
pub struct RpcClient {
    urls: Vec<String>,
    current_index: Arc<AtomicUsize>,
    client_name: String,
}

impl RpcClient {
    pub fn new(urls: Vec<String>, client_name: String) -> Self {
        Self {
            urls,
            current_index: Arc::new(AtomicUsize::new(0)),
            client_name,
        }
    }

    #[allow(dead_code)] // 内部使用，用于 RPC URL 轮换
    fn get_current_url(&self) -> &str {
        let idx = self.current_index.load(Ordering::Relaxed);
        if idx < self.urls.len() {
            &self.urls[idx]
        } else {
            // Fallback to 0 if out of bounds (shouldn't happen with correct logic)
            &self.urls[0]
        }
    }

    #[allow(dead_code)] // 内部使用，用于 RPC URL 轮换
    fn rotate_url(&self) {
        let current = self.current_index.load(Ordering::Relaxed);
        let next = (current + 1) % self.urls.len();
        self.current_index.store(next, Ordering::Relaxed);
        // tracing::warn!("[{}] Rotated RPC node to: {}", self.client_name, self.urls[next]);
    }

    #[allow(dead_code)] // 用于 RPC POST 请求
    pub async fn post<T: DeserializeOwned>(&self, method: &str, params: Value) -> Result<T> {
        let mut attempts = 0;
        let max_attempts = self.urls.len();
        let mut last_error = anyhow::anyhow!("No RPC URLs provided");

        while attempts < max_attempts {
            let url = self.get_current_url();
            let payload = serde_json::json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": 1
            });

            match self.do_post(url, &payload).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // tracing::warn!("[{}] RPC call failed on {}: {}", self.client_name, url, e);
                    last_error = e;
                    self.rotate_url();
                    attempts += 1;
                }
            }
        }
        Err(last_error)
    }

    #[allow(dead_code)] // 内部使用，用于 RPC POST 请求
    async fn do_post<T: DeserializeOwned>(&self, url: &str, payload: &Value) -> Result<T> {
        let resp = Request::post(url)
            .header("Content-Type", "application/json")
            .json(payload)
            .map_err(|e| anyhow::anyhow!("Request build failed: {}", e))?
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Network error: {}", e))?;

        if !resp.ok() {
            return Err(anyhow::anyhow!("HTTP error: {}", resp.status()));
        }

        let json: Value = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))?;

        if let Some(error) = json.get("error") {
            return Err(anyhow::anyhow!("RPC error: {:?}", error));
        }

        let result = json
            .get("result")
            .ok_or_else(|| anyhow::anyhow!("No result in response"))?;

        // Handle Solana's "value" wrapper if present, otherwise return result directly
        if let Some(value) = result.get("value") {
            serde_json::from_value(value.clone())
                .map_err(|e| anyhow::anyhow!("Deserialization failed: {}", e))
        } else {
            serde_json::from_value(result.clone())
                .map_err(|e| anyhow::anyhow!("Deserialization failed: {}", e))
        }
    }

    // For REST APIs (like Bitcoin Esplora)
    #[allow(dead_code)] // 用于 REST API GET 请求
    pub async fn get_json<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let mut attempts = 0;
        let max_attempts = self.urls.len();
        let mut last_error = anyhow::anyhow!("No API URLs provided");

        while attempts < max_attempts {
            let base_url = self.get_current_url();
            let full_url = format!("{}{}", base_url, endpoint);

            match Request::get(&full_url).send().await {
                Ok(resp) => {
                    if resp.ok() {
                        match resp.json::<T>().await {
                            Ok(data) => return Ok(data),
                            Err(e) => {
                                last_error = anyhow::anyhow!("JSON parse failed: {}", e);
                                self.rotate_url();
                            }
                        }
                    } else {
                        last_error = anyhow::anyhow!("HTTP error: {}", resp.status());
                        self.rotate_url();
                    }
                }
                Err(e) => {
                    last_error = anyhow::anyhow!("Network error: {}", e);
                    self.rotate_url();
                }
            }
            attempts += 1;
        }
        Err(last_error)
    }
}
