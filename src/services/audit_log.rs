//! Audit Log Service - 审计日志服务
//! 企业级审计日志服务，支持审计日志查询、合规报告生成等

use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: String,
    pub user_id: Option<String>,
    pub action: String,
    pub resource_type: String, // "order", "transaction", "user", etc.
    pub resource_id: String,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub result: AuditLogResult,
}

/// 审计日志结果
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLogResult {
    Success,
    Failure,
    Partial,
}

impl AuditLogResult {
    pub fn label(&self) -> &'static str {
        match self {
            AuditLogResult::Success => "成功",
            AuditLogResult::Failure => "失败",
            AuditLogResult::Partial => "部分成功",
        }
    }
}

/// 审计日志查询请求
#[derive(Debug, Clone, Serialize)]
pub struct AuditLogQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub user_id: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub result: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// 审计日志查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub entries: Vec<AuditLogEntry>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

/// 合规报告请求
#[derive(Debug, Clone, Serialize)]
pub struct ComplianceReportRequest {
    pub report_type: String, // "kyc", "aml", "transaction"
    pub start_date: String,
    pub end_date: String,
    pub format: Option<String>, // "pdf", "csv", "json"
}

/// 合规报告响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReportResponse {
    pub report_id: String,
    pub report_type: String,
    pub start_date: String,
    pub end_date: String,
    pub generated_at: String,
    pub download_url: Option<String>,
    pub status: String, // "pending", "processing", "completed", "failed"
}

/// 审计日志服务
pub struct AuditLogService {
    api_client: Arc<ApiClient>,
}

impl AuditLogService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            api_client: Arc::new(app_state.get_api_client()),
        }
    }

    /// 查询审计日志
    ///
    /// # 参数
    /// - `query`: 查询条件
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn query_logs(&self, query: AuditLogQuery) -> Result<AuditLogResponse, String> {
        let url = "/api/v1/audit/logs";

        self.api_client
            .post::<AuditLogResponse, AuditLogQuery>(url, &query)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else if error_msg.contains("forbidden") || error_msg.contains("403") {
                    "权限不足，只有管理员可以查看审计日志".to_string()
                } else {
                    format!("查询审计日志失败：{}", e)
                }
            })
    }

    /// 生成合规报告
    ///
    /// # 参数
    /// - `request`: 报告请求
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn generate_compliance_report(
        &self,
        request: ComplianceReportRequest,
    ) -> Result<ComplianceReportResponse, String> {
        let url = "/api/v1/audit/compliance/report";

        self.api_client
            .post::<ComplianceReportResponse, ComplianceReportRequest>(url, &request)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else if error_msg.contains("forbidden") || error_msg.contains("403") {
                    "权限不足，只有管理员可以生成合规报告".to_string()
                } else {
                    format!("生成合规报告失败：{}", e)
                }
            })
    }

    /// 获取报告状态
    ///
    /// # 参数
    /// - `report_id`: 报告ID
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_report_status(
        &self,
        report_id: &str,
    ) -> Result<ComplianceReportResponse, String> {
        let url = format!("/api/v1/audit/compliance/report/{}", report_id);

        self.api_client
            .get::<ComplianceReportResponse>(&url)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("not found") || error_msg.contains("404") {
                    "报告不存在".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else {
                    format!("获取报告状态失败：{}", e)
                }
            })
    }
}
