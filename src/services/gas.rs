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

/// Pure helper: pick an estimate by speed from an aggregate response.
pub fn pick_estimate(all: &GasEstimateResponse, speed: GasSpeed) -> &GasEstimate {
    match speed {
        GasSpeed::Slow => &all.slow,
        GasSpeed::Average => &all.average,
        GasSpeed::Fast => &all.fast,
    }
}

/// Pure helper: compute a max-fee-per-gas based gas cost in ETH.
///
/// - `max_fee_per_gas_gwei`: gwei / gas
/// - `gas_limit`: gas
///
/// Returns ETH (gwei * gas / 1e9).
pub fn gas_fee_eth_from_max_fee_per_gas_gwei(max_fee_per_gas_gwei: f64, gas_limit: u64) -> f64 {
    if !max_fee_per_gas_gwei.is_finite() || max_fee_per_gas_gwei < 0.0 {
        return 0.0;
    }
    (max_fee_per_gas_gwei * gas_limit as f64) / 1e9
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_est(gwei: f64, secs: u64) -> GasEstimate {
        GasEstimate {
            base_fee: "0x0".to_string(),
            max_priority_fee: "0x0".to_string(),
            max_fee_per_gas: "0x0".to_string(),
            estimated_time_seconds: secs,
            base_fee_gwei: 0.0,
            max_priority_fee_gwei: 0.0,
            max_fee_per_gas_gwei: gwei,
        }
    }

    #[test]
    fn gas_fee_eth_changes_with_gwei() {
        let slow = gas_fee_eth_from_max_fee_per_gas_gwei(1.0, 21_000);
        let fast = gas_fee_eth_from_max_fee_per_gas_gwei(2.0, 21_000);
        assert!(fast > slow);
    }

    #[test]
    fn gas_fee_eth_invalid_values_are_zero() {
        assert_eq!(gas_fee_eth_from_max_fee_per_gas_gwei(-1.0, 21_000), 0.0);
        assert_eq!(gas_fee_eth_from_max_fee_per_gas_gwei(f64::NAN, 21_000), 0.0);
        assert_eq!(
            gas_fee_eth_from_max_fee_per_gas_gwei(f64::INFINITY, 21_000),
            0.0
        );
    }

    #[test]
    fn pick_estimate_selects_expected_tier() {
        let all = GasEstimateResponse {
            slow: dummy_est(1.0, 300),
            average: dummy_est(2.0, 180),
            fast: dummy_est(3.0, 60),
        };

        assert_eq!(
            pick_estimate(&all, GasSpeed::Slow).max_fee_per_gas_gwei,
            1.0
        );
        assert_eq!(
            pick_estimate(&all, GasSpeed::Average).max_fee_per_gas_gwei,
            2.0
        );
        assert_eq!(
            pick_estimate(&all, GasSpeed::Fast).max_fee_per_gas_gwei,
            3.0
        );
    }
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
        Ok(pick_estimate(&all, speed).clone())
    }

    /// 获取后端自动推荐的最优Gas费（使用average作为默认推荐）
    /// 后端已实现智能化选择，前端直接使用推荐值
    pub async fn get_recommended(&self, chain: &str) -> Result<GasEstimate, AppError> {
        // 后端自动选择最优Gas费，我们使用average作为默认推荐
        // 这是最平衡的选择，兼顾速度和成本
        self.estimate(chain, GasSpeed::Average).await
    }
}
