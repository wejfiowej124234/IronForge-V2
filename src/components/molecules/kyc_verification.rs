//! KYC Verification Component - KYC验证组件
//! 支持Sumsub/Onfido/Jumio等KYC服务商集成
#![allow(dead_code)]

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// KYC服务商类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KycProvider {
    Sumsub,
    Onfido,
    Jumio,
}

impl KycProvider {
    pub fn label(&self) -> &'static str {
        match self {
            KycProvider::Sumsub => "Sumsub",
            KycProvider::Onfido => "Onfido",
            KycProvider::Jumio => "Jumio",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            KycProvider::Sumsub => "全球领先的KYC/AML解决方案，支持200+国家",
            KycProvider::Onfido => "AI驱动的身份验证，快速准确的验证流程",
            KycProvider::Jumio => "企业级身份验证，支持多种文档类型",
        }
    }
}

/// KYC验证状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KycVerificationStatus {
    NotStarted, // 未开始
    InProgress, // 进行中
    Pending,    // 待审核
    Approved,   // 已通过
    Rejected,   // 已拒绝
    Expired,    // 已过期
}

impl KycVerificationStatus {
    pub fn label(&self) -> &'static str {
        match self {
            KycVerificationStatus::NotStarted => "未开始",
            KycVerificationStatus::InProgress => "进行中",
            KycVerificationStatus::Pending => "待审核",
            KycVerificationStatus::Approved => "已通过",
            KycVerificationStatus::Rejected => "已拒绝",
            KycVerificationStatus::Expired => "已过期",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            KycVerificationStatus::NotStarted => "rgba(107, 114, 128, 1)",
            KycVerificationStatus::InProgress => "rgba(59, 130, 246, 1)",
            KycVerificationStatus::Pending => "rgba(234, 179, 8, 1)",
            KycVerificationStatus::Approved => "rgba(34, 197, 94, 1)",
            KycVerificationStatus::Rejected => "rgba(239, 68, 68, 1)",
            KycVerificationStatus::Expired => "rgba(239, 68, 68, 1)",
        }
    }

    pub fn bg_color(&self) -> &'static str {
        match self {
            KycVerificationStatus::NotStarted => "rgba(107, 114, 128, 0.1)",
            KycVerificationStatus::InProgress => "rgba(59, 130, 246, 0.1)",
            KycVerificationStatus::Pending => "rgba(234, 179, 8, 0.1)",
            KycVerificationStatus::Approved => "rgba(34, 197, 94, 0.1)",
            KycVerificationStatus::Rejected => "rgba(239, 68, 68, 0.1)",
            KycVerificationStatus::Expired => "rgba(239, 68, 68, 0.1)",
        }
    }
}

/// KYC验证信息
#[derive(Debug, Clone, PartialEq)]
pub struct KycVerificationInfo {
    pub status: KycVerificationStatus,
    pub provider: Option<KycProvider>,
    pub verification_id: Option<String>,
    pub submitted_at: Option<String>,
    pub completed_at: Option<String>,
    pub rejection_reason: Option<String>,
    pub level: Option<String>, // "basic", "intermediate", "advanced"
}

/// KYC验证组件属性
#[derive(Props, PartialEq, Clone)]
pub struct KycVerificationProps {
    /// 当前验证信息
    pub verification_info: Option<KycVerificationInfo>,
    /// 开始验证回调
    pub on_start_verification: Option<EventHandler<KycProvider>>,
    /// 重新验证回调
    pub on_retry: Option<EventHandler<()>>,
}

/// KYC验证组件
#[component]
pub fn KycVerification(props: KycVerificationProps) -> Element {
    let verification_info = props.verification_info.clone();
    let status = verification_info
        .as_ref()
        .map(|info| info.status)
        .unwrap_or(KycVerificationStatus::NotStarted);

    rsx! {
        div {
            class: "space-y-4",
            // 当前状态显示
            div {
                class: "p-4 rounded-lg",
                style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                div {
                    class: "flex items-center justify-between mb-3",
                    div {
                        class: "flex items-center gap-2",
                        span {
                            class: "text-sm font-medium",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "KYC验证状态"
                        }
                        span {
                            class: "px-2 py-1 rounded text-xs font-medium",
                            style: format!("background: {}; color: {};", status.bg_color(), status.color()),
                            {status.label()}
                        }
                    }
                    if let Some(info) = &verification_info {
                        if let Some(provider) = info.provider {
                            span {
                                class: "text-xs",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                {provider.label()}
                            }
                        }
                    }
                }

                // 状态详情
                if let Some(info) = &verification_info {
                    if let Some(submitted) = &info.submitted_at {
                        div {
                            class: "text-xs mb-2",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "提交时间: {submitted}"
                        }
                    }
                    if let Some(completed) = &info.completed_at {
                        div {
                            class: "text-xs mb-2",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "完成时间: {completed}"
                        }
                    }
                    if let Some(reason) = &info.rejection_reason {
                        div {
                            class: "p-2 rounded mt-2",
                            style: format!("background: rgba(239, 68, 68, 0.1);"),
                            div {
                                class: "text-xs font-medium mb-1",
                                style: "color: rgba(239, 68, 68, 1);",
                                "拒绝原因"
                            }
                            div {
                                class: "text-xs",
                                style: "color: rgba(239, 68, 68, 0.9);",
                                "{reason}"
                            }
                        }
                    }
                }
            }

            // 操作按钮
            if status == KycVerificationStatus::NotStarted {
                // 选择KYC服务商
                div {
                    class: "space-y-3",
                    div {
                        class: "text-sm font-medium",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "选择KYC服务商"
                    }
                    div {
                        class: "grid grid-cols-1 md:grid-cols-3 gap-3",
                        for provider in [KycProvider::Sumsub, KycProvider::Onfido, KycProvider::Jumio] {
                            div {
                                class: "p-4 rounded-lg border cursor-pointer transition-all hover:scale-105",
                                style: format!(
                                    "background: {}; border-color: {};",
                                    Colors::BG_PRIMARY,
                                    Colors::BORDER_PRIMARY
                                ),
                                onclick: {
                                    let provider_clone = provider.clone();
                                    let handler_opt = props.on_start_verification.clone();
                                    move |_| {
                                        if let Some(handler) = handler_opt.as_ref() {
                                            handler.call(provider_clone.clone());
                                        }
                                    }
                                },
                                div {
                                    class: "text-sm font-medium mb-1",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    {provider.label()}
                                }
                                div {
                                    class: "text-xs",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    {provider.description()}
                                }
                            }
                        }
                    }
                }
            } else if status == KycVerificationStatus::InProgress {
                div {
                    class: "p-4 rounded-lg",
                    style: format!("background: {};", Colors::BG_SECONDARY),
                    div {
                        class: "text-sm",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "KYC验证正在进行中，请按照指引完成验证流程..."
                    }
                }
            } else if status == KycVerificationStatus::Pending {
                div {
                    class: "p-4 rounded-lg",
                    style: format!("background: {};", Colors::BG_SECONDARY),
                    div {
                        class: "text-sm",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "您的KYC验证正在审核中，通常需要1-3个工作日..."
                    }
                }
            } else if status == KycVerificationStatus::Approved {
                if let Some(info) = &verification_info {
                    if let Some(level) = &info.level {
                        div {
                            class: "p-4 rounded-lg",
                            style: format!("background: rgba(34, 197, 94, 0.1); border: 1px solid rgba(34, 197, 94, 0.3);"),
                            div {
                                class: "text-sm font-medium mb-1",
                                style: "color: rgba(34, 197, 94, 1);",
                                "✓ KYC验证已通过"
                            }
                            div {
                                class: "text-xs",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "认证等级: {level}"
                            }
                        }
                    }
                }
            } else if matches!(status, KycVerificationStatus::Rejected | KycVerificationStatus::Expired) {
                if let Some(ref on_retry) = props.on_retry {
                    {
                        let handler = on_retry.clone();
                        rsx! {
                            div {
                                class: "space-y-3",
                                div {
                                    class: "p-4 rounded-lg",
                                    style: format!("background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3);"),
                                    div {
                                        class: "text-sm font-medium mb-1",
                                        style: "color: rgba(239, 68, 68, 1);",
                                        if status == KycVerificationStatus::Rejected {
                                            "KYC验证被拒绝"
                                        } else {
                                            "KYC验证已过期"
                                        }
                                    }
                                    div {
                                        class: "text-xs",
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        "请重新提交验证"
                                    }
                                }
                                button {
                                    class: "w-full px-4 py-2 rounded-lg font-medium text-sm transition-all",
                                    style: format!("background: {}; color: white;", Colors::TECH_PRIMARY),
                                    onclick: move |_| {
                                        handler.call(());
                                    },
                                    "重新验证"
                                }
                            }
                        }
                    }
                } else {
                    div {
                        class: "p-4 rounded-lg",
                        style: format!("background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3);"),
                        div {
                            class: "text-sm font-medium mb-1",
                            style: "color: rgba(239, 68, 68, 1);",
                            if status == KycVerificationStatus::Rejected {
                                "KYC验证被拒绝"
                            } else {
                                "KYC验证已过期"
                            }
                        }
                        div {
                            class: "text-xs",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "请重新提交验证"
                        }
                    }
                }
            }
        }
    }
}
