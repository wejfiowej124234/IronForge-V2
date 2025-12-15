//! Price Change Indicator - 价格变化提示组件
//! 显示报价变化百分比，提供视觉反馈（上涨/下跌）

use dioxus::prelude::*;

/// 价格变化信息
#[derive(Debug, Clone, PartialEq)]
pub struct PriceChangeInfo {
    /// 变化百分比（正数表示上涨，负数表示下跌）
    pub change_percent: f64,
    /// 变化方向
    pub direction: PriceChangeDirection,
    /// 时间戳（用于显示）
    pub timestamp: u64,
}

/// 价格变化方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PriceChangeDirection {
    Up,       // 上涨
    Down,     // 下跌
    NoChange, // 无变化
}

impl PriceChangeInfo {
    /// 创建价格变化信息
    pub fn new(previous_price: f64, current_price: f64, timestamp: u64) -> Self {
        let change_percent = if previous_price > 0.0 {
            ((current_price - previous_price) / previous_price) * 100.0
        } else {
            0.0
        };

        let direction = if change_percent > 0.001 {
            PriceChangeDirection::Up
        } else if change_percent < -0.001 {
            PriceChangeDirection::Down
        } else {
            PriceChangeDirection::NoChange
        };

        Self {
            change_percent,
            direction,
            timestamp,
        }
    }

    /// 格式化变化百分比
    pub fn format_change(&self) -> String {
        if self.change_percent.abs() < 0.01 {
            return "无变化".to_string();
        }

        let sign = if self.change_percent > 0.0 { "+" } else { "" };
        format!("{}{:.2}%", sign, self.change_percent)
    }
}

/// 价格变化提示组件
#[component]
pub fn PriceChangeIndicator(
    /// 价格变化信息
    change_info: Option<PriceChangeInfo>,
    /// 是否显示动画
    #[props(default = true)]
    show_animation: bool,
) -> Element {
    let change_info = change_info.clone();

    match change_info {
        Some(info) => {
            if info.direction == PriceChangeDirection::NoChange {
                return rsx! { div {} };
            }

            let (bg_color, text_color, icon) = match info.direction {
                PriceChangeDirection::Up => (
                    "rgba(16, 185, 129, 0.1)", // 绿色背景
                    "rgb(16, 185, 129)",       // 绿色文字
                    "↑",
                ),
                PriceChangeDirection::Down => (
                    "rgba(239, 68, 68, 0.1)", // 红色背景
                    "rgb(239, 68, 68)",       // 红色文字
                    "↓",
                ),
                PriceChangeDirection::NoChange => return rsx! { div {} },
            };

            rsx! {
                div {
                    class: "flex items-center gap-2 px-3 py-2 rounded-lg transition-all",
                    style: format!(
                        "background: {}; color: {}; border: 1px solid {};",
                        bg_color,
                        text_color,
                        if info.direction == PriceChangeDirection::Up {
                            "rgba(16, 185, 129, 0.3)"
                        } else {
                            "rgba(239, 68, 68, 0.3)"
                        }
                    ),
                    span {
                        class: "text-lg font-bold",
                        {icon}
                    }
                    span {
                        class: "text-sm font-medium",
                        {info.format_change()}
                    }
                    span {
                        class: "text-xs opacity-75",
                        "价格更新"
                    }
                }
            }
        }
        None => rsx! { div {} },
    }
}
