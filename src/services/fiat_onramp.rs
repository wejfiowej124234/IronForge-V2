//! Fiat Onramp Service - 法币充值服务
//! 企业级法币充值服务，集成第三方服务商API

use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

/// 法币报价请求
#[derive(Debug, Clone, Serialize)]
pub struct FiatQuoteRequest {
    pub amount: String,
    pub currency: String,       // 法币货币代码，如 "USD"
    pub token: String,          // 目标稳定币，如 "USDT" 或 "USDC"
    pub payment_method: String, // 支付方式：credit_card, bank_transfer, paypal
}

/// 法币报价响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatQuoteResponse {
    pub fiat_amount: String,
    pub crypto_amount: String,
    pub exchange_rate: String,
    pub fee_amount: String,
    pub fee_percentage: f64,
    pub estimated_arrival: String, // 预计到账时间
    pub quote_expires_at: String,  // 报价过期时间（ISO 8601格式）- 后端返回String
    pub min_amount: String,        // 后端返回String
    pub max_amount: String,        // 后端返回String
    pub quote_id: String,          // 报价ID - 后端返回
}

/// 创建法币订单请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateFiatOrderRequest {
    pub amount: String,
    pub currency: String,
    pub token: String,
    pub payment_method: String,
    pub quote_id: String,               // 报价ID（必需字段，与后端API一致）
    pub wallet_address: Option<String>, // 接收稳定币的钱包地址
}

/// 法币订单响应
#[derive(Debug, Clone, Deserialize)]
pub struct FiatOrderResponse {
    pub order_id: String,
    pub status: String, // pending, processing, completed, failed, cancelled
    pub payment_url: Option<String>, // 支付链接（如果状态为pending）
    pub fiat_amount: String,
    pub crypto_amount: String,
    pub exchange_rate: String,
    pub fee_amount: String,
    pub estimated_arrival: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

/// 法币订单状态
#[derive(Debug, Clone, Deserialize)]
pub struct FiatOrderStatus {
    pub order_id: String,
    pub status: String,
    pub fiat_amount: String,
    pub crypto_amount: String,
    pub exchange_rate: Option<String>,
    pub fee_amount: Option<String>,
    pub payment_url: Option<String>,
    pub tx_hash: Option<String>, // 区块链交易哈希（如果已完成）
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
}

/// 法币充值服务
pub struct FiatOnrampService {
    api_client: Arc<ApiClient>,
}

impl FiatOnrampService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        let api_client = app_state.get_api_client();
        
        // 调试：检查API客户端是否有token（强制输出到console）
        use tracing::{info, warn};
        use wasm_bindgen::prelude::*;
        
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = console)]
            fn log(s: &str);
        }
        
        if let Some(token) = api_client.get_token() {
            let msg = format!("✅ FiatOnrampService: API client has token (length: {})", token.len());
            info!("{}", msg);
            log(&msg);
        } else {
            let msg = "❌ FiatOnrampService: API client has NO token - requests will fail with 401";
            warn!("{}", msg);
            log(msg);
        }
        
        Self {
            api_client: Arc::new(api_client),
        }
    }

    /// 获取法币购买报价
    ///
    /// # 参数
    /// - `amount`: 法币金额
    /// - `currency`: 法币货币代码（如 "USD"）
    /// - `token`: 目标稳定币（如 "USDT" 或 "USDC"）
    /// - `payment_method`: 支付方式（credit_card, bank_transfer, paypal）
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息，所有错误都通过Result类型处理
    pub async fn get_quote(
        &self,
        amount: &str,
        currency: &str,
        token: &str,
        payment_method: &str,
    ) -> Result<FiatQuoteResponse, String> {
        // 验证输入参数
        if amount.is_empty() {
            return Err("请输入购买金额".to_string());
        }

        let amount_val: f64 = amount.parse().map_err(|_| "请输入有效的金额".to_string())?;
        if amount_val <= 0.0 {
            return Err("金额必须大于0".to_string());
        }

        if currency.is_empty() {
            return Err("请选择法币货币".to_string());
        }

        if token.is_empty() {
            return Err("请选择稳定币类型".to_string());
        }

        if payment_method.is_empty() {
            return Err("请选择支付方式".to_string());
        }

        let request = FiatQuoteRequest {
            amount: amount.to_string(),
            currency: currency.to_string(),
            token: token.to_string(),
            payment_method: payment_method.to_string(),
        };

        // 构建查询参数
        let query_params = format!(
            "amount={}&currency={}&token={}&payment_method={}",
            encode_uri_component(&request.amount),
            encode_uri_component(&request.currency),
            encode_uri_component(&request.token),
            encode_uri_component(&request.payment_method),
        );

        let url = format!("/api/v1/fiat/onramp/quote?{}", query_params);

        // 发送API请求（带超时）
        self.api_client
            .get::<FiatQuoteResponse>(&url)
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
                } else if error_msg.contains("service unavailable") || error_msg.contains("503") {
                    // 后端返回503表示服务暂时不可用（例如：无可用支付服务商）
                    "法币充值服务暂时不可用，请稍后重试".to_string()
                } else if error_msg.contains("bad gateway") || error_msg.contains("502") {
                    "支付服务暂时不可用，请稍后重试".to_string()
                } else if error_msg.contains("not found") || error_msg.contains("404") {
                    "服务暂时不可用，请稍后重试".to_string()
                } else if error_msg.contains("limit") || error_msg.contains("限额") {
                    "交易金额超出限额，请调整金额".to_string()
                } else if error_msg.contains("country") || error_msg.contains("region") || error_msg.contains("地区") {
                    "您所在地区暂不支持此服务".to_string()
                } else if error_msg.contains("没有可用") || error_msg.contains("暂时不可用") {
                    "法币充值服务暂时不可用，请稍后重试或联系客服".to_string()
                } else {
                    format!("获取报价失败：{}", e)
                }
            })
    }

    /// 创建法币购买订单
    ///
    /// # 参数
    /// - `amount`: 法币金额
    /// - `currency`: 法币货币代码
    /// - `token`: 目标稳定币
    /// - `payment_method`: 支付方式
    /// - `wallet_address`: 接收稳定币的钱包地址（可选）
    /// - `quote_id`: 报价ID（可选）
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn create_order(
        &self,
        amount: &str,
        currency: &str,
        token: &str,
        payment_method: &str,
        quote_id: &str, // 必需字段
        wallet_address: Option<&str>,
    ) -> Result<FiatOrderResponse, String> {
        // 验证输入参数
        if amount.is_empty() {
            return Err("请输入购买金额".to_string());
        }

        let amount_val: f64 = amount.parse().map_err(|_| "请输入有效的金额".to_string())?;
        if amount_val <= 0.0 {
            return Err("金额必须大于0".to_string());
        }

        // 检查最小金额（通常为$10）
        if amount_val < 10.0 {
            return Err("最小购买金额为 $10".to_string());
        }

        if quote_id.is_empty() {
            return Err("报价ID不能为空，请先获取报价".to_string());
        }

        let request = CreateFiatOrderRequest {
            amount: amount.to_string(),
            currency: currency.to_string(),
            token: token.to_string(),
            payment_method: payment_method.to_string(),
            quote_id: quote_id.to_string(),
            wallet_address: wallet_address.map(|s| s.to_string()),
        };

        let url = "/api/v1/fiat/onramp/orders";

        // 发送API请求
        self.api_client
            .post::<FiatOrderResponse, CreateFiatOrderRequest>(&url, &request)
            .await
            .map_err(|e| {
                // 企业级错误处理
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("expired") || error_msg.contains("过期") {
                    "报价已过期，请重新获取报价".to_string()
                } else if error_msg.contains("insufficient") || error_msg.contains("余额不足") {
                    "账户余额不足，请选择其他支付方式".to_string()
                } else if error_msg.contains("kyc") || error_msg.contains("验证") {
                    "请先完成身份验证".to_string()
                } else if error_msg.contains("limit") || error_msg.contains("限额") {
                    "交易金额超出限额，请调整金额".to_string()
                } else {
                    format!("创建订单失败：{}", e)
                }
            })
    }

    /// 查询订单状态
    ///
    /// # 参数
    /// - `order_id`: 订单ID
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_order_status(&self, order_id: &str) -> Result<FiatOrderStatus, String> {
        if order_id.is_empty() {
            return Err("订单ID不能为空".to_string());
        }

        let url = format!("/api/v1/fiat/onramp/orders/{}", encode_uri_component(order_id));

        self.api_client
            .get::<FiatOrderStatus>(&url)
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

    /// 取消订单
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

        let url = format!("/api/v1/fiat/onramp/orders/{}/cancel", encode_uri_component(order_id));

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
    pub async fn retry_order(&self, order_id: &str) -> Result<FiatOrderResponse, String> {
        if order_id.is_empty() {
            return Err("订单ID不能为空".to_string());
        }

        let url = format!("/api/v1/fiat/onramp/orders/{}/retry", encode_uri_component(order_id));

        self.api_client
            .post::<FiatOrderResponse, serde_json::Value>(&url, &serde_json::json!({}))
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

    /// 获取订单列表
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
    ) -> Result<FiatOrderListResponse, String> {
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
            "/api/v1/fiat/onramp/orders".to_string()
        } else {
            format!("/api/v1/fiat/onramp/orders?{}", query_params.join("&"))
        };

        self.api_client
            .get::<FiatOrderListResponse>(&url)
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

/// 订单列表响应
#[derive(Debug, Clone, Deserialize)]
pub struct FiatOrderListResponse {
    pub orders: Vec<FiatOrderStatus>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}
