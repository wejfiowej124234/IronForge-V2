//! Order Tracking Component - 订单跟踪组件
//! 显示订单状态、进度和详细信息

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 订单状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,    // 待处理
    Processing, // 处理中
    Completed,  // 已完成
    Failed,     // 失败
    Cancelled,  // 已取消
    Expired,    // 已过期
}

impl OrderStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => OrderStatus::Pending,
            "processing" => OrderStatus::Processing,
            "completed" => OrderStatus::Completed,
            "failed" => OrderStatus::Failed,
            "cancelled" => OrderStatus::Cancelled,
            "expired" => OrderStatus::Expired,
            _ => OrderStatus::Pending,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "待处理",
            OrderStatus::Processing => "处理中",
            OrderStatus::Completed => "已完成",
            OrderStatus::Failed => "失败",
            OrderStatus::Cancelled => "已取消",
            OrderStatus::Expired => "已过期",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "rgba(59, 130, 246, 1)", // 蓝色
            OrderStatus::Processing => "rgba(234, 179, 8, 1)", // 黄色
            OrderStatus::Completed => "rgba(34, 197, 94, 1)", // 绿色
            OrderStatus::Failed => "rgba(239, 68, 68, 1)",   // 红色
            OrderStatus::Cancelled => "rgba(107, 114, 128, 1)", // 灰色
            OrderStatus::Expired => "rgba(239, 68, 68, 1)",  // 红色
        }
    }

    pub fn bg_color(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "rgba(59, 130, 246, 0.1)",
            OrderStatus::Processing => "rgba(234, 179, 8, 0.1)",
            OrderStatus::Completed => "rgba(34, 197, 94, 0.1)",
            OrderStatus::Failed => "rgba(239, 68, 68, 0.1)",
            OrderStatus::Cancelled => "rgba(107, 114, 128, 0.1)",
            OrderStatus::Expired => "rgba(239, 68, 68, 0.1)",
        }
    }

    pub fn progress(&self) -> u8 {
        match self {
            OrderStatus::Pending => 20,
            OrderStatus::Processing => 60,
            OrderStatus::Completed => 100,
            OrderStatus::Failed => 0,
            OrderStatus::Cancelled => 0,
            OrderStatus::Expired => 0,
        }
    }
}

/// 订单跟踪信息
#[derive(Debug, Clone, PartialEq)]
pub struct OrderTrackingInfo {
    pub order_id: String,
    pub status: OrderStatus,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
    pub payment_url: Option<String>,
    pub tx_hash: Option<String>,
}

/// 订单跟踪组件属性
#[derive(Props, PartialEq, Clone)]
pub struct OrderTrackingProps {
    /// 订单信息
    pub order: OrderTrackingInfo,
    /// 是否显示详细信息
    #[props(default = true)]
    pub show_details: bool,
    /// 是否显示操作按钮
    #[props(default = true)]
    pub show_actions: bool,
    /// 取消订单回调
    pub on_cancel: Option<EventHandler<String>>,
    /// 重试回调
    pub on_retry: Option<EventHandler<String>>,
}

/// 订单跟踪组件
#[component]
pub fn OrderTracking(props: OrderTrackingProps) -> Element {
    let status = props.order.status;
    let progress = status.progress();
    let status_color = status.color();
    let status_bg = status.bg_color();

    rsx! {
        div {
            class: "space-y-4",
            // 订单头部
            div {
                class: "flex items-start justify-between",
                div {
                    class: "flex-1",
                    div {
                        class: "flex items-center gap-3 mb-2",
                        h3 {
                            class: "text-lg font-semibold",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "{props.order.title}"
                        }
                        span {
                            class: "px-3 py-1 rounded-full text-sm font-medium",
                            style: format!("background: {}; color: {};", status_bg, status_color),
                            "{status.label()}"
                        }
                    }
                    if let Some(desc) = &props.order.description {
                        p {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "{desc}"
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
                        class: "text-sm font-mono",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {
                            if props.order.order_id.len() > 12 {
                                format!("{}...{}", &props.order.order_id[..8], &props.order.order_id[props.order.order_id.len()-4..])
                            } else {
                                props.order.order_id.clone()
                            }
                        }
                    }
                }
            }

            // 状态机可视化
            div {
                class: "space-y-3",
                // 进度条
                if matches!(status, OrderStatus::Pending | OrderStatus::Processing) {
                    div {
                        class: "space-y-2",
                        div {
                            class: "flex items-center justify-between text-xs",
                            span {
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "处理进度"
                            }
                            span {
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "{progress}%"
                            }
                        }
                        div {
                            class: "w-full h-2 rounded-full overflow-hidden",
                            style: format!("background: {};", Colors::BG_SECONDARY),
                            div {
                                class: "h-full transition-all duration-500",
                                style: format!(
                                    "width: {}%; background: {};",
                                    progress,
                                    status_color
                                ),
                            }
                        }
                    }
                }

                // 状态步骤
                div {
                    class: "flex items-center justify-between",
                    // 步骤1: 待处理
                    div {
                        class: "flex flex-col items-center flex-1",
                        div {
                            class: "w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium mb-1",
                            style: format!(
                                "background: {}; color: {}; border: 2px solid {};",
                                if matches!(status, OrderStatus::Pending | OrderStatus::Processing | OrderStatus::Completed) {
                                    status_color
                                } else {
                                    Colors::BG_SECONDARY
                                },
                                if matches!(status, OrderStatus::Pending | OrderStatus::Processing | OrderStatus::Completed) {
                                    "white"
                                } else {
                                    Colors::TEXT_SECONDARY
                                },
                                if matches!(status, OrderStatus::Pending | OrderStatus::Processing | OrderStatus::Completed) {
                                    status_color
                                } else {
                                    Colors::BORDER_PRIMARY
                                }
                            ),
                            "1"
                        }
                        span {
                            class: "text-xs text-center",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "待处理"
                        }
                    }
                    // 连接线
                    div {
                        class: "flex-1 h-0.5 mx-2",
                        style: format!(
                            "background: {};",
                            if matches!(status, OrderStatus::Processing | OrderStatus::Completed) {
                                status_color
                            } else {
                                Colors::BORDER_PRIMARY
                            }
                        ),
                    }
                    // 步骤2: 处理中
                    div {
                        class: "flex flex-col items-center flex-1",
                        div {
                            class: "w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium mb-1",
                            style: format!(
                                "background: {}; color: {}; border: 2px solid {};",
                                if matches!(status, OrderStatus::Processing | OrderStatus::Completed) {
                                    status_color
                                } else {
                                    Colors::BG_SECONDARY
                                },
                                if matches!(status, OrderStatus::Processing | OrderStatus::Completed) {
                                    "white"
                                } else {
                                    Colors::TEXT_SECONDARY
                                },
                                if matches!(status, OrderStatus::Processing | OrderStatus::Completed) {
                                    status_color
                                } else {
                                    Colors::BORDER_PRIMARY
                                }
                            ),
                            "2"
                        }
                        span {
                            class: "text-xs text-center",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "处理中"
                        }
                    }
                    // 连接线
                    div {
                        class: "flex-1 h-0.5 mx-2",
                        style: format!(
                            "background: {};",
                            if matches!(status, OrderStatus::Completed) {
                                status_color
                            } else {
                                Colors::BORDER_PRIMARY
                            }
                        ),
                    }
                    // 步骤3: 已完成
                    div {
                        class: "flex flex-col items-center flex-1",
                        div {
                            class: "w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium mb-1",
                            style: format!(
                                "background: {}; color: {}; border: 2px solid {};",
                                if matches!(status, OrderStatus::Completed) {
                                    status_color
                                } else {
                                    Colors::BG_SECONDARY
                                },
                                if matches!(status, OrderStatus::Completed) {
                                    "white"
                                } else {
                                    Colors::TEXT_SECONDARY
                                },
                                if matches!(status, OrderStatus::Completed) {
                                    status_color
                                } else {
                                    Colors::BORDER_PRIMARY
                                }
                            ),
                            if matches!(status, OrderStatus::Completed) {
                                "✓"
                            } else {
                                "3"
                            }
                        }
                        span {
                            class: "text-xs text-center",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "已完成"
                        }
                    }
                }
            }

            // 详细信息
            if props.show_details {
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-4 p-4 rounded-lg",
                    style: format!("background: {};", Colors::BG_SECONDARY),
                    div {
                        div {
                            class: "text-xs mb-1",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "创建时间"
                        }
                        div {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "{props.order.created_at}"
                        }
                    }
                    if let Some(updated) = &props.order.updated_at {
                        div {
                            div {
                                class: "text-xs mb-1",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "更新时间"
                            }
                            div {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                "{updated}"
                            }
                        }
                    }
                    if let Some(completed) = &props.order.completed_at {
                        div {
                            div {
                                class: "text-xs mb-1",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "完成时间"
                            }
                            div {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                "{completed}"
                            }
                        }
                    }
                    if let Some(tx_hash) = &props.order.tx_hash {
                        div {
                            div {
                                class: "text-xs mb-1",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "交易哈希"
                            }
                            div {
                                class: "text-sm font-mono break-all",
                                style: format!("color: {};", Colors::TECH_PRIMARY),
                                {
                                    if tx_hash.len() > 16 {
                                        format!("{}...{}", &tx_hash[..8], &tx_hash[tx_hash.len()-8..])
                                    } else {
                                        tx_hash.clone()
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 错误信息
            if let Some(error) = &props.order.error_message {
                div {
                    class: "p-3 rounded-lg",
                    style: format!("background: {}; border: 1px solid {};",
                        "rgba(239, 68, 68, 0.1)", "rgba(239, 68, 68, 0.3)"),
                    div {
                        class: "text-sm font-medium mb-1",
                        style: "color: rgba(239, 68, 68, 1);",
                        "错误信息"
                    }
                    div {
                        class: "text-sm",
                        style: "color: rgba(239, 68, 68, 0.9);",
                        "{error}"
                    }
                }
            }

            // 操作按钮
            if props.show_actions {
                div {
                    class: "flex items-center gap-3",
                    if matches!(status, OrderStatus::Pending) {
                        if let Some(payment_url) = &props.order.payment_url {
                            a {
                                href: payment_url.clone(),
                                target: "_blank",
                                class: "px-4 py-2 rounded-lg font-medium text-sm transition-all",
                                style: format!(
                                    "background: {}; color: white;",
                                    Colors::TECH_PRIMARY
                                ),
                                "前往支付"
                            }
                        }
                        if let Some(ref on_cancel) = props.on_cancel {
                            {
                                let order_id = props.order.order_id.clone();
                                let handler = on_cancel.clone();
                                rsx! {
                                    button {
                                        class: "px-4 py-2 rounded-lg font-medium text-sm transition-all",
                                        style: format!(
                                            "background: {}; color: {}; border: 1px solid {};",
                                            Colors::BG_PRIMARY,
                                            Colors::TEXT_PRIMARY,
                                            Colors::BORDER_PRIMARY
                                        ),
                                        onclick: move |_| {
                                            handler.call(order_id.clone());
                                        },
                                        "Cancel Order"
                                    }
                                }
                            }
                        }
                    }
                    if matches!(status, OrderStatus::Failed) {
                        if let Some(ref on_retry) = props.on_retry {
                            {
                                let order_id = props.order.order_id.clone();
                                let handler = on_retry.clone();
                                rsx! {
                                    button {
                                        class: "px-4 py-2 rounded-lg font-medium text-sm transition-all",
                                        style: format!(
                                            "background: {}; color: white;",
                                            Colors::TECH_PRIMARY
                                        ),
                                        onclick: move |_| {
                                            handler.call(order_id.clone());
                                        },
                                        "Retry"
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
