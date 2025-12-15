//! Stablecoin Balance Card - 稳定币余额卡片组件
//! 显示USDT和USDC余额，支持快速购买

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::services::address_detector::ChainType;
use crate::services::token::TokenService;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use futures::join;

/// 稳定币余额卡片组件
#[component]
pub fn StablecoinBalanceCard(
    /// 是否显示快速购买按钮
    show_buy_button: Option<bool>,
    /// 当前链类型（可选，默认Ethereum）
    chain: Option<ChainType>,
    /// 快速购买按钮点击回调
    on_buy_click: Option<EventHandler<()>>,
) -> Element {
    let app_state = use_context::<AppState>();
    let show_buy = show_buy_button.unwrap_or(true);
    let current_chain = chain.unwrap_or(ChainType::Ethereum);

    // 稳定币余额状态
    let usdt_balance = use_signal(|| 0.0f64);
    let usdc_balance = use_signal(|| 0.0f64);
    let loading = use_signal(|| true);

    // 获取当前钱包
    let current_wallet = use_memo(move || {
        let wallet_state = app_state.wallet.read();
        wallet_state.get_selected_wallet().cloned()
    });

    // 从钱包获取实际稳定币余额
    use_effect({
        let app_state_clone = app_state.clone();
        let chain_clone = current_chain;
        let wallet_opt = current_wallet.read().clone();
        let mut loading_mut = loading;
        let mut usdt_mut = usdt_balance;
        let mut usdc_mut = usdc_balance;

        move || {
            if wallet_opt.is_none() {
                loading_mut.set(false);
                return;
            }

            let wallet = wallet_opt.clone().unwrap();
            let wallet_address = wallet
                .accounts
                .first()
                .map(|a| a.address.clone())
                .unwrap_or_default();
            let app_state_for_spawn = app_state_clone.clone();

            spawn(async move {
                loading_mut.set(true);

                // 使用硬编码默认地址（简化实现，避免复杂的async嵌套）
                // 如果需要动态获取，可以在组件初始化时预先获取
                let usdt_address = match chain_clone {
                    ChainType::Ethereum => "0xdAC17F958D2ee523a2206206994597C13D831ec7",
                    ChainType::Polygon => "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",
                    ChainType::BSC => "0x55d398326f99059fF775485246999027B3197955",
                    _ => "0xdAC17F958D2ee523a2206206994597C13D831ec7", // 默认Ethereum
                }
                .to_string();

                let usdc_address = match chain_clone {
                    ChainType::Ethereum => "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                    ChainType::Polygon => "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
                    ChainType::BSC => "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d",
                    _ => "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // 默认Ethereum
                }
                .to_string();

                // 并行获取USDT和USDC余额
                // 创建两个TokenService实例用于并行调用
                let token_service1 = TokenService::new(app_state_for_spawn.clone());
                let token_service2 = TokenService::new(app_state_for_spawn.clone());
                let chain_clone1 = chain_clone;
                let chain_clone2 = chain_clone;
                let usdt_address_clone1 = usdt_address.clone();
                let _usdt_address_clone2 = usdt_address.clone();
                let _usdc_address_clone1 = usdc_address.clone();
                let usdc_address_clone2 = usdc_address.clone();
                let wallet_address_clone1 = wallet_address.clone();
                let wallet_address_clone2 = wallet_address.clone();

                let usdt_future = async move {
                    token_service1
                        .get_token_balance(
                            chain_clone1,
                            &usdt_address_clone1,
                            &wallet_address_clone1,
                        )
                        .await
                };

                let usdc_future = async move {
                    token_service2
                        .get_token_balance(
                            chain_clone2,
                            &usdc_address_clone2,
                            &wallet_address_clone2,
                        )
                        .await
                };

                let (usdt_result, usdc_result) = join!(usdt_future, usdc_future);

                if let Ok(balance) = usdt_result {
                    usdt_mut.set(balance.balance_formatted);
                } else {
                    usdt_mut.set(0.0);
                }

                if let Ok(balance) = usdc_result {
                    usdc_mut.set(balance.balance_formatted);
                } else {
                    usdc_mut.set(0.0);
                }

                loading_mut.set(false);
            });
        }
    });

    // 计算总法币价值（简化：1 USDT/USDC = 1 USD）
    let total_usd_value = *usdt_balance.read() + *usdc_balance.read();

    rsx! {
        div {
            class: "mb-6 p-6 rounded-lg",
            style: format!(
                "background: {}; border: 1px solid {};",
                Colors::BG_SECONDARY,
                Colors::BORDER_PRIMARY
            ),
            div {
                class: "flex items-center justify-between mb-4",
                h2 {
                    class: "text-lg font-semibold",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    "💰 稳定币余额"
                }
                if show_buy {
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Small,
                        onclick: move |_| {
                            if let Some(handler) = on_buy_click {
                                handler.call(());
                            } else {
                                log::info!("快速购买稳定币");
                            }
                        },
                        "快速购买"
                    }
                }
            }

            div {
                class: "grid grid-cols-2 gap-4",
                // USDT余额卡片
                div {
                    class: "p-4 rounded-lg",
                    style: format!("background: {};", Colors::BG_PRIMARY),
                    div {
                        class: "text-sm font-medium mb-1",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "USDT"
                    }
                    if loading() {
                        div {
                            class: "space-y-2",
                            // 骨架屏加载效果
                            div {
                                class: "h-6 w-20 bg-gray-300 rounded animate-pulse",
                            }
                            div {
                                class: "h-4 w-16 bg-gray-200 rounded animate-pulse",
                            }
                        }
                    } else {
                        div {
                            class: "text-lg font-bold mb-1",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "{usdt_balance.read():.2}"
                        }
                        div {
                            class: "text-xs",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "≈ ${usdt_balance.read():.2}"
                        }
                    }
                }

                // USDC余额卡片
                div {
                    class: "p-4 rounded-lg",
                    style: format!("background: {};", Colors::BG_PRIMARY),
                    div {
                        class: "text-sm font-medium mb-1",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "USDC"
                    }
                    if loading() {
                        div {
                            class: "space-y-2",
                            // 骨架屏加载效果
                            div {
                                class: "h-6 w-20 bg-gray-300 rounded animate-pulse",
                            }
                            div {
                                class: "h-4 w-16 bg-gray-200 rounded animate-pulse",
                            }
                        }
                    } else {
                        div {
                            class: "text-lg font-bold mb-1",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "{usdc_balance.read():.2}"
                        }
                        div {
                            class: "text-xs",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "≈ ${usdc_balance.read():.2}"
                        }
                    }
                }
            }

            // 总价值显示
            if !loading() && total_usd_value > 0.0 {
                div {
                    class: "mt-4 pt-4 border-t",
                    style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                    div {
                        class: "flex justify-between items-center",
                        span {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "总价值"
                        }
                        span {
                            class: "text-lg font-bold",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "${total_usd_value:.2}"
                        }
                    }
                }
            }

            // 余额不足提示（优化版）
            if !loading() && total_usd_value == 0.0 {
                div {
                    class: "mt-4 p-4 rounded-lg",
                    style: format!(
                        "background: rgba(251, 191, 36, 0.1); border: 1px solid rgba(251, 191, 36, 0.3);"
                    ),
                    div {
                        class: "flex items-start gap-3",
                        span {
                            class: "text-xl",
                            "💡"
                        }
                        div {
                            class: "flex-1",
                            div {
                                class: "text-sm font-medium mb-1",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                "稳定币余额不足"
                            }
                            div {
                                class: "text-xs",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "您的稳定币余额为0，请先购买稳定币（USDT/USDC）后再进行代币交换。"
                            }
                            if show_buy {
                                div {
                                    class: "mt-2",
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        size: ButtonSize::Small,
                                        onclick: move |_| {
                                            if let Some(handler) = on_buy_click {
                                                handler.call(());
                                            }
                                        },
                                        "立即购买稳定币"
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
