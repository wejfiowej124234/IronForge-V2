//! Limit Order Service - 限价单服务
//! 企业级限价单管理，集成后端API

use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 限价单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LimitOrderType {
    /// 买入限价单
    Buy,
    /// 卖出限价单
    Sell,
}

/// 限价单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LimitOrderStatus {
    /// 待执行
    Pending,
    /// 部分执行
    PartiallyFilled,
    /// 已完成
    Filled,
    /// 已取消
    Cancelled,
    /// 已过期
    Expired,
    /// 失败
    Failed,
}

/// 创建限价单请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateLimitOrderRequest {
    /// 订单类型：buy 或 sell
    pub order_type: String,
    /// 支付代币符号
    pub from_token: String,
    /// 接收代币符号
    pub to_token: String,
    /// 数量
    pub amount: String,
    /// 限价
    pub limit_price: String,
    /// 链名称
    pub network: String,
    /// 过期天数
    pub expiry_days: u32,
    /// 钱包ID（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_id: Option<String>,
}

/// 限价单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitOrderResponse {
    /// 订单ID
    pub order_id: String,
    /// 订单类型
    pub order_type: String,
    /// 支付代币
    pub from_token: String,
    /// 接收代币
    pub to_token: String,
    /// 数量
    pub amount: String,
    /// 限价
    pub limit_price: String,
    /// 状态
    pub status: String,
    /// 已执行数量
    #[serde(default)]
    pub filled_amount: Option<String>,
    /// 创建时间
    pub created_at: String,
    /// 过期时间
    #[serde(default)]
    pub expires_at: Option<String>,
    /// 消息（可选）
    #[serde(default)]
    pub message: Option<String>,
}

/// 限价单列表查询请求
#[derive(Debug, Clone, Serialize)]
pub struct LimitOrderQuery {
    /// 订单类型筛选（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    /// 状态筛选（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// 页码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// 每页数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
}

/// 限价单列表响应
#[derive(Debug, Clone, Deserialize)]
pub struct LimitOrderListResponse {
    /// 订单列表
    pub orders: Vec<LimitOrderResponse>,
    /// 总页数
    pub total_pages: u32,
    /// 当前页
    pub current_page: u32,
    /// 总数量
    pub total_count: u32,
}

/// 限价单服务
pub struct LimitOrderService {
    app_state: AppState,
}

impl LimitOrderService {
    /// 创建新的限价单服务实例
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// 获取最新的 API 客户端（包含最新的 token）
    fn get_api_client(&self) -> ApiClient {
        self.app_state.get_api_client()
    }

    /// 创建限价单
    #[allow(clippy::too_many_arguments)]
    pub async fn create_order(
        &self,
        order_type: LimitOrderType,
        from_token: &str,
        to_token: &str,
        amount: &str,
        limit_price: &str,
        network: &str,
        expiry_days: u32,
        wallet_id: Option<&str>,
    ) -> Result<LimitOrderResponse, String> {
        // 验证输入
        if amount.is_empty() || amount.parse::<f64>().unwrap_or(0.0) <= 0.0 {
            return Err("数量必须大于0".to_string());
        }

        if limit_price.is_empty() || limit_price.parse::<f64>().unwrap_or(0.0) <= 0.0 {
            return Err("限价必须大于0".to_string());
        }

        if from_token.is_empty() || to_token.is_empty() {
            return Err("请选择代币".to_string());
        }

        if expiry_days == 0 || expiry_days > 365 {
            return Err("过期天数必须在1-365之间".to_string());
        }

        let order_type_str = match order_type {
            LimitOrderType::Buy => "buy",
            LimitOrderType::Sell => "sell",
        };

        let request = CreateLimitOrderRequest {
            order_type: order_type_str.to_string(),
            from_token: from_token.to_string(),
            to_token: to_token.to_string(),
            amount: amount.to_string(),
            limit_price: limit_price.to_string(),
            network: network.to_string(),
            expiry_days,
            wallet_id: wallet_id.map(|s| s.to_string()),
        };

        match self
            .get_api_client()
            .post::<LimitOrderResponse, CreateLimitOrderRequest>("/api/v1/limit-orders", &request)
            .await
        {
            Ok(resp) => Ok(resp),
            Err(e) => {
                // ✅ 统一处理401错误：仅在用户已登录且token过期时自动登出
                if crate::shared::auth_handler::is_unauthorized_error(&e) {
                    crate::shared::auth_handler::handle_unauthorized_and_redirect(self.app_state);
                    // 注意：如果用户本来就没登录，上面的函数不会做任何事
                }

                // 企业级错误处理：将技术错误转换为用户友好消息
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    return Err("请先登录账户".to_string());
                }
                if error_msg.contains("insufficient") || error_msg.contains("balance") {
                    Err("余额不足，无法创建限价单".to_string())
                } else if error_msg.contains("network") || error_msg.contains("connection") {
                    Err("网络错误，请稍后重试".to_string())
                } else if error_msg.contains("timeout") {
                    Err("请求超时，请稍后重试".to_string())
                } else if error_msg.contains("invalid") || error_msg.contains("validation") {
                    Err("输入参数无效，请检查后重试".to_string())
                } else {
                    Err(format!("创建限价单失败：{}", e))
                }
            }
        }
    }

    /// 获取限价单列表
    pub async fn get_orders(
        &self,
        query: Option<LimitOrderQuery>,
    ) -> Result<LimitOrderListResponse, String> {
        let query = query.unwrap_or(LimitOrderQuery {
            order_type: None,
            status: None,
            page: Some(1),
            page_size: Some(20),
        });

        // 构建查询字符串
        let mut query_params = Vec::new();
        if let Some(order_type) = &query.order_type {
            query_params.push(format!("order_type={}", order_type));
        }
        if let Some(status) = &query.status {
            query_params.push(format!("status={}", status));
        }
        if let Some(page) = query.page {
            query_params.push(format!("page={}", page));
        }
        if let Some(page_size) = query.page_size {
            query_params.push(format!("page_size={}", page_size));
        }

        // 确保URL格式正确，去除末尾的&符号
        let url = if query_params.is_empty() {
            "/api/v1/limit-orders".to_string()
        } else {
            let query_string = query_params.join("&");
            format!(
                "/api/v1/limit-orders?{}",
                query_string.trim_end_matches('&')
            )
        };

        match self
            .get_api_client()
            .get::<LimitOrderListResponse>(&url)
            .await
        {
            Ok(resp) => Ok(resp),
            Err(e) => {
                // ✅ 统一处理401错误：仅在用户已登录且token过期时自动登出
                if crate::shared::auth_handler::is_unauthorized_error(&e) {
                    crate::shared::auth_handler::handle_unauthorized_and_redirect(self.app_state);
                    // 注意：如果用户本来就没登录，上面的函数不会做任何事
                }

                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    return Err("请先登录账户以查看限价单".to_string());
                }
                if error_msg.contains("network") || error_msg.contains("connection") {
                    Err("网络错误，请稍后重试".to_string())
                } else if error_msg.contains("timeout") {
                    Err("请求超时，请稍后重试".to_string())
                } else {
                    Err(format!("获取限价单列表失败：{}", e))
                }
            }
        }
    }

    /// 获取单个限价单详情
    pub async fn get_order(&self, order_id: &str) -> Result<LimitOrderResponse, String> {
        if order_id.is_empty() {
            return Err("订单ID不能为空".to_string());
        }

        let url = format!("/api/v1/limit-orders/{}", order_id);

        self.get_api_client()
            .get::<LimitOrderResponse>(&url)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("not found") || error_msg.contains("404") {
                    "限价单不存在".to_string()
                } else if error_msg.contains("network") || error_msg.contains("connection") {
                    "网络错误，请稍后重试".to_string()
                } else {
                    format!("获取限价单详情失败：{}", e)
                }
            })
    }

    /// 取消限价单
    pub async fn cancel_order(&self, order_id: &str) -> Result<LimitOrderResponse, String> {
        if order_id.is_empty() {
            return Err("订单ID不能为空".to_string());
        }

        let url = format!("/api/v1/limit-orders/{}/cancel", order_id);

        self.get_api_client()
            .post::<LimitOrderResponse, serde_json::Value>(&url, &serde_json::json!({}))
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("not found") || error_msg.contains("404") {
                    "限价单不存在".to_string()
                } else if error_msg.contains("cannot cancel") || error_msg.contains("status") {
                    "该限价单无法取消（可能已执行或已过期）".to_string()
                } else if error_msg.contains("network") || error_msg.contains("connection") {
                    "网络错误，请稍后重试".to_string()
                } else {
                    format!("取消限价单失败：{}", e)
                }
            })
    }
}
