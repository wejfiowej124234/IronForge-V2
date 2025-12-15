//! Provider Status Badge - 服务商状态徽章组件
//! 显示服务商的健康状态和可用性
#![allow(dead_code)]

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 服务商状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProviderStatus {
    Healthy,  // 健康
    Degraded, // 降级
    Down,     // 不可用
    Unknown,  // 未知
}

impl ProviderStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ProviderStatus::Healthy => "健康",
            ProviderStatus::Degraded => "降级",
            ProviderStatus::Down => "不可用",
            ProviderStatus::Unknown => "未知",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            ProviderStatus::Healthy => "rgba(34, 197, 94, 1)",
            ProviderStatus::Degraded => "rgba(251, 191, 36, 1)",
            ProviderStatus::Down => "rgba(239, 68, 68, 1)",
            ProviderStatus::Unknown => "rgba(156, 163, 175, 1)",
        }
    }

    pub fn bg_color(&self) -> &'static str {
        match self {
            ProviderStatus::Healthy => "rgba(34, 197, 94, 0.1)",
            ProviderStatus::Degraded => "rgba(251, 191, 36, 0.1)",
            ProviderStatus::Down => "rgba(239, 68, 68, 0.1)",
            ProviderStatus::Unknown => "rgba(156, 163, 175, 0.1)",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ProviderStatus::Healthy => "✅",
            ProviderStatus::Degraded => "⚠️",
            ProviderStatus::Down => "❌",
            ProviderStatus::Unknown => "❓",
        }
    }
}

/// 服务商状态信息
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderStatusInfo {
    pub provider_name: String,
    pub status: ProviderStatus,
    pub response_time_ms: Option<u64>,
    pub success_rate: Option<f64>,
    pub last_check: Option<u64>, // Unix timestamp
}

/// 服务商状态徽章组件
#[component]
pub fn ProviderStatusBadge(
    /// 服务商状态信息
    status_info: ProviderStatusInfo,
) -> Element {
    let status = status_info.status;

    rsx! {
        div {
            class: "inline-flex items-center gap-2 px-3 py-1 rounded-full",
            style: format!("background: {};", status.bg_color()),
            span {
                {status.icon()}
            }
            span {
                class: "text-xs font-medium",
                style: format!("color: {};", status.color()),
                {status_info.provider_name}
            }
            span {
                class: "text-xs",
                style: format!("color: {};", Colors::TEXT_SECONDARY),
                {status.label()}
            }
            if let Some(response_time) = status_info.response_time_ms {
                span {
                    class: "text-xs",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    "{response_time}ms"
                }
            }
            if let Some(success_rate) = status_info.success_rate {
                span {
                    class: "text-xs",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    "{success_rate:.1}%"
                }
            }
        }
    }
}

/// 服务商状态列表组件
#[component]
pub fn ProviderStatusList(
    /// 服务商状态列表
    providers: Vec<ProviderStatusInfo>,
) -> Element {
    if providers.is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "p-4 rounded-lg space-y-2",
            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
            h4 {
                class: "text-sm font-semibold mb-3",
                style: format!("color: {};", Colors::TEXT_PRIMARY),
                "服务商状态"
            }
            div {
                class: "flex flex-wrap gap-2",
                for provider in providers.iter() {
                    ProviderStatusBadge {
                        status_info: provider.clone(),
                    }
                }
            }
        }
    }
}
