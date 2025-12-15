//! Country Detection Hint - 国家检测提示组件
//! 显示用户国家检测结果和服务商支持状态

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 国家检测结果
#[derive(Debug, Clone, PartialEq)]
pub struct CountryDetectionResult {
    pub country_code: String, // ISO 3166-1 alpha-2
    pub country_name: String,
    pub detection_method: String,           // "KYC", "Payment", "IP"
    pub supported_providers: Vec<String>,   // 支持的服务商列表
    pub unsupported_providers: Vec<String>, // 不支持的服务商列表
}

/// 国家检测提示组件
#[component]
pub fn CountryDetectionHint(
    /// 检测结果
    detection_result: Option<CountryDetectionResult>,
) -> Element {
    let result = match detection_result {
        Some(r) => r,
        None => {
            return rsx! {
                div {
                    class: "p-3 rounded-lg",
                    style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                    div {
                        class: "flex items-center gap-2 text-sm",
                        span { "🌍" }
                        span {
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "正在检测您的国家/地区..."
                        }
                    }
                }
            };
        }
    };

    let has_unsupported = !result.unsupported_providers.is_empty();

    rsx! {
        div {
            class: "p-4 rounded-lg space-y-2",
            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),

            // 国家信息
            div {
                class: "flex items-center justify-between",
                div {
                    class: "flex items-center gap-2",
                    span { "🌍" }
                    div {
                        span {
                            class: "text-sm font-medium",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "检测到国家/地区: "
                        }
                        span {
                            class: "text-sm font-bold",
                            style: format!("color: {};", Colors::TECH_PRIMARY),
                            "{result.country_name}"
                        }
                    }
                }
                div {
                    class: "text-xs px-2 py-1 rounded",
                    style: format!("background: {}; color: {};", Colors::BG_PRIMARY, Colors::TEXT_SECONDARY),
                    "检测方式: {result.detection_method}"
                }
            }

            // 支持的服务商
            if !result.supported_providers.is_empty() {
                div {
                    class: "pt-2 border-t",
                    style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                    div {
                        class: "text-xs mb-1",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "✅ 支持的服务商:"
                    }
                    div {
                        class: "flex flex-wrap gap-1",
                        for provider in result.supported_providers.iter() {
                            span {
                                class: "px-2 py-1 rounded text-xs",
                                style: format!("background: rgba(34, 197, 94, 0.1); color: rgba(34, 197, 94, 1);"),
                                {provider.clone()}
                            }
                        }
                    }
                }
            }

            // 不支持的服务商警告
            if has_unsupported {
                div {
                    class: "pt-2 border-t",
                    style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                    div {
                        class: "text-xs mb-1",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "⚠️ 不支持的服务商（已自动过滤）:"
                    }
                    div {
                        class: "flex flex-wrap gap-1",
                        for provider in result.unsupported_providers.iter() {
                            span {
                                class: "px-2 py-1 rounded text-xs",
                                style: format!("background: rgba(239, 68, 68, 0.1); color: rgba(239, 68, 68, 1);"),
                                {provider.clone()}
                            }
                        }
                    }
                    div {
                        class: "text-xs mt-2",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "系统已自动切换到支持的服务商，您无需额外操作。"
                    }
                }
            }
        }
    }
}
