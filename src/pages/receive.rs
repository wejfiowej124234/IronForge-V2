//! Receive Page - 接收页面
//! 显示接收地址和二维码，支持多链

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::molecules::{ChainSelector, QrCodeDisplay};
use crate::router::Route;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;

/// Receive Page - 接收页面
/// 企业级实现：完整的状态检查和友好的用户引导
#[component]
pub fn Receive() -> Element {
    let app_state = use_context::<AppState>();
    let navigator = use_navigator();
    let mut selected_chain = use_signal(|| "ethereum".to_string());

    // 企业级：获取钱包状态并进行完整性检查
    let wallet_state_check = use_memo(move || {
        let wallet_state = app_state.wallet.read();
        let has_wallet = wallet_state.is_initialized() && !wallet_state.wallets.is_empty();
        let selected_wallet = wallet_state.get_selected_wallet();
        (has_wallet, selected_wallet.cloned())
    });

    // 获取当前选中链的账户
    let current_account = use_memo(move || {
        let state_check = wallet_state_check.read();
        let (_, wallet_opt) = &*state_check;
        wallet_opt.as_ref().and_then(|w| {
            w.accounts
                .iter()
                .find(|acc| acc.chain.to_lowercase() == selected_chain.read().to_lowercase())
                .cloned()
        })
    });

    rsx! {
        div {
            class: "min-h-screen pt-20 pb-8 px-4",
            style: format!("background: {};", Colors::BG_PRIMARY),

            div {
                class: "container mx-auto max-w-2xl px-4 sm:px-6",

                // 页面标题 - 响应式优化
                div {
                    class: "mb-8",
                    h1 {
                        class: "text-2xl font-bold mb-2 flex items-center gap-2",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        span { "💸" }
                        span { "接收资产" }
                    }
                    p {
                        class: "text-sm",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "选择对应链并分享地址或二维码给发送方"
                    }
                }

                // 企业级：钱包状态检查
                if !wallet_state_check.read().0 {
                    // 无钱包状态：友好引导
                    Card {
                        variant: crate::components::atoms::card::CardVariant::Base,
                        padding: Some("32px".to_string()),
                        children: rsx! {
                            div {
                                class: "text-center py-8",
                                div {
                                    class: "text-6xl mb-4",
                                    "💼"
                                }
                                h3 {
                                    class: "text-xl font-semibold mb-2",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    "还没有钱包"
                                }
                                p {
                                    class: "text-sm mb-6",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "创建钱包后即可接收资产"
                                }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    size: ButtonSize::Large,
                                    onclick: move |_| {
                                        navigator.push(Route::CreateWallet {});
                                    },
                                    "创建钱包"
                                }
                            }
                        }
                    }
                } else {
                    // 链选择器
                    ChainSelector {
                        selected_chain: selected_chain
                    }

                    if let Some(account) = current_account.as_ref() {
                    Card {
                        variant: crate::components::atoms::card::CardVariant::Base,
                        padding: Some("32px".to_string()),
                        children: rsx! {
                            // 链信息显示
                            div {
                                class: "mb-8 p-4 rounded-xl",
                                style: format!("background: linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(79, 70, 229, 0.05) 100%); border: 2px solid {}; box-shadow: 0 2px 8px rgba(99, 102, 241, 0.1);",
                                    "rgba(99, 102, 241, 0.3)"),
                                div {
                                    class: "flex items-center justify-between",
                                    div {
                                        class: "flex items-center gap-3",
                                        span { class: "text-2xl", "⛓️" }
                                        span {
                                            class: "text-lg font-bold",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            {account.chain_label()}
                                        }
                                    }
                                    span {
                                        class: "text-xs px-3 py-1.5 rounded-full font-semibold",
                                        style: format!("background: {}; color: white;", Colors::TECH_PRIMARY),
                                        "✓ 当前网络"
                                    }
                                }
                            }

                            // 二维码显示组件
                            QrCodeDisplay {
                                address: account.address.clone(),
                                show_copy_button: Some(true)
                            }

                            // 安全提示 - 更醒目的警告样式
                            div {
                                class: "mt-6 p-5 rounded-xl border-2",
                                style: "background: linear-gradient(135deg, rgba(245, 158, 11, 0.15) 0%, rgba(217, 119, 6, 0.1) 100%); border-color: rgba(245, 158, 11, 0.5); box-shadow: 0 4px 12px rgba(245, 158, 11, 0.15);",
                                div {
                                    class: "flex items-start gap-4",
                                    div {
                                        class: "flex-shrink-0 w-10 h-10 flex items-center justify-center rounded-full",
                                        style: "background: rgba(245, 158, 11, 0.2);",
                                        span {
                                            class: "text-2xl",
                                            "⚠️"
                                        }
                                    }
                                    div {
                                        class: "flex-1",
                                        p {
                                            class: "font-bold text-base mb-3",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            "重要安全提示"
                                        }
                                        div {
                                            class: "text-sm space-y-2",
                                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                                            p {
                                                class: "flex items-start gap-2",
                                                span { "•" }
                                                span { {format!("仅向此地址发送 {} 网络的资产", account.chain_label())} }
                                            }
                                            p {
                                                class: "flex items-start gap-2",
                                                span { "•" }
                                                span { "跨链转账将导致资产永久丢失，无法找回" }
                                            }
                                            p {
                                                class: "flex items-start gap-2",
                                                span { "•" }
                                                span { "建议首次使用时先发送小额测试" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    } else {
                        // 企业级：链未支持的友好提示
                        Card {
                            variant: crate::components::atoms::card::CardVariant::Base,
                            padding: Some("32px".to_string()),
                            children: rsx! {
                                div {
                                    class: "text-center py-8",
                                    div {
                                        class: "text-5xl mb-4",
                                        "⚠️"
                                    }
                                    p {
                                        class: "text-lg mb-2",
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        "未找到 {selected_chain.read()} 链的账户"
                                    }
                                    p {
                                        class: "text-sm mb-4",
                                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                                        "该钱包暂不支持此链，请选择其他链"
                                    }
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        size: ButtonSize::Medium,
                                        onclick: move |_| {
                                            selected_chain.set("ethereum".to_string());
                                        },
                                        "切换到以太坊"
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
