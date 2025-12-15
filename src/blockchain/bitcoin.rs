use crate::blockchain::traits::{ChainAdapter, Transaction, TransactionReceipt};
use anyhow::Result;
use async_trait::async_trait;
use gloo_net::http::Request;
use serde::Deserialize;

/// Bitcoin链适配器
/// 为未来扩展准备的区块链适配器实现
#[allow(dead_code)] // 为未来扩展准备
pub struct BitcoinAdapter {
    api_urls: Vec<String>,
    network: String, // "mainnet" or "testnet"
}

impl BitcoinAdapter {
    pub fn new(api_urls: Vec<String>, network: String) -> Self {
        Self { api_urls, network }
    }

    async fn api_get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let mut last_error = anyhow::anyhow!("No API URLs provided");

        for url in &self.api_urls {
            let full_url = format!("{}{}", url, endpoint);
            match Request::get(&full_url).send().await {
                Ok(resp) => {
                    if resp.ok() {
                        match resp.json::<T>().await {
                            Ok(data) => return Ok(data),
                            Err(e) => last_error = anyhow::anyhow!("JSON parse failed: {}", e),
                        }
                    } else {
                        last_error = anyhow::anyhow!("API error status: {}", resp.status());
                    }
                }
                Err(e) => last_error = anyhow::anyhow!("Request failed: {}", e),
            }
        }
        Err(last_error)
    }

    #[allow(dead_code)] // 用于 Bitcoin RPC API 调用
    async fn api_post(&self, endpoint: &str, body: &str) -> Result<String> {
        let mut last_error = anyhow::anyhow!("No API URLs provided");

        for url in &self.api_urls {
            let full_url = format!("{}{}", url, endpoint);
            // Handle Result from body()
            let request = match Request::post(&full_url).body(body) {
                Ok(req) => req,
                Err(e) => {
                    last_error = anyhow::anyhow!("Failed to set body: {}", e);
                    continue;
                }
            };

            match request.send().await {
                Ok(resp) => {
                    if resp.ok() {
                        match resp.text().await {
                            Ok(text) => return Ok(text),
                            Err(e) => last_error = anyhow::anyhow!("Text parse failed: {}", e),
                        }
                    } else {
                        last_error = anyhow::anyhow!("API error status: {}", resp.status());
                    }
                }
                Err(e) => last_error = anyhow::anyhow!("Request failed: {}", e),
            }
        }
        Err(last_error)
    }
}

#[allow(dead_code)] // 为未来扩展准备
#[derive(Deserialize)]
struct Utxo {
    txid: String,
    vout: u32,
    status: Status,
    value: u64,
}

#[allow(dead_code)] // 为未来扩展准备
#[derive(Deserialize)]
struct Status {
    confirmed: bool,
    block_height: Option<u32>,
    block_hash: Option<String>,
    block_time: Option<u64>,
}

#[async_trait(?Send)]
impl ChainAdapter for BitcoinAdapter {
    fn chain_name(&self) -> &'static str {
        "Bitcoin"
    }

    fn chain_id(&self) -> u64 {
        if self.network == "mainnet" {
            0
        } else {
            1
        }
    }

    async fn get_balance(&self, address: &str) -> Result<String> {
        // Using Esplora API: /address/:address/utxo
        let utxos: Vec<Utxo> = self.api_get(&format!("/address/{}/utxo", address)).await?;
        let total: u64 = utxos.iter().map(|u| u.value).sum();
        Ok(total.to_string())
    }

    async fn get_transactions(
        &self,
        _address: &str,
        _limit: usize,
    ) -> Result<Vec<TransactionReceipt>> {
        // Implement transaction history fetching
        Ok(vec![])
    }

    async fn estimate_gas(&self, _tx: &Transaction) -> Result<u64> {
        // Bitcoin uses fee rate (sat/vB).
        // Fetch fee estimates from backend API
        // TODO: In production, this should fetch from backend API
        // For now, we'll use a default value
        let default_fee = 20u64; // Default: 20 sat/vB

        // Try to fetch from backend API
        // Note: This requires AppState to be available, which may not be the case here
        // In a real implementation, this should be refactored to pass AppState or use a service
        Ok(default_fee) // Return default for now, can be enhanced with actual API call
    }

    async fn broadcast_transaction(&self, signed_tx: &[u8]) -> Result<String> {
        let tx_hex = hex::encode(signed_tx);
        let txid = self.api_post("/tx", &tx_hex).await?;
        Ok(txid)
    }
}
