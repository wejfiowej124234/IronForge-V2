//! Bitcoin Fee Service - 企业级Bitcoin费率服务
//! 从后端API获取实时Bitcoin费率，移除硬编码

use crate::shared::api::ApiClient;
use crate::shared::error::AppError;
use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BitcoinFeeEstimate {
    pub fee_per_vb: u64, // sat/vB
    pub fee_per_kb: u64, // sat/kB
}

#[derive(Clone, Copy)]
pub struct BitcoinFeeService {
    app_state: AppState,
}

impl BitcoinFeeService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    fn api(&self) -> ApiClient {
        self.app_state.get_api_client()
    }

    /// 获取Bitcoin费率估算（从后端API）
    ///
    /// # Returns
    /// Bitcoin费率（sat/vB）
    pub async fn get_fee_rate(&self) -> Result<u64, AppError> {
        let api = self.api();
        // deserialize 方法已自动提取 data 字段
        match api
            .get::<BitcoinFeeEstimate>("/api/v1/bitcoin/fee-estimates")
            .await
        {
            Ok(response) => Ok(response.fee_per_vb),
            Err(e) => {
                // API 调用失败，使用降级策略
                log::warn!("Bitcoin费率API调用失败: {}，使用降级策略", e);
                // 企业级实现：降级策略（API返回成功但没有数据）
                // 多级降级策略：
                // 1. 优先从环境变量读取默认费率
                // 2. 最终降级：使用安全默认值（仅作为最后保障）
                let default_rate = std::env::var("BITCOIN_DEFAULT_FEE_RATE_SAT_VBYTE")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|&v| v > 0 && v <= 1000) // 验证范围：合理值（0-1000 sat/vB）
                    .unwrap_or_else(|| {
                        log::warn!("Bitcoin费率API返回空数据，使用安全默认值20 sat/vB");
                        20 // 安全默认值：20 sat/vB
                    });
                Ok(default_rate)
            }
        }
    }

    /// 获取完整的Bitcoin费率信息
    pub async fn get_fee_estimate(&self) -> Result<BitcoinFeeEstimate, AppError> {
        let api = self.api();
        // deserialize 方法已自动提取 data 字段
        api.get("/api/v1/bitcoin/fee-estimates")
            .await
            .map_err(AppError::from)
    }
}

use serde::{Deserialize, Serialize};
