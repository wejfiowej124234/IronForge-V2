//! User Feedback Component - 用户反馈组件
//! 提供用户操作反馈和交互增强
#![allow(dead_code)]

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 反馈类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeedbackType {
    Success,
    Error,
    Warning,
    Info,
    Loading,
}

impl FeedbackType {
    pub fn icon(&self) -> &'static str {
        match self {
            FeedbackType::Success => "✅",
            FeedbackType::Error => "❌",
            FeedbackType::Warning => "⚠️",
            FeedbackType::Info => "ℹ️",
            FeedbackType::Loading => "⏳",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            FeedbackType::Success => "rgba(34, 197, 94, 1)",
            FeedbackType::Error => "rgba(239, 68, 68, 1)",
            FeedbackType::Warning => "rgba(234, 179, 8, 1)",
            FeedbackType::Info => "rgba(59, 130, 246, 1)",
            FeedbackType::Loading => Colors::TEXT_SECONDARY,
        }
    }
}

/// 用户反馈组件属性
#[derive(Props, PartialEq, Clone)]
pub struct UserFeedbackProps {
    /// 反馈类型
    pub feedback_type: FeedbackType,
    /// 反馈消息
    pub message: String,
    /// 是否显示
    #[props(default = true)]
    pub visible: bool,
    /// 自动隐藏时间（毫秒）
    #[props(default = 3000)]
    pub auto_hide_ms: u32,
    /// 关闭回调
    pub on_close: Option<EventHandler<()>>,
}

/// 用户反馈组件
#[component]
pub fn UserFeedback(mut props: UserFeedbackProps) -> Element {
    let mut visible = use_signal(|| props.visible);

    // 自动隐藏
    use_effect({
        let mut visible_sig = visible;
        let auto_hide = props.auto_hide_ms;
        let on_close = props.on_close.clone();

        move || {
            if props.visible && auto_hide > 0 {
                spawn(async move {
                    gloo_timers::future::TimeoutFuture::new(auto_hide as u32).await;
                    visible_sig.set(false);
                    if let Some(ref handler) = on_close {
                        handler.call(());
                    }
                });
            }
        }
    });

    if !*visible.read() || !props.visible {
        return rsx! { div {} };
    }

    let feedback_type = props.feedback_type;
    let icon = feedback_type.icon();
    let color = feedback_type.color();

    rsx! {
        div {
            class: "fixed bottom-4 right-4 z-50 max-w-md",
            div {
                class: "p-4 rounded-lg shadow-lg flex items-start gap-3",
                style: format!(
                    "background: {}; border: 1px solid {};",
                    Colors::BG_SECONDARY,
                    color
                ),
                span {
                    class: "text-xl",
                    "{icon}"
                }
                div {
                    class: "flex-1",
                    p {
                        class: "text-sm font-medium",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "{props.message}"
                    }
                }
                if let Some(on_close) = props.on_close.take() {
                    button {
                        class: "text-lg leading-none",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        onclick: move |_| {
                            visible.set(false);
                            on_close.call(());
                        },
                        "×"
                    }
                }
            }
        }
    }
}

/// 操作确认对话框
#[derive(Props, PartialEq, Clone)]
pub struct ConfirmDialogProps {
    /// 标题
    pub title: String,
    /// 消息
    pub message: String,
    /// 确认按钮文本
    #[props(default = "确认".to_string())]
    pub confirm_text: String,
    /// 取消按钮文本
    #[props(default = "取消".to_string())]
    pub cancel_text: String,
    /// 是否显示
    pub visible: bool,
    /// 确认回调
    pub on_confirm: EventHandler<()>,
    /// 取消回调
    pub on_cancel: EventHandler<()>,
}

/// 确认对话框组件
#[component]
pub fn ConfirmDialog(props: ConfirmDialogProps) -> Element {
    if !props.visible {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center",
            style: "background: rgba(0, 0, 0, 0.5);",
            onclick: move |_| {
                props.on_cancel.call(());
            },
            div {
                class: "p-6 rounded-lg max-w-md w-full mx-4",
                style: format!("background: {};", Colors::BG_PRIMARY),
                onclick: move |e| {
                    e.stop_propagation();
                },
                h3 {
                    class: "text-lg font-semibold mb-3",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    "{props.title}"
                }
                p {
                    class: "text-sm mb-6",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    "{props.message}"
                }
                div {
                    class: "flex gap-3 justify-end",
                    button {
                        class: "px-4 py-2 rounded-lg text-sm font-medium",
                        style: format!(
                            "background: {}; color: {}; border: 1px solid {};",
                            Colors::BG_SECONDARY,
                            Colors::TEXT_PRIMARY,
                            Colors::BORDER_PRIMARY
                        ),
                        onclick: move |_| {
                            props.on_cancel.call(());
                        },
                        "{props.cancel_text}"
                    }
                    button {
                        class: "px-4 py-2 rounded-lg text-sm font-medium",
                        style: format!("background: {}; color: white;", Colors::TECH_PRIMARY),
                        onclick: move |_| {
                            props.on_confirm.call(());
                        },
                        "{props.confirm_text}"
                    }
                }
            }
        }
    }
}
