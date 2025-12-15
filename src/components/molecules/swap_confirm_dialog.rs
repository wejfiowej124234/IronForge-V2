//! Swap Confirm Dialog - 交换确认对话框组件
//! 在用户执行交换前显示确认信息

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 交换确认信息
#[derive(Debug, Clone, PartialEq)]
pub struct SwapConfirmInfo {
    pub from_token: String,
    pub to_token: String,
    pub from_amount: String,
    pub to_amount: String,
    pub exchange_rate: String,
    /// 协议手续费（1inch等DEX协议的费用，如果有）
    pub protocol_fee: Option<String>,
    /// Gas费用：区块链网络收取的交易执行费用（gas_used * gas_price）
    pub gas_fee: Option<String>,
    /// 平台服务费：钱包服务商收取的服务费用（与Gas费用完全独立）
    pub platform_service_fee: Option<String>,
    pub slippage: f64,
    /// 是否需要先执行approval交易（企业级实现）
    pub needs_approval: Option<bool>,
    /// 1inch路由器地址（用于前端显示和验证）
    pub router_address: Option<String>,
}

/// 交换确认对话框组件
#[component]
pub fn SwapConfirmDialog(
    /// 是否显示对话框
    show: Signal<bool>,
    /// 确认信息
    confirm_info: Option<SwapConfirmInfo>,
    /// 确认回调
    on_confirm: Option<EventHandler<()>>,
    /// 取消回调
    on_cancel: Option<EventHandler<()>>,
) -> Element {
    let show_val = *show.read();
    let info_opt = confirm_info.clone();

    if !show_val || info_opt.is_none() {
        return rsx! { div {} };
    }

    let info = info_opt.unwrap();

    rsx! {
        // 遮罩层
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center",
            style: "background: rgba(0, 0, 0, 0.5);",
            onclick: move |_| {
                if let Some(handler) = on_cancel {
                    handler.call(());
                }
            },

            // 对话框
            div {
                class: "bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md mx-4",
                style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                onclick: move |e| {
                    e.stop_propagation();
                },

                div {
                    class: "p-6",
                    // 标题
                    div {
                        class: "flex items-center justify-between mb-6",
                        h3 {
                            class: "text-xl font-bold",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "确认交换"
                        }
                        button {
                            class: "text-2xl leading-none opacity-50 hover:opacity-100 transition-opacity",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            onclick: move |_| {
                                if let Some(handler) = on_cancel {
                                    handler.call(());
                                }
                            },
                            "×"
                        }
                    }

                    // 交换信息
                    div {
                        class: "space-y-4 mb-6",
                        // 支付金额
                        div {
                            class: "p-4 rounded-lg",
                            style: format!("background: {}; border: 1px solid {};", Colors::BG_PRIMARY, Colors::BORDER_PRIMARY),
                            div {
                                class: "text-sm mb-1",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "您将支付"
                            }
                            div {
                                class: "text-2xl font-bold",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                "{info.from_amount} {info.from_token}"
                            }
                        }

                        // 箭头
                        div {
                            class: "flex justify-center my-2",
                            span {
                                class: "text-2xl",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "↓"
                            }
                        }

                        // 接收金额
                        div {
                            class: "p-4 rounded-lg",
                            style: format!("background: {}; border: 1px solid {};", Colors::BG_PRIMARY, Colors::BORDER_PRIMARY),
                            div {
                                class: "text-sm mb-1",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "您将收到"
                            }
                            div {
                                class: "text-2xl font-bold",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                "{info.to_amount} {info.to_token}"
                            }
                        }

                        // 详细信息
                        div {
                            class: "space-y-2 pt-4 border-t",
                            style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                            div {
                                class: "flex justify-between text-sm",
                                span {
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "汇率"
                                }
                                span {
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    "{info.exchange_rate}"
                                }
                            }
                            // 协议手续费（1inch等DEX协议的费用，如果有）
                            if let Some(protocol_fee) = &info.protocol_fee {
                                div {
                                    class: "flex justify-between text-sm",
                                    span {
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        "协议手续费"
                                    }
                                    span {
                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                        "{protocol_fee}"
                                    }
                                }
                            }
                            // Gas费用：区块链网络收取的交易执行费用
                            if let Some(gas) = &info.gas_fee {
                                div {
                                    class: "flex justify-between text-sm",
                                    span {
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        "Gas费用（网络费用）"
                                    }
                                    span {
                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                        "{gas}"
                                    }
                                }
                            }
                            // 平台服务费：钱包服务商收取的服务费用（与Gas费用完全独立）
                            if let Some(service_fee) = &info.platform_service_fee {
                                div {
                                    class: "flex justify-between text-sm",
                                    span {
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        "平台服务费"
                                    }
                                    span {
                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                        "{service_fee}"
                                    }
                                }
                            }
                            div {
                                class: "flex justify-between text-sm",
                                span {
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "滑点容忍度"
                                }
                                span {
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    "{info.slippage:.1}%"
                                }
                            }
                            // 是否需要approval提示
                            if let Some(true) = info.needs_approval {
                                div {
                                    class: "p-2 rounded mt-2",
                                    style: format!("background: rgba(59, 130, 246, 0.1); border: 1px solid rgba(59, 130, 246, 0.3);"),
                                    div {
                                        class: "flex items-center gap-2 text-xs",
                                        span { "ℹ️" }
                                        span {
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            "此交易需要先执行approval操作，系统将自动处理"
                                        }
                                    }
                                }
                            }
                            // 路由器地址显示（用于验证）
                            if let Some(router) = &info.router_address {
                                div {
                                    class: "flex justify-between text-xs mt-1",
                                    span {
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        "路由器地址"
                                    }
                                    span {
                                        class: "font-mono",
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        {
                                            if router.len() > 10 {
                                                format!("{}...{}", &router[..6], &router[router.len()-4..])
                                            } else {
                                                router.clone()
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 警告提示
                    div {
                        class: "p-3 rounded-lg mb-6",
                        style: format!("background: rgba(251, 191, 36, 0.1); border: 1px solid rgba(251, 191, 36, 0.3);"),
                        div {
                            class: "flex items-start gap-2 text-sm",
                            span {
                                "⚠️"
                            }
                            div {
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                "请仔细检查交换信息。一旦确认，交易将无法撤销。"
                            }
                        }
                    }

                    // 按钮组
                    div {
                        class: "flex gap-3",
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Large,
                            onclick: move |_| {
                                if let Some(handler) = on_cancel {
                                    handler.call(());
                                }
                            },
                            class: "flex-1",
                            "取消"
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            onclick: move |_| {
                                if let Some(handler) = on_confirm {
                                    handler.call(());
                                }
                            },
                            class: "flex-1",
                            "确认交换"
                        }
                    }
                }
            }
        }
    }
}
