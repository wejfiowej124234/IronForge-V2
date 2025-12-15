//! Reconciliation Service - 对账服务
//! 企业级对账服务，支持每日对账、订单状态同步、异常监控等

use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 对账结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationResult {
    pub date: String, // YYYY-MM-DD
    pub total_orders: u64,
    pub matched_orders: u64,
    pub unmatched_orders: u64,
    pub total_amount: String,
    pub matched_amount: String,
    pub unmatched_amount: String,
    pub discrepancies: Vec<Discrepancy>,
    pub status: ReconciliationStatus,
    pub completed_at: Option<String>,
}

/// 对账状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconciliationStatus {
    Pending,    // 待处理
    Processing, // 处理中
    Completed,  // 已完成
    Failed,     // 失败
}

impl ReconciliationStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ReconciliationStatus::Pending => "待处理",
            ReconciliationStatus::Processing => "处理中",
            ReconciliationStatus::Completed => "已完成",
            ReconciliationStatus::Failed => "失败",
        }
    }
}

/// 对账差异
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    pub order_id: String,
    pub discrepancy_type: String, // "amount", "status", "missing"
    pub expected_value: String,
    pub actual_value: String,
    pub severity: String, // "warning", "error", "critical"
}

/// 对账服务
pub struct ReconciliationService {
    api_client: Arc<ApiClient>,
}

impl ReconciliationService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            api_client: Arc::new(app_state.get_api_client()),
        }
    }

    /// 执行每日对账
    ///
    /// # 参数
    /// - `date`: 对账日期（YYYY-MM-DD格式），如果为None则使用今天
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn reconcile_daily(
        &self,
        date: Option<&str>,
    ) -> Result<ReconciliationResult, String> {
        let url = if let Some(d) = date {
            format!("/api/v1/reconciliation/daily?date={}", d)
        } else {
            "/api/v1/reconciliation/daily".to_string()
        };

        self.api_client
            .post::<ReconciliationResult, serde_json::Value>(&url, &serde_json::json!({}))
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else if error_msg.contains("forbidden") || error_msg.contains("403") {
                    "权限不足，请联系管理员".to_string()
                } else {
                    format!("执行对账失败：{}", e)
                }
            })
    }

    /// 获取对账历史
    ///
    /// # 参数
    /// - `start_date`: 开始日期（YYYY-MM-DD）
    /// - `end_date`: 结束日期（YYYY-MM-DD）
    /// - `page`: 页码（从1开始）
    /// - `limit`: 每页数量
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_reconciliation_history(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> Result<ReconciliationHistoryResponse, String> {
        let mut url = "/api/v1/reconciliation/history".to_string();
        let mut params = Vec::new();

        if let Some(sd) = start_date {
            params.push(format!("start_date={}", sd));
        }
        if let Some(ed) = end_date {
            params.push(format!("end_date={}", ed));
        }
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
            .get::<ReconciliationHistoryResponse>(&url)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else {
                    format!("获取对账历史失败：{}", e)
                }
            })
    }

    /// 同步订单状态
    ///
    /// # 参数
    /// - `provider`: 服务商名称（可选，如果为None则同步所有服务商）
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn sync_order_status(
        &self,
        provider: Option<&str>,
    ) -> Result<OrderSyncResult, String> {
        let url = if let Some(p) = provider {
            format!("/api/v1/reconciliation/sync?provider={}", p)
        } else {
            "/api/v1/reconciliation/sync".to_string()
        };

        self.api_client
            .post::<OrderSyncResult, serde_json::Value>(&url, &serde_json::json!({}))
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else {
                    format!("同步订单状态失败：{}", e)
                }
            })
    }

    /// 获取异常监控数据
    ///
    /// # 参数
    /// - `start_date`: 开始日期（可选）
    /// - `end_date`: 结束日期（可选）
    ///
    /// # 错误处理
    /// 返回用户友好的错误消息
    pub async fn get_monitoring_data(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<MonitoringData, String> {
        let mut url = "/api/v1/reconciliation/monitoring".to_string();
        let mut params = Vec::new();

        if let Some(sd) = start_date {
            params.push(format!("start_date={}", sd));
        }
        if let Some(ed) = end_date {
            params.push(format!("end_date={}", ed));
        }

        if !params.is_empty() {
            url.push_str(&format!("?{}", params.join("&")));
        }

        self.api_client
            .get::<MonitoringData>(&url)
            .await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("timeout") || error_msg.contains("network") {
                    "网络连接超时，请稍后重试".to_string()
                } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
                    "请先登录账户".to_string()
                } else {
                    format!("获取监控数据失败：{}", e)
                }
            })
    }
}

/// 对账历史响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationHistoryResponse {
    pub results: Vec<ReconciliationResult>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

/// 订单同步结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSyncResult {
    pub provider: String,
    pub synced_orders: u64,
    pub updated_orders: u64,
    pub failed_orders: u64,
    pub sync_time: String,
    pub errors: Vec<String>,
}

/// 监控数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringData {
    pub success_rate: f64,
    pub failure_rate: f64,
    pub total_orders: u64,
    pub successful_orders: u64,
    pub failed_orders: u64,
    pub pending_orders: u64,
    pub average_response_time_ms: Option<u64>,
    pub alerts: Vec<Alert>,
}

/// 告警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub level: String, // "info", "warning", "error", "critical"
    pub message: String,
    pub timestamp: String,
    pub resolved: bool,
}
