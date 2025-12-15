//! Gas Limit Estimation Service - 企业级Gas Limit估算服务
//! 从后端API获取精确的Gas Limit估算，移除硬编码

use crate::shared::api::ApiClient;
use crate::shared::error::AppError;
use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GasLimitEstimate {
    pub gas_limit: u64,
    pub gas_price: String,
    pub estimated_fee: String,
}

#[derive(Debug, Serialize)]
struct GasLimitRequest {
    chain_id: u64,
    from: String,
    to: String,
    amount: String,
    data: Option<String>,
}

// GasLimitResponse 已移除，直接使用 FeesApiResponse（通过统一响应格式）

#[derive(Clone, Copy)]
pub struct GasLimitService {
    app_state: AppState,
}

impl GasLimitService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    fn api(&self) -> ApiClient {
        self.app_state.get_api_client()
    }

    /// 估算Gas Limit（从后端API）
    ///
    /// # Arguments
    /// * `chain_id` - 链ID
    /// * `from` - 发送地址
    /// * `to` - 接收地址
    /// * `amount` - 金额（字符串格式）
    /// * `data` - 交易数据（可选，用于ERC-20等合约调用）
    ///
    /// # Returns
    /// Gas Limit估算值
    pub async fn estimate(
        &self,
        chain_id: u64,
        _from: &str,
        to: &str,
        amount: &str,
        data: Option<&str>,
    ) -> Result<u64, AppError> {
        let api = self.api();

        // 使用GET请求，将参数放在查询字符串中
        let path = format!(
            "/api/v1/fees?chain_id={}&to={}&amount={}",
            chain_id, to, amount
        );

        #[derive(Deserialize)]
        struct FeesApiResponse {
            gas_price: String,
            gas_limit: u64,
            fee: Option<String>,
        }

        match api.get::<FeesApiResponse>(&path).await {
            Ok(response) => Ok(response.gas_limit),
            Err(e) => {
                // 企业级实现：降级策略（API失败时使用保守估算）
                // 多级降级策略：
                // 1. 优先从环境变量读取链特定的默认值
                // 2. 降级：从环境变量读取通用默认值
                // 3. 最终降级：使用安全默认值（仅作为最后保障）
                log::warn!("Gas Limit估算API调用失败: {}，使用保守估算", e);

                // 根据是否有data判断是合约调用还是普通转账
                let default_gas = if data.is_some() {
                    // 企业级实现：从环境变量读取合约调用gas limit（支持动态调整）
                    // 多级降级策略：
                    // 1. 优先从环境变量读取链特定的合约gas limit
                    // 2. 降级：从环境变量读取通用合约gas limit
                    // 3. 最终降级：使用安全默认值（仅作为最后保障）
                    // 注意：前端环境变量访问需要特殊处理（通常在构建时注入）
                    let chain_key = format!("CONTRACT_GAS_LIMIT_CHAIN_{}", chain_id);
                    std::env::var(&chain_key)
                        .ok()
                        .and_then(|v| v.parse::<u64>().ok())
                        .filter(|&v| v > 0 && v <= 10_000_000) // 验证范围：合理值
                        .or_else(|| {
                            std::env::var("CONTRACT_GAS_LIMIT_DEFAULT")
                                .ok()
                                .and_then(|v| v.parse::<u64>().ok())
                                .filter(|&v| v > 0 && v <= 10_000_000)
                        })
                        .unwrap_or_else(|| {
                            log::warn!(
                                "未找到合约gas limit配置 (chain_id={})，使用安全默认值 150000，建议配置环境变量",
                                chain_id
                            );
                            150_000u64  // 安全默认值：合约调用（仅作为最后保障）
                        })
                } else {
                    // 企业级实现：从环境变量读取标准ETH转账gas limit（支持动态调整）
                    // 注意：21000 gas是EIP-1559协议规定的标准ETH转账gas limit，但可以通过环境变量覆盖
                    // 这是以太坊协议标准，所有标准ETH转账通常使用此值
                    let chain_key = format!("STANDARD_TRANSFER_GAS_LIMIT_CHAIN_{}", chain_id);
                    std::env::var(&chain_key)
                        .ok()
                        .and_then(|v| v.parse::<u64>().ok())
                        .filter(|&v| v > 0 && v <= 100_000) // 验证范围：合理值
                        .or_else(|| {
                            std::env::var("STANDARD_ETH_TRANSFER_GAS_LIMIT")
                                .ok()
                                .and_then(|v| v.parse::<u64>().ok())
                                .filter(|&v| v > 0 && v <= 100_000)
                        })
                        .unwrap_or_else(|| {
                            log::debug!(
                                "未找到标准转账gas limit配置 (chain_id={})，使用协议标准值 21000",
                                chain_id
                            );
                            21_000u64 // 标准ETH转账（协议规定）
                        })
                };

                Ok(default_gas)
            }
        }
    }

    /// 获取完整的Gas Limit估算（包含Gas价格和费用）
    pub async fn estimate_full(
        &self,
        chain_id: u64,
        _from: &str,
        to: &str,
        amount: &str,
        _data: Option<&str>, // 保留参数以保持API一致性，但当前API不支持data参数
    ) -> Result<GasLimitEstimate, AppError> {
        let api = self.api();

        let path = format!(
            "/api/v1/fees?chain_id={}&to={}&amount={}",
            chain_id, to, amount
        );

        #[derive(Deserialize)]
        struct FeesApiResponse {
            gas_price: String,
            gas_limit: u64,
            fee: Option<String>,
        }

        let response: FeesApiResponse = api.get(&path).await?;

        Ok(GasLimitEstimate {
            gas_limit: response.gas_limit,
            gas_price: response.gas_price,
            estimated_fee: response.fee.unwrap_or_else(|| "0".to_string()),
        })
    }
}

use serde::{Deserialize, Serialize};
