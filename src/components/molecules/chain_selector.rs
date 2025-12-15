//! Chain Selector - 链选择器组件
//! 用于选择区块链（Ethereum、Bitcoin、Solana、TON）

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

// 使用服务层的ChainType
use crate::services::address_detector::ChainType;

/// 链选择器组件（支持所有链）
#[component]
pub fn ChainSelector(selected_chain: Signal<String>) -> Element {
    let current_chain_str = selected_chain.read().clone();
    let current_chain = ChainType::from_str(&current_chain_str).unwrap_or(ChainType::Ethereum);

    // 所有支持的链
    let all_chains = [
        ChainType::Ethereum,
        ChainType::BSC,
        ChainType::Polygon,
        ChainType::Bitcoin,
        ChainType::Solana,
        ChainType::TON,
    ];

    rsx! {
        div {
            class: "mb-6",
            div {
                class: "flex items-center justify-between mb-2",
                label {
                    class: "block text-sm font-medium",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    "选择链"
                }
                span {
                    class: "text-xs",
                    style: format!("color: {};", Colors::TEXT_TERTIARY),
                    "(选择代币后自动匹配)"
                }
            }
            div {
                class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-2",
                for chain in all_chains.iter() {
                    Button {
                        variant: if current_chain == *chain {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Secondary
                        },
                        size: ButtonSize::Medium,
                        onclick: {
                            let mut selected_chain = selected_chain;
                            let chain_str = chain.as_str().to_string();
                            move |_| {
                                selected_chain.set(chain_str.clone());
                            }
                        },
                        {chain.label()}
                    }
                }
            }
        }
    }
}
