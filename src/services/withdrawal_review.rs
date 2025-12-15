//! Withdrawal Review Service - 提现审核服务
//! 企业级提现审核服务，支持自动/人工审核

use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 提现审核请求
#[derive(Debug, Clone, Serialize)]
pub struct WithdrawalReviewRequest {
    pub order_id: String,
    pub action: ReviewAction,
    pub reason: Option<String>,
}

/// 审核操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewAction {
    Approve, // 批准
    Reject,  // 拒绝
    Pending, // 待审核
}

impl ReviewAction {
    pub fn label(&self) -> &'static str {
        match self {
            ReviewAction::Approve => "批准",
            ReviewAction::Reject => "拒绝",
            ReviewAction::Pending => "待审核",
        }
    }
}

/// 提现审核结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalReviewResult {
    pub order_id: String,
    pub review_status: ReviewStatus,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<String>,
    pub reason: Option<String>,
    pub auto_reviewed: bool,
}

/// 审核状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    Pending,      // 待审核
    Approved,     // 已批准
    Rejected,     // 已拒绝
    AutoApproved, // 自动批准
}

impl ReviewStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ReviewStatus::Pending => "待审核",
            ReviewStatus::Approved => "已批准",
            ReviewStatus::Rejected => "已拒绝",
            ReviewStatus::AutoApproved => "自动批准",
        }
    }
}

/// 提现审核服务
pub struct WithdrawalReviewService {
    api_client: Arc<ApiClient>,
}

impl WithdrawalReviewService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            api_client: Arc::new(app_state.get_api_client()),
        }
    }

    /// 获取订单审核状态
    ///
    /// # 参数
    /// - `order_id`: 订单ID
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_review_status(
        &self,
        order_id: &str,
    ) -> Result<WithdrawalReviewResult, String> {
        if order_id.is_empty() {
            return Err("订单ID不能为空".to_string());
        }

        let url = format!("/api/v1/withdrawal/review/{}", order_id);

        self.api_client
            .get::<WithdrawalReviewResult>(&url)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("not found") || error_msg.contains("404") {
                    "订单不存在".to_string()
                } else if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else {
                    format!("获取审核状态失败：{}", e)
                }
            })
    }

    /// 提交审核
    ///
    /// # 参数
    /// - `request`: 审核请求
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn submit_review(
        &self,
        request: WithdrawalReviewRequest,
    ) -> Result<WithdrawalReviewResult, String> {
        if request.order_id.is_empty() {
            return Err("订单ID不能为空".to_string());
        }

        let url = "/api/v1/withdrawal/review";

        self.api_client
            .post::<WithdrawalReviewResult, WithdrawalReviewRequest>(url, &request)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("not found") || error_msg.contains("404") {
                    "订单不存在".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else if error_msg.contains("forbidden") || error_msg.contains("403") {
                    "权限不足，只有审核人员可以提交审核".to_string()
                } else if error_msg.contains("already reviewed") || error_msg.contains("已审核")
                {
                    "该订单已经审核过".to_string()
                } else {
                    format!("提交审核失败：{}", e)
                }
            })
    }

    /// 获取待审核订单列表
    ///
    /// # 参数
    /// - `page`: 页码（从1开始）
    /// - `limit`: 每页数量
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_pending_reviews(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> Result<PendingReviewResponse, String> {
        let mut url = "/api/v1/withdrawal/review/pending".to_string();
        let mut params = Vec::new();

        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }

        if !params.is_empty() {
            url.push_str(&format!("?{}", params.join("&")));
        }

        self.api_client
            .get::<PendingReviewResponse>(&url)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else if error_msg.contains("forbidden") || error_msg.contains("403") {
                    "权限不足，只有审核人员可以查看待审核订单".to_string()
                } else {
                    format!("获取待审核订单列表失败：{}", e)
                }
            })
    }
}

/// 待审核订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingReviewResponse {
    pub orders: Vec<PendingReviewOrder>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

/// 待审核订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingReviewOrder {
    pub order_id: String,
    pub user_id: String,
    pub amount: String,
    pub currency: String,
    pub withdraw_method: String,
    pub created_at: String,
    pub requires_manual_review: bool,
    pub review_reason: Option<String>,
}
