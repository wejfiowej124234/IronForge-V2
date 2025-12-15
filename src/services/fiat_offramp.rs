//! Fiat Offramp Service - 法币提现服务
//! 企业级法币提现服务，集成第三方服务商API
//! 支持自动两步流程：代币 → 稳定币 → 法币

use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 默认空字符串（用于serde default）
fn default_empty_string() -> String {
    String::new()
}

/// URL编码工具函数（使用JavaScript的encodeURIComponent）
fn encode_uri_component(s: &str) -> String {
    // 使用JavaScript的encodeURIComponent进行URL编码
    let encoded = js_sys::Reflect::get(&js_sys::global(), &"encodeURIComponent".into())
        .ok()
        .and_then(|f| {
            js_sys::Function::from(f)
                .call1(&js_sys::global(), &s.into())
                .ok()
        })
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| s.to_string());
    encoded
}

/// 法币提现报价请求
#[derive(Debug, Clone, Serialize)]
pub struct FiatOfframpQuoteRequest {
    pub token: String,           // 源代币（如 "ETH", "BTC"等）
    pub amount: String,          // 代币数量
    pub chain: String,           // 区块链网络（如 "ethereum", "bitcoin"）
    pub fiat_currency: String,   // 目标法币（如 "USD"）
    pub withdraw_method: String, // 提现方式：bank_card, bank_account, paypal
}

/// 法币提现报价响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatOfframpQuoteResponse {
    pub token_amount: String,                  // 源代币数量
    pub token_symbol: String,                  // 源代币符号
    pub stablecoin_amount: String,             // 中间稳定币数量（USDT/USDC）
    pub stablecoin_symbol: String,             // 中间稳定币符号
    pub fiat_amount: String,                   // 最终法币金额
    pub fiat_currency: String,                 // 法币货币代码
    pub exchange_rate_token_to_stable: String, // 代币→稳定币汇率
    pub exchange_rate_stable_to_fiat: String,  // 稳定币→法币汇率
    pub fee_amount: String,                    // 总手续费
    pub fee_percentage: f64,                   // 手续费百分比
    #[serde(default = "default_empty_string")]
    pub swap_fee: String, // 交换手续费（代币→稳定币）- 后端返回String
    #[serde(default = "default_empty_string")]
    pub withdrawal_fee: String, // 提现手续费（稳定币→法币）- 后端返回String
    pub estimated_arrival: String,             // 预计到账时间
    pub quote_expires_at: String,              // 报价过期时间 - 后端返回String，不是Option
    pub min_amount: Option<String>,            // 可选字段
    pub max_amount: Option<String>,            // 可选字段
    pub quote_id: String,                      // 报价ID - 后端返回
}

/// 创建法币提现订单请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateFiatOfframpOrderRequest {
    pub token: String,                     // 源代币
    pub amount: String,                    // 代币数量
    pub chain: String,                     // 区块链网络
    pub fiat_currency: String,             // 目标法币
    pub withdraw_method: String,           // 提现方式
    pub recipient_info: serde_json::Value, // 收款账户信息（JSON对象，如 {"bank_account": "...", "bank_name": "...", "account_holder": "..."}）
    pub quote_id: String,                  // 报价ID（必需字段，与后端API一致）
}

/// 法币提现订单响应
#[derive(Debug, Clone, Deserialize)]
pub struct FiatOfframpOrderResponse {
    pub order_id: String,
    pub status: String, // pending, processing, completed, failed, cancelled
    pub review_status: Option<String>, // 审核状态：auto_approved, pending_review
    pub token_amount: String,
    pub token_symbol: String,
    pub stablecoin_amount: String,
    pub stablecoin_symbol: String,
    pub fiat_amount: String,
    pub fiat_currency: String,
    pub fee_amount: String,
    pub estimated_arrival: String,    // 后端返回String，不是Option
    pub swap_tx_hash: Option<String>, // 代币→稳定币交换的交易哈希
    pub created_at: String,
    pub expires_at: String, // 后端返回String，不是Option
}

/// 法币提现订单状态
#[derive(Debug, Clone, Deserialize)]
pub struct FiatOfframpOrderStatus {
    pub order_id: String,
    pub status: String,
    pub token_amount: String,
    pub token_symbol: String,
    pub stablecoin_amount: String, // 后端返回String，不是Option
    pub stablecoin_symbol: String, // 后端返回String，不是Option
    pub fiat_amount: String,
    pub fiat_currency: String,
    pub fee_amount: String, // 后端返回String，不是Option
    pub swap_tx_hash: Option<String>,
    pub withdrawal_tx_hash: Option<String>, // 提现交易哈希（如果有）
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
}

/// 法币提现服务
pub struct FiatOfframpService {
    api_client: Arc<ApiClient>,
}

impl FiatOfframpService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        let api_client = app_state.get_api_client();
        
        // 调试：检查API客户端是否有token
        #[cfg(debug_assertions)]
        {
            use tracing::{info, warn};
            if let Some(token) = api_client.get_token() {
                info!("FiatOfframpService: API client has token (length: {})", token.len());
            } else {
                warn!("FiatOfframpService: API client has NO token - requests will fail with 401");
            }
        }
        
        Self {
            api_client: Arc::new(api_client),
        }
    }

    /// 获取法币提现报价
    ///
    /// 系统将自动执行两步流程：
    /// 1. 代币 → 稳定币（USDT/USDC）交换
    /// 2. 稳定币 → 法币提现
    ///
    /// # 参数
    /// - `token`: 源代币符号（如 "ETH", "BTC"）
    /// - `amount`: 代币数量
    /// - `chain`: 区块链网络（如 "ethereum", "bitcoin"）
    /// - `fiat_currency`: 目标法币（如 "USD"）
    /// - `withdraw_method`: 提现方式（bank_card, bank_account, paypal）
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_quote(
        &self,
        token: &str,
        amount: &str,
        chain: &str,
        fiat_currency: &str,
        withdraw_method: &str,
    ) -> Result<FiatOfframpQuoteResponse, String> {
        // 验证输入参数
        if token.is_empty() {
            return Err("请选择提现代币".to_string());
        }

        if amount.is_empty() {
            return Err("请输入提现数量".to_string());
        }

        let amount_val: f64 = amount.parse().map_err(|_| "请输入有效的数量".to_string())?;
        if amount_val <= 0.0 {
            return Err("数量必须大于0".to_string());
        }

        if chain.is_empty() {
            return Err("请选择区块链网络".to_string());
        }

        if fiat_currency.is_empty() {
            return Err("请选择法币货币".to_string());
        }

        if withdraw_method.is_empty() {
            return Err("请选择提现方式".to_string());
        }

        let request = FiatOfframpQuoteRequest {
            token: token.to_string(),
            amount: amount.to_string(),
            chain: chain.to_string(),
            fiat_currency: fiat_currency.to_string(),
            withdraw_method: withdraw_method.to_string(),
        };

        // 构建查询参数
        let query_params = format!(
            "token={}&amount={}&chain={}&fiat_currency={}&withdraw_method={}",
            encode_uri_component(&request.token),
            encode_uri_component(&request.amount),
            encode_uri_component(&request.chain),
            encode_uri_component(&request.fiat_currency),
            encode_uri_component(&request.withdraw_method),
        );

        let url = format!("/api/v1/fiat/offramp/quote?{}", query_params);

        // 发送API请求（带超时）
        self.api_client
            .get::<FiatOfframpQuoteResponse>(&url)
            .await
            .map_err(|e| {
                // 企业级错误处理：将技术错误转换为用户友好的消息
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else if error_msg.contains("forbidden") || error_msg.contains("403") {
                    "权限不足，请联系客服".to_string()
                } else if error_msg.contains("not found") || error_msg.contains("404") {
                    "服务暂时不可用，请稍后重试".to_string()
                } else if error_msg.contains("insufficient") || error_msg.contains("余额不足") {
                    "代币余额不足，请调整提现数量".to_string()
                } else if error_msg.contains("limit") || error_msg.contains("限额") {
                    "交易金额超出限额，请调整金额".to_string()
                } else if error_msg.contains("country") || error_msg.contains("region") {
                    "您所在地区暂不支持此提现方式，请使用其他方式".to_string()
                } else {
                    format!("获取报价失败：{}", e)
                }
            })
    }

    /// 创建法币提现订单
    ///
    /// 系统将自动执行：
    /// 1. 代币 → 稳定币交换
    /// 2. 稳定币 → 法币提现
    ///
    /// # 参数
    /// - `token`: 源代币符号
    /// - `amount`: 代币数量
    /// - `chain`: 区块链网络
    /// - `fiat_currency`: 目标法币
    /// - `withdraw_method`: 提现方式
    /// - `recipient_info`: 收款账户信息
    /// - `quote_id`: 报价ID（可选）
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn create_order(
        &self,
        token: &str,
        amount: &str,
        chain: &str,
        fiat_currency: &str,
        withdraw_method: &str,
        recipient_info: &str,   // 接收JSON字符串，内部转换为serde_json::Value
        quote_id: Option<&str>, // quote_id是可选的，但后端期望必需字段
    ) -> Result<FiatOfframpOrderResponse, String> {
        // 验证输入参数
        if token.is_empty() {
            return Err("请选择提现代币".to_string());
        }

        if amount.is_empty() {
            return Err("请输入提现数量".to_string());
        }

        let amount_val: f64 = amount.parse().map_err(|_| "请输入有效的数量".to_string())?;
        if amount_val <= 0.0 {
            return Err("数量必须大于0".to_string());
        }

        if chain.is_empty() {
            return Err("请选择区块链网络".to_string());
        }

        if fiat_currency.is_empty() {
            return Err("请选择法币货币".to_string());
        }

        if withdraw_method.is_empty() {
            return Err("请选择提现方式".to_string());
        }

        if recipient_info.is_empty() {
            return Err("请输入收款账户信息".to_string());
        }

        // recipient_info已经是JSON字符串，需要解析为JSON对象
        let recipient_info_json: serde_json::Value = match serde_json::from_str(recipient_info) {
            Ok(json) => json,
            Err(_) => {
                // 如果不是JSON字符串，创建一个简单的对象
                serde_json::json!({
                    "account": recipient_info
                })
            }
        };

        // quote_id是必需的，不应该为None
        let quote_id_str = match quote_id {
            Some(id) => id,
            None => {
                return Err("报价ID不能为空，请先获取报价".to_string());
            }
        };

        let request = CreateFiatOfframpOrderRequest {
            token: token.to_string(),
            amount: amount.to_string(),
            chain: chain.to_string(),
            fiat_currency: fiat_currency.to_string(),
            withdraw_method: withdraw_method.to_string(),
            recipient_info: recipient_info_json,
            quote_id: quote_id_str.to_string(),
        };

        let url = "/api/v1/fiat/offramp/orders";

        // 发送API请求
        self.api_client
            .post::<FiatOfframpOrderResponse, CreateFiatOfframpOrderRequest>(&url, &request)
            .await
            .map_err(|e| {
                // 企业级错误处理
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("expired") || error_msg.contains("过期") {
                    "报价已过期，请重新获取报价".to_string()
                } else if error_msg.contains("insufficient") || error_msg.contains("余额不足") {
                    "代币余额不足，请调整提现数量".to_string()
                } else if error_msg.contains("kyc") || error_msg.contains("验证") {
                    "请先完成身份验证".to_string()
                } else if error_msg.contains("limit") || error_msg.contains("限额") {
                    "交易金额超出限额，请调整金额".to_string()
                } else if error_msg.contains("invalid") || error_msg.contains("无效") {
                    "收款账户信息无效，请检查后重试".to_string()
                } else {
                    format!("创建提现订单失败：{}", e)
                }
            })
    }

    /// 查询提现订单状态
    ///
    /// # 参数
    /// - `order_id`: 订单ID
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_order_status(&self, order_id: &str) -> Result<FiatOfframpOrderStatus, String> {
        if order_id.is_empty() {
            return Err("订单ID不能为空".to_string());
        }

        let url = format!("/api/v1/fiat/offramp/orders/{}", encode_uri_component(order_id));

        self.api_client
            .get::<FiatOfframpOrderStatus>(&url)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("not found") || error_msg.contains("404") {
                    "订单不存在".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else {
                    format!("查询订单状态失败：{}", e)
                }
            })
    }

    /// 取消提现订单
    ///
    /// # 参数
    /// - `order_id`: 订单ID
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn cancel_order(&self, order_id: &str) -> Result<(), String> {
        if order_id.is_empty() {
            return Err("订单ID不能为空".to_string());
        }

        let url = format!(
            "/api/v1/fiat/offramp/orders/{}/cancel",
            encode_uri_component(order_id)
        );

        // deserialize 方法已自动提取 data 字段
        // 后端返回: {code: 0, message: "success", data: {}}
        let _: crate::shared::api::EmptyResponse = self
            .api_client
            .post(&url, &serde_json::json!({}))
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("not found") || error_msg.contains("404") {
                    "订单不存在".to_string()
                } else if error_msg.contains("cannot cancel") || error_msg.contains("无法取消")
                {
                    "该订单无法取消".to_string()
                } else {
                    format!("取消订单失败：{}", e)
                }
            })?;

        Ok(())
    }

    /// 重试失败订单
    ///
    /// # 参数
    /// - `order_id`: 订单ID
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn retry_order(&self, order_id: &str) -> Result<FiatOfframpOrderResponse, String> {
        if order_id.is_empty() {
            return Err("订单ID不能为空".to_string());
        }

        let url = format!(
            "/api/v1/fiat/offramp/orders/{}/retry",
            encode_uri_component(order_id)
        );

        self.api_client
            .post::<FiatOfframpOrderResponse, serde_json::Value>(&url, &serde_json::json!({}))
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("not found") || error_msg.contains("404") {
                    "订单不存在".to_string()
                } else if error_msg.contains("cannot retry") || error_msg.contains("无法重试") {
                    "该订单无法重试".to_string()
                } else if error_msg.contains("not failed") || error_msg.contains("不是失败状态")
                {
                    "只有失败订单可以重试".to_string()
                } else {
                    format!("重试订单失败：{}", e)
                }
            })
    }

    /// 获取提现订单列表
    ///
    /// # 参数
    /// - `status`: 订单状态筛选（可选）
    /// - `page`: 页码（可选）
    /// - `page_size`: 每页数量（可选）
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_orders(
        &self,
        status: Option<&str>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<FiatOfframpOrderListResponse, String> {
        let mut query_params = Vec::new();

        if let Some(s) = status {
            query_params.push(format!("status={}", encode_uri_component(s)));
        }
        if let Some(p) = page {
            query_params.push(format!("page={}", p));
        }
        if let Some(ps) = page_size {
            query_params.push(format!("page_size={}", ps));
        }

        let url = if query_params.is_empty() {
            "/api/v1/fiat/offramp/orders".to_string()
        } else {
            format!("/api/v1/fiat/offramp/orders?{}", query_params.join("&"))
        };

        self.api_client
            .get::<FiatOfframpOrderListResponse>(&url)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else if error_msg.contains("forbidden") || error_msg.contains("403") {
                    "权限不足，请联系客服".to_string()
                } else {
                    format!("获取订单列表失败：{}", e)
                }
            })
    }
}

/// 提现订单列表响应
#[derive(Debug, Clone, Deserialize)]
pub struct FiatOfframpOrderListResponse {
    pub orders: Vec<FiatOfframpOrderStatus>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}
