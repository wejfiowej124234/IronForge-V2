//! Wallet Transaction Service - Backend API Integration
//! 交易服务：对接后端交易管理API

use crate::shared::api::ApiClient;
use crate::shared::error::AppError;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Transaction DTO from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 用于交易管理 API
pub struct TransactionDto {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub chain: String,
    pub to_address: String,
    pub amount: String,
    pub status: String,
    pub tx_hash: Option<String>,
    pub gas_limit: Option<String>,
    pub gas_price: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Create transaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 用于交易管理 API
pub struct CreateTransactionRequest {
    pub to_address: String,
    pub amount: String,
    pub chain: String,
    pub gas_limit: Option<String>,
    pub gas_price: Option<String>,
    pub data: Option<String>,
}

/// Transaction list response (may vary by endpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 用于交易管理 API
pub struct TransactionListResponse {
    #[serde(default)]
    pub transactions: Vec<TransactionDto>,
    #[serde(default)]
    pub page: u64,
    #[serde(default)]
    pub page_size: u64,
    #[serde(default)]
    pub total: usize,
}

#[derive(Clone, Copy)]
#[allow(dead_code)] // 用于交易管理 API 服务
pub struct WalletTransactionService {
    app_state: AppState,
}

impl WalletTransactionService {
    #[allow(dead_code)] // 用于交易管理 API 服务
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    #[allow(dead_code)] // 内部使用
    fn api(&self) -> ApiClient {
        self.app_state.get_api_client()
    }

    /// Create a new transaction
    #[allow(dead_code)] // 用于交易管理 API
    pub async fn create_transaction(
        &self,
        wallet_id: Uuid,
        request: CreateTransactionRequest,
    ) -> Result<TransactionDto, AppError> {
        let api = self.api();
        let path = format!("/api/v1/wallets/{}/transactions", wallet_id);
        let tx: TransactionDto = api.post(&path, &request).await?;
        Ok(tx)
    }

    /// List transactions for a wallet
    #[allow(dead_code)] // 用于交易管理 API
    pub async fn list_transactions(
        &self,
        wallet_id: Uuid,
    ) -> Result<Vec<TransactionDto>, AppError> {
        let api = self.api();
        let path = format!("/api/v1/wallets/{}/transactions", wallet_id);
        // deserialize 方法已自动提取 data 字段
        // 后端返回统一格式: {code, message, data: TransactionListResponse}
        let response: TransactionListResponse = api.get(&path).await.map_err(AppError::from)?;
        Ok(response.transactions)
    }

    /// Get transaction by ID
    #[allow(dead_code)] // 用于交易管理 API
    pub async fn get_transaction(&self, tx_id: Uuid) -> Result<TransactionDto, AppError> {
        let api = self.api();
        let path = format!("/api/v1/transactions/{}", tx_id);
        let tx: TransactionDto = api.get(&path).await?;
        Ok(tx)
    }
}
