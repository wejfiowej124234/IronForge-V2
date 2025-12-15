//! Limit Display - 限额显示组件
//! 显示用户KYC等级和交易限额
#![allow(dead_code)]

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// KYC等级
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KycLevel {
    None,
    Basic,
    Intermediate,
    Advanced,
}

impl KycLevel {
    pub fn label(&self) -> &'static str {
        match self {
            KycLevel::None => "未认证",
            KycLevel::Basic => "基础认证",
            KycLevel::Intermediate => "中级认证",
            KycLevel::Advanced => "高级认证",
        }
    }

    pub fn daily_limit(&self) -> f64 {
        match self {
            KycLevel::None => 0.0,
            KycLevel::Basic => 1000.0,
            KycLevel::Intermediate => 10000.0,
            KycLevel::Advanced => 100000.0,
        }
    }

    pub fn monthly_limit(&self) -> f64 {
        match self {
            KycLevel::None => 0.0,
            KycLevel::Basic => 5000.0,
            KycLevel::Intermediate => 50000.0,
            KycLevel::Advanced => 500000.0,
        }
    }
}

/// 限额信息
#[derive(Debug, Clone, PartialEq)]
pub struct LimitInfo {
    pub kyc_level: KycLevel,
    pub daily_used: f64,
    pub daily_limit: f64,
    pub monthly_used: f64,
    pub monthly_limit: f64,
}

/// 限额显示组件
#[component]
pub fn LimitDisplay(
    /// 限额信息
    limit_info: Option<LimitInfo>,
) -> Element {
    let info = match limit_info {
        Some(i) => i,
        None => {
            // 默认显示未认证状态
            return rsx! {
                div {
                    class: "p-4 rounded-lg",
                    style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                    div {
                        class: "flex items-center justify-between mb-2",
                        span {
                            class: "text-sm font-medium",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "KYC认证状态"
                        }
                        span {
                            class: "px-2 py-1 rounded text-xs",
                            style: format!("background: rgba(239, 68, 68, 0.1); color: rgba(239, 68, 68, 1);"),
                            "未认证"
                        }
                    }
                    div {
                        class: "text-xs",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "完成KYC认证可提高交易限额"
                    }
                }
            };
        }
    };

    let daily_percent = if info.daily_limit > 0.0 {
        (info.daily_used / info.daily_limit * 100.0).min(100.0)
    } else {
        0.0
    };

    let monthly_percent = if info.monthly_limit > 0.0 {
        (info.monthly_used / info.monthly_limit * 100.0).min(100.0)
    } else {
        0.0
    };

    let kyc_color = match info.kyc_level {
        KycLevel::None => "rgba(239, 68, 68, 1)",
        KycLevel::Basic => "rgba(251, 191, 36, 1)",
        KycLevel::Intermediate => "rgba(59, 130, 246, 1)",
        KycLevel::Advanced => "rgba(34, 197, 94, 1)",
    };

    let kyc_bg = match info.kyc_level {
        KycLevel::None => "rgba(239, 68, 68, 0.1)",
        KycLevel::Basic => "rgba(251, 191, 36, 0.1)",
        KycLevel::Intermediate => "rgba(59, 130, 246, 0.1)",
        KycLevel::Advanced => "rgba(34, 197, 94, 0.1)",
    };

    rsx! {
        div {
            class: "p-4 rounded-lg space-y-3",
            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),

            // KYC等级
            div {
                class: "flex items-center justify-between",
                span {
                    class: "text-sm font-medium",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    "KYC认证等级"
                }
                span {
                    class: "px-2 py-1 rounded text-xs font-medium",
                    style: format!("background: {}; color: {};", kyc_bg, kyc_color),
                    {info.kyc_level.label()}
                }
            }

            // 每日限额
            div {
                class: "space-y-1",
                div {
                    class: "flex items-center justify-between text-xs",
                    span {
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "每日限额"
                    }
                    span {
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {format!("${:.2} / ${:.2}", info.daily_used, info.daily_limit)}
                    }
                }
                div {
                    class: "w-full h-2 rounded-full overflow-hidden",
                    style: format!("background: {};", Colors::BG_PRIMARY),
                    div {
                        class: "h-full transition-all",
                        style: format!(
                            "width: {}%; background: {};",
                            daily_percent,
                            if daily_percent >= 90.0 {
                                "rgba(239, 68, 68, 0.8)"
                            } else if daily_percent >= 70.0 {
                                "rgba(251, 191, 36, 0.8)"
                            } else {
                                "rgba(34, 197, 94, 0.8)"
                            }
                        ),
                    }
                }
            }

            // 每月限额
            div {
                class: "space-y-1",
                div {
                    class: "flex items-center justify-between text-xs",
                    span {
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "每月限额"
                    }
                    span {
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {format!("${:.2} / ${:.2}", info.monthly_used, info.monthly_limit)}
                    }
                }
                div {
                    class: "w-full h-2 rounded-full overflow-hidden",
                    style: format!("background: {};", Colors::BG_PRIMARY),
                    div {
                        class: "h-full transition-all",
                        style: format!(
                            "width: {}%; background: {};",
                            monthly_percent,
                            if monthly_percent >= 90.0 {
                                "rgba(239, 68, 68, 0.8)"
                            } else if monthly_percent >= 70.0 {
                                "rgba(251, 191, 36, 0.8)"
                            } else {
                                "rgba(34, 197, 94, 0.8)"
                            }
                        ),
                    }
                }
            }
        }
    }
}
