//! Order List Component - 订单列表组件
//! 显示法币订单列表（充值/提现）

use crate::components::molecules::order_tracking::OrderStatus;
use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    Onramp,  // 充值
    Offramp, // 提现
}

impl OrderType {
    pub fn label(&self) -> &'static str {
        match self {
            OrderType::Onramp => "充值",
            OrderType::Offramp => "提现",
        }
    }
}

/// 订单列表项
#[derive(Debug, Clone, PartialEq)]
pub struct OrderListItem {
    pub order_id: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub amount: String,
    pub currency: String,
    pub token_symbol: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
}

/// 订单列表组件属性
#[derive(Props, PartialEq, Clone)]
pub struct OrderListProps {
    /// 订单列表
    pub orders: Vec<OrderListItem>,
    /// 是否加载中
    #[props(default = false)]
    pub loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 取消订单回调
    pub on_cancel: Option<EventHandler<String>>,
    /// 重试回调
    pub on_retry: Option<EventHandler<String>>,
    /// 查看详情回调
    pub on_view_details: Option<EventHandler<String>>,
}

/// 订单列表组件
#[component]
pub fn OrderList(props: OrderListProps) -> Element {
    if props.loading {
        return rsx! {
            div {
                class: "space-y-4 py-8",
                // 骨架屏加载效果
                for _ in 0..3 {
                    div {
                        class: "p-4 rounded-lg animate-pulse",
                        style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                        div {
                            class: "flex items-center justify-between mb-3",
                            div {
                                class: "h-4 w-32 rounded",
                                style: format!("background: {};", Colors::BG_PRIMARY),
                            }
                            div {
                                class: "h-4 w-20 rounded",
                                style: format!("background: {};", Colors::BG_PRIMARY),
                            }
                        }
                        div {
                            class: "h-3 w-48 rounded mb-2",
                            style: format!("background: {};", Colors::BG_PRIMARY),
                        }
                        div {
                            class: "h-3 w-36 rounded",
                            style: format!("background: {};", Colors::BG_PRIMARY),
                        }
                    }
                }
                div {
                    class: "text-center mt-4",
                    div {
                        class: "text-sm",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "正在加载订单..."
                    }
                }
            }
        };
    }

    if let Some(error) = &props.error {
        return rsx! {
            div {
                class: "p-6 rounded-lg",
                style: format!("background: {}; border: 1px solid {};",
                    "rgba(239, 68, 68, 0.1)", "rgba(239, 68, 68, 0.3)"),
                div {
                    class: "flex items-start gap-3 mb-3",
                    span {
                        class: "text-2xl",
                        "⚠️"
                    }
                    div {
                        class: "flex-1",
                        div {
                            class: "text-sm font-medium mb-1",
                            style: "color: rgba(239, 68, 68, 1);",
                            "加载订单失败"
                        }
                        div {
                            class: "text-sm",
                            style: "color: rgba(239, 68, 68, 0.9);",
                            "{error}"
                        }
                    }
                }
                button {
                    class: "w-full px-4 py-2 rounded-lg font-medium text-sm transition-all",
                    style: format!("background: {}; color: white;", Colors::TECH_PRIMARY),
                    onclick: {
                        // 重试功能由父组件处理
                        move |_| {
                            // 这里可以触发父组件的刷新
                        }
                    },
                    "🔄 重试"
                }
            }
        };
    }

    if props.orders.is_empty() {
        return rsx! {
            div {
                class: "text-center py-16",
                div {
                    class: "mb-6",
                    style: format!("color: {}; font-size: 64px;", Colors::TEXT_SECONDARY),
                    "📋"
                }
                div {
                    class: "text-lg font-semibold mb-2",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    "暂无订单"
                }
                div {
                    class: "text-sm mb-6",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    "您还没有任何法币订单记录"
                }
                div {
                    class: "text-xs",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    "提示：您可以尝试购买稳定币或提现来创建订单"
                }
            }
        };
    }

    let orders_clone = props.orders.clone();
    rsx! {
        div {
            class: "space-y-4",
            for order in orders_clone {
                div {
                    class: "p-4 rounded-lg",
                    style: format!("background: {}; border: 1px solid {};",
                        Colors::BG_PRIMARY, Colors::BORDER_PRIMARY),
                    // 订单头部
                    div {
                        class: "flex items-start justify-between mb-3",
                        div {
                            class: "flex-1",
                            div {
                                class: "flex items-center gap-2 mb-1",
                                span {
                                    class: "px-2 py-1 rounded text-xs font-medium",
                                    style: format!("background: {}; color: {};",
                                        if order.order_type == OrderType::Onramp {
                                            "rgba(34, 197, 94, 0.1)"
                                        } else {
                                            "rgba(59, 130, 246, 0.1)"
                                        },
                                        if order.order_type == OrderType::Onramp {
                                            "rgba(34, 197, 94, 1)"
                                        } else {
                                            "rgba(59, 130, 246, 1)"
                                        }
                                    ),
                                    "{order.order_type.label()}"
                                }
                                span {
                                    class: "px-2 py-1 rounded text-xs font-medium",
                                    style: format!("background: {}; color: {};",
                                        order.status.bg_color(), order.status.color()
                                    ),
                                    "{order.status.label()}"
                                }
                            }
                            div {
                                class: "text-lg font-semibold",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                "{order.amount} {order.currency}"
                                if let Some(token) = &order.token_symbol {
                                    span {
                                        class: "text-sm font-normal ml-2",
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        "({token})"
                                    }
                                }
                            }
                        }
                        div {
                            class: "text-right",
                            div {
                                class: "text-xs",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "订单号"
                            }
                            div {
                                class: "text-xs font-mono",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                {
                                    if order.order_id.len() > 8 {
                                        format!("{}...", &order.order_id[..8])
                                    } else {
                                        order.order_id.clone()
                                    }
                                }
                            }
                        }
                    }

                    // 订单信息
                    div {
                        class: "grid grid-cols-2 gap-4 text-sm mb-3",
                        div {
                            div {
                                class: "text-xs mb-1",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "创建时间"
                            }
                            div {
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                "{order.created_at}"
                            }
                        }
                        if let Some(updated) = &order.updated_at {
                            div {
                                div {
                                    class: "text-xs mb-1",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "更新时间"
                                }
                                div {
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    "{updated}"
                                }
                            }
                        }
                    }

                    // 错误信息
                    if let Some(error) = &order.error_message {
                        div {
                            class: "p-2 rounded mb-3",
                            style: format!("background: {};", "rgba(239, 68, 68, 0.1)"),
                            div {
                                class: "text-xs",
                                style: "color: rgba(239, 68, 68, 1);",
                                "{error}"
                            }
                        }
                    }

                    // 操作按钮
                    div {
                        class: "flex items-center gap-2 flex-wrap",
                        // 查看详情按钮（所有状态）
                        if let Some(ref on_view_details) = props.on_view_details {
                            {
                                let order_id = order.order_id.clone();
                                let handler = *on_view_details;
                                rsx! {
                                    button {
                                        class: "px-3 py-1.5 rounded text-xs font-medium transition-all",
                                        style: format!(
                                            "background: {}; color: {}; border: 1px solid {};",
                                            Colors::BG_SECONDARY,
                                            Colors::TEXT_PRIMARY,
                                            Colors::BORDER_PRIMARY
                                        ),
                                        onclick: move |_| {
                                            handler.call(order_id.clone());
                                        },
                                        "查看详情"
                                    }
                                }
                            }
                        }
                        // 取消按钮（待处理状态）
                        if matches!(order.status, OrderStatus::Pending) {
                            if let Some(ref on_cancel) = props.on_cancel {
                                {
                                    let order_id = order.order_id.clone();
                                    let handler = *on_cancel;
                                    rsx! {
                                        button {
                                            class: "px-3 py-1.5 rounded text-xs font-medium transition-all",
                                            style: format!(
                                                "background: {}; color: {}; border: 1px solid {};",
                                                Colors::BG_PRIMARY,
                                                Colors::TEXT_PRIMARY,
                                                Colors::BORDER_PRIMARY
                                            ),
                                            onclick: move |_| {
                                                handler.call(order_id.clone());
                                            },
                                            "取消订单"
                                        }
                                    }
                                }
                            }
                        }
                        // 重试按钮（失败状态）
                        if matches!(order.status, OrderStatus::Failed) {
                            if let Some(ref on_retry) = props.on_retry {
                                {
                                    let order_id = order.order_id.clone();
                                    let handler = *on_retry;
                                    rsx! {
                                        button {
                                            class: "px-3 py-1.5 rounded text-xs font-medium transition-all",
                                            style: format!(
                                                "background: {}; color: white;",
                                                Colors::TECH_PRIMARY
                                            ),
                                            onclick: move |_| {
                                                handler.call(order_id.clone());
                                            },
                                            "重试"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
