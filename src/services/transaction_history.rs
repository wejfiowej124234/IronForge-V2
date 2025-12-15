//! Transaction History Service - 交易历史服务
//! 企业级交易历史服务，支持交换、充值、提现历史查询

use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};

/// URL编码工具函数（使用JavaScript的encodeURIComponent）
fn encode_uri_component(s: &str) -> String {
    // 使用JavaScript的encodeURIComponent进行URL编码
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

/// 交易类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Swap,    // 代币交换
    Onramp,  // 法币充值
    Offramp, // 法币提现
}

impl TransactionType {
    pub fn label(&self) -> &'static str {
        match self {
            TransactionType::Swap => "交换",
            TransactionType::Onramp => "充值",
            TransactionType::Offramp => "提现",
        }
    }
}

/// 交易状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,    // 待处理
    Processing, // 处理中
    Completed,  // 已完成
    Failed,     // 失败
    Cancelled,  // 已取消
}

impl TransactionStatus {
    pub fn label(&self) -> &'static str {
        match self {
            TransactionStatus::Pending => "待处理",
            TransactionStatus::Processing => "处理中",
            TransactionStatus::Completed => "已完成",
            TransactionStatus::Failed => "失败",
            TransactionStatus::Cancelled => "已取消",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            TransactionStatus::Pending => "#F59E0B",    // 警告橙
            TransactionStatus::Processing => "#3B82F6", // 蓝色
            TransactionStatus::Completed => "#10B981",  // 成功绿
            TransactionStatus::Failed => "#EF4444",     // 错误红
            TransactionStatus::Cancelled => "#6B7280",  // 灰色
        }
    }
}

/// 交易历史项
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionHistoryItem {
    pub id: String,
    pub tx_type: String, // "swap", "onramp", "offramp"
    pub status: String,  // "pending", "processing", "completed", "failed", "cancelled"
    pub from_token: String,
    pub to_token: String,
    pub from_amount: String,
    pub to_amount: String,
    pub fee_amount: Option<String>,
    pub gas_fee: Option<String>,
    pub tx_hash: Option<String>, // 区块链交易哈希
    pub created_at: String,      // ISO 8601格式
    pub completed_at: Option<String>,
    pub fiat_order_id: Option<String>, // 法币订单ID（如果是充值/提现）
    pub metadata: Option<serde_json::Value>, // 额外信息
}

/// 交易历史查询参数
#[derive(Debug, Clone, Serialize)]
pub struct TransactionHistoryQuery {
    pub tx_type: Option<String>,    // 交易类型筛选
    pub status: Option<String>,     // 状态筛选
    pub page: Option<u32>,          // 页码（从1开始）
    pub page_size: Option<u32>,     // 每页数量
    pub start_date: Option<String>, // 开始日期（ISO 8601）
    pub end_date: Option<String>,   // 结束日期（ISO 8601）
}

/// 交易历史响应
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionHistoryResponse {
    pub transactions: Vec<TransactionHistoryItem>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

/// 交易历史服务
pub struct TransactionHistoryService {
    app_state: AppState,
}

impl TransactionHistoryService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// 获取最新的 API 客户端（包含最新的 token）
    fn get_api_client(&self) -> ApiClient {
        self.app_state.get_api_client()
    }

    /// 获取交易历史列表
    ///
    /// # 参数
    /// - `query`: 查询参数
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_history(
        &self,
        query: Option<TransactionHistoryQuery>,
    ) -> Result<TransactionHistoryResponse, String> {
        let mut url = "/api/v1/swap/history".to_string();

        // 构建查询参数
        if let Some(q) = query {
            let mut params = Vec::new();

            if let Some(tx_type) = q.tx_type {
                params.push(format!("tx_type={}", encode_uri_component(&tx_type)));
            }

            if let Some(status) = q.status {
                params.push(format!("status={}", encode_uri_component(&status)));
            }

            if let Some(page) = q.page {
                params.push(format!("page={}", page));
            }

            if let Some(page_size) = q.page_size {
                params.push(format!("page_size={}", page_size));
            }

            if let Some(start_date) = q.start_date {
                params.push(format!("start_date={}", encode_uri_component(&start_date)));
            }

            if let Some(end_date) = q.end_date {
                params.push(format!("end_date={}", encode_uri_component(&end_date)));
            }

            if !params.is_empty() {
                url = format!("{}?{}", url, params.join("&"));
            }
        } else {
            // 如果没有query参数，但调用时传入了Some(query)，使用默认分页
            url = format!("{}?page=1&page_size=10", url);
        }

        // 发送API请求
        match self
            .get_api_client()
            .get::<TransactionHistoryResponse>(&url)
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
                    Err("请先登录账户".to_string())
                } else if error_msg.contains("timeout") || error_msg.contains("network") {
                    Err("网络连接超时，请稍后重试".to_string())
                } else {
                    Err(format!("获取交易历史失败：{}", e))
                }
            }
        }
    }

    /// 获取单笔交易详情
    ///
    /// # 参数
    /// - `transaction_id`: 交易ID
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_transaction_detail(
        &self,
        transaction_id: &str,
    ) -> Result<TransactionHistoryItem, String> {
        if transaction_id.is_empty() {
            return Err("交易ID不能为空".to_string());
        }

        let url = format!(
            "/api/v1/swap/history/{}",
            encode_uri_component(transaction_id)
        );

        match self
            .get_api_client()
            .get::<TransactionHistoryItem>(&url)
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
                    Err("请先登录账户".to_string())
                } else if error_msg.contains("not found") || error_msg.contains("404") {
                    Err("交易记录不存在".to_string())
                } else {
                    Err(format!("获取交易详情失败：{}", e))
                }
            }
        }
    }
}
