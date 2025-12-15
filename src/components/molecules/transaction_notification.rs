//! Transaction Notification - 交易通知组件
//! 实时显示交易状态变化通知
#![allow(dead_code)]

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 交易通知类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}

impl NotificationType {
    fn icon(&self) -> &'static str {
        match self {
            NotificationType::Success => "✅",
            NotificationType::Error => "❌",
            NotificationType::Warning => "⚠️",
            NotificationType::Info => "ℹ️",
        }
    }

    fn bg_color(&self) -> &'static str {
        match self {
            NotificationType::Success => "rgba(34, 197, 94, 0.1)",
            NotificationType::Error => "rgba(239, 68, 68, 0.1)",
            NotificationType::Warning => "rgba(251, 191, 36, 0.1)",
            NotificationType::Info => "rgba(59, 130, 246, 0.1)",
        }
    }

    fn border_color(&self) -> &'static str {
        match self {
            NotificationType::Success => "rgba(34, 197, 94, 0.3)",
            NotificationType::Error => "rgba(239, 68, 68, 0.3)",
            NotificationType::Warning => "rgba(251, 191, 36, 0.3)",
            NotificationType::Info => "rgba(59, 130, 246, 0.3)",
        }
    }
}

/// 交易通知项
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionNotification {
    pub id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub timestamp: u64, // Unix timestamp (seconds)
    pub transaction_id: Option<String>,
}

/// 交易通知组件
#[component]
pub fn TransactionNotificationItem(
    /// 通知项
    notification: TransactionNotification,
    /// 关闭回调
    on_close: Option<EventHandler<String>>,
) -> Element {
    let notification_type = notification.notification_type;
    let id = notification.id.clone();
    let title = notification.title.clone();
    let message = notification.message.clone();
    let transaction_id = notification.transaction_id.clone();
    let timestamp = notification.timestamp;

    rsx! {
        div {
            class: "p-4 rounded-lg mb-3 transition-all animate-slide-in",
            style: format!(
                "background: {}; border: 1px solid {};",
                notification_type.bg_color(),
                notification_type.border_color()
            ),
            div {
                class: "flex items-start gap-3",
                // 图标
                span {
                    class: "text-xl",
                    {notification_type.icon()}
                }
                // 内容
                div {
                    class: "flex-1",
                    h4 {
                        class: "text-sm font-semibold mb-1",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {title}
                    }
                    p {
                        class: "text-xs",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        {message}
                    }
                    if let Some(tx_id) = transaction_id {
                        div {
                            class: "mt-2 text-xs font-mono",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "交易ID: {tx_id}"
                        }
                    }
                    div {
                        class: "mt-1 text-xs",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        {format_time(timestamp)}
                    }
                }
                // 关闭按钮
                button {
                    class: "text-lg leading-none opacity-50 hover:opacity-100 transition-opacity",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    onclick: move |_| {
                        if let Some(handler) = on_close {
                            handler.call(id.clone());
                        }
                    },
                    "×"
                }
            }
        }
    }
}

/// 交易通知容器
#[component]
pub fn TransactionNotificationContainer(
    /// 通知列表
    notifications: Signal<Vec<TransactionNotification>>,
    /// 关闭通知回调
    on_close: Option<EventHandler<String>>,
) -> Element {
    let notifications_val = notifications.read();

    if notifications_val.is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "fixed top-4 right-4 z-50 w-full max-w-sm",
            for notification in notifications_val.iter() {
                TransactionNotificationItem {
                    notification: notification.clone(),
                    on_close: on_close.clone(),
                }
            }
        }
    }
}

/// 格式化时间显示
fn format_time(timestamp: u64) -> String {
    // 使用js_sys::Date获取当前时间
    let now_js = (js_sys::Date::now() / 1000.0) as u64;
    let diff = now_js.saturating_sub(timestamp);

    if diff < 60 {
        "刚刚".to_string()
    } else if diff < 3600 {
        format!("{}分钟前", diff / 60)
    } else if diff < 86400 {
        format!("{}小时前", diff / 3600)
    } else {
        // 简单格式化，显示日期和时间
        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64((timestamp * 1000) as f64));
        format!(
            "{}-{:02}-{:02} {:02}:{:02}",
            date.get_full_year(),
            date.get_month() + 1,
            date.get_date(),
            date.get_hours(),
            date.get_minutes()
        )
    }
}
