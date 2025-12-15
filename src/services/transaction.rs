use serde::{Deserialize, Serialize};

use crate::shared::api::ApiClient;
use crate::shared::error::AppError;
use crate::shared::state::AppState;
use gloo_timers::future::TimeoutFuture;

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastRequest {
    pub chain: String,
    pub signed_tx: String, // Hex encoded signed transaction
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastResponse {
    pub tx_hash: String,
    pub status: String,
}

// 响应结构体已移除，直接使用 BroadcastResponse 和 TransactionStatus
// deserialize 方法已自动提取 data 字段

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionStatus {
    pub tx_hash: String,
    pub status: String,
    pub confirmations: u64,
    pub last_seen: Option<u64>,
}

#[derive(Clone, Copy)]
pub struct TransactionService {
    app_state: AppState,
}

impl TransactionService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    fn api(&self) -> ApiClient {
        self.app_state.get_api_client()
    }

    pub async fn broadcast(
        &self,
        chain: &str,
        signed_tx: &str,
    ) -> Result<BroadcastResponse, AppError> {
        let payload = BroadcastRequest {
            chain: chain.to_string(),
            signed_tx: signed_tx.to_string(),
        };

        let api = self.api();
        // ✅ v1标准路径
        api.post("/api/v1/transactions/broadcast", &payload)
            .await
            .map_err(AppError::Api)
    }

    pub async fn status(&self, tx_hash: &str) -> Result<TransactionStatus, AppError> {
        let path = format!("/api/v1/transactions/{}/status", tx_hash);
        let api = self.api();
        // ✅ v1标准路径
        api.get(&path).await.map_err(AppError::Api)
    }

    /// 获取账户的nonce（用于Ethereum交易）
    pub async fn get_nonce(&self, address: &str, chain_id: u64) -> Result<u64, AppError> {
        let path = format!(
            "/api/v1/transactions/nonce?address={}&chain_id={}",
            address, chain_id
        );
        let api = self.api();

        #[derive(Deserialize)]
        struct NonceData {
            nonce: u64,
        }

        // deserialize 方法已自动提取 data 字段
        let response: NonceData = api.get(&path).await.map_err(AppError::from)?;
        Ok(response.nonce)
    }

    // 注意：get_recent_blockhash和get_seqno的完整实现在下面（202-259行）

    #[allow(dead_code)] // 用于交易确认等待功能
    pub async fn wait_for_confirmation(
        &self,
        tx_hash: &str,
        max_attempts: u32,
        interval_ms: u32,
    ) -> Result<TransactionStatus, AppError> {
        let mut attempts = 0;
        loop {
            let status = self.status(tx_hash).await?;
            if status.status == "confirmed" || status.status == "failed" {
                return Ok(status);
            }

            attempts += 1;
            if attempts >= max_attempts {
                return Ok(status);
            }

            TimeoutFuture::new(interval_ms).await;
        }
    }

    #[allow(dead_code)] // 用于交易历史查询功能
    pub async fn history(&self, page: u32) -> Result<Vec<TransactionHistoryItem>, AppError> {
        let path = format!("/api/v1/transactions/history?page={}", page);
        let api = self.api();
        // deserialize 方法已自动提取 data 字段
        api.get(&path).await.map_err(AppError::Api)
    }

    /// 按地址查询交易历史（✅ V1 API标准：使用公开路由）
    pub async fn get_history(
        &self,
        address: &str,
        chain: &str,
    ) -> Result<Vec<TransactionHistoryItem>, AppError> {
        // ✅ 使用后端公开路由：GET /api/wallets/:address/transactions?chain=xxx
        let path = format!("/api/v1/wallets/{}/transactions?chain={}", address, chain);
        let api = self.api();
        // deserialize 方法已自动提取 data 字段
        api.get(&path).await.map_err(AppError::Api)
    }

    /// Get Solana recent blockhash
    pub async fn get_recent_blockhash(&self, _chain: &str) -> Result<String, AppError> {
        let api = self.api();

        #[derive(Deserialize)]
        struct BlockhashData {
            blockhash: String,
        }

        // ✅ v1标准路径
        let response: BlockhashData = api
            .get("/api/v1/solana/recent-blockhash")
            .await
            .map_err(AppError::from)?;
        Ok(response.blockhash)
    }

    /// Get TON sequence number for an address
    pub async fn get_seqno(&self, address: &str, _chain: &str) -> Result<u64, AppError> {
        let api = self.api();
        let path = format!("/api/v1/ton/seqno?address={}", address);

        #[derive(Deserialize)]
        struct SeqnoData {
            seqno: u64,
        }

        // deserialize 方法已自动提取 data 字段
        let response: SeqnoData = api.get(&path).await.map_err(AppError::from)?;
        Ok(response.seqno)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionHistoryItem {
    pub hash: String,
    pub tx_type: String, // "send", "receive"
    pub status: String,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub token: String,
    pub timestamp: u64,
    pub fee: String,
}

// HistoryApiResponse 已移除，直接使用 Option<Vec<TransactionHistoryItem>>
// deserialize 方法已自动提取 data 字段
