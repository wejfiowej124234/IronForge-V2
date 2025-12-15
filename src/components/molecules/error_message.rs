//! Error Message - 错误消息显示组件
//! 企业级用户友好的错误消息显示，隐藏技术细节

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 将技术错误消息转换为用户友好的消息
fn user_friendly_error(error: &str) -> String {
    let error_lower = error.to_lowercase();

    // 网络相关错误
    if error_lower.contains("timeout")
        || error_lower.contains("network")
        || error_lower.contains("connection")
    {
        return "网络连接超时，请检查您的网络连接后重试".to_string();
    }

    // API相关错误
    if error_lower.contains("api")
        || error_lower.contains("server error")
        || error_lower.contains("500")
    {
        return "服务暂时不可用，请稍后再试".to_string();
    }

    // 余额相关错误
    if error_lower.contains("insufficient")
        || error_lower.contains("balance")
        || error_lower.contains("余额不足")
    {
        return "余额不足，请检查您的账户余额".to_string();
    }

    // 验证相关错误
    if error_lower.contains("unauthorized")
        || error_lower.contains("401")
        || error_lower.contains("身份验证")
    {
        return "请先完成身份验证".to_string();
    }

    // 限额相关错误
    if error_lower.contains("limit")
        || error_lower.contains("限额")
        || error_lower.contains("exceeded")
    {
        return "交易金额超出限额，请调整金额后重试".to_string();
    }

    // 报价相关错误
    if error_lower.contains("quote")
        || error_lower.contains("报价")
        || error_lower.contains("expired")
    {
        return "报价已过期，请重新获取报价".to_string();
    }

    // 无效输入
    if error_lower.contains("invalid")
        || error_lower.contains("无效")
        || error_lower.contains("invalid input")
    {
        return "输入信息无效，请检查后重试".to_string();
    }

    // 默认：返回原始错误，但移除技术细节
    error
        .replace("Error: ", "")
        .replace("error: ", "")
        .replace("Failed to ", "")
        .replace("failed to ", "")
        .trim()
        .to_string()
}

/// 错误消息显示组件
#[component]
pub fn ErrorMessage(
    /// 错误消息文本
    message: Option<String>,
    /// 是否显示技术细节（默认false，只显示用户友好消息）
    #[props(default = false)]
    show_technical: bool,
    /// 自定义类名
    #[props(default)]
    class: Option<String>,
) -> Element {
    if let Some(error) = message {
        let friendly_msg = user_friendly_error(&error);
        let display_msg = if show_technical && error != friendly_msg {
            format!("{}\n\n技术详情: {}", friendly_msg, error)
        } else {
            friendly_msg
        };

        rsx! {
            div {
                class: format!("mt-4 p-4 rounded-lg {}", class.unwrap_or_default()),
                style: format!(
                    "background: rgba(239, 68, 68, 0.1); border: 1px solid {}; color: {};",
                    Colors::PAYMENT_ERROR,
                    Colors::PAYMENT_ERROR
                ),
                div {
                    class: "flex items-start gap-2",
                    // 错误图标
                    span {
                        class: "text-lg",
                        "⚠️"
                    }
                    div {
                        class: "flex-1",
                        div {
                            class: "font-medium mb-1",
                            "操作失败"
                        }
                        div {
                            class: "text-sm whitespace-pre-line",
                            {display_msg}
                        }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}
