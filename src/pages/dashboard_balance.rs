//! Dashboard Balance Overview Component
//! 余额概览组件 - 显示选中钱包的多链余额聚合

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::molecules::ErrorMessage;
use crate::features::wallet::state::Wallet;
use crate::router::Route;
use crate::services::balance::BalanceService;
use crate::services::price::PriceService;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;

/// 链ID映射（用于API调用）
///
/// 注意：此函数当前未使用，但保留用于未来扩展
#[allow(dead_code)]
fn get_chain_id(chain: &str) -> u64 {
    match chain.to_lowercase().as_str() {
        "ethereum" | "eth" => 1,
        "bitcoin" | "btc" => 0,
        "solana" | "sol" => 101,
        "ton" => 0,
        _ => 1,
    }
}

/// 余额概览组件 - 显示选中钱包的多链余额聚合
#[component]
pub fn BalanceOverview(wallet: Wallet) -> Element {
    let app_state = use_context::<AppState>();
    let navigator = use_navigator();
    let t = crate::i18n::use_translation();

    // 余额状态
    let balances = use_signal(|| std::collections::HashMap::<String, String>::new());
    let prices = use_signal(|| std::collections::HashMap::<String, f64>::new());
    let total_usd = use_signal(|| 0.0);
    let mut is_loading = use_signal(|| true);
    let error_message = use_signal(|| Option::<String>::None);

    // 自动刷新余额和价格
    let wallet_clone = wallet.clone();
    use_effect(move || {
        let app_state = app_state;
        // 在闭包内部克隆wallet，避免移动问题
        let wallet = wallet_clone.clone();
        let mut balances = balances;
        let mut prices = prices;
        let mut total_usd = total_usd;
        let mut is_loading = is_loading;
        let mut error_message = error_message;

        spawn(async move {
            loop {
                is_loading.set(true);
                error_message.set(None);

                let balance_service = BalanceService::new(app_state);
                let price_service = PriceService::new(app_state);

                let mut balance_map = std::collections::HashMap::new();
                let mut price_map = std::collections::HashMap::new();
                let mut total = 0.0;

                // 查询所有账户的余额
                for account in &wallet.accounts {
                    let chain_id = get_chain_id(&account.chain);
                    let chain_symbol = match account.chain.to_lowercase().as_str() {
                        "ethereum" | "eth" => "ETH",
                        "bitcoin" | "btc" => "BTC",
                        "solana" | "sol" => "SOL",
                        "ton" => "TON",
                        _ => "ETH",
                    };

                    match balance_service
                        .get_balance(&account.address, chain_id)
                        .await
                    {
                        Ok(balance_resp) => {
                            balance_map
                                .insert(chain_symbol.to_string(), balance_resp.balance.clone());

                            // 获取价格
                            match price_service.get_price(chain_symbol).await {
                                Ok(price) => {
                                    price_map.insert(chain_symbol.to_string(), price.usd);
                                    // 计算USD价值（余额需要转换为正确的单位）
                                    let balance_val: f64 =
                                        balance_resp.balance.parse().unwrap_or(0.0);
                                    let usd_value = match chain_symbol {
                                        "ETH" => balance_val * price.usd / 1e18,
                                        "BTC" => balance_val * price.usd / 1e8,
                                        "SOL" => balance_val * price.usd / 1e9,
                                        "TON" => balance_val * price.usd / 1e9,
                                        _ => balance_val * price.usd / 1e18,
                                    };
                                    total += usd_value;
                                }
                                Err(_) => {
                                    // 价格获取失败，继续
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to get balance for {}: {}", account.address, e);
                        }
                    }
                }

                balances.set(balance_map);
                prices.set(price_map);
                total_usd.set(total);
                is_loading.set(false);

                // 每30秒刷新一次
                gloo_timers::future::TimeoutFuture::new(30000).await;
            }
        });
    });

    rsx! {
        Card {
            variant: crate::components::atoms::card::CardVariant::Strong,
            padding: Some("24px".to_string()),
            class: Some("mb-6".to_string()),
            children: rsx! {
                div {
                    class: "mb-6",
                    div {
                        class: "flex items-center justify-between",
                        h2 {
                            class: "text-xl font-bold flex items-center gap-2",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            span { "💼" }
                            span { {format!("资产概览 - {}", wallet.name.clone())} }
                        }
                        if !is_loading() {
                            span {
                                class: "text-xs px-2 py-1 rounded-full",
                                style: format!("background: {}; color: white;", "rgba(34, 197, 94, 0.8)"),
                                "✓ 实时"
                            }
                        }
                    }
                }

                if is_loading() {
                    div {
                        class: "text-center py-8",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        "正在加载余额..."
                    }
                } else if error_message.read().is_some() {
                    ErrorMessage {
                        message: error_message.read().clone(),
                        class: Some("p-4".to_string())
                    }
                } else {
                    // 总资产价值 - 增强视觉
                    div {
                        class: "mb-6 pb-6 border-b p-6 rounded-2xl",
                        style: format!("border-color: {}; background: linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(79, 70, 229, 0.05) 100%);", Colors::BORDER_PRIMARY),
                        div {
                            class: "flex items-center gap-2 mb-2",
                            span { class: "text-lg", "💰" }
                            span {
                                class: "text-sm font-semibold uppercase tracking-wide",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "总资产价值"
                            }
                        }
                        div {
                            class: "text-4xl font-bold",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            {format!("${:.2}", total_usd())}
                        }
                        div {
                            class: "mt-2 text-xs",
                            style: format!("color: {};", Colors::TEXT_TERTIARY),
                            "≈ 实时汇率，每30秒更新"
                        }
                    }

                    // 各链余额列表
                    div {
                        class: "space-y-2",
                        for account in wallet.accounts.iter().cloned() {
                            div {
                                class: "flex justify-between items-center p-4 rounded-xl border transition-all hover:scale-[1.01] hover:shadow-md cursor-pointer",
                                style: format!("background: {}; border-color: {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                                div {
                                    span {
                                        class: "font-semibold",
                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                        {account.chain_label()}
                                    }
                                    p {
                                        class: "text-xs mt-1 font-mono",
                                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                                        {account.short_address()}
                                    }
                                }
                                div {
                                    class: "text-right",
                                    div {
                                        class: "font-semibold",
                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                        {
                                            let account_chain = account.chain.clone();
                                            let chain_symbol = match account_chain.to_lowercase().as_str() {
                                                "ethereum" | "eth" => "ETH",
                                                "bitcoin" | "btc" => "BTC",
                                                "solana" | "sol" => "SOL",
                                                "ton" => "TON",
                                                _ => "ETH",
                                            };
                                            let balance = balances.read().get(chain_symbol).cloned().unwrap_or_else(|| "0".to_string());
                                            let balance_val: f64 = balance.parse().unwrap_or(0.0);
                                            let display_balance = match chain_symbol {
                                                "ETH" => balance_val / 1e18,
                                                "BTC" => balance_val / 1e8,
                                                "SOL" => balance_val / 1e9,
                                                "TON" => balance_val / 1e9,
                                                _ => balance_val / 1e18,
                                            };
                                            format!("{:.6} {}", display_balance, chain_symbol)
                                        }
                                    }
                                    {
                                        let account_chain = account.chain.clone();
                                        let chain_symbol = match account_chain.to_lowercase().as_str() {
                                            "ethereum" | "eth" => "ETH",
                                            "bitcoin" | "btc" => "BTC",
                                            "solana" | "sol" => "SOL",
                                            "ton" => "TON",
                                            _ => "ETH",
                                        };
                                        let balance = balances.read().get(chain_symbol).cloned().unwrap_or_else(|| "0".to_string());
                                        let price = prices.read().get(chain_symbol).copied().unwrap_or(0.0);
                                        if price > 0.0 {
                                            rsx! {
                                                p {
                                                    class: "text-xs mt-1",
                                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                                    {
                                                        let balance_val: f64 = balance.parse().unwrap_or(0.0);
                                                        let usd_value = match chain_symbol {
                                                            "ETH" => balance_val * price / 1e18,
                                                            "BTC" => balance_val * price / 1e8,
                                                            "SOL" => balance_val * price / 1e9,
                                                            "TON" => balance_val * price / 1e9,
                                                            _ => balance_val * price / 1e18,
                                                        };
                                                        format!("${:.2}", usd_value)
                                                    }
                                                }
                                            }
                                        } else {
                                            rsx! { div {} }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 快速操作 - 优化：不重复顶部导航，提供更有价值的操作
                    div {
                        class: "mt-6 pt-6 border-t",
                        style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                        div {
                            class: "flex items-center justify-between mb-4",
                            span {
                                class: "text-sm font-semibold",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                {t("dashboard.quick_actions")}
                            }
                            button {
                                class: "text-xs px-3 py-1 rounded-full transition-all hover:scale-105",
                                style: format!("background: {}; color: white;", Colors::TECH_PRIMARY),
                                onclick: move |_| {
                                    // 手动刷新余额
                                    is_loading.set(true);
                                },
                                "🔄 刷新余额"
                            }
                        }
                        div {
                            class: "grid grid-cols-3 gap-3",
                            button {
                                class: "p-4 rounded-xl transition-all hover:scale-105 active:scale-95",
                                style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                                onclick: move |_| {
                                    navigator.push(Route::Swap {});
                                },
                                div {
                                    class: "text-center",
                                    div { class: "text-2xl mb-1", "🔄" }
                                    div { 
                                        class: "text-xs font-semibold",
                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                        "交换"
                                    }
                                }
                            }
                            button {
                                class: "p-4 rounded-xl transition-all hover:scale-105 active:scale-95",
                                style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                                onclick: move |_| {
                                    navigator.push(Route::Sell {});
                                },
                                div {
                                    class: "text-center",
                                    div { class: "text-2xl mb-1", "💳" }
                                    div { 
                                        class: "text-xs font-semibold",
                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                        "提现"
                                    }
                                }
                            }
                            button {
                                class: "p-4 rounded-xl transition-all hover:scale-105 active:scale-95",
                                style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                                onclick: move |_| {
                                    // TODO: 跳转到完整交易历史页面
                                    // navigator.push(Route::Transactions {});
                                },
                                div {
                                    class: "text-center",
                                    div { class: "text-2xl mb-1", "📊" }
                                    div { 
                                        class: "text-xs font-semibold",
                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                        "记录"
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
