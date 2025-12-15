//! Wallet Service - Backend API Integration
//! 钱包服务：对接后端钱包管理API

use crate::shared::api::ApiClient;
use crate::shared::error::AppError;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Wallet DTO from backend (matches SimpleWalletResp from backend)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)] // 用于钱包管理 API
pub struct WalletDto {
    pub id: String,      // 后端返回 String，不是 Uuid
    pub user_id: String, // 后端返回 String，不是 Uuid
    pub chain: String,
    pub address: String,
    pub public_key: String, // ✅ 后端返回的公钥（非托管模式必须）
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub group_id: Option<String>, // ✅ 钱包组ID（用于多链钱包合并）
}

// ✅废弃端点已移除，统一使用 UnifiedCreateWalletRequest

/// Unified create wallet request (匹配后端 UnifiedCreateWalletRequest)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCreateWalletRequest {
    /// 钱包名称
    pub name: String,
    /// 链标识 (chain_id 或 symbol, 例如: "ethereum", "ETH", "1")
    pub chain: String,
    /// 助记词 (可选，不提供则自动生成)
    pub mnemonic: Option<String>,
    /// 助记词长度 (12 或 24，默认 12)
    pub word_count: Option<u8>,
    /// 账户索引 (默认 0)
    pub account: Option<u32>,
    /// 地址索引 (默认 0)
    pub index: Option<u32>,
    /// 租户ID（可选，从JWT获取）
    pub tenant_id: Option<String>,
    /// 用户ID（可选，从JWT获取）
    pub user_id: Option<String>,
}

/// 批量创建钱包请求（匹配后端 CreateMultiChainWalletsRequest）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateWalletsRequest {
    pub wallets: Vec<WalletRegistrationInfo>,
}

/// 钱包注册信息（匹配后端 WalletRegistrationInfo）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletRegistrationInfo {
    pub chain: String,
    pub address: String,
    pub public_key: String,
    pub derivation_path: Option<String>,
    pub name: Option<String>,
}

/// 批量创建钱包响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateWalletsResponse {
    pub success: bool,
    pub wallets: Vec<WalletCreateResult>,
    pub failed: Vec<WalletCreateError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCreateResult {
    pub id: String,
    pub chain: String,
    pub address: String,
    pub created_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCreateError {
    pub chain: String,
    pub address: String,
    pub error: String,
}

/// Unified create wallet response (匹配后端 UnifiedCreateWalletResponse)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCreateWalletResponse {
    pub message: String,
    /// 钱包数据库记录
    pub wallet: WalletDbRecord,
    /// 助记词（仅在生成新助记词时返回）
    pub mnemonic: Option<String>,
}

/// Wallet database record (匹配后端 WalletDbRecord)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDbRecord {
    pub id: String,
    pub name: String,
    pub address: String,
    pub public_key: String,
    pub chain_id: i64,
    pub chain_symbol: String,
    pub curve_type: String,
    pub derivation_path: String,
    pub created_at: String,
}

/// Update wallet request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 用于钱包管理 API
pub struct UpdateWalletRequest {
    pub name: Option<String>,
}

#[derive(Clone, Copy)]
#[allow(dead_code)] // 用于钱包管理 API 服务
pub struct WalletService {
    app_state: AppState,
}

#[allow(dead_code)] // Wallet service methods, used in future features
impl WalletService {
    #[allow(dead_code)]
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    #[allow(dead_code)]
    fn api(&self) -> ApiClient {
        self.app_state.get_api_client()
    }

    /// List all wallets for the current user
    /// 后端返回: ApiResponse<ListWalletsResp>
    /// V1 API标准：使用page和page_size分页，tenant_id和user_id从JWT自动提取
    pub async fn list_wallets(&self) -> Result<Vec<WalletDto>, AppError> {
        self.list_wallets_paginated(1, 100).await
    }

    /// List wallets with pagination
    /// 后端返回: ApiResponse<ListWalletsResp { wallets: Vec<WalletResp>, total: i64 }>
    pub async fn list_wallets_paginated(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<Vec<WalletDto>, AppError> {
        let api = self.api();
        // ✅ V1 API标准：使用page和page_size参数，不需要tenant_id（从JWT获取）
        let path = format!("/api/v1/wallets?page={}&page_size={}", page, page_size);

        #[cfg(debug_assertions)]
        {
            use tracing::info;
            info!("🔍 Request path (before API call): {}", path);
        }

        #[derive(serde::Deserialize)]
        struct ListWalletsResp {
            wallets: Vec<WalletDto>,
            total: i64,
        }

        match api.get::<ListWalletsResp>(&path).await {
            Ok(resp) => Ok(resp.wallets),
            Err(e) => {
                // 检测401错误：token过期或无效
                if crate::shared::auth_handler::is_unauthorized_error(&e) {
                    #[cfg(debug_assertions)]
                    {
                        use tracing::warn;
                        warn!("⚠️ Token已过期或无效，清理状态");
                    }
                    // 强制清理过期token
                    self.app_state.handle_unauthorized();
                }
                Err(e.into())
            }
        }
    }

    /// Get wallet by ID
    /// 后端返回: ApiResponse<SimpleWalletResp>
    /// deserialize 方法已自动提取 data 字段
    #[allow(dead_code)]
    pub async fn get_wallet(&self, wallet_id: Uuid) -> Result<WalletDto, AppError> {
        let api = self.api();
        let path = format!("/api/v1/wallets/{}", wallet_id);
        // 后端返回 ApiResponse<SimpleWalletResp>，deserialize 自动提取 data 字段
        let wallet: WalletDto = api.get(&path).await?;
        Ok(wallet)
    }

    /// Create a new wallet using unified-create endpoint
    /// 后端返回: ApiResponse<UnifiedCreateWalletResponse>
    /// deserialize 方法已自动提取 data 字段
    pub async fn create_wallet(
        &self,
        request: UnifiedCreateWalletRequest,
    ) -> Result<UnifiedCreateWalletResponse, AppError> {
        let api = self.api();
        // 后端返回 ApiResponse<UnifiedCreateWalletResponse>，deserialize 自动提取 data 字段
        // ✅ 企业级标准：使用 v1 统一路径
        let response: UnifiedCreateWalletResponse =
            api.post("/api/v1/wallets/batch", &request).await?;
        Ok(response)
    }

    /// 批量创建钱包（匹配后端 BatchCreateWalletsRequest）
    /// 后端返回: ApiResponse<BatchCreateWalletsResponse>
    pub async fn batch_create_wallets(
        &self,
        request: BatchCreateWalletsRequest,
    ) -> Result<BatchCreateWalletsResponse, AppError> {
        let api = self.api();
        let response: BatchCreateWalletsResponse =
            api.post("/api/v1/wallets/batch", &request).await?;
        Ok(response)
    }

    // ✅已完全移除废弃方法

    /// Update wallet
    /// 后端返回: ApiResponse<SimpleWalletResp>
    /// deserialize 方法已自动提取 data 字段
    #[allow(dead_code)]
    pub async fn update_wallet(
        &self,
        wallet_id: Uuid,
        request: UpdateWalletRequest,
    ) -> Result<WalletDto, AppError> {
        let api = self.api();
        let path = format!("/api/v1/wallets/{}", wallet_id);
        // ✅ v1标准路径
        let wallet: WalletDto = api.put(&path, &request).await?;
        Ok(wallet)
    }

    /// Delete wallet
    #[allow(dead_code)]
    pub async fn delete_wallet(&self, wallet_id: Uuid) -> Result<(), AppError> {
        let api = self.api();
        let path = format!("/api/v1/wallets/{}", wallet_id);
        // deserialize 方法已自动提取 data 字段
        // 后端返回: {code: 0, message: "success", data: {}}
        let _: crate::shared::api::EmptyResponse =
            api.delete(&path).await.map_err(AppError::from)?;
        Ok(())
    }
}
