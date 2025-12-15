//! Country Support Service - 国家支持服务
//! 企业级国家支持服务，维护服务商国家支持列表

use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 国家支持信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountrySupportInfo {
    pub country_code: String, // ISO 3166-1 alpha-2
    pub country_name: String,
    pub supported_providers: Vec<String>,
    pub unsupported_providers: Vec<String>,
    pub last_updated: String,
}

/// 服务商国家支持列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCountrySupport {
    pub provider: String,
    pub supported_countries: Vec<String>,
    pub unsupported_countries: Vec<String>,
    pub last_synced: String,
}

/// 国家支持服务
pub struct CountrySupportService {
    api_client: Arc<ApiClient>,
}

impl CountrySupportService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            api_client: Arc::new(app_state.get_api_client()),
        }
    }

    /// 获取国家支持信息
    ///
    /// # 参数
    /// - `country_code`: 国家代码（ISO 3166-1 alpha-2）
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_country_support(
        &self,
        country_code: &str,
    ) -> Result<CountrySupportInfo, String> {
        if country_code.is_empty() {
            return Err("国家代码不能为空".to_string());
        }

        let url = format!("/api/v1/country-support/{}", country_code);

        self.api_client
            .get::<CountrySupportInfo>(&url)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("not found") || error_msg.contains("404") {
                    format!("未找到国家 {} 的支持信息", country_code)
                } else if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else {
                    format!("获取国家支持信息失败：{}", e)
                }
            })
    }

    /// 获取服务商国家支持列表
    ///
    /// # 参数
    /// - `provider`: 服务商名称（可选，如果为None则获取所有服务商）
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_provider_support(
        &self,
        provider: Option<&str>,
    ) -> Result<Vec<ProviderCountrySupport>, String> {
        let url = if let Some(p) = provider {
            format!("/api/v1/country-support/provider/{}", p)
        } else {
            "/api/v1/country-support/providers".to_string()
        };

        self.api_client
            .get::<Vec<ProviderCountrySupport>>(&url)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else {
                    format!("获取服务商国家支持列表失败：{}", e)
                }
            })
    }

    /// 同步服务商国家支持列表
    ///
    /// # 参数
    /// - `provider`: 服务商名称（可选，如果为None则同步所有服务商）
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn sync_provider_support(
        &self,
        provider: Option<&str>,
    ) -> Result<SyncResult, String> {
        let url = if let Some(p) = provider {
            format!("/api/v1/country-support/sync?provider={}", p)
        } else {
            "/api/v1/country-support/sync".to_string()
        };

        self.api_client
            .post::<SyncResult, serde_json::Value>(&url, &serde_json::json!({}))
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else if error_msg.contains("forbidden") || error_msg.contains("403") {
                    "权限不足，只有管理员可以同步国家支持列表".to_string()
                } else {
                    format!("同步国家支持列表失败：{}", e)
                }
            })
    }
}

/// 同步结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub provider: String,
    pub synced_countries: u64,
    pub updated_countries: u64,
    pub sync_time: String,
    pub errors: Vec<String>,
}
