use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::api::ApiClient;
use crate::shared::error::AppError;
use crate::shared::state::AppState;

/// 平台服务费报价（企业级实现）
///
/// 注意：这是平台服务费（钱包服务商收取的服务费用），与Gas费用（区块链网络费用）完全独立！
///
/// Gas费用由区块链网络收取，用于执行交易（gas_used * gas_price）
/// 平台服务费由钱包服务商收取，用于提供钱包服务
///
/// 这两个费用是完全独立的，不能混淆！
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlatformFeeQuote {
    /// 平台服务费：钱包服务商收取的服务费用
    /// 注意：这不是Gas费用，Gas费用由区块链网络自动扣除
    pub platform_fee: f64,
    /// 平台服务费收款地址（钱包服务商的费用归集地址）
    pub collector_address: String,
    /// 应用的费用规则ID
    pub applied_rule_id: Uuid,
    /// 费用规则版本号
    pub rule_version: i32,
}

#[derive(Debug, Serialize, Clone)]
struct FeeCalculationRequest {
    chain: String,
    operation: String,
    amount: f64,
}

// 响应结构体已移除，直接使用 PlatformFeeQuote
// deserialize 方法已自动提取 data 字段

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct FeeService {
    app_state: AppState,
}

impl FeeService {
    #[allow(dead_code)]
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    fn api(&self) -> ApiClient {
        self.app_state.get_api_client()
    }

    /// 计算平台服务费（企业级实现）
    ///
    /// 注意：这是平台服务费（钱包服务商收取的服务费用），与Gas费用（区块链网络费用）完全独立！
    ///
    /// Gas费用由区块链网络收取，用于执行交易（gas_used * gas_price）
    /// 平台服务费由钱包服务商收取，用于提供钱包服务
    ///
    /// 这两个费用是完全独立的，不能混淆！
    #[allow(dead_code)]
    pub async fn calculate(
        &self,
        chain: &str,
        operation: &str,
        amount: f64,
    ) -> Result<PlatformFeeQuote, AppError> {
        if amount < 0.0 {
            return Err(AppError::Validation(
                "amount must be non-negative".to_string(),
            ));
        }

        let normalized_chain = chain.to_lowercase();
        let normalized_operation = operation.to_lowercase();
        let api = self.api();

        let request_body = FeeCalculationRequest {
            chain: normalized_chain,
            operation: normalized_operation,
            amount,
        };

        // deserialize 方法已自动提取 data 字段
        api.post::<PlatformFeeQuote, _>("/api/v1/fees/calculate", &request_body)
            .await
            .map_err(AppError::from)
    }
}
