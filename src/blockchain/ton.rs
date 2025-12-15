use crate::blockchain::rpc::RpcClient;
use crate::blockchain::traits::{ChainAdapter, Transaction, TransactionReceipt};
use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;

/// TON链适配器
/// 为未来扩展准备的区块链适配器实现
#[allow(dead_code)] // 为未来扩展准备
pub struct TonAdapter {
    rpc: RpcClient,
    chain_id: u64,
}

impl TonAdapter {
    pub fn new(rpc_urls: Vec<String>, chain_id: u64) -> Self {
        Self {
            rpc: RpcClient::new(rpc_urls, "TON".to_string()),
            chain_id,
        }
    }
}

#[async_trait(?Send)]
impl ChainAdapter for TonAdapter {
    fn chain_name(&self) -> &'static str {
        "TON"
    }

    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    async fn get_balance(&self, address: &str) -> Result<String> {
        // Production implementation: Use TON Center API directly
        // TON Center API v2: getAddressInformation
        // Response: { "ok": true, "result": { "balance": "1000000000", ... } }

        #[derive(serde::Deserialize)]
        struct TonBalanceResp {
            ok: bool,
            result: TonBalanceResult,
            #[serde(default)]
            error: Option<String>,
        }
        #[derive(serde::Deserialize)]
        struct TonBalanceResult {
            balance: String, // Nanotons (1 TON = 10^9 nanotons)
        }

        // Use TON Center public API
        let api_url = std::env::var("TON_API_URL")
            .unwrap_or_else(|_| "https://toncenter.com/api/v2".to_string());

        let url = format!("{}/getAddressInformation?address={}", api_url, address);

        // Use reqwest or gloo_net to call TON Center API
        // Since we're in a WASM environment, use gloo_net
        use gloo_net::http::Request;

        match Request::get(&url)
            .header("Accept", "application/json")
            .send()
            .await
        {
            Ok(resp) => {
                if resp.ok() {
                    match resp.json::<TonBalanceResp>().await {
                        Ok(data) => {
                            if data.ok {
                                Ok(data.result.balance)
                            } else {
                                Err(anyhow::anyhow!(data
                                    .error
                                    .unwrap_or_else(|| "TON API returned error".to_string())))
                            }
                        }
                        Err(e) => Err(anyhow::anyhow!("Failed to parse TON API response: {}", e)),
                    }
                } else {
                    let status = resp.status();
                    let text = resp
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    Err(anyhow::anyhow!("TON API error {}: {}", status, text))
                }
            }
            Err(e) => {
                // Fallback: Try using backend API if available
                // This requires wallet_id, which we don't have here
                // For now, return error with helpful message
                Err(anyhow::anyhow!(
                    "Failed to query TON balance: {}. Address: {}",
                    e,
                    address
                ))
            }
        }
    }

    async fn get_transactions(
        &self,
        _address: &str,
        _limit: usize,
    ) -> Result<Vec<TransactionReceipt>> {
        // getTransactions
        Ok(vec![])
    }

    async fn estimate_gas(&self, _tx: &Transaction) -> Result<u64> {
        // TON fees are complex (storage + computation + fwd).
        // Return a safe default for now.
        Ok(10_000_000) // 0.01 TON
    }

    async fn broadcast_transaction(&self, signed_tx: &[u8]) -> Result<String> {
        // Encode transaction as base64 BOC
        let boc_base64 = base64::engine::general_purpose::STANDARD.encode(signed_tx);

        // Use backend API for broadcasting
        // Note: This requires AppState/TransactionService, which may not be directly available here
        // In production, this should be refactored to use TransactionService
        // For now, we'll use a direct HTTP call to the backend API

        use crate::shared::api::ApiClient;
        use serde::{Deserialize, Serialize};

        #[derive(Serialize)]
        struct BroadcastRequest {
            boc: String,
        }

        #[derive(Deserialize)]
        struct BroadcastData {
            tx_hash: String,
        }

        // Get API client from default config
        // In a real implementation, this should be passed as a parameter or accessed via context
        let api = ApiClient::new(crate::shared::api::ApiConfig::default());
        let request = BroadcastRequest { boc: boc_base64 };

        // ✅ v1标准路径
        let response: BroadcastData = api
            .post("/api/v1/ton/broadcast", &request)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to broadcast TON transaction: {:?}", e))?;
        Ok(response.tx_hash)
    }
}
