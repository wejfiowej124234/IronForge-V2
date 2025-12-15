//! Error Boundary - 错误边界组件
//! 捕获组件树中的错误，显示友好的错误界面

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 错误边界组件
#[component]
pub fn ErrorBoundary(children: Element, fallback: Option<Element>) -> Element {
    let mut error = use_signal(|| Option::<String>::None);

    // 注意：Dioxus 0.6 没有内置的错误边界机制
    // 这个组件主要用于显示错误状态和提供恢复机制
    // 实际的错误捕获需要在组件内部手动处理

    if let Some(err_msg) = error.read().as_ref() {
        if let Some(fallback_ui) = fallback {
            return fallback_ui;
        }

        return rsx! {
            div {
                class: "min-h-screen flex items-center justify-center p-6",
                style: format!("background: {};", Colors::BG_PRIMARY),
                div {
                    class: "max-w-md w-full p-8 rounded-lg backdrop-blur-sm",
                    style: format!(
                        "background: rgba(255, 255, 255, 0.05); border: 1px solid {};",
                        Colors::PAYMENT_ERROR
                    ),
                    div {
                        class: "text-center",
                        h2 {
                            class: "text-2xl font-bold mb-4",
                            style: format!("color: {};", Colors::PAYMENT_ERROR),
                            "发生错误"
                        }
                        p {
                            class: "mb-6",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            {err_msg.clone()}
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Medium,
                            onclick: move |_| {
                                error.set(None);
                                // 刷新页面
                                if let Some(window) = web_sys::window() {
                                    if let Err(e) = window.location().reload() {
                                        log::error!("Failed to reload page: {:?}", e);
                                    }
                                }
                            },
                            "重新加载"
                        }
                    }
                }
            }
        };
    }

    rsx! {
        {children}
    }
}

/// 错误边界Hook
/// 用于在组件中捕获和处理错误
///
/// 注意：此函数当前未使用，但保留用于未来扩展
#[allow(dead_code)]
pub fn use_error_boundary() -> Signal<Option<String>> {
    use_signal(|| Option::<String>::None)
}
