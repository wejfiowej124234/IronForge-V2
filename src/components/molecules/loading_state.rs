//! Loading State - 加载状态显示组件
//! 企业级加载状态显示，支持进度和预计时间

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 加载状态显示组件
#[component]
pub fn LoadingState(
    /// 加载消息
    #[props(default)]
    message: Option<String>,
    /// 进度百分比 (0-100)
    #[props(default)]
    progress: Option<u8>,
    /// 预计剩余时间（秒）
    #[props(default)]
    estimated_time: Option<u64>,
    /// 自定义类名
    #[props(default)]
    class: Option<String>,
) -> Element {
    let default_message = message.unwrap_or_else(|| "处理中...".to_string());

    rsx! {
        div {
            class: format!("p-4 rounded-lg {}", class.unwrap_or_default()),
            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),

            div {
                class: "flex items-center gap-3",

                // 加载动画
                div {
                    class: "animate-spin rounded-full h-5 w-5 border-2 border-t-transparent",
                    style: format!("border-color: {};", Colors::TECH_PRIMARY),
                }

                // 消息和进度
                div {
                    class: "flex-1",
                    div {
                        class: "text-sm font-medium",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {default_message}
                    }

                    // 进度条
                    if let Some(progress_val) = progress {
                        div {
                            class: "mt-2",
                            div {
                                class: "w-full bg-gray-200 rounded-full h-2",
                                style: format!("background: {};", Colors::BG_PRIMARY),
                                div {
                                    class: "h-2 rounded-full transition-all duration-300",
                                    style: format!(
                                        "width: {}%; background: {};",
                                        progress_val,
                                        Colors::TECH_PRIMARY
                                    ),
                                }
                            }
                            div {
                                class: "text-xs mt-1",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "{progress_val}%"
                            }
                        }
                    }

                    // 预计时间
                    if let Some(time) = estimated_time {
                        div {
                            class: "text-xs mt-1",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            if time < 60 {
                                "预计剩余 {time} 秒"
                            } else {
                                "预计剩余 {time / 60} 分钟"
                            }
                        }
                    }
                }
            }
        }
    }
}
