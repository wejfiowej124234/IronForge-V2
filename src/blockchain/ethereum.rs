use crate::blockchain::traits::{ChainAdapter, Transaction, TransactionReceipt};
use anyhow::Result;
use async_trait::async_trait;
use gloo_net::http::Request;
use primitive_types::U256;
use serde::Deserialize;
use serde_json::json;

/// Ethereum链适配器
/// 为未来扩展准备的区块链适配器实现
#[allow(dead_code)] // 为未来扩展准备
pub struct EthereumAdapter {
    rpc_urls: Vec<String>,
    chain_id: u64,
}

impl EthereumAdapter {
    pub fn new(rpc_urls: Vec<String>, chain_id: u64) -> Self {
        Self { rpc_urls, chain_id }
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
                Err(e) => {
                    // Log error and try next
                    // tracing::warn!("RPC call to {} failed: {}", url, e);
                    last_error = e;
                }
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
        serde_json::from_value(result.clone())
            .map_err(|e| anyhow::anyhow!("Result deserialize failed: {}", e))
    }
}

#[async_trait(?Send)]
impl ChainAdapter for EthereumAdapter {
    fn chain_name(&self) -> &'static str {
        "Ethereum"
    }

    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    async fn get_balance(&self, address: &str) -> Result<String> {
        let balance_hex: String = self
            .rpc_call("eth_getBalance", json!([address, "latest"]))
            .await?;
        let balance = U256::from_str_radix(balance_hex.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow::anyhow!("Hex parse error: {:?}", e))?;
        Ok(balance.to_string())
    }

    async fn get_transactions(
        &self,
        _address: &str,
        _limit: usize,
    ) -> Result<Vec<TransactionReceipt>> {
        // eth_getLogs or external indexer needed for history.
        Ok(vec![])
    }

    async fn estimate_gas(&self, tx: &Transaction) -> Result<u64> {
        let value_hex = if tx.value.starts_with("0x") {
            tx.value.clone()
        } else {
            let val = U256::from_dec_str(&tx.value).unwrap_or(U256::zero());
            format!("0x{:x}", val)
        };

        let tx_obj = json!({
            "to": tx.to,
            "value": value_hex,
            "data": tx.data.as_ref().map(|d| format!("0x{}", hex::encode(d))),
        });

        let gas_hex: String = self.rpc_call("eth_estimateGas", json!([tx_obj])).await?;
        let gas = u64::from_str_radix(gas_hex.trim_start_matches("0x"), 16)?;
        Ok(gas)
    }

    async fn broadcast_transaction(&self, signed_tx: &[u8]) -> Result<String> {
        let tx_hex = format!("0x{}", hex::encode(signed_tx));
        let tx_hash: String = self
            .rpc_call("eth_sendRawTransaction", json!([tx_hex]))
            .await?;
        Ok(tx_hash)
    }
}
