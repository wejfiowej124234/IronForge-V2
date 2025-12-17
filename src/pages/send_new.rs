//! Send Page - 发送页面（优化版）
//! 实现双模式设计：简单支付（智能模式）和选择支付钱包（高级模式）

use crate::components::atoms::button::{Button, ButtonVariant, ButtonSize};
use crate::components::atoms::card::Card;
use crate::components::atoms::input::{Input, InputType};
use crate::components::atoms::modal::Modal;
use crate::components::molecules::{ChainSelector, GasFeeCard, ErrorMessage};
use crate::features::wallet::hooks::use_wallet;
use crate::features::wallet::state::Account;
use crate::router::Route;
use crate::services::address_detector::{AddressDetector, ChainType};
use crate::services::payment_router::{PaymentRouter, PaymentStrategy};
use crate::services::gas::{GasEstimate, GasService};
use crate::services::transaction::TransactionService;
use crate::services::bridge::BridgeService;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use dioxus::events::FormEvent;
use anyhow::Result;

/// 支付模式
#[derive(Debug, Clone, Copy, PartialEq)]
enum PaymentMode {
    /// 简单支付（智能模式）- 默认
    Simple,
    /// 选择支付钱包（高级模式）
    Advanced,
}

/// 执行直接转账
async fn execute_direct_transfer(
    app_state: &AppState,
    wallet_ctrl: &crate::features::wallet::hooks::WalletController,
    recipient: &str,
    amount: f64,
    chain: &ChainType,
    account: &Account,
) -> Result<()> {
    // 这里应该调用实际的交易发送逻辑
    // 暂时返回成功，后续集成实际的交易发送代码
    log::info!("执行直接转账: {} -> {}, 金额: {}, 链: {}", account.address, recipient, amount, chain.label());
    Ok(())
}

/// 执行跨链桥转账
async fn execute_bridge_transfer(
    app_state: &AppState,
    wallet_ctrl: &crate::features::wallet::hooks::WalletController,
    recipient: &str,
    amount: f64,
    from_chain: &ChainType,
    from_account: &Account,
    to_chain: &ChainType,
) -> Result<()> {
    // 这里应该调用实际的跨链桥逻辑
    // 暂时返回成功，后续集成实际的跨链桥代码
    log::info!("执行跨链桥: {} -> {}, 金额: {}, 从{}到{}", 
        from_account.address, recipient, amount, from_chain.label(), to_chain.label());
    Ok(())
}

/// Send Page - 发送页面（优化版）
#[component]
pub fn Send() -> Element {
    let app_state = use_context::<AppState>();
    let navigator = use_navigator();
    let wallet_controller = use_wallet();
    
    // 模式选择
    let payment_mode = use_signal(|| PaymentMode::Simple);
    
    // 表单状态
    let recipient_address = use_signal(|| String::new());
    let amount = use_signal(|| String::new());
    let selected_chain = use_signal(|| Option::<ChainType>::None); // 高级模式使用
    
    // 检测结果
    let detected_chain = use_signal(|| Option::<ChainType>::None);
    let payment_strategy = use_signal(|| Option::<PaymentStrategy>::None);
    
    // UI状态
    let error_message = use_signal(|| Option::<String>::None);
    let is_loading = use_signal(|| false);
    let show_confirm_modal = use_signal(|| false);
    let gas_estimate = use_signal(|| Option::<GasEstimate>::None);
    let gas_loading = use_signal(|| false);
    
    // 获取当前钱包
    let current_wallet = use_memo(move || {
        let wallet_state = app_state.wallet.read();
        wallet_state.get_selected_wallet().cloned()
    });
    
    // 地址变化时自动检测
    use_effect(move || {
        let addr = recipient_address.read().clone();
        if !addr.trim().is_empty() {
            match AddressDetector::detect_chain(&addr) {
                Ok(chain) => {
                    detected_chain.set(Some(chain));
                    error_message.set(None);
                    
                    // 如果是简单模式，自动选择支付策略
                    if *payment_mode.read() == PaymentMode::Simple {
                        if let Some(wallet) = current_wallet.read().as_ref() {
                            let amount_val: f64 = amount.read().parse().unwrap_or(0.0);
                            if amount_val > 0.0 {
                                match PaymentRouter::select_payment_strategy(chain, amount_val, wallet) {
                                    Ok(strategy) => {
                                        payment_strategy.set(Some(strategy));
                                    }
                                    Err(e) => {
                                        error_message.set(Some(e.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    detected_chain.set(None);
                    if addr.len() > 5 {
                        // 只有地址足够长时才显示错误
                        #[cfg(debug_assertions)]
                        tracing::debug!("address_detect_error={}", e);

                        error_message.set(Some("无法识别地址格式，请检查后重试".to_string()));
                    }
                }
            }
        } else {
            detected_chain.set(None);
            payment_strategy.set(None);
        }
    });
    
    // 金额变化时重新计算策略（简单模式）
    use_effect(move || {
        if *payment_mode.read() == PaymentMode::Simple {
            if let (Some(chain), Some(wallet)) = (detected_chain.read().as_ref(), current_wallet.read().as_ref()) {
                let amount_val: f64 = amount.read().parse().unwrap_or(0.0);
                if amount_val > 0.0 {
                    match PaymentRouter::select_payment_strategy(*chain, amount_val, wallet) {
                        Ok(strategy) => {
                            payment_strategy.set(Some(strategy));
                        }
                        Err(e) => {
                            error_message.set(Some(e.to_string()));
                        }
                    }
                }
            }
        }
    });
    
    // 加载Gas费用
    use_effect(move || {
        let chain_str = if let Some(chain) = detected_chain.read().as_ref() {
            chain.as_str()
        } else if let Some(chain) = selected_chain.read().as_ref() {
            chain.as_str()
        } else {
            "ethereum"
        };
        
        let app_state_clone = app_state;
        let mut gas_est = gas_estimate;
        let mut gas_load = gas_loading;
        
        spawn(async move {
            gas_load.set(true);
            let gas_service = GasService::new(app_state_clone);
            match gas_service.get_recommended(chain_str).await {
                Ok(est) => {
                    gas_est.set(Some(est));
                }
                Err(_) => {
                    // 静默失败，不阻塞用户
                }
            }
            gas_load.set(false);
        });
    });
    
    rsx! {
        div {
            class: "min-h-screen p-4",
            style: format!("background: {};", Colors::BG_PRIMARY),
            
            div {
                class: "container mx-auto max-w-2xl px-4 sm:px-6",
                
                // 页面标题
                div {
                    class: "mb-6",
                    h1 {
                        class: "text-2xl font-bold mb-4",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "发送"
                    }
                    
                    // 模式切换
                    div {
                        class: "flex gap-2 mb-4",
                        Button {
                            variant: if *payment_mode.read() == PaymentMode::Simple {
                                ButtonVariant::Primary
                            } else {
                                ButtonVariant::Secondary
                            },
                            size: ButtonSize::Medium,
                            class: Some("flex-1".to_string()),
                            onclick: move |_| {
                                payment_mode.set(PaymentMode::Simple);
                                selected_chain.set(None);
                                payment_strategy.set(None);
                            },
                            "💡 简单支付"
                        }
                        Button {
                            variant: if *payment_mode.read() == PaymentMode::Advanced {
                                ButtonVariant::Primary
                            } else {
                                ButtonVariant::Secondary
                            },
                            size: ButtonSize::Medium,
                            class: Some("flex-1".to_string()),
                            onclick: move |_| {
                                payment_mode.set(PaymentMode::Advanced);
                                payment_strategy.set(None);
                            },
                            "⚙️ 选择支付钱包"
                        }
                    }
                    
                    // 模式说明
                    if *payment_mode.read() == PaymentMode::Simple {
                        p {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "智能模式：自动检测地址链类型，使用余额最多的链进行支付"
                        }
                    } else {
                        p {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "高级模式：手动选择支付链，同链支付"
                        }
                    }
                }
                
                Card {
                    variant: crate::components::atoms::card::CardVariant::Base,
                    padding: Some("24px".to_string()),
                    children: rsx! {
                        // 高级模式：链选择器
                        if *payment_mode.read() == PaymentMode::Advanced {
                            div {
                                class: "mb-6",
                                ChainSelector {
                                    selected_chain: {
                                        let chain_str = use_signal(|| {
                                            selected_chain.read()
                                                .map(|c| c.as_str().to_string())
                                                .unwrap_or_else(|| "ethereum".to_string())
                                        });
                                        
                                        // 同步ChainSelector的选择到selected_chain
                                        use_effect(move || {
                                            let chain_val = chain_str.read().clone();
                                            if let Some(chain) = ChainType::from_str(&chain_val) {
                                                selected_chain.set(Some(chain));
                                            }
                                        });
                                        
                                        chain_str
                                    }
                                }
                            }
                        }
                        
                        // 接收地址输入
                        div {
                            class: "mb-6",
                            Input {
                                input_type: InputType::Text,
                                label: Some("接收地址".to_string()),
                                placeholder: Some("请输入接收地址".to_string()),
                                value: Some(recipient_address.read().clone()),
                                onchange: {
                                    let mut recipient_address = recipient_address;
                                    Some(EventHandler::new(move |e: FormEvent| {
                                        recipient_address.set(e.value());
                                    }))
                                },
                            }
                            
                            // 地址检测结果
                            if let Some(chain) = detected_chain.read().as_ref() {
                                div {
                                    class: "mt-2 p-2 rounded-lg",
                                    style: format!("background: rgba(34, 197, 94, 0.1); border: 1px solid rgba(34, 197, 94, 0.3);"),
                                    div {
                                        class: "flex items-center gap-2",
                                        span {
                                            class: "text-sm",
                                            style: format!("color: rgb(34, 197, 94);"),
                                            "✓ 检测到: {}链", chain.label()
                                        }
                                    }
                                }
                            }
                        }
                        
                        // 金额输入
                        div {
                            class: "mb-6",
                            Input {
                                input_type: InputType::Text,
                                label: Some("金额".to_string()),
                                placeholder: Some("0.0".to_string()),
                                value: Some(amount.read().clone()),
                                onchange: {
                                    let mut amount = amount;
                                    Some(EventHandler::new(move |e: FormEvent| {
                                        amount.set(e.value());
                                    }))
                                },
                            }
                            
                            // 显示当前账户余额（如果有）
                            if let Some(wallet) = current_wallet.read().as_ref() {
                                if let Some(acc) = wallet.accounts.first() {
                                    div {
                                        class: "mt-2 text-sm",
                                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                                        "可用余额: {}", acc.balance
                                    }
                                }
                            }
                        }
                        
                        // 支付策略预览（简单模式）
                        if *payment_mode.read() == PaymentMode::Simple {
                            if let Some(strategy) = payment_strategy.read().as_ref() {
                                PaymentStrategyPreview {
                                    strategy: strategy.clone(),
                                }
                            }
                        }
                        
                        // 高级模式：链匹配检查
                        if *payment_mode.read() == PaymentMode::Advanced {
                            if let (Some(detected), Some(selected)) = (detected_chain.read().as_ref(), selected_chain.read().as_ref()) {
                                if detected != selected {
                                    div {
                                        class: "mb-4 p-4 rounded-lg",
                                        style: format!("background: rgba(239, 68, 68, 0.1); border: 1px solid {};", Colors::PAYMENT_ERROR),
                                        div {
                                            class: "flex items-start gap-2",
                                            span {
                                                class: "text-xl",
                                                "⚠️"
                                            }
                                            div {
                                                p {
                                                    class: "text-sm font-semibold mb-1",
                                                    style: format!("color: {};", Colors::PAYMENT_ERROR),
                                                    "链不匹配"
                                                }
                                                p {
                                                    class: "text-sm mb-3",
                                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                                    "接收地址属于{}链，但您选择的是{}链。请切换到正确的链。",
                                                    detected.label(),
                                                    selected.label()
                                                }
                                                if let Some(wallet) = current_wallet.read().as_ref() {
                                                    if let Some(acc) = wallet.accounts.iter().find(|a| {
                                                        ChainType::from_str(&a.chain).map_or(false, |c| c == *detected)
                                                    }) {
                                                        let balance: f64 = acc.balance.parse().unwrap_or(0.0);
                                                        let amount_val: f64 = amount.read().parse().unwrap_or(0.0);
                                                        if balance < amount_val {
                                                            // 余额不足，建议使用余额最多的链
                                                            if let Ok(strategy) = PaymentRouter::select_payment_strategy(*detected, amount_val, wallet) {
                                                                if let PaymentStrategy::Bridge { from_chain, from_account, to_chain, estimated_fee } = strategy {
                                                                    div {
                                                                        class: "mt-3 p-3 rounded-lg",
                                                                        style: format!("background: rgba(251, 191, 36, 0.1); border: 1px solid rgba(251, 191, 36, 0.3);"),
                                                                        p {
                                                                            class: "text-sm mb-2",
                                                                            style: format!("color: rgb(251, 191, 36);"),
                                                                            "💡 建议：您的{}链余额不足，可以使用{}链（余额：{}）进行跨链支付",
                                                                            detected.label(),
                                                                            from_chain.label(),
                                                                            from_account.balance
                                                                        }
                                                                        Button {
                                                                            variant: ButtonVariant::Primary,
                                                                            size: ButtonSize::Small,
                                                                            onclick: move |_| {
                                                                                // 切换到简单模式并使用建议的链
                                                                                payment_mode.set(PaymentMode::Simple);
                                                                                payment_strategy.set(Some(PaymentStrategy::Bridge {
                                                                                    from_chain,
                                                                                    from_account: from_account.clone(),
                                                                                    to_chain,
                                                                                    estimated_fee,
                                                                                }));
                                                                            },
                                                                            "使用{}链支付", from_chain.label()
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
                                }
                            }
                        }
                        
                        // Gas费用显示
                        GasFeeCard {
                            gas_estimate: gas_estimate.read().clone(),
                            is_loading: gas_loading()
                        }
                        
                        // 错误提示
                        ErrorMessage {
                            message: error_message.read().clone(),
                        }
                        
                        // 操作按钮
                        div {
                            class: "flex gap-4 mt-6",
                            Button {
                                variant: ButtonVariant::Primary,
                                size: ButtonSize::Large,
                                class: Some("flex-1".to_string()),
                                disabled: {
                                    recipient_address.read().trim().is_empty() ||
                                    amount.read().trim().is_empty() ||
                                    detected_chain.read().is_none() ||
                                    (*payment_mode.read() == PaymentMode::Advanced && 
                                     selected_chain.read().is_none()) ||
                                    error_message.read().is_some() ||
                                    *is_loading.read()
                                },
                                loading: *is_loading.read(),
                                onclick: move |_| {
                                    show_confirm_modal.set(true);
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
            }
            
            // 确认模态框
            if show_confirm_modal() {
                TransactionConfirmModal {
                    recipient_address: recipient_address.read().clone(),
                    amount: amount.read().clone(),
                    detected_chain: detected_chain.read().clone(),
                    selected_chain: selected_chain.read().clone(),
                    payment_strategy: payment_strategy.read().clone(),
                    payment_mode: *payment_mode.read(),
                    gas_estimate: gas_estimate.read().clone(),
                    on_confirm: EventHandler::new({
                        let app_state = app_state;
                        let recipient = recipient_address.read().clone();
                        let amt = amount.read().clone();
                        let strategy = payment_strategy.read().clone();
                        let detected = detected_chain.read().clone();
                        let wallet_ctrl = wallet_controller;
                        let nav = navigator.clone();
                        let mut loading = is_loading;
                        let mut modal = show_confirm_modal;
                        let mut err = error_message;
                        let toasts = app_state.toasts;
                        move |_| {
                            loading.set(true);
                            modal.set(false);
                            
                            spawn(async move {
                                // 验证输入
                                if recipient.trim().is_empty() {
                                    loading.set(false);
                                    err.set(Some("请输入接收地址".to_string()));
                                    return;
                                }
                                
                                let amount_val: f64 = match amt.parse() {
                                    Ok(v) if v > 0.0 => v,
                                    _ => {
                                        loading.set(false);
                                        err.set(Some("请输入有效的金额".to_string()));
                                        return;
                                    }
                                };
                                
                                // 根据支付策略执行交易
                                match strategy {
                                    Some(PaymentStrategy::Direct { chain, account }) => {
                                        // 直接发送
                                        match execute_direct_transfer(
                                            &app_state,
                                            &wallet_ctrl,
                                            &recipient,
                                            amount_val,
                                            &chain,
                                            &account,
                                        ).await {
                                            Ok(_) => {
                                                AppState::show_success(toasts, "交易发送成功".to_string());
                                                loading.set(false);
                                                nav.push(Route::Dashboard {});
                                            }
                                            Err(e) => {
                                                err.set(Some(
                                                    crate::shared::ui_error::sanitize_user_message(
                                                        format!("发送失败: {}", e),
                                                    ),
                                                ));
                                                loading.set(false);
                                            }
                                        }
                                    }
                                    Some(PaymentStrategy::Bridge { from_chain, from_account, to_chain, estimated_fee }) => {
                                        // 跨链桥
                                        match execute_bridge_transfer(
                                            &app_state,
                                            &wallet_ctrl,
                                            &recipient,
                                            amount_val,
                                            &from_chain,
                                            &from_account,
                                            &to_chain,
                                        ).await {
                                            Ok(_) => {
                                                AppState::show_success(toasts, "跨链转账已发起".to_string());
                                                loading.set(false);
                                                nav.push(Route::Dashboard {});
                                            }
                                            Err(e) => {
                                                err.set(Some(
                                                    crate::shared::ui_error::sanitize_user_message(
                                                        format!("跨链转账失败: {}", e),
                                                    ),
                                                ));
                                                loading.set(false);
                                            }
                                        }
                                    }
                                    Some(PaymentStrategy::InsufficientBalance { message, .. }) => {
                                        err.set(Some(message));
                                        loading.set(false);
                                    }
                                    None => {
                                        err.set(Some("请先输入地址和金额".to_string()));
                                        loading.set(false);
                                    }
                                }
                            });
                        }
                    }),
                    on_cancel: EventHandler::new(move |_| {
                        show_confirm_modal.set(false);
                    }),
                }
            }
        }
    }
}

/// 支付策略预览组件
#[component]
fn PaymentStrategyPreview(strategy: PaymentStrategy) -> Element {
    rsx! {
        div {
            class: "mb-6 p-4 rounded-lg",
            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
            match strategy {
                PaymentStrategy::Direct { chain, account } => {
                    rsx! {
                        div {
                            class: "space-y-2",
                            div {
                                class: "flex items-center gap-2",
                                span {
                                    class: "text-sm font-semibold",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    "✓ 直接发送"
                                }
                            }
                            div {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "从: {}链 (余额: {})", chain.label(), account.balance
                            }
                            div {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "到: {}链", chain.label()
                            }
                        }
                    }
                }
                PaymentStrategy::Bridge { from_chain, from_account, to_chain, estimated_fee } => {
                    rsx! {
                        div {
                            class: "space-y-2",
                            div {
                                class: "flex items-center gap-2",
                                span {
                                    class: "text-sm font-semibold",
                                    style: format!("color: rgb(34, 197, 94);"),
                                    "🌉 跨链支付"
                                }
                            }
                            div {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "从: {}链 (余额: {})", from_chain.label(), from_account.balance
                            }
                            div {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "到: {}链", to_chain.label()
                            }
                            div {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "跨链费用: ~{:.6}", estimated_fee
                            }
                            div {
                                class: "text-xs mt-2 p-2 rounded",
                                style: format!("background: rgba(34, 197, 94, 0.1); color: rgb(34, 197, 94);"),
                                "💡 系统将自动执行跨链桥，将资产从{}链转移到{}链",
                                from_chain.label(),
                                to_chain.label()
                            }
                        }
                    }
                }
                PaymentStrategy::InsufficientBalance { message, suggestion } => {
                    rsx! {
                        div {
                            class: "space-y-2",
                            div {
                                class: "flex items-center gap-2",
                                span {
                                    class: "text-sm font-semibold",
                                    style: format!("color: {};", Colors::PAYMENT_ERROR),
                                    "⚠️ 余额不足"
                                }
                            }
                            p {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                {message}
                            }
                            if let Some(sug) = suggestion {
                                div {
                                    class: "mt-3 p-3 rounded-lg",
                                    style: format!("background: rgba(251, 191, 36, 0.1); border: 1px solid rgba(251, 191, 36, 0.3);"),
                                    p {
                                        class: "text-sm mb-2",
                                        style: format!("color: rgb(251, 191, 36);"),
                                        "💡 建议：使用{}链 (余额: {:.6}) 进行跨链支付",
                                        sug.from_chain.label(),
                                        sug.from_balance
                                    }
                                    p {
                                        class: "text-xs",
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        "跨链费用: ~{:.6}", sug.estimated_fee
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

/// 交易确认模态框
#[component]
fn TransactionConfirmModal(
    recipient_address: String,
    amount: String,
    detected_chain: Option<ChainType>,
    selected_chain: Option<ChainType>,
    payment_strategy: Option<PaymentStrategy>,
    payment_mode: PaymentMode,
    gas_estimate: Option<GasEstimate>,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        Modal {
            open: true,
            on_close: on_cancel,
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
                                {format!("{}...{}", &recipient_address[..6], &recipient_address[recipient_address.len()-4..])}
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
                        if let Some(chain) = detected_chain {
                            div {
                                class: "flex justify-between",
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "目标链"
                                }
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    {chain.label()}
                                }
                            }
                        }
                        if let Some(strategy) = payment_strategy {
                            match strategy {
                                PaymentStrategy::Bridge { from_chain, to_chain, estimated_fee, .. } => {
                                    rsx! {
                                        div {
                                            class: "p-3 rounded-lg",
                                            style: format!("background: rgba(34, 197, 94, 0.1); border: 1px solid rgba(34, 197, 94, 0.3);"),
                                            div {
                                                class: "text-sm font-semibold mb-2",
                                                style: format!("color: rgb(34, 197, 94);"),
                                                "🌉 跨链支付"
                                            }
                                            div {
                                                class: "text-xs space-y-1",
                                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                                div { "从: {from_chain.label()}" }
                                                div { "到: {to_chain.label()}" }
                                                div { "跨链费用: ~{estimated_fee:.6}" }
                                            }
                                        }
                                    }
                                }
                                _ => {}
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

