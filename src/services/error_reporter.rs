//! Error Reporter Service - 错误上报服务
//! 提供错误上报到Sentry的功能（可选，需要Sentry账户和DSN）

use crate::services::error_logger::{ErrorLevel, ErrorLog};
use serde_json::Value;

/// 错误上报服务
pub struct ErrorReporter {
    enabled: bool,
    sentry_dsn: Option<String>,
    environment: String,
}

impl ErrorReporter {
    /// 创建新的错误上报服务
    pub fn new(sentry_dsn: Option<String>) -> Self {
        let enabled = sentry_dsn.is_some();
        let environment = Self::get_environment();

        Self {
            enabled,
            sentry_dsn,
            environment,
        }
    }

    /// 上报错误到Sentry
    ///
    /// 注意：需要添加sentry依赖并配置DSN才能实际使用
    /// 当前实现为框架，待集成Sentry SDK
    pub fn report_error(&self, level: ErrorLevel, message: String, _context: Option<Value>) {
        if !self.enabled {
            log::debug!("Error reporting disabled, skipping: {}", message);
            return;
        }

        // TODO: 集成Sentry SDK
        // 示例代码（需要添加sentry依赖）：
        // use sentry::{capture_message, Level};
        //
        // let sentry_level = match level {
        //     ErrorLevel::Info => Level::Info,
        //     ErrorLevel::Warning => Level::Warning,
        //     ErrorLevel::Error => Level::Error,
        //     ErrorLevel::Critical => Level::Fatal,
        // };
        //
        // if let Some(ctx) = context {
        //     sentry::configure_scope(|scope| {
        //         scope.set_context("error_context", ctx);
        //     });
        // }
        //
        // capture_message(&message, sentry_level);

        // 当前实现：记录到控制台
        log::warn!(
            "[ErrorReporter] Would report to Sentry: [{}] {} (Environment: {})",
            level.label(),
            message,
            self.environment
        );
    }

    /// 上报错误日志
    pub fn report_log(&self, log: &ErrorLog) {
        self.report_error(log.level, log.message.clone(), log.context.clone());
    }

    /// 获取环境信息
    fn get_environment() -> String {
        // 从环境变量或配置中获取
        // 默认值：development
        if let Some(window) = web_sys::window() {
            // 使用href()获取完整URL，然后解析hostname
            if let Ok(href) = window.location().href() {
                if href.contains("localhost") || href.contains("127.0.0.1") {
                    return "development".to_string();
                } else if href.contains("staging") || href.contains("test") {
                    return "staging".to_string();
                } else if !href.is_empty() {
                    return "production".to_string();
                }
            }
        }
        "development".to_string()
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 设置Sentry DSN
    pub fn set_dsn(&mut self, dsn: Option<String>) {
        self.sentry_dsn = dsn.clone();
        self.enabled = dsn.is_some();
    }
}

/// 初始化错误上报服务
///
/// 从环境变量或配置中读取Sentry DSN
pub fn init_error_reporter() -> ErrorReporter {
    // 从环境变量或localStorage读取Sentry DSN
    let dsn = if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(dsn_value)) = storage.get_item("sentry_dsn") {
                Some(dsn_value)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    ErrorReporter::new(dsn)
}
