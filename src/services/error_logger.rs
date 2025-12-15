//! Error Logger Service - 前端错误日志服务
//! 提供错误追踪、日志记录和错误上报功能

use crate::services::error_reporter::ErrorReporter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
/// 获取当前 Unix 时间戳（秒）- WebAssembly 兼容
fn now_timestamp() -> u64 {
    js_sys::Date::new_0().get_time() as u64 / 1000
}

/// 错误级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorLevel {
    Info,
    Warning,
    Error,
    Critical,
}

impl ErrorLevel {
    pub fn label(&self) -> &'static str {
        match self {
            ErrorLevel::Info => "INFO",
            ErrorLevel::Warning => "WARNING",
            ErrorLevel::Error => "ERROR",
            ErrorLevel::Critical => "CRITICAL",
        }
    }
}

/// 错误日志项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLog {
    pub timestamp: u64,
    pub level: ErrorLevel,
    pub message: String,
    pub context: Option<serde_json::Value>,
    pub stack_trace: Option<String>,
    pub user_agent: Option<String>,
    pub url: Option<String>,
}

/// 错误日志服务
pub struct ErrorLogger {
    logs: Vec<ErrorLog>,
    max_logs: usize,
    enable_console: bool,
    enable_storage: bool,
    error_reporter: Option<Arc<ErrorReporter>>,
}

impl ErrorLogger {
    /// 创建新的错误日志服务
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Vec::new(),
            max_logs,
            enable_console: true,
            enable_storage: true,
            error_reporter: None,
        }
    }

    /// 设置错误上报服务
    pub fn set_reporter(&mut self, reporter: Arc<ErrorReporter>) {
        self.error_reporter = Some(reporter);
    }

    /// 记录错误
    pub fn log(&mut self, level: ErrorLevel, message: String, context: Option<serde_json::Value>) {
        let timestamp = now_timestamp();

        let user_agent = web_sys::window().and_then(|w| w.navigator().user_agent().ok());

        let url = web_sys::window().and_then(|w| w.location().href().ok());

        let error_log = ErrorLog {
            timestamp,
            level,
            message: message.clone(),
            context,
            stack_trace: None,
            user_agent,
            url,
        };

        // 添加到内存日志
        self.logs.push(error_log.clone());
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }

        // 控制台输出
        if self.enable_console {
            let level_label = level.label();
            match level {
                ErrorLevel::Info => {
                    log::info!("[{}] {}", level_label, message);
                }
                ErrorLevel::Warning => {
                    log::warn!("[{}] {}", level_label, message);
                }
                ErrorLevel::Error => {
                    log::error!("[{}] {}", level_label, message);
                }
                ErrorLevel::Critical => {
                    log::error!("[{}] {}", level_label, message);
                }
            }
        }

        // 持久化存储（IndexedDB或localStorage）
        if self.enable_storage {
            self.save_to_storage(&error_log);
        }

        // 上报到Sentry（如果配置了ErrorReporter）
        if let Some(reporter) = &self.error_reporter {
            if level == ErrorLevel::Error || level == ErrorLevel::Critical {
                reporter.report_log(&error_log);
            }
        }
    }

    /// 保存到本地存储
    fn save_to_storage(&self, log: &ErrorLog) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                // 获取现有日志
                let existing = storage
                    .get_item("error_logs")
                    .ok()
                    .flatten()
                    .and_then(|s| serde_json::from_str::<Vec<ErrorLog>>(&s).ok())
                    .unwrap_or_default();

                // 添加新日志
                let mut logs = existing;
                logs.push(log.clone());

                // 限制日志数量
                if logs.len() > self.max_logs {
                    logs.remove(0);
                }

                // 保存
                if let Ok(json) = serde_json::to_string(&logs) {
                    let _ = storage.set_item("error_logs", &json);
                }
            }
        }
    }

    /// 获取所有日志
    pub fn get_logs(&self) -> &[ErrorLog] {
        &self.logs
    }

    /// 清空日志
    pub fn clear(&mut self) {
        self.logs.clear();
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item("error_logs");
            }
        }
    }

    /// 导出日志为JSON
    pub fn export_json(&self) -> String {
        serde_json::to_string(&self.logs).unwrap_or_default()
    }
}

/// 全局错误处理器
pub fn setup_global_error_handler() {
    // 使用web_sys的ErrorEvent处理
    // 注意：在Dioxus中，错误处理通常通过组件级别的错误边界处理
    // 这里提供一个简单的日志记录功能
    log::info!("Error logger initialized");
}
