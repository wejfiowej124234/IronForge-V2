//! Toast - 全局提示组件
//! 用于显示成功、错误、警告等信息提示
#![allow(dead_code)]

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

/// Toast类型
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ToastType {
    Success,
    Error,
    #[allow(dead_code)] // 为未来扩展准备
    Warning,
    Info,
}

/// Toast消息
#[derive(Clone, Debug, PartialEq)]
pub struct ToastMessage {
    pub id: u64,
    pub message: String,
    pub toast_type: ToastType,
    pub duration: u32, // 显示时长（毫秒）
}

/// Toast容器组件
#[component]
pub fn ToastContainer(messages: Signal<Vec<ToastMessage>>) -> Element {
    rsx! {
        div {
            class: "fixed top-4 right-4 z-50 flex flex-col gap-2 max-w-md",
            for message in messages.read().iter() {
                ToastItem {
                    key: "{message.id}",
                    message: message.clone(),
                    on_close: move |id| {
                        let mut msgs = messages.write();
                        msgs.retain(|m| m.id != id);
                    }
                }
            }
        }
    }
}

/// 单个Toast项
#[component]
fn ToastItem(message: ToastMessage, on_close: EventHandler<u64>) -> Element {
    let msg_id = message.id;
    let msg_duration = message.duration;
    let toast_type = message.toast_type;
    let msg_text = message.message.clone();

    // 自动关闭
    use_effect(move || {
        let close_handler = on_close;
        spawn(async move {
            TimeoutFuture::new(msg_duration).await;
            close_handler.call(msg_id);
        });
    });

    let bg_color = match toast_type {
        ToastType::Success => format!(
            "background: rgba(34, 197, 94, 0.1); border: 1px solid {};",
            Colors::PAYMENT_SUCCESS
        ),
        ToastType::Error => format!(
            "background: rgba(239, 68, 68, 0.1); border: 1px solid {};",
            Colors::PAYMENT_ERROR
        ),
        ToastType::Warning => format!(
            "background: rgba(251, 191, 36, 0.1); border: 1px solid {};",
            Colors::PAYMENT_WARNING
        ),
        ToastType::Info => format!(
            "background: rgba(59, 130, 246, 0.1); border: 1px solid {};",
            Colors::TECH_PRIMARY
        ),
    };

    let text_color = match toast_type {
        ToastType::Success => Colors::PAYMENT_SUCCESS,
        ToastType::Error => Colors::PAYMENT_ERROR,
        ToastType::Warning => Colors::PAYMENT_WARNING,
        ToastType::Info => Colors::TECH_PRIMARY,
    };

    let icon = match toast_type {
        ToastType::Success => "✓",
        ToastType::Error => "✕",
        ToastType::Warning => "⚠",
        ToastType::Info => "ℹ",
    };

    rsx! {
        div {
            class: "p-4 rounded-lg shadow-lg backdrop-blur-sm animate-slide-in-right",
            style: format!("{} color: {};", bg_color, text_color),
            div {
                class: "flex items-start gap-3",
                div {
                    class: "flex-shrink-0 text-xl font-bold",
                    {icon}
                }
                div {
                    class: "flex-1",
                    {msg_text}
                }
                button {
                    class: "flex-shrink-0 text-lg opacity-70 hover:opacity-100 transition-opacity",
                    onclick: move |_| {
                        on_close.call(msg_id);
                    },
                    "×"
                }
            }
        }
    }
}
