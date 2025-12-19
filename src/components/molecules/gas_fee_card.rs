//! Gas Fee Card - Gas费显示卡片组件
//! 显示Gas费估算信息，支持加载状态

use crate::services::gas::{gas_fee_eth_from_max_fee_per_gas_gwei, GasEstimate};
use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// Gas费显示卡片组件（含平台服务费）
#[component]
pub fn GasFeeCard(
    gas_estimate: Option<GasEstimate>,
    platform_fee: Option<f64>,
    is_loading: bool,
) -> Element {
    rsx! {
        div {
            class: "mb-6",
            label {
                class: "block text-sm font-medium mb-2",
                style: format!("color: {};", Colors::TEXT_SECONDARY),
                "交易费用明细"
            }
            if is_loading {
                div {
                    class: "p-4 rounded-lg",
                    style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                    div {
                        class: "text-sm",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        "正在获取最优Gas费..."
                    }
                }
            } else if let Some(gas) = gas_estimate {
                div {
                    class: "p-4 rounded-lg",
                    style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                    div {
                        class: "space-y-2",
                        // Gas费（区块链网络费用）
                        div {
                            class: "flex justify-between items-center",
                            span {
                                class: "text-sm flex items-center gap-1",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                span { "⛽" }
                                span { "Gas费" }
                            }
                            span {
                                class: "text-sm font-semibold",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {
                                    let gas_fee = gas_fee_eth_from_max_fee_per_gas_gwei(
                                        gas.max_fee_per_gas_gwei,
                                        21_000,
                                    );
                                    format!("{:.8} ETH", gas_fee)
                                }
                            }
                        }
                        // 平台服务费
                        if let Some(fee) = platform_fee {
                            div {
                                class: "flex justify-between items-center",
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "平台服务费"
                                }
                                span {
                                    class: "text-sm font-semibold",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    {format!("{:.6} ETH", fee)}
                                }
                            }
                        }
                        // 总费用
                        div {
                            class: "flex justify-between items-center pt-2 border-t",
                            style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                            span {
                                class: "text-sm font-semibold flex items-center gap-1",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                span { "💰" }
                                span { "总计" }
                            }
                            span {
                                class: "text-sm font-bold",
                                style: format!("color: {};", Colors::TECH_PRIMARY),
                                {
                                    let gas_fee = gas_fee_eth_from_max_fee_per_gas_gwei(
                                        gas.max_fee_per_gas_gwei,
                                        21_000,
                                    );
                                    let total = gas_fee + platform_fee.unwrap_or(0.0);
                                    format!("{:.8} ETH", total)
                                }
                            }
                        }
                        // 预估时间
                        div {
                            class: "flex justify-between items-center",
                            span {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "预估时间"
                            }
                            span {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_TERTIARY),
                                {format!("约 {:.0} 秒", gas.estimated_time_seconds)}
                            }
                        }
                        // 透明度说明
                        div {
                            class: "mt-2 pt-2 border-t",
                            style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                            div {
                                class: "flex items-center gap-2",
                                span {
                                    class: "text-xs",
                                    style: format!("color: {};", Colors::TECH_PRIMARY),
                                    "💡"
                                }
                                span {
                                    class: "text-xs",
                                    style: format!("color: {};", Colors::TEXT_TERTIARY),
                                    "Gas费由区块链收取，服务费由平台收取（按交易金额0.1%-1.0%动态计算）"
                                }
                            }
                        }
                    }
                }
            } else {
                div {
                    class: "p-4 rounded-lg",
                    style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                    div {
                        class: "text-sm",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        "Gas费将在发送时自动计算"
                    }
                }
            }
        }
    }
}
