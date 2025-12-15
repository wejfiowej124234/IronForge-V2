//! Price Chart - 价格图表组件
//! 显示代币价格走势图（基础版本）

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 价格数据点
#[derive(Debug, Clone, PartialEq)]
pub struct PriceDataPoint {
    pub timestamp: u64, // Unix timestamp (seconds)
    pub price: f64,
    pub volume: Option<f64>,
}

/// 价格图表组件（基础版本）
/// 使用SVG绘制简单的价格走势图
#[component]
pub fn PriceChart(
    /// 代币符号
    token_symbol: String,
    /// 价格数据点列表
    data: Vec<PriceDataPoint>,
    /// 时间范围（小时）
    time_range_hours: Option<u32>,
) -> Element {
    if data.is_empty() {
        return rsx! {
            div {
                class: "p-6 rounded-lg text-center",
                style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                div {
                    class: "text-sm",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    "暂无价格数据"
                }
            }
        };
    }

    // 计算价格范围
    let min_price = data.iter().map(|p| p.price).fold(f64::INFINITY, f64::min);
    let max_price = data
        .iter()
        .map(|p| p.price)
        .fold(f64::NEG_INFINITY, f64::max);
    let price_range = max_price - min_price;
    let padding = price_range * 0.1; // 10% padding

    // 图表尺寸
    let width = 600.0;
    let height = 300.0;
    let padding_x = 60.0;
    let padding_y = 40.0;
    let chart_width = width - padding_x * 2.0;
    let chart_height = height - padding_y * 2.0;

    // 计算数据点位置
    let mut points = Vec::new();
    for (i, point) in data.iter().enumerate() {
        let x = padding_x + (i as f64 / (data.len() - 1) as f64) * chart_width;
        let y = padding_y + chart_height
            - ((point.price - min_price + padding) / (price_range + padding * 2.0)) * chart_height;
        points.push((x, y));
    }

    // 生成路径
    let path_data = if points.len() > 1 {
        let mut path = format!("M {} {}", points[0].0, points[0].1);
        for point in points.iter().skip(1) {
            path.push_str(&format!(" L {} {}", point.0, point.1));
        }
        path
    } else {
        String::new()
    };

    // 格式化价格
    let format_price = |price: f64| -> String {
        if price >= 1.0 {
            format!("{:.2}", price)
        } else if price >= 0.01 {
            format!("{:.4}", price)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        } else {
            format!("{:.6}", price)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        }
    };

    // 预先计算Y轴标签（包含所有格式化字符串）
    let mut y_labels = Vec::new();
    for idx in 0..=4 {
        let price_val =
            min_price + padding + (price_range + padding * 2.0) * (4 - idx) as f64 / 4.0;
        let y_val = padding_y + (idx as f64 / 4.0) * chart_height;
        let price_text = format_price(price_val);
        let y_str = format!("{}", y_val);
        let y_plus_4 = format!("{}", y_val + 4.0);
        let x2_val = width - padding_x;
        let x2_str = format!("{}", x2_val);
        let x_text = padding_x - 10.0;
        let x_text_str = format!("{}", x_text);
        y_labels.push((y_str, y_plus_4, x2_str, x_text_str, price_text));
    }

    rsx! {
        div {
            class: "p-6 rounded-lg",
            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),

            // 标题
            div {
                class: "mb-4",
                h3 {
                    class: "text-lg font-semibold",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    "{token_symbol} 价格走势"
                }
                if let Some(range) = time_range_hours {
                    div {
                        class: "text-xs mt-1",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "过去 {range} 小时"
                    }
                }
            }

            // 价格信息
            if let (Some(first), Some(last)) = (data.first(), data.last()) {
                div {
                    class: "flex items-center justify-between mb-4",
                    div {
                        span {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "当前价格: "
                        }
                        span {
                            class: "text-lg font-bold ml-2",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "${format_price(last.price.clone())}"
                        }
                    }
                    {
                        let change_percent = if last.price > first.price {
                            ((last.price - first.price) / first.price) * 100.0
                        } else if last.price < first.price {
                            ((first.price - last.price) / first.price) * 100.0
                        } else {
                            0.0
                        };
                        let change_text = format!("{:.2}%", change_percent);

                        if last.price > first.price {
                            rsx! {
                                span {
                                    class: "text-sm px-2 py-1 rounded",
                                    style: format!("background: rgba(34, 197, 94, 0.1); color: rgb(34, 197, 94);"),
                                    "↑ {change_text}"
                                }
                            }
                        } else if last.price < first.price {
                            rsx! {
                                span {
                                    class: "text-sm px-2 py-1 rounded",
                                    style: format!("background: rgba(239, 68, 68, 0.1); color: rgb(239, 68, 68);"),
                                    "↓ {change_text}"
                                }
                            }
                        } else {
                            rsx! {
                                span {
                                    class: "text-sm px-2 py-1 rounded",
                                    style: format!("background: {}; color: {};", Colors::BG_PRIMARY, Colors::TEXT_SECONDARY),
                                    "0.00%"
                                }
                            }
                        }
                    }
                }
            }

            // SVG图表
            div {
                class: "overflow-x-auto",
                svg {
                    width: "{width}",
                    height: "{height}",
                    view_box: format!("0 0 {} {}", width, height),
                    style: "max-width: 100%; height: auto;",

                    // 背景网格
                    defs {
                        pattern {
                            id: "grid",
                            width: "40",
                            height: "40",
                            pattern_units: "userSpaceOnUse",
                            rect {
                                width: "40",
                                height: "40",
                                fill: "none",
                            }
                            path {
                                d: "M 40 0 L 0 0 0 40",
                                fill: "none",
                                stroke: "{Colors::BORDER_PRIMARY}",
                                stroke_width: "0.5",
                                opacity: "0.3",
                            }
                        }
                    }
                    rect {
                        width: "{width}",
                        height: "{height}",
                        fill: "url(#grid)",
                    }

                    // Y轴标签（价格）
                    if !data.is_empty() {
                        for (y_str, y_plus_4, x2_str, x_text_str, price_text) in y_labels.iter() {
                            g {
                                line {
                                    x1: "{padding_x}",
                                    y1: "{y_str}",
                                    x2: "{x2_str}",
                                    y2: "{y_str}",
                                    stroke: "{Colors::BORDER_PRIMARY}",
                                    stroke_width: "0.5",
                                    opacity: "0.2",
                                }
                                text {
                                    x: "{x_text_str}",
                                    y: "{y_plus_4}",
                                    text_anchor: "end",
                                    font_size: "10",
                                    fill: "{Colors::TEXT_SECONDARY}",
                                    {price_text.clone()}
                                }
                            }
                        }
                    }

                    // 价格线
                    if !path_data.is_empty() {
                        path {
                            d: "{path_data}",
                            fill: "none",
                                stroke: "{Colors::TECH_PRIMARY}",
                                stroke_width: "2",
                        }
                        // 填充区域
                        path {
                            d: "{path_data} L {width - padding_x} {height - padding_y} L {padding_x} {height - padding_y} Z",
                            fill: "url(#gradient)",
                            opacity: "0.2",
                        }
                    }

                    // 渐变定义
                    defs {
                        linearGradient {
                            id: "gradient",
                            x1: "0%",
                            y1: "0%",
                            x2: "0%",
                            y2: "100%",
                            stop {
                                offset: "0%",
                                stop_color: "{Colors::TECH_PRIMARY}",
                                stop_opacity: "0.3",
                            }
                            stop {
                                offset: "100%",
                                stop_color: "{Colors::TECH_PRIMARY}",
                                stop_opacity: "0",
                            }
                        }
                    }
                }
            }

            // 说明
            div {
                class: "mt-4 text-xs text-center",
                style: format!("color: {};", Colors::TEXT_SECONDARY),
                "价格数据仅供参考，实际交易价格可能有所不同"
            }
        }
    }
}
