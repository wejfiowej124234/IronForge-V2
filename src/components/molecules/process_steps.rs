//! Process Steps - 流程步骤指示器组件
//! 用于显示多步骤流程的进度

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 流程步骤指示器组件
#[component]
pub fn ProcessSteps(
    /// 当前步骤（从1开始）
    current_step: u8,
    /// 总步骤数
    total_steps: u8,
    /// 步骤标签
    steps: Vec<String>,
) -> Element {
    let current_step_usize = current_step as usize;
    rsx! {
        div {
            class: "w-full",
            div {
                class: "relative flex items-center justify-between mb-4",
                for (index, step_label) in steps.iter().enumerate() {
                    div {
                        class: "relative flex flex-col items-center flex-1 z-10",
                        // 步骤圆圈
                        div {
                            class: "w-10 h-10 rounded-full flex items-center justify-center font-semibold transition-all",
                            style: format!(
                                "background: {}; border: 2px solid {}; color: {};",
                                if (index + 1) <= current_step_usize {
                                    Colors::TECH_PRIMARY
                                } else {
                                    Colors::BG_PRIMARY
                                },
                                if (index + 1) <= current_step_usize {
                                    Colors::TECH_PRIMARY
                                } else {
                                    Colors::BORDER_PRIMARY
                                },
                                if (index + 1) <= current_step_usize {
                                    "white"
                                } else {
                                    Colors::TEXT_SECONDARY
                                }
                            ),
                            {
                                if (index + 1) < current_step_usize {
                                    "✓".to_string()
                                } else {
                                    (index + 1).to_string()
                                }
                            }
                        }
                        // 步骤标签
                        div {
                            class: "mt-2 text-xs text-center max-w-[80px]",
                            style: format!(
                                "color: {}; font-weight: {};",
                                if (index + 1) == current_step_usize {
                                    Colors::TEXT_PRIMARY
                                } else {
                                    Colors::TEXT_SECONDARY
                                },
                                if (index + 1) == current_step_usize { "600" } else { "400" }
                            ),
                            {step_label.as_str()}
                        }
                    }
                    // 连接线（除了最后一个）
                    if index < steps.len() - 1 {
                        div {
                            class: "absolute top-5",
                            style: format!(
                                "left: {}%; width: {}%; height: 2px; background: {}; z-0;",
                                (index as f64 + 0.5) * 100.0 / (steps.len() as f64),
                                100.0 / (steps.len() as f64),
                                if (index + 1) < current_step_usize {
                                    Colors::TECH_PRIMARY
                                } else {
                                    Colors::BORDER_PRIMARY
                                }
                            ),
                        }
                    }
                }
            }
        }
    }
}
