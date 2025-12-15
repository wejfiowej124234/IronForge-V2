//! Onboarding Tour - 新手引导组件
//! 交互式教程，引导新用户了解功能
#![allow(dead_code)]

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 新手引导步骤
#[derive(Debug, Clone)]
pub struct TourStep {
    pub id: String,
    pub title: String,
    pub content: String,
    pub target_selector: Option<String>, // CSS选择器，用于高亮目标元素
}

/// 新手引导组件
#[component]
pub fn OnboardingTour(
    /// 是否显示引导
    show: Signal<bool>,
    /// 当前步骤索引
    current_step: Signal<usize>,
    /// 引导步骤列表
    steps: Signal<Vec<TourStep>>,
    /// 关闭回调
    on_close: Option<EventHandler<()>>,
    /// 下一步回调
    on_next: Option<EventHandler<()>>,
    /// 上一步回调
    on_prev: Option<EventHandler<()>>,
    /// 跳过回调
    on_skip: Option<EventHandler<()>>,
) -> Element {
    let show_val = *show.read();
    let current_step_val = *current_step.read();
    let steps_val = steps.read();

    if !show_val || steps_val.is_empty() || current_step_val >= steps_val.len() {
        return rsx! { div {} };
    }

    let step = &steps_val[current_step_val];
    let is_first = current_step_val == 0;
    let is_last = current_step_val == steps_val.len() - 1;

    rsx! {
        // 遮罩层
        div {
            class: "fixed inset-0 z-50",
            style: "background: rgba(0, 0, 0, 0.5);",
            onclick: move |_| {
                if let Some(handler) = on_close {
                    handler.call(());
                }
            },

            // 引导卡片
            div {
                class: "absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-full max-w-md mx-4",
                style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                onclick: move |e| {
                    e.stop_propagation();
                },

                div {
                    class: "p-6 rounded-lg",
                    // 标题栏
                    div {
                        class: "flex items-center justify-between mb-4",
                        h3 {
                            class: "text-lg font-semibold",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            {step.title.clone()}
                        }
                        button {
                            class: "text-xl leading-none",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            onclick: move |_| {
                                if let Some(handler) = on_close {
                                    handler.call(());
                                }
                            },
                            "×"
                        }
                    }

                    // 内容
                    div {
                        class: "mb-6",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {step.content.clone()}
                    }

                    // 步骤指示器
                    div {
                        class: "flex items-center justify-center gap-2 mb-4",
                        for (idx, _) in steps_val.iter().enumerate() {
                            div {
                                class: "w-2 h-2 rounded-full transition-all",
                                style: format!(
                                    "background: {};",
                                    if idx == current_step_val {
                                        Colors::TECH_PRIMARY
                                    } else {
                                        Colors::BORDER_PRIMARY
                                    }
                                ),
                            }
                        }
                    }

                    // 按钮组
                    div {
                        class: "flex items-center justify-between gap-3",
                        // 跳过按钮
                        button {
                            class: "px-4 py-2 rounded transition-all",
                            style: format!(
                                "background: {}; border: 1px solid {}; color: {};",
                                Colors::BG_PRIMARY,
                                Colors::BORDER_PRIMARY,
                                Colors::TEXT_SECONDARY
                            ),
                            onclick: move |_| {
                                if let Some(handler) = on_skip {
                                    handler.call(());
                                }
                            },
                            "跳过"
                        }

                        // 导航按钮
                        div {
                            class: "flex gap-2",
                            // 上一步
                            if !is_first {
                                button {
                                    class: "px-4 py-2 rounded transition-all",
                                    style: format!(
                                        "background: {}; border: 1px solid {}; color: {};",
                                        Colors::BG_PRIMARY,
                                        Colors::BORDER_PRIMARY,
                                        Colors::TEXT_PRIMARY
                                    ),
                                    onclick: move |_| {
                                        if let Some(handler) = on_prev {
                                            handler.call(());
                                        }
                                    },
                                    "上一步"
                                }
                            }

                            // 下一步/完成
                            button {
                                class: "px-4 py-2 rounded transition-all",
                                style: format!(
                                    "background: {}; color: white;",
                                    Colors::TECH_PRIMARY
                                ),
                                onclick: move |_| {
                                    if is_last {
                                        if let Some(handler) = on_close {
                                            handler.call(());
                                        }
                                    } else {
                                        if let Some(handler) = on_next {
                                            handler.call(());
                                        }
                                    }
                                },
                                if is_last {
                                    "完成"
                                } else {
                                    "下一步"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 新手引导管理器
pub struct OnboardingManager {
    steps: Vec<TourStep>,
}

impl OnboardingManager {
    pub fn new() -> Self {
        Self {
            steps: vec![
                TourStep {
                    id: "welcome".to_string(),
                    title: "欢迎使用交换功能".to_string(),
                    content: "在这里您可以交换代币、购买稳定币、提现到法币。让我们开始探索吧！".to_string(),
                    target_selector: None,
                },
                TourStep {
                    id: "swap".to_string(),
                    title: "代币交换".to_string(),
                    content: "选择要交换的代币和数量，系统会自动为您获取最佳报价。支持所有主流代币和稳定币。".to_string(),
                    target_selector: Some("#swap-tab".to_string()),
                },
                TourStep {
                    id: "buy".to_string(),
                    title: "购买稳定币".to_string(),
                    content: "使用信用卡、银行卡或PayPal购买USDT/USDC稳定币。支持多种支付方式，即时到账。".to_string(),
                    target_selector: Some("#buy-tab".to_string()),
                },
                TourStep {
                    id: "withdraw".to_string(),
                    title: "提现到法币".to_string(),
                    content: "将您的代币提现到银行卡、银行账户或PayPal。系统会自动执行代币→稳定币→法币的两步流程。".to_string(),
                    target_selector: Some("#withdraw-tab".to_string()),
                },
                TourStep {
                    id: "history".to_string(),
                    title: "交易历史".to_string(),
                    content: "查看所有交易记录，包括交换、充值和提现。支持按类型和状态筛选。".to_string(),
                    target_selector: Some("#history-tab".to_string()),
                },
            ],
        }
    }

    pub fn get_steps(&self) -> &[TourStep] {
        &self.steps
    }
}

impl Default for OnboardingManager {
    fn default() -> Self {
        Self::new()
    }
}
