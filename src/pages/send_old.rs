//! Send Page - 发送页面
//! 实现发送加密货币的完整流程

use crate::components::atoms::button::{Button, ButtonVariant, ButtonSize};
use crate::components::atoms::modal::Modal;
use crate::components::molecules::{AddressInput, AmountInput, ChainSelector, GasFeeCard, ErrorMessage};
use crate::features::wallet::hooks::use_wallet;
use crate::router::Route;
use crate::services::gas::{GasEstimate, GasService};
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use dioxus::events::FormEvent;

/// Send Page - 发送页面
#[component]
pub fn Send() -> Element {
    let app_state = use_context::<AppState>();
    let navigator = use_navigator();
    
    // 表单状态
    let recipient_address = use_signal(|| String::new());
    let amount = use_signal(|| String::new());
    let selected_chain = use_signal(|| "ethereum".to_string());
    
    // UI状态
    let error_message = use_signal(|| Option::<String>::None);
    let is_loading = use_signal(|| false);
    let show_confirm_modal = use_signal(|| false);
    let _show_unlock_modal = use_signal(|| false);
    let show_recover_modal = use_signal(|| false);
    let gas_estimate = use_signal(|| Option::<GasEstimate>::None);
    let gas_loading = use_signal(|| false);
    
    // 获取当前账户
    let current_account = use_memo(move || {
        let wallet_state = app_state.wallet.read();
        wallet_state.get_selected_wallet()
            .and_then(|w| {
                w.selected_account_index
                    .and_then(|idx| w.accounts.get(idx))
                    .cloned()
            })
    });
    
    // 自动加载后端推荐的最优Gas费（后端已实现智能化选择）
    use_effect(move || {
        let chain = selected_chain.read().clone();
        let app_state_clone = app_state;
        let mut gas_est = gas_estimate;
        let mut gas_load = gas_loading;
        
        spawn(async move {
            gas_load.set(true);
            let gas_service = GasService::new(app_state_clone);
            // 使用后端自动推荐的最优Gas费
            match gas_service.get_recommended(&chain).await {
                Ok(est) => {
                    gas_est.set(Some(est));
                }
                Err(e) => {
                    log::warn!("Failed to get recommended gas fee: {}", e);
                    // 如果获取失败，不阻塞用户，继续显示表单
                }
            }
            gas_load.set(false);
        });
    });
    
    // 地址验证错误
    let address_error = use_signal(|| Option::<String>::None);
    
    rsx! {
        div {
            class: "min-h-screen p-4",
            style: format!("background: {};", Colors::BG_PRIMARY),
            
            div {
                class: "container mx-auto max-w-2xl px-4 sm:px-6",
                
                // 页面标题 - 响应式优化
                div {
                    class: "mb-4 sm:mb-6",
                    h1 {
                        class: "text-xl sm:text-2xl font-bold mb-2",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "发送"
                    }
                    if let Some(account) = current_account.read().as_ref() {
                        p {
                            class: "text-xs sm:text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "从 {account.chain_label()} 账户发送"
                        }
                    }
                }
                
                div {
                    class: "p-6 rounded-lg",
                    style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                    // 链选择器
                    ChainSelector {
                        selected_chain: selected_chain
                    }
                    
                    // 接收地址输入（使用提取的组件）
                    AddressInput {
                        value: recipient_address,
                        selected_chain: selected_chain,
                        error: address_error,
                        label: "接收地址".to_string(),
                        placeholder: "请输入接收地址".to_string(),
                        onchange: {
                            let mut err = error_message;
                            Some(EventHandler::new(move |_e: FormEvent| {
                                err.set(None);
                            }))
                        },
                    }
                    
                    // 金额输入（使用提取的组件）
                    AmountInput {
                        value: amount,
                        current_account: current_account,
                        gas_estimate: gas_estimate,
                        label: "金额".to_string(),
                        placeholder: "0.0".to_string(),
                        onchange: {
                            let mut err = error_message;
                            Some(EventHandler::new(move |_e: FormEvent| {
                                err.set(None);
                            }))
                        },
                        show_balance: true,
                    }
                    
                    // Gas费显示（后端自动选择最优Gas费，无需用户手动选择）
                    GasFeeCard {
                        gas_estimate: gas_estimate.read().clone(),
                        is_loading: gas_loading()
                    }
                    
                    // 总费用显示
                    div {
                        class: "mb-6 p-4 rounded-lg",
                        style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                        div {
                            class: "flex justify-between items-center",
                            span {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "总费用"
                            }
                                span {
                                    class: "text-lg font-semibold",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    {
                                        let amount_val: f64 = amount.read().parse().unwrap_or(0.0);
                                        let gas_cost = gas_estimate.read().as_ref().map(|g| {
                                            g.max_fee_per_gas_gwei * 21000.0 / 1e9
                                        }).unwrap_or(0.0);
                                        format!("{:.6}", amount_val + gas_cost)
                                    }
                                }
                        }
                    }
                    
                    // 错误提示
                    ErrorMessage {
                        message: {
                            let err_msg = error_message.read().clone();
                            // 检查是否是恢复钱包的错误
                            if let Some(ref msg) = err_msg {
                                if msg.contains("WALLET_NEEDS_RECOVERY") {
                                    // 不显示错误消息（因为会显示恢复提示）
                                    None
                                } else {
                                    err_msg.clone()
                                }
                            } else {
                                err_msg.clone()
                            }
                        },
                        class: Some("mb-4".to_string())
                    }
                    
                    // 恢复钱包提示（当检测到需要恢复时）
                    if let Some(ref err_msg) = error_message.read().as_ref() {
                        if err_msg.contains("WALLET_NEEDS_RECOVERY") {
                            div {
                                class: "mb-6 p-4 rounded-lg",
                                style: format!("background: rgba(251, 191, 36, 0.1); border: 1px solid rgba(251, 191, 36, 0.3);"),
                                div {
                                    class: "flex items-start gap-3",
                                    span {
                                        class: "text-xl",
                                        "⚠️"
                                    }
                                    div {
                                        class: "flex-1",
                                        p {
                                            class: "text-sm font-semibold mb-2",
                                            style: format!("color: rgb(251, 191, 36);"),
                                            "新设备检测"
                                        }
                                        p {
                                            class: "text-sm mb-3",
                                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                                            "检测到这是新设备，钱包数据不在本地存储中。要在此设备上签名交易，请先恢复钱包。"
                                        }
                                        Button {
                                            variant: ButtonVariant::Primary,
                                            size: ButtonSize::Medium,
                                            onclick: {
                                                let mut show_recover = show_recover_modal;
                                                move |_| {
                                                    show_recover.set(true);
                                                }
                                            },
                                            "恢复钱包"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                        // 余额不足提示
                        if {
                            let has_balance = if let Some(acc) = current_account.read().as_ref() {
                                let balance: f64 = acc.balance.parse().unwrap_or(0.0);
                                let amount_val: f64 = amount.read().parse().unwrap_or(0.0);
                                let gas_cost = gas_estimate.read().as_ref().map(|g| {
                                    g.max_fee_per_gas_gwei * 21000.0 / 1e9
                                }).unwrap_or(0.0);
                                balance >= (amount_val + gas_cost)
                            } else {
                                false
                            };
                            !has_balance && !amount.read().is_empty()
                        } {
                        div {
                            class: "mb-4 p-4 rounded-lg",
                            style: format!("background: rgba(239, 68, 68, 0.1); border: 1px solid {}; color: {};", Colors::PAYMENT_ERROR, Colors::PAYMENT_ERROR),
                            "余额不足，无法完成交易"
                        }
                    }
                    
                    // 操作按钮
                    div {
                        class: "flex gap-4",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            class: Some("flex-1".to_string()),
                                disabled: {
                                let addr_empty = recipient_address.read().trim().is_empty();
                                let amt_empty = amount.read().trim().is_empty();
                                let has_addr_err = address_error.read().is_some();
                                let has_balance = if let Some(acc) = current_account.read().as_ref() {
                                    let balance: f64 = acc.balance.parse().unwrap_or(0.0);
                                    let amount_val: f64 = amount.read().parse().unwrap_or(0.0);
                                    let gas_cost = gas_estimate.read().as_ref().map(|g| {
                                        g.max_fee_per_gas_gwei * 21000.0 / 1e9
                                    }).unwrap_or(0.0);
                                    balance >= (amount_val + gas_cost)
                                } else {
                                    false
                                };
                                addr_empty || amt_empty || has_addr_err || !has_balance || *is_loading.read()
                            },
                            loading: *is_loading.read(),
                            onclick: {
                                let mut show_confirm_modal = show_confirm_modal;
                                move |_| {
                                    // 显示确认模态框
                                    show_confirm_modal.set(true);
                                }
                            },
                            "继续"
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Large,
                            onclick: move |_| {
                                navigator.go_back();
                            },
                            "取消"
                        }
                    }
                }
            }
            
            // 交易确认模态框
            if show_confirm_modal() {
                TransactionConfirmModal {
                    recipient_address: recipient_address.read().clone(),
                    amount: amount.read().clone(),
                    chain: selected_chain.read().clone(),
                    gas_estimate: gas_estimate.read().clone(),
                    total_cost: {
                        let amount_val: f64 = amount.read().parse().unwrap_or(0.0);
                        let gas_cost = gas_estimate.read().as_ref().map(|g| {
                            g.max_fee_per_gas_gwei * 21000.0 / 1e9
                        }).unwrap_or(0.0);
                        amount_val + gas_cost
                    },
                    on_confirm: EventHandler::new({
                        let app_state_clone = app_state;
                        let recipient = recipient_address.read().clone();
                        let amt = amount.read().clone();
                        let chain = selected_chain.read().clone();
                        let gas_est = gas_estimate.read().clone();
                                let current_acc = current_account.read().clone();
                        let wallet_ctrl = use_wallet();
                        let nav = navigator.clone();
                        let mut loading = is_loading;
                        let mut modal = show_confirm_modal;
                        let mut err = error_message;
                        move |_| {
                            let app_state = app_state_clone;
                            let recipient = recipient.clone();
                            let amt = amt.clone();
                            let chain = chain.clone();
                            let gas_est = gas_est.clone();
                            let current_acc = current_acc.clone();
                            let wallet_ctrl = wallet_ctrl;
                            let nav = nav.clone();
                            
                            loading.set(true);
                            modal.set(false);
                            
                            spawn(async move {
                                // 1. 验证输入
                                if recipient.trim().is_empty() {
                                    loading.set(false);
                                    err.set(Some("请输入接收地址".to_string()));
                                    return;
                                }
                                
                                if amt.trim().is_empty() {
                                    loading.set(false);
                                    err.set(Some("请输入金额".to_string()));
                                    return;
                                }
                                
                                let amount_val: f64 = match amt.parse() {
                                    Ok(v) => v,
                                    Err(_) => {
                                        loading.set(false);
                                        err.set(Some("金额格式无效".to_string()));
                                        return;
                                    }
                                };
                                
                                if amount_val <= 0.0 {
                                    loading.set(false);
                                    err.set(Some("金额必须大于0".to_string()));
                                    return;
                                }
                                
                                // 2. 获取当前账户和钱包信息
                                let wallet_state = app_state.wallet.read();
                                let selected_wallet_id = wallet_state.selected_wallet_id.clone();
                                let _account = if let Some(acc) = current_acc.as_ref() {
                                    acc.clone()
                                } else {
                                    loading.set(false);
                                    err.set(Some("未选择账户".to_string()));
                                    return;
                                };
                                
                                // 3. 检查钱包是否解锁
                                if let Some(wallet_id) = &selected_wallet_id {
                                    // 首先检查钱包是否在本地存储中
                                    if !wallet_ctrl.is_wallet_in_local_storage(wallet_id) {
                                        loading.set(false);
                                        err.set(Some(
                                            "WALLET_NEEDS_RECOVERY: 检测到这是新设备，钱包数据不在本地。请先恢复钱包才能签名交易。".to_string()
                                        ));
                                        return;
                                    }
                                    
                                    // 检查钱包是否解锁
                                    if !wallet_ctrl.is_wallet_unlocked(wallet_id) {
                                        loading.set(false);
                                        err.set(Some("钱包已锁定，请先解锁".to_string()));
                                        return;
                                    }
                                }
                                
                                // 4. 构建交易数据
                                let _gas_price = gas_est.as_ref().map(|g| {
                                    // 转换为wei (gwei * 1e9)
                                    (g.max_fee_per_gas_gwei * 1e9) as u64
                                }).unwrap_or(21000000000u64); // 默认21 gwei
                                
                                let _gas_limit = 21000u64; // 标准ETH转账
                                let _value_wei = (amount_val * 1e18) as u64;
                                
                                // 5. 获取nonce（从后端获取）
                                let _chain_id = match chain.to_lowercase().as_str() {
                                    "ethereum" | "eth" => 1,
                                    "bitcoin" | "btc" => 0,
                                    "solana" | "sol" => 101,
                                    "ton" => 0,
                                    _ => 1,
                                };
                                
                                // 6. 签名交易
                                // 注意：在前端完成签名，确保私钥不离开前端
                                let chain_lower = chain.to_lowercase();
                                let tx_service = crate::services::transaction::TransactionService::new(app_state);
                                let signed_tx = if chain_lower == "ethereum" || chain_lower == "eth" {
                                    // Ethereum交易签名
                                    let key_manager_opt = app_state.key_manager.read().clone();
                                    if let Some(key_manager) = key_manager_opt {
                                        // 获取账户索引
                                        let wallet_state_read = app_state.wallet.read();
                                        let wallet_id_to_find_opt = selected_wallet_id.clone();
                                        let account_index = if let Some(wallet_id_to_find) = wallet_id_to_find_opt {
                                            current_acc.as_ref()
                                                .and_then(|acc| {
                                                    wallet_state_read.wallets.iter()
                                                        .find(|w| w.id == wallet_id_to_find)
                                                        .and_then(|w| {
                                                            w.accounts.iter()
                                                                .position(|a| a.address == acc.address)
                                                        })
                                                })
                                                .unwrap_or(0) as u32
                                        } else {
                                            0u32
                                        };
                                        
                                        // 派生私钥
                                        match key_manager.derive_eth_private_key(account_index) {
                                            Ok(private_key_hex) => {
                                                // 获取账户地址
                                                let from_address = current_acc.as_ref()
                                                    .map(|acc| acc.address.clone())
                                                    .unwrap_or_default();
                                                
                                                // 获取nonce（从后端API获取）
                                                let chain_id = 1u64; // Ethereum主网
                                                let nonce = match tx_service.get_nonce(&from_address, chain_id).await {
                                                    Ok(n) => n,
                                                    Err(e) => {
                                                        log::warn!("获取nonce失败，使用0: {}", e);
                                                        0u64 // 如果获取失败，使用0作为fallback
                                                    }
                                                };
                                                
                                                // 构建交易并签名
                                                let gas_price = gas_est.as_ref().map(|g| {
                                                    (g.max_fee_per_gas_gwei * 1e9) as u64
                                                }).unwrap_or(21000000000u64);
                                                
                                                let gas_limit = 21000u64;
                                                let value_wei = format!("{}", (amount_val * 1e18) as u64);
                                                
                                                match crate::crypto::tx_signer::EthereumTxSigner::sign_transaction(
                                                    &private_key_hex,
                                                    &recipient.trim(),
                                                    &value_wei,
                                                    nonce,
                                                    gas_price,
                                                    gas_limit,
                                                    chain_id,
                                                ) {
                                                    Ok(signed) => signed,
                                                    Err(e) => {
                                                        loading.set(false);
                                                        err.set(Some(crate::shared::ui_error::sanitize_user_message(
                                                            format!("签名失败: {}", e),
                                                        )));
                                                        return;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                loading.set(false);
                                                err.set(Some(crate::shared::ui_error::sanitize_user_message(
                                                    format!("获取私钥失败: {}", e),
                                                )));
                                                return;
                                            }
                                        }
                                    } else {
                                        loading.set(false);
                                        err.set(Some("钱包未解锁，无法签名交易".to_string()));
                                        return;
                                    }
                                } else if chain_lower == "bitcoin" || chain_lower == "btc" {
                                    // Bitcoin交易签名
                                    let key_manager_opt = app_state.key_manager.read().clone();
                                    if let Some(key_manager) = key_manager_opt {
                                        let wallet_state_read = app_state.wallet.read();
                                        let wallet_id_to_find_opt = selected_wallet_id.clone();
                                        let account_index = if let Some(wallet_id_to_find) = wallet_id_to_find_opt {
                                            current_acc.as_ref()
                                                .and_then(|acc| {
                                                    wallet_state_read.wallets.iter()
                                                        .find(|w| w.id == wallet_id_to_find)
                                                        .and_then(|w| {
                                                            w.accounts.iter()
                                                                .position(|a| a.address == acc.address)
                                                        })
                                                })
                                                .unwrap_or(0) as u32
                                        } else {
                                            0u32
                                        };
                                        
                                        match key_manager.derive_btc_private_key(account_index) {
                                            Ok(private_key_hex) => {
                                                let value_satoshi = format!("{}", (amount_val * 1e8) as u64);
                                                let fee_rate = 10u64; // 默认费率 10 sat/vB
                                                
                                                match crate::crypto::tx_signer::BitcoinTxSigner::sign_transaction(
                                                    &private_key_hex,
                                                    &recipient.trim(),
                                                    &value_satoshi,
                                                    fee_rate,
                                                ) {
                                                    Ok(signed) => signed,
                                                    Err(e) => {
                                                        loading.set(false);
                                                        err.set(Some(crate::shared::ui_error::sanitize_user_message(
                                                            format!("签名失败: {}", e),
                                                        )));
                                                        return;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                loading.set(false);
                                                err.set(Some(crate::shared::ui_error::sanitize_user_message(
                                                    format!("获取私钥失败: {}", e),
                                                )));
                                                return;
                                            }
                                        }
                                    } else {
                                        loading.set(false);
                                        err.set(Some("钱包未解锁，无法签名交易".to_string()));
                                        return;
                                    }
                                } else if chain_lower == "solana" || chain_lower == "sol" {
                                    // Solana交易签名
                                    let key_manager_opt = app_state.key_manager.read().clone();
                                    if let Some(key_manager) = key_manager_opt {
                                        let wallet_state_read = app_state.wallet.read();
                                        let wallet_id_to_find_opt = selected_wallet_id.clone();
                                        let account_index = if let Some(wallet_id_to_find) = wallet_id_to_find_opt {
                                            current_acc.as_ref()
                                                .and_then(|acc| {
                                                    wallet_state_read.wallets.iter()
                                                        .find(|w| w.id == wallet_id_to_find)
                                                        .and_then(|w| {
                                                            w.accounts.iter()
                                                                .position(|a| a.address == acc.address)
                                                        })
                                                })
                                                .unwrap_or(0) as u32
                                        } else {
                                            0u32
                                        };
                                        
                                        match key_manager.derive_sol_private_key(account_index) {
                                            Ok(private_key_hex) => {
                                                let value_lamports = format!("{}", (amount_val * 1e9) as u64);
                                                
                                                // Get recent blockhash from backend API
                                                let transaction_service = crate::services::transaction::TransactionService::new(app_state.clone());
                                                let recent_blockhash_res = transaction_service.get_recent_blockhash(&chain).await;
                                                let recent_blockhash = match recent_blockhash_res {
                                                    Ok(hash) => hash,
                                                    Err(e) => {
                                                        err.set(Some(crate::shared::ui_error::sanitize_user_message(
                                                            format!("获取最新区块哈希失败: {}", e),
                                                        )));
                                                        loading.set(false);
                                                        return;
                                                    }
                                                };
                                                
                                                match crate::crypto::tx_signer::SolanaTxSigner::sign_transaction(
                                                    &private_key_hex,
                                                    &recipient.trim(),
                                                    &value_lamports,
                                                    &recent_blockhash,
                                                ) {
                                                    Ok(signed) => signed,
                                                    Err(e) => {
                                                        loading.set(false);
                                                        err.set(Some(crate::shared::ui_error::sanitize_user_message(
                                                            format!("签名失败: {}", e),
                                                        )));
                                                        return;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                loading.set(false);
                                                err.set(Some(crate::shared::ui_error::sanitize_user_message(
                                                    format!("获取私钥失败: {}", e),
                                                )));
                                                return;
                                            }
                                        }
                                    } else {
                                        loading.set(false);
                                        err.set(Some("钱包未解锁，无法签名交易".to_string()));
                                        return;
                                    }
                                } else if chain_lower == "ton" {
                                    // TON交易签名
                                    let key_manager_opt = app_state.key_manager.read().clone();
                                    if let Some(key_manager) = key_manager_opt {
                                        let wallet_state_read = app_state.wallet.read();
                                        let wallet_id_to_find_opt = selected_wallet_id.clone();
                                        let account_index = if let Some(wallet_id_to_find) = wallet_id_to_find_opt {
                                            current_acc.as_ref()
                                                .and_then(|acc| {
                                                    wallet_state_read.wallets.iter()
                                                        .find(|w| w.id == wallet_id_to_find)
                                                        .and_then(|w| {
                                                            w.accounts.iter()
                                                                .position(|a| a.address == acc.address)
                                                        })
                                                })
                                                .unwrap_or(0) as u32
                                        } else {
                                            0u32
                                        };
                                        
                                        match key_manager.derive_ton_private_key(account_index) {
                                            Ok(private_key_hex) => {
                                                let value_nanoton = format!("{}", (amount_val * 1e9) as u64);
                                                
                                                // Get sequence number from backend API
                                                let wallet_address = current_acc.as_ref()
                                                    .map(|acc| acc.address.clone())
                                                    .unwrap_or_default();
                                                let transaction_service = crate::services::transaction::TransactionService::new(app_state.clone());
                                                let seqno_res = transaction_service.get_seqno(&wallet_address, &chain).await;
                                                let seqno = match seqno_res {
                                                    Ok(s) => s,
                                                    Err(e) => {
                                                        err.set(Some(crate::shared::ui_error::sanitize_user_message(
                                                            format!("获取账户序列号失败: {}", e),
                                                        )));
                                                        loading.set(false);
                                                        return;
                                                    }
                                                };
                                                
                                                match crate::crypto::tx_signer::TonTxSigner::sign_transaction(
                                                    &private_key_hex,
                                                    &recipient.trim(),
                                                    &value_nanoton,
                                                    seqno,
                                                ) {
                                                    Ok(signed) => signed,
                                                    Err(e) => {
                                                        loading.set(false);
                                                        err.set(Some(crate::shared::ui_error::sanitize_user_message(
                                                            format!("签名失败: {}", e),
                                                        )));
                                                        return;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                loading.set(false);
                                                err.set(Some(crate::shared::ui_error::sanitize_user_message(
                                                    format!("获取私钥失败: {}", e),
                                                )));
                                                return;
                                            }
                                        }
                                    } else {
                                        loading.set(false);
                                        err.set(Some("钱包未解锁，无法签名交易".to_string()));
                                        return;
                                    }
                                } else {
                                    // Unsupported chain - return error instead of placeholder
                                    log::error!("Unsupported chain {} for transaction signing", chain);
                                    loading.set(false);
                                    err.set(Some(format!("不支持的链: {}", chain)));
                                    return;
                                };
                                
                                // 7. 调用后端API广播交易
                                match tx_service.broadcast(&chain, &signed_tx).await {
                                    Ok(broadcast_resp) => {
                                        // 8. 等待交易确认
                                        log::info!("Transaction broadcasted: {}", broadcast_resp.tx_hash);
                                        
                                        // 9. 轮询交易状态
                                        let tx_hash = broadcast_resp.tx_hash.clone();
                                        let mut confirmed = false;
                                        for _ in 0..30 { // 最多等待30次（约5分钟）
                                            match tx_service.status(&tx_hash).await {
                                                Ok(status) => {
                                                    if status.status == "confirmed" {
                                                        confirmed = true;
                                                        break;
                                                    } else if status.status == "failed" {
                                                        loading.set(false);
                                                        AppState::show_error(app_state.toasts, format!("交易失败: {}", tx_hash));
                                                        err.set(Some(format!("交易失败: {}", tx_hash)));
                                                        return;
                                                    }
                                                }
                                                Err(_) => {
                                                    // 继续等待
                                                }
                                            }
                                            gloo_timers::future::TimeoutFuture::new(10000).await; // 等待10秒
                                        }
                                        
                                        loading.set(false);
                                        
                                        if confirmed {
                                            // 交易成功，显示Toast并返回Dashboard
                                            AppState::show_success(app_state.toasts, format!("交易成功: {}", tx_hash));
                                            nav.push(Route::Dashboard {});
                                        } else {
                                            // 交易已广播，但未确认
                                            AppState::show_info(app_state.toasts, format!("交易已广播: {}，等待确认中...", tx_hash));
                                            err.set(Some(format!("交易已广播: {}，等待确认中...", tx_hash)));
                                        }
                                    }
                                    Err(e) => {
                                        loading.set(false);
                                        AppState::show_error(app_state.toasts, format!("交易失败: {}", e));
                                        err.set(Some(crate::shared::ui_error::sanitize_user_message(
                                            format!("交易失败: {}", e),
                                        )));
                                    }
                                }
                            });
                        }
                    }),
                    on_cancel: EventHandler::new({
                        let mut modal = show_confirm_modal;
                        move |_| {
                            modal.set(false);
                        }
                    }),
                }
            }
        }
    }
}

/// Transaction Confirm Modal - 交易确认模态框
#[component]
fn TransactionConfirmModal(
    recipient_address: String,
    amount: String,
    chain: String,
    gas_estimate: Option<GasEstimate>,
    total_cost: f64,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        Modal {
            open: true,
            onclose: move |_| {
                on_cancel.call(());
            },
            children: rsx! {
                div {
                    class: "p-6",
                    h2 {
                        class: "text-xl font-bold mb-4",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "确认交易"
                    }
                    
                    div {
                        class: "space-y-4 mb-6",
                        div {
                            class: "flex justify-between",
                            span {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "接收地址"
                            }
                            span {
                                class: "text-sm font-mono",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {
                                    let start = if recipient_address.len() > 6 {
                                        &recipient_address[..6]
                                    } else {
                                        &recipient_address
                                    };
                                    let end = if recipient_address.len() > 4 {
                                        &recipient_address[recipient_address.len()-4..]
                                    } else {
                                        ""
                                    };
                                    format!("{}...{}", start, end)
                                }
                            }
                        }
                        div {
                            class: "flex justify-between",
                            span {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "金额"
                            }
                            span {
                                class: "text-sm font-semibold",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {amount.clone()}
                            }
                        }
                        if let Some(gas) = &gas_estimate {
                            div {
                                class: "flex justify-between",
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "Gas费"
                                }
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    {format!("{:.6}", gas.max_fee_per_gas_gwei * 21000.0 / 1e9)}
                                }
                            }
                        }
                        div {
                            class: "flex justify-between pt-4 border-t",
                            style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                            span {
                                class: "text-base font-semibold",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                "总计"
                            }
                            span {
                                class: "text-base font-bold",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {format!("{:.6}", total_cost)}
                            }
                        }
                    }
                    
                    div {
                        class: "flex gap-4",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            class: Some("flex-1".to_string()),
                            onclick: move |_| {
                                on_confirm.call(());
                            },
                            "确认发送"
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Large,
                            class: Some("flex-1".to_string()),
                            onclick: move |_| {
                                on_cancel.call(());
                            },
                            "取消"
                        }
                    }
                }
            }
        }
    }
}

