use crate::blockchain::traits::{ChainAdapter, Transaction, TransactionReceipt};
use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;
use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::json;

/// Solana链适配器
/// 为未来扩展准备的区块链适配器实现
#[allow(dead_code)] // 为未来扩展准备
pub struct SolanaAdapter {
    rpc_urls: Vec<String>,
    cluster: String, // "mainnet-beta", "testnet", "devnet"
}

impl SolanaAdapter {
    pub fn new(rpc_urls: Vec<String>, cluster: String) -> Self {
        Self { rpc_urls, cluster }
    }

    async fn rpc_call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let mut last_error = anyhow::anyhow!("No RPC URLs provided");

        for url in &self.rpc_urls {
            match self.do_rpc_call(url, &payload).await {
                Ok(result) => return Ok(result),
                Err(e) => last_error = e,
            }
        }
        Err(last_error)
    }

    async fn do_rpc_call<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        payload: &serde_json::Value,
    ) -> Result<T> {
        let resp = Request::post(url)
            .header("Content-Type", "application/json")
            .json(payload)
            .map_err(|e| anyhow::anyhow!("Request build failed: {}", e))?
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("RPC call failed: {}", e))?;

        if !resp.ok() {
            return Err(anyhow::anyhow!("RPC error status: {}", resp.status()));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))?;

        if let Some(error) = json.get("error") {
            return Err(anyhow::anyhow!("RPC error: {:?}", error));
        }

        let result = json
            .get("result")
            .ok_or_else(|| anyhow::anyhow!("No result in RPC response"))?;

        // Solana RPC result often contains "value" wrapper for some calls
        if let Some(value) = result.get("value") {
            serde_json::from_value(value.clone())
                .map_err(|e| anyhow::anyhow!("Result value deserialize failed: {}", e))
        } else {
            serde_json::from_value(result.clone())
                .map_err(|e| anyhow::anyhow!("Result deserialize failed: {}", e))
        }
    }
}

#[async_trait(?Send)]
impl ChainAdapter for SolanaAdapter {
    fn chain_name(&self) -> &'static str {
        "Solana"
    }

    fn chain_id(&self) -> u64 {
        // Solana doesn't use numeric chain ID in the same way, but we can map it.
        match self.cluster.as_str() {
            "mainnet-beta" => 101,
            "testnet" => 102,
            "devnet" => 103,
            _ => 0,
        }
    }

    async fn get_balance(&self, address: &str) -> Result<String> {
        let balance: u64 = self.rpc_call("getBalance", json!([address])).await?;
        Ok(balance.to_string())
    }

    async fn get_transactions(
        &self,
        _address: &str,
        _limit: usize,
    ) -> Result<Vec<TransactionReceipt>> {
        // getSignaturesForAddress
        Ok(vec![])
    }

    async fn estimate_gas(&self, _tx: &Transaction) -> Result<u64> {
        // getFeeForMessage
        // For now return standard fee (5000 lamports)
        Ok(5000)
    }

    async fn broadcast_transaction(&self, signed_tx: &[u8]) -> Result<String> {
        let tx_base64 = base64::engine::general_purpose::STANDARD.encode(signed_tx);
        let signature: String = self
            .rpc_call(
                "sendTransaction",
                json!([tx_base64, {"encoding": "base64"}]),
            )
            .await?;
        Ok(signature)
    }
}
