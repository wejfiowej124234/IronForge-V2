use serde::{Deserialize, Serialize};

use crate::shared::api::ApiClient;
use crate::shared::error::AppError;
use crate::shared::request::{CachePolicy, SmartRequestContext};
use crate::shared::state::AppState;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GasSpeed {
    Slow,
    Average,
    Fast,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GasEstimate {
    pub base_fee: String,
    pub max_priority_fee: String,
    pub max_fee_per_gas: String,
    pub estimated_time_seconds: u64,
    pub base_fee_gwei: f64,
    pub max_priority_fee_gwei: f64,
    pub max_fee_per_gas_gwei: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GasEstimateResponse {
    pub slow: GasEstimate,
    #[serde(alias = "normal")]
    pub average: GasEstimate,
    pub fast: GasEstimate,
}

// ApiResponse 已移除，直接使用 GasEstimateResponse
// deserialize 方法已自动提取 data 字段

#[derive(Clone, Copy)]
pub struct GasService {
    app_state: AppState,
}

impl GasService {
    /// 创建GasService实例
    ///
    /// 注意：此方法当前未使用，但保留用于未来扩展
    #[allow(dead_code)]
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    fn api(&self) -> ApiClient {
        self.app_state.get_api_client()
    }

    fn context(&self) -> SmartRequestContext {
        SmartRequestContext::new(self.app_state)
    }

    pub async fn estimate_all(&self, chain: &str) -> Result<GasEstimateResponse, AppError> {
        let key = format!("gas:{}:aggregate", chain.to_lowercase());
        let path = format!("/api/v1/gas/estimate-all?chain={}", chain);
        let api = self.api();
        let ctx = self.context();

        // deserialize 方法已自动提取 data 字段
        let response: GasEstimateResponse = ctx
            .run(&key, CachePolicy::short(), move || {
                let api = api.clone();
                let path = path.clone();
                async move { api.get(&path).await }
            })
            .await
            .map_err(AppError::from)?;

        Ok(response)
    }

    /// 获取指定速度级别的Gas费估算
    ///
    /// 注意：此方法当前未使用，但保留用于未来扩展
    #[allow(dead_code)]
    pub async fn estimate(&self, chain: &str, speed: GasSpeed) -> Result<GasEstimate, AppError> {
        let all = self.estimate_all(chain).await?;
        let estimate = match speed {
            GasSpeed::Slow => all.slow,
            GasSpeed::Average => all.average,
            GasSpeed::Fast => all.fast,
        };
        Ok(estimate)
    }

    /// 获取后端自动推荐的最优Gas费（使用average作为默认推荐）
    /// 后端已实现智能化选择，前端直接使用推荐值
    pub async fn get_recommended(&self, chain: &str) -> Result<GasEstimate, AppError> {
        // 后端自动选择最优Gas费，我们使用average作为默认推荐
        // 这是最平衡的选择，兼顾速度和成本
        self.estimate(chain, GasSpeed::Average).await
    }
}
