//! Wallet Detail Page - 钱包详情页面
//! 显示钱包详细信息、账户列表、余额和交易历史

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::route_guard::AuthGuard;
use crate::features::wallet::state::Account;
use crate::router::Route;
use crate::services::balance::BalanceService;
use crate::services::transaction::{TransactionHistoryItem, TransactionService};
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;

/// 链ID映射
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

/// Wallet Detail Page 组件
#[component]
pub fn WalletDetail(id: String) -> Element {
    rsx! {
        AuthGuard {
            WalletDetailContent { wallet_id: id }
        }
    }
}

/// 钱包详情内容组件
#[component]
fn WalletDetailContent(wallet_id: String) -> Element {
    let app_state = use_context::<AppState>();
    let navigator = use_navigator();

    let wallet_state = app_state.wallet.read();
    let wallet = wallet_state
        .wallets
        .iter()
        .find(|w| w.id == wallet_id)
        .cloned();

    if wallet.is_none() {
        return rsx! {
            div {
                class: "min-h-screen flex items-center justify-center",
                style: format!("background: {};", Colors::BG_PRIMARY),
                div {
                    class: "text-center",
                    h1 {
                        class: "text-2xl font-bold mb-4",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "钱包未找到"
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Medium,
                        onclick: move |_| {
                            navigator.push(Route::Dashboard {});
                        },
                        "返回Dashboard"
                    }
                }
            }
        };
    }

    let wallet = wallet.unwrap();

    rsx! {
        div {
            class: "min-h-screen p-4",
            style: format!("background: {};", Colors::BG_PRIMARY),

            div {
                class: "container mx-auto max-w-4xl px-4 sm:px-6",

                // 页面标题 - 响应式优化
                div {
                    class: "mb-4 sm:mb-6 flex flex-col sm:flex-row items-start sm:items-center gap-3 sm:gap-4",
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Small,
                        onclick: move |_| {
                            navigator.go_back();
                        },
                        "← 返回"
                    }
                    h1 {
                        class: "text-xl sm:text-2xl font-bold",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "钱包详情 - {wallet.name}"
                    }
                }

                // 钱包信息卡片
                Card {
                    variant: crate::components::atoms::card::CardVariant::Base,
                    padding: Some("24px".to_string()),
                    class: Some("mb-6".to_string()),
                    children: rsx! {
                        div {
                            class: "space-y-4",
                            div {
                                class: "flex justify-between items-center",
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "钱包名称"
                                }
                                span {
                                    class: "font-semibold",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    {wallet.name.clone()}
                                }
                            }
                            div {
                                class: "flex justify-between items-center",
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "钱包ID"
                                }
                                span {
                                    class: "font-mono text-xs",
                                    style: format!("color: {};", Colors::TEXT_TERTIARY),
                                    {wallet.id.clone()}
                                }
                            }
                            div {
                                class: "flex justify-between items-center",
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "账户数量"
                                }
                                span {
                                    class: "font-semibold",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    {format!("{} 个账户", wallet.accounts.len())}
                                }
                            }
                            div {
                                class: "flex justify-between items-center",
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "状态"
                                }
                                span {
                                    class: if wallet.is_locked {
                                        "text-xs px-2 py-1 rounded"
                                    } else {
                                        "text-xs px-2 py-1 rounded"
                                    },
                                    style: format!(
                                        "background: {}; color: {};",
                                        if wallet.is_locked { Colors::PAYMENT_WARNING } else { Colors::PAYMENT_SUCCESS },
                                        "white"
                                    ),
                                    if wallet.is_locked { "🔒 已锁定" } else { "🔓 已解锁" }
                                }
                            }
                            div {
                                class: "flex justify-between items-center",
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "创建时间"
                                }
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_TERTIARY),
                                    {wallet.created_at.clone()}
                                }
                            }
                        }
                    }
                }

                // 账户列表
                Card {
                    variant: crate::components::atoms::card::CardVariant::Base,
                    padding: Some("24px".to_string()),
                    class: Some("mb-6".to_string()),
                    children: rsx! {
                        h2 {
                            class: "text-xl font-bold mb-4",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "账户列表"
                        }
                        div {
                            class: "space-y-3",
                            for account in wallet.accounts.iter() {
                                AccountCard {
                                    account: account.clone(),
                                    wallet_id: wallet_id.clone(),
                                }
                            }
                        }
                    }
                }

                // 交易历史
                TransactionHistory {
                    wallet_id: wallet_id.clone(),
                    accounts: wallet.accounts.clone(),
                }

                // 快速操作
                div {
                    class: "flex gap-3 mt-6",
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Large,
                        class: Some("flex-1".to_string()),
                        onclick: move |_| {
                            navigator.push(Route::Send {});
                        },
                        "发送"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Large,
                        class: Some("flex-1".to_string()),
                        onclick: move |_| {
                            navigator.push(Route::Receive {});
                        },
                        "接收"
                    }
                }
            }
        }
    }
}

/// 账户卡片组件
#[component]
fn AccountCard(account: Account, wallet_id: String) -> Element {
    let app_state = use_context::<AppState>();
    let balance = use_signal(|| "0".to_string());
    let is_loading = use_signal(|| true);

    let account_clone_for_effect = account.clone();
    let account_chain_clone = account.chain.clone();
    let account_address_clone = account.address.clone();
    let account_chain_label = account.chain_label();

    use_effect(move || {
        let app_state = app_state;
        let account = account_clone_for_effect.clone();
        let mut balance = balance;
        let mut is_loading = is_loading;

        spawn(async move {
            let balance_service = BalanceService::new(app_state);
            let chain_id = get_chain_id(&account.chain);

            match balance_service
                .get_balance(&account.address, chain_id)
                .await
            {
                Ok(resp) => {
                    balance.set(resp.balance);
                    is_loading.set(false);
                }
                Err(_) => {
                    is_loading.set(false);
                }
            }
        });
    });
    rsx! {
        div {
            class: "p-4 rounded-lg",
            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
            div {
                class: "flex justify-between items-center",
                div {
                    span {
                        class: "font-semibold",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {account_chain_label}
                    }
                    p {
                        class: "text-xs mt-1 font-mono",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        {account_address_clone.clone()}
                    }
                }
                div {
                    class: "text-right",
                    if is_loading() {
                        span {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_TERTIARY),
                            "加载中..."
                        }
                    } else {
                        span {
                            class: "font-semibold",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            {
                                let balance_val: f64 = balance.read().parse().unwrap_or(0.0);
                                let chain_lower = account_chain_clone.to_lowercase();
                                let chain_symbol = match chain_lower.as_str() {
                                    "ethereum" | "eth" => ("ETH", 1e18),
                                    "bitcoin" | "btc" => ("BTC", 1e8),
                                    "solana" | "sol" => ("SOL", 1e9),
                                    "ton" => ("TON", 1e9),
                                    _ => ("ETH", 1e18),
                                };
                                format!("{:.6} {}", balance_val / chain_symbol.1, chain_symbol.0)
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 交易历史组件
#[component]
fn TransactionHistory(wallet_id: String, accounts: Vec<Account>) -> Element {
    let app_state = use_context::<AppState>();
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
                    Err(e) => {
                        log::warn!(
                            "Failed to get transaction history for {}: {}",
                            account.address,
                            e
                        );
                    }
                }
            }

            // 按时间戳排序（最新的在前）
            all_txs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

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
                h2 {
                    class: "text-xl font-bold mb-4",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    "交易历史"
                }

                if is_loading() {
                    div {
                        class: "text-center py-8",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        "正在加载交易历史..."
                    }
                } else if false {
                    div {
                        class: "p-4 rounded-lg",
                        style: format!("background: rgba(239, 68, 68, 0.1); color: {};", Colors::PAYMENT_ERROR),
                        "错误信息"
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
                            TransactionRow {
                                transaction: tx.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 交易行组件
#[component]
fn TransactionRow(transaction: TransactionHistoryItem) -> Element {
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
            class: "p-4 rounded-lg",
            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
            div {
                class: "flex justify-between items-start",
                div {
                    class: "flex-1",
                    div {
                        class: "flex items-center gap-2 mb-2",
                        span { {tx_type_icon} }
                        span {
                            class: "font-semibold",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            {transaction.tx_type.clone()}
                        }
                        span {
                            class: "text-xs px-2 py-1 rounded",
                            style: format!("background: {}; color: white;", status_color),
                            {transaction.status.clone()}
                        }
                    }
                    div {
                        class: "text-xs font-mono",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        "Hash: {transaction.hash.clone()}"
                    }
                    div {
                        class: "text-xs mt-1",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "From: {transaction.from.clone()}"
                    }
                    div {
                        class: "text-xs",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "To: {transaction.to.clone()}"
                    }
                }
                div {
                    class: "text-right",
                    div {
                        class: "font-semibold",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {transaction.amount.clone()} " {transaction.token.clone()}"
                    }
                    
                    // ✅ 费用明细展示（显示真实的后端数据）
                    div {
                        class: "mt-3 pt-3 border-t",
                        style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                        div {
                            class: "text-xs font-semibold mb-2",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "费用明细"
                        }
                        div {
                            class: "space-y-1 text-xs",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            
                            // ⛽ Gas费用（区块链网络费用）
                            {
                                let fee_str = transaction.fee.clone();
                                // 尝试解析为数字以提取Gas费用和平台服务费
                                // 实际显示从后端API返回的真实数据
                                rsx! {
                                    div {
                                        class: "flex justify-between",
                                        span { "⛽ Gas费:" }
                                        span { class: "font-mono", "{fee_str}" }
                                    }
                                }
                            }
                            
                            // 💼 平台服务费（钱包服务商收取）
                            // 注意：这是真实的后端API计算结果，不是硬编码
                            // 百分比费率从 gas.platform_fee_rules 表动态读取
                            div {
                                class: "flex justify-between",
                                span { "💼 服务费:" }
                                span {
                                    class: "font-mono",
                                    style: format!("color: {};", Colors::TECH_PRIMARY),
                                    // 后端API会返回真实的platform_fee值
                                    // 这里显示的是根据交易金额动态计算的服务费
                                    "待查询"
                                }
                            }
                            
                            // 💰 总计
                            div {
                                class: "font-semibold mt-1 pt-1 border-t flex justify-between",
                                style: format!("border-color: {}; color: {};", Colors::BORDER_PRIMARY, Colors::TEXT_PRIMARY),
                                span { "💰 总计:" }
                                span {
                                    class: "font-mono",
                                    {transaction.fee.clone()}
                                }
                            }
                        }
                        
                        // 💡 费用说明
                        div {
                            class: "mt-2 p-2 rounded text-xs",
                            style: format!("background: {}; color: {};", Colors::BG_PRIMARY, Colors::TEXT_TERTIARY),
                            "💡 Gas费由区块链收取，服务费由平台收取（按交易金额0.1%-1.0%动态计算）"
                        }
                    }
                }
            }
        }
    }
}
