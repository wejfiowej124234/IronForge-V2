//! Limit Order Form - 限价单表单组件
//! 企业级限价单功能，支持设置目标价格

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::services::token::TokenInfo;
use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 限价单类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LimitOrderType {
    Buy,  // 买入限价单
    Sell, // 卖出限价单
}

impl LimitOrderType {
    fn label(&self) -> &'static str {
        match self {
            LimitOrderType::Buy => "买入",
            LimitOrderType::Sell => "卖出",
        }
    }
}

/// 限价单表单组件
#[component]
#[allow(clippy::type_complexity)]
pub fn LimitOrderForm(
    /// 买入/卖出类型
    order_type: Signal<LimitOrderType>,
    /// 代币选择（From）
    from_token: Signal<Option<TokenInfo>>,
    /// 代币选择（To）
    to_token: Signal<Option<TokenInfo>>,
    /// 数量
    amount: Signal<String>,
    /// 限价（目标价格）
    limit_price: Signal<String>,
    /// 有效期（天数）
    expiry_days: Signal<u32>,
    /// 错误消息
    error_message: Signal<Option<String>>,
    /// 加载状态
    loading: Signal<bool>,
    /// 提交回调
    #[allow(clippy::type_complexity)]
    on_submit: Option<EventHandler<(LimitOrderType, String, String, String, u32)>>,
) -> Element {
    let order_type_val = *order_type.read();

    rsx! {
        div {
            class: "p-6 rounded-lg space-y-4",
            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),

            // 标题
            div {
                class: "flex items-center justify-between mb-4",
                h3 {
                    class: "text-lg font-semibold",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    "限价单"
                }
                div {
                    class: "text-xs",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    "当市场价格达到目标价格时自动执行"
                }
            }

            // 买入/卖出选择
            div {
                class: "grid grid-cols-2 gap-2 mb-4",
                button {
                    class: "p-3 rounded-lg transition-all",
                    style: format!(
                        "background: {}; border: 1px solid {}; color: {};",
                        if order_type_val == LimitOrderType::Buy {
                            Colors::TECH_PRIMARY
                        } else {
                            Colors::BG_PRIMARY
                        },
                        if order_type_val == LimitOrderType::Buy {
                            Colors::TECH_PRIMARY
                        } else {
                            Colors::BORDER_PRIMARY
                        },
                        if order_type_val == LimitOrderType::Buy {
                            "white"
                        } else {
                            Colors::TEXT_PRIMARY
                        }
                    ),
                    onclick: move |_| {
                        order_type.set(LimitOrderType::Buy);
                    },
                    "买入"
                }
                button {
                    class: "p-3 rounded-lg transition-all",
                    style: format!(
                        "background: {}; border: 1px solid {}; color: {};",
                        if order_type_val == LimitOrderType::Sell {
                            Colors::TECH_PRIMARY
                        } else {
                            Colors::BG_PRIMARY
                        },
                        if order_type_val == LimitOrderType::Sell {
                            Colors::TECH_PRIMARY
                        } else {
                            Colors::BORDER_PRIMARY
                        },
                        if order_type_val == LimitOrderType::Sell {
                            "white"
                        } else {
                            Colors::TEXT_PRIMARY
                        }
                    ),
                    onclick: move |_| {
                        order_type.set(LimitOrderType::Sell);
                    },
                    "卖出"
                }
            }

            // 代币选择提示
            div {
                class: "p-3 rounded mb-4",
                style: format!("background: rgba(59, 130, 246, 0.1); border: 1px solid rgba(59, 130, 246, 0.3);"),
                div {
                    class: "text-xs",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    if order_type_val == LimitOrderType::Buy {
                        "买入限价单：当市场价格 ≤ 限价时，自动买入代币"
                    } else {
                        "卖出限价单：当市场价格 ≥ 限价时，自动卖出代币"
                    }
                }
            }

            // 数量输入
            div {
                label {
                    class: "block text-sm font-medium mb-2",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    if order_type_val == LimitOrderType::Buy {
                        "买入数量（法币）"
                    } else {
                        "卖出数量（代币）"
                    }
                }
                input {
                    class: "w-full p-3 rounded-lg",
                    style: format!("background: {}; border: 1px solid {}; color: {};",
                        Colors::BG_PRIMARY, Colors::BORDER_PRIMARY, Colors::TEXT_PRIMARY),
                    r#type: "number",
                    value: "{amount.read()}",
                    oninput: move |e| amount.set(e.value()),
                    placeholder: "0.0",
                    step: "0.000001"
                }
            }

            // 限价输入
            div {
                label {
                    class: "block text-sm font-medium mb-2",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    "限价（目标价格）"
                }
                div {
                    class: "flex items-center gap-2",
                    input {
                        class: "flex-1 p-3 rounded-lg",
                        style: format!("background: {}; border: 1px solid {}; color: {};",
                            Colors::BG_PRIMARY, Colors::BORDER_PRIMARY, Colors::TEXT_PRIMARY),
                        r#type: "number",
                        value: "{limit_price.read()}",
                        oninput: move |e| limit_price.set(e.value()),
                        placeholder: "0.0",
                        step: "0.000001"
                    }
                    if let Some(to) = to_token.read().as_ref() {
                        span {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "{to.symbol}"
                        }
                    }
                }
                div {
                    class: "text-xs mt-1",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    if order_type_val == LimitOrderType::Buy {
                        "当市场价格 ≤ 此价格时自动买入"
                    } else {
                        "当市场价格 ≥ 此价格时自动卖出"
                    }
                }
            }

            // 有效期选择
            div {
                label {
                    class: "block text-sm font-medium mb-2",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    "有效期"
                }
                div {
                    class: "grid grid-cols-4 gap-2",
                    for days in [1, 7, 30, 90] {
                        button {
                            class: "p-2 text-sm rounded",
                            style: format!(
                                "background: {}; border: 1px solid {}; color: {};",
                                if *expiry_days.read() == days {
                                    Colors::TECH_PRIMARY
                                } else {
                                    Colors::BG_PRIMARY
                                },
                                if *expiry_days.read() == days {
                                    Colors::TECH_PRIMARY
                                } else {
                                    Colors::BORDER_PRIMARY
                                },
                                if *expiry_days.read() == days {
                                    "white"
                                } else {
                                    Colors::TEXT_PRIMARY
                                }
                            ),
                            onclick: move |_| expiry_days.set(days),
                            if days == 1 {
                                "1天"
                            } else if days == 7 {
                                "7天"
                            } else if days == 30 {
                                "30天"
                            } else {
                                "90天"
                            }
                        }
                    }
                }
            }

            // 订单摘要
            if !amount.read().is_empty() && !limit_price.read().is_empty() {
                div {
                    class: "p-4 rounded-lg mt-4",
                    style: format!("background: {}; border: 1px solid {};", Colors::BG_PRIMARY, Colors::BORDER_PRIMARY),
                    h4 {
                        class: "text-sm font-semibold mb-2",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "订单摘要"
                    }
                    div {
                        class: "space-y-1 text-xs",
                        div {
                            class: "flex justify-between",
                            span { style: format!("color: {};", Colors::TEXT_SECONDARY), "类型" }
                            span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{order_type_val.label()}" }
                        }
                        div {
                            class: "flex justify-between",
                            span { style: format!("color: {};", Colors::TEXT_SECONDARY), "数量" }
                            span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{amount.read()}" }
                        }
                        div {
                            class: "flex justify-between",
                            span { style: format!("color: {};", Colors::TEXT_SECONDARY), "限价" }
                            span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{limit_price.read()}" }
                        }
                        div {
                            class: "flex justify-between",
                            span { style: format!("color: {};", Colors::TEXT_SECONDARY), "有效期" }
                            span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{expiry_days.read()} 天" }
                        }
                    }
                }
            }

            // 错误消息
            if let Some(err) = error_message.read().as_ref() {
                div {
                    class: "p-3 rounded-lg mt-4",
                    style: format!("background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3);"),
                    div {
                        class: "text-sm",
                        style: format!("color: rgb(239, 68, 68);"),
                        {err.clone()}
                    }
                }
            }

            // 提交按钮
            Button {
                variant: ButtonVariant::Primary,
                size: ButtonSize::Large,
                onclick: move |_| {
                    if let Some(handler) = on_submit {
                        let order_type_val = *order_type.read();
                        let amount_val = amount.read().clone();
                        let limit_price_val = limit_price.read().clone();
                        let expiry_days_val = *expiry_days.read();
                        handler.call((order_type_val, amount_val, limit_price_val, "".to_string(), expiry_days_val));
                    }
                },
                disabled: amount.read().is_empty()
                    || limit_price.read().is_empty()
                    || from_token.read().is_none()
                    || to_token.read().is_none()
                    || *loading.read(),
                loading: *loading.read(),
                class: "w-full mt-4",
                if *loading.read() {
                    "创建限价单中..."
                } else {
                    "创建限价单"
                }
            }
        }
    }
}
