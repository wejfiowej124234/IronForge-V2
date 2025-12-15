//! Dashboard Transaction History Preview Component
//! 交易历史预览组件 - 在Dashboard中显示最近的交易

use crate::components::atoms::card::Card;
use crate::features::wallet::state::Account;
use crate::router::Route;
use crate::services::transaction::{TransactionHistoryItem, TransactionService};
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;

/// 交易历史预览组件
#[component]
pub fn TransactionHistoryPreview(wallet_id: String, accounts: Vec<Account>) -> Element {
    let app_state = use_context::<AppState>();
    let navigator = use_navigator();

    let transactions = use_signal(|| Vec::<TransactionHistoryItem>::new());
    let is_loading = use_signal(|| true);

    use_effect(move || {
        let app_state = app_state;
        let accounts = accounts.clone();
        let mut transactions = transactions;
        let mut is_loading = is_loading;

        spawn(async move {
            is_loading.set(true);
            let tx_service = TransactionService::new(app_state);
            let mut all_txs = Vec::new();

            // 查询所有账户的交易历史
            for account in &accounts {
                match tx_service
                    .get_history(&account.address, &account.chain)
                    .await
                {
                    Ok(txs) => {
                        all_txs.extend(txs);
                    }
                    Err(_) => {
                        // 忽略错误，继续查询其他账户
                    }
                }
            }

            // 按时间戳排序（最新的在前），只取前5条
            all_txs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            all_txs.truncate(5);

            transactions.set(all_txs);
            is_loading.set(false);
        });
    });

    rsx! {
        Card {
            variant: crate::components::atoms::card::CardVariant::Base,
            padding: Some("24px".to_string()),
            class: Some("mb-6".to_string()),
            children: rsx! {
                div {
                    class: "flex justify-between items-center mb-4",
                    h2 {
                        class: "text-xl font-bold",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "最近交易"
                    }
                    if !transactions.read().is_empty() {
                        crate::components::atoms::button::Button {
                            variant: crate::components::atoms::button::ButtonVariant::Secondary,
                            size: crate::components::atoms::button::ButtonSize::Small,
                            onclick: move |_| {
                                navigator.push(Route::WalletDetail { id: wallet_id.clone() });
                            },
                            "查看全部"
                        }
                    }
                }

                if is_loading() {
                    div {
                        class: "text-center py-8",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        "正在加载交易历史..."
                    }
                } else if transactions.read().is_empty() {
                    div {
                        class: "text-center py-8",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        "暂无交易记录"
                    }
                } else {
                    div {
                        class: "space-y-3",
                        for tx in transactions.read().iter() {
                            TransactionRowPreview {
                                transaction: tx.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 交易行预览组件（简化版）
#[component]
fn TransactionRowPreview(transaction: TransactionHistoryItem) -> Element {
    let status_color = match transaction.status.to_lowercase().as_str() {
        "confirmed" => Colors::PAYMENT_SUCCESS,
        "pending" => Colors::PAYMENT_WARNING,
        "failed" => Colors::PAYMENT_ERROR,
        _ => Colors::TEXT_TERTIARY,
    };

    let tx_type_icon = match transaction.tx_type.to_lowercase().as_str() {
        "send" => "📤",
        "receive" => "📥",
        _ => "📋",
    };

    rsx! {
        div {
            class: "p-3 rounded-lg",
            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
            div {
                class: "flex justify-between items-center",
                div {
                    class: "flex items-center gap-2",
                    span { {tx_type_icon} }
                    div {
                        span {
                            class: "font-semibold text-sm",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            {transaction.tx_type.clone()}
                        }
                        p {
                            class: "text-xs font-mono mt-1",
                            style: format!("color: {};", Colors::TEXT_TERTIARY),
                            {format!("{}...{}", &transaction.hash[..8], &transaction.hash[transaction.hash.len()-6..])}
                        }
                    }
                }
                div {
                    class: "text-right",
                    div {
                        class: "font-semibold text-sm",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {transaction.amount.clone()} " {transaction.token.clone()}"
                    }
                    span {
                        class: "text-xs px-2 py-1 rounded mt-1 inline-block",
                        style: format!("background: {}; color: white;", status_color),
                        {transaction.status.clone()}
                    }
                }
            }
        }
    }
}
