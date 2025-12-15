//! Swap Service - 代币交换服务
//! 集成后端Swap API

use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use web_sys;

/// 默认 f64 值（用于 serde default）
fn default_f64_zero() -> f64 {
    0.0
}

/// Swap报价请求
#[derive(Debug, Clone, Serialize)]
pub struct SwapQuoteRequest {
    pub from: String,
    pub to: String,
    pub amount: String,
    pub network: String,
}

/// Swap报价响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuoteResponse {
    pub from_token: String,
    pub to_token: String,
    pub from_amount: String,
    pub to_amount: String,
    // 兼容不同的后端响应格式：gas_estimate 或 estimated_gas
    #[serde(alias = "gas_estimate", default)]
    pub estimated_gas: Option<String>,
    #[serde(default)]
    pub protocol_fee: Option<String>,
    // 后端可能返回的其他字段（允许缺失）
    #[serde(default)]
    pub exchange_rate: Option<f64>,
    #[serde(default)]
    pub price_impact: Option<f64>,
    #[serde(default)]
    pub route: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub estimated_gas_usd: Option<f64>,
    #[serde(default)]
    pub valid_for: Option<u32>,
}

/// Swap执行请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapExecuteRequest {
    // 后端字段名
    pub wallet_name: String,
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
    pub network: String,
    // 后端期望 f64（不是 Option）
    pub slippage: f64,
    // 后端可选字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<String>,
}

/// Swap执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapExecuteResponse {
    // 后端返回的字段名是 tx_id
    #[serde(rename = "tx_id")]
    pub swap_id: String,
    pub status: String,
    pub from_amount: String,
    pub to_amount: String,
    #[serde(default)]
    pub transaction: Option<SwapTransactionData>,
    #[serde(default)]
    pub message: Option<String>,
    // 后端可能返回的其他字段
    #[serde(default = "default_f64_zero")]
    pub actual_rate: f64,
    /// Gas费用：区块链网络收取的交易执行费用（gas_used * gas_price）
    #[serde(default)]
    pub gas_used: Option<String>,
    /// Gas价格（wei单位，用于计算Gas费用）
    #[serde(default)]
    pub gas_price: Option<String>,
    /// 平台服务费：钱包服务商收取的服务费用（与Gas费用完全独立）
    #[serde(default)]
    pub platform_service_fee: Option<String>,
    /// 服务费收款地址（平台地址）
    #[serde(default)]
    pub service_fee_collector: Option<String>,
    #[serde(default)]
    pub confirmations: u32,
    /// 是否需要先执行approval交易（企业级实现）
    #[serde(default)]
    pub needs_approval: Option<bool>,
    /// 1inch路由器地址（用于前端显示和验证）
    #[serde(default)]
    pub router_address: Option<String>,
}

/// Swap交易数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapTransactionData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub gas: Option<String>,
    pub gas_price: Option<String>,
}

/// Swap服务
pub struct SwapService {
    api_client: Arc<ApiClient>,
}

impl SwapService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            api_client: Arc::new(app_state.get_api_client()),
        }
    }

    /// 获取Swap报价（增强错误处理）
    pub async fn get_quote(
        &self,
        from: &str,
        to: &str,
        amount: &str,
        network: &str,
    ) -> Result<SwapQuoteResponse, String> {
        // 输入验证
        if from.is_empty() || to.is_empty() || amount.is_empty() {
            return Err("参数不能为空".to_string());
        }

        // 金额验证
        if let Ok(amount_f64) = amount.parse::<f64>() {
            if amount_f64 <= 0.0 || amount_f64.is_infinite() || amount_f64.is_nan() {
                return Err("金额必须大于0".to_string());
            }
            if amount_f64 > 1e15 {
                return Err("金额过大，请输入有效金额".to_string());
            }
        } else {
            return Err("金额格式无效".to_string());
        }

        let request = SwapQuoteRequest {
            from: from.to_string(),
            to: to.to_string(),
            amount: amount.to_string(),
            network: network.to_string(),
        };

        // 前端使用GET方法调用（与IronCore API兼容）
        // URL编码工具函数
        fn encode_uri_component(s: &str) -> String {
            js_sys::Reflect::get(&js_sys::global(), &"encodeURIComponent".into())
                .ok()
                .and_then(|f| {
                    js_sys::Function::from(f)
                        .call1(&js_sys::global(), &s.into())
                        .ok()
                })
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| s.to_string())
        }

        // ✅使用标准端点
        let url = format!(
            "/api/v1/swap/quote?from={}&to={}&amount={}&network={}",
            encode_uri_component(&request.from),
            encode_uri_component(&request.to),
            encode_uri_component(&request.amount),
            encode_uri_component(&request.network)
        );

        // 调用API并转换错误消息（增强错误处理）
        match self.api_client.get::<SwapQuoteResponse>(&url).await {
            Ok(response) => Ok(response),
            Err(e) => {
                // 将ApiError转换为友好的错误消息
                let error_msg = match e {
                    crate::shared::error::ApiError::Timeout => {
                        "请求超时，请检查网络连接后重试".to_string()
                    }
                    crate::shared::error::ApiError::Unauthorized => {
                        "认证失败，请重新登录".to_string()
                    }
                    crate::shared::error::ApiError::ResponseError(msg) => {
                        if msg.contains("429") || msg.contains("rate limit") {
                            "请求过于频繁，请稍后再试".to_string()
                        } else if msg.contains("503") || msg.contains("unavailable") {
                            "服务暂时不可用，请稍后再试".to_string()
                        } else {
                            format!("服务错误: {}", msg)
                        }
                    }
                    crate::shared::error::ApiError::RequestFailed(msg) => {
                        if msg.contains("Failed to fetch") || msg.contains("network") {
                            "网络连接失败，请检查网络连接".to_string()
                        } else {
                            format!("请求失败: {}", msg)
                        }
                    }
                };
                Err(error_msg)
            }
        }
    }

    /// 执行Swap
    pub async fn execute(
        &self,
        wallet_id: &str,
        from: &str,
        to: &str,
        amount: &str,
        network: &str,
        slippage: Option<f64>,
    ) -> Result<SwapExecuteResponse, String> {
        // 构建请求，匹配后端期望的字段名
        // 注意：后端期望 wallet_name，但前端传入的是 wallet_id
        // 如果后端支持 wallet_id，我们可以直接使用；否则需要获取 wallet_name
        // 这里暂时使用 wallet_id 作为 wallet_name（后端可能会接受）
        let request = SwapExecuteRequest {
            wallet_name: wallet_id.to_string(), // 后端期望 wallet_name
            from_token: from.to_string(),       // 后端期望 from_token
            to_token: to.to_string(),           // 后端期望 to_token
            amount: amount.to_string(),
            network: network.to_string(),
            // 企业级实现：滑点配置
            // 注意：在WASM环境中无法访问环境变量，因此使用安全默认值
            // 生产环境应该通过后端API获取配置，或使用localStorage存储用户偏好
            // 多级降级策略：
            // 1. 优先使用用户提供的滑点值
            // 2. 降级：从localStorage读取用户保存的滑点偏好
            // 3. 最终降级：使用安全默认值（0.5%）
            slippage: slippage.unwrap_or_else(|| {
                // 尝试从localStorage读取用户保存的滑点偏好
                let stored_slippage = if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        if let Ok(Some(value)) = storage.get_item("swap_default_slippage") {
                            value
                                .parse::<f64>()
                                .ok()
                                .filter(|&v| v > 0.0 && v <= 100.0 && v.is_finite())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                stored_slippage.unwrap_or_else(|| {
                    log::info!(
                        "使用默认滑点 0.5% (network={})，用户可在设置中自定义",
                        network
                    );
                    0.5 // 安全默认值：0.5%
                })
            }),
            password: None,
            client_request_id: None,
        };

        self.api_client
            .post::<SwapExecuteResponse, SwapExecuteRequest>("/api/v1/swap/execute", &request)
            .await
            .map_err(|e| format!("Failed to execute swap: {}", e))
    }

    /// 获取Swap交易状态（企业级实现）
    pub async fn get_status(&self, swap_id: &str) -> Result<SwapStatusResponse, String> {
        let url = format!("/api/v1/swap/{}", swap_id);

        self.api_client
            .get::<SwapStatusResponse>(&url)
            .await
            .map_err(|e| format!("Failed to get swap status: {}", e))
    }

    /// 更新Swap交易状态（企业级实现）
    pub async fn update_status(
        &self,
        swap_id: &str,
        tx_hash: Option<&str>,
        status: &str,
        gas_used: Option<&str>,
        confirmations: Option<u32>,
    ) -> Result<(), String> {
        let request = serde_json::json!({
            "tx_hash": tx_hash,
            "status": status,
            "gas_used": gas_used,
            "confirmations": confirmations,
        });

        let url = format!("/api/v1/swap/{}/status", swap_id);

        // deserialize 方法已自动提取 data 字段
        // 后端返回: {code: 0, message: "success", data: {}}
        let _: crate::shared::api::EmptyResponse = self
            .api_client
            .put(&url, &request)
            .await
            .map_err(|e| format!("Failed to update swap status: {}", e))?;

        Ok(())
    }
}

/// Swap状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapStatusResponse {
    pub swap_id: String,
    pub status: String,
    pub from_token: String,
    pub to_token: String,
    pub from_amount: String,
    pub to_amount: Option<String>,
    pub network: String,
    pub tx_hash: Option<String>,
    pub gas_used: Option<String>,
    pub confirmations: u32,
    pub created_at: String,
    pub updated_at: String,
}
