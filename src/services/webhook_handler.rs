//! Webhook Handler Service - Webhook回调处理服务
//! 企业级Webhook处理服务，支持订单状态更新、签名验证等

use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Webhook事件类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebhookEventType {
    OrderStatusUpdated,
    PaymentCompleted,
    PaymentFailed,
    KycStatusUpdated,
    Unknown(String),
}

impl WebhookEventType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "order.status.updated" => WebhookEventType::OrderStatusUpdated,
            "payment.completed" => WebhookEventType::PaymentCompleted,
            "payment.failed" => WebhookEventType::PaymentFailed,
            "kyc.status.updated" => WebhookEventType::KycStatusUpdated,
            _ => WebhookEventType::Unknown(s.to_string()),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            WebhookEventType::OrderStatusUpdated => "订单状态更新",
            WebhookEventType::PaymentCompleted => "支付完成",
            WebhookEventType::PaymentFailed => "支付失败",
            WebhookEventType::KycStatusUpdated => "KYC状态更新",
            WebhookEventType::Unknown(s) => s,
        }
    }
}

/// Webhook事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub event_type: String,
    pub provider: String,
    pub order_id: Option<String>,
    pub data: serde_json::Value,
    pub timestamp: String,
    pub signature: Option<String>,
}

/// Webhook处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookProcessResult {
    pub event_id: String,
    pub message: String,
    pub processed_at: String,
}

/// Webhook处理服务
pub struct WebhookHandlerService {
    api_client: Arc<ApiClient>,
}

impl WebhookHandlerService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            api_client: Arc::new(app_state.get_api_client()),
        }
    }

    /// 处理Webhook事件
    ///
    /// # 参数
    /// - `provider`: 服务商名称
    /// - `event`: Webhook事件
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn process_webhook(
        &self,
        provider: &str,
        event: WebhookEvent,
    ) -> Result<WebhookProcessResult, String> {
        let url = format!("/api/v1/webhook/{}", provider);

        self.api_client
            .post::<WebhookProcessResult, WebhookEvent>(&url, &event)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("invalid signature") || error_msg.contains("签名") {
                    "Webhook签名验证失败".to_string()
                } else if error_msg.contains("duplicate") || error_msg.contains("重复") {
                    "该Webhook事件已处理过".to_string()
                } else if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else {
                    format!("处理Webhook事件失败：{}", e)
                }
            })
    }

    /// 验证Webhook签名
    ///
    /// # 参数
    /// - `provider`: 服务商名称
    /// - `payload`: Webhook负载
    /// - `signature`: 签名
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn verify_signature(
        &self,
        provider: &str,
        payload: &str,
        signature: &str,
    ) -> Result<bool, String> {
        let url = format!("/api/v1/webhook/{}/verify", provider);
        let request = serde_json::json!({
            "payload": payload,
            "signature": signature,
        });

        // deserialize 方法已自动提取 data 字段
        // 后端返回: {code: 0, message: "success", data: {valid: true/false}}
        #[derive(Debug, Deserialize)]
        struct WebhookVerifyResponse {
            valid: bool,
        }

        self.api_client
            .post::<WebhookVerifyResponse, _>(&url, &request)
            .await
            .map(|response| response.valid)
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else {
                    format!("验证签名失败：{}", e)
                }
            })
    }
}
