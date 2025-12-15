//! 用户服务 - 获取用户信息和KYC状态

use crate::shared::api::ApiClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 用户KYC状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserKycStatusResponse {
    pub kyc_status: String,  // "unverified", "basic", "standard", "premium"
    pub daily_limit: f64,
    pub monthly_limit: f64,
    pub daily_used: f64,
    pub monthly_used: f64,
}

/// 用户信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoResponse {
    pub id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub kyc_status: String,
    pub created_at: String,
}

/// 用户服务
pub struct UserService {
    api_client: Arc<ApiClient>,
}

impl UserService {
    pub fn new(api_client: Arc<ApiClient>) -> Self {
        Self { api_client }
    }

    /// 获取用户KYC状态和额度信息
    pub async fn get_kyc_status(&self) -> Result<UserKycStatusResponse> {
        // 企业级实现：从后端获取真实KYC状态
        let response = self
            .api_client
            .get::<UserKycStatusResponse>("/api/v1/users/kyc/status")
            .await?;

        tracing::info!("[UserService] get_kyc_status response: {:?}", response);

        Ok(response)
    }

    /// 获取当前用户信息
    pub async fn get_user_info(&self) -> Result<UserInfoResponse> {
        let response = self
            .api_client
            .get::<UserInfoResponse>("/api/v1/users/me")
            .await?;

        tracing::info!("[UserService] get_user_info response: {:?}", response);

        Ok(response)
    }
}
