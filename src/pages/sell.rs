//! Sell Page - 法币提现页面
//! 企业级法币提现实现，支持代币→稳定币→法币的自动两步流程

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::atoms::input::{Input, InputType};
use crate::components::molecules::ErrorMessage;
use crate::components::molecules::token_selector::TokenSelector; // ✅ 添加TokenSelector
use crate::features::wallet::unlock::ensure_wallet_unlocked;
use crate::router::Route;
use crate::services::address_detector::ChainType; // ✅ 添加ChainType
use crate::services::fiat_offramp::{FiatOfframpQuoteResponse, FiatOfframpService};
use crate::services::token::TokenInfo; // ✅ 添加TokenInfo
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use std::sync::Arc;

/// 提现方式选项（6个国际标准方式）
#[derive(Debug, Clone, Copy, PartialEq)]
enum WithdrawMethod {
    BankCard,      // 银行卡（推荐）
    PayPal,        // PayPal
    ApplePay,      // Apple Pay
    GooglePay,     // Google Pay
    Alipay,        // 支付宝
    WechatPay,     // 微信支付
}

impl WithdrawMethod {
    fn value(&self) -> &'static str {
        match self {
            WithdrawMethod::BankCard => "bank_card",
            WithdrawMethod::PayPal => "paypal",
            WithdrawMethod::ApplePay => "apple_pay",
            WithdrawMethod::GooglePay => "google_pay",
            WithdrawMethod::Alipay => "alipay",
            WithdrawMethod::WechatPay => "wechat_pay",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            WithdrawMethod::BankCard => "💳 银行卡/借记卡",
            WithdrawMethod::PayPal => "📱 PayPal",
            WithdrawMethod::ApplePay => "🍎 Apple Pay",
            WithdrawMethod::GooglePay => "📱 Google Pay",
            WithdrawMethod::Alipay => "💰 支付宝 Alipay",
            WithdrawMethod::WechatPay => "💬 微信支付 WeChat Pay",
        }
    }
    
    fn description(&self) -> &'static str {
        match self {
            WithdrawMethod::BankCard => "1-3工作日 · 全球支持",
            WithdrawMethod::PayPal => "即时到账 · 全球支付",
            WithdrawMethod::ApplePay => "即时到账 · iOS设备",
            WithdrawMethod::GooglePay => "即时到账 · Android设备",
            WithdrawMethod::Alipay => "即时到账 · 中国地区",
            WithdrawMethod::WechatPay => "即时到账 · 中国地区",
        }
    }
    
    fn is_recommended(&self) -> bool {
        matches!(self, WithdrawMethod::BankCard)
    }

    fn all() -> Vec<Self> {
        vec![
            WithdrawMethod::BankCard,
            WithdrawMethod::PayPal,
            WithdrawMethod::ApplePay,
            WithdrawMethod::GooglePay,
            WithdrawMethod::Alipay,
            WithdrawMethod::WechatPay,
        ]
    }
}

/// 法币货币选项
#[derive(Debug, Clone, Copy, PartialEq)]
enum FiatCurrency {
    USD,
    EUR,
    CNY,
    GBP,
}

impl FiatCurrency {
    fn value(&self) -> &'static str {
        match self {
            FiatCurrency::USD => "USD",
            FiatCurrency::EUR => "EUR",
            FiatCurrency::CNY => "CNY",
            FiatCurrency::GBP => "GBP",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            FiatCurrency::USD => "🇺🇸 USD - 美元",
            FiatCurrency::EUR => "🇪🇺 EUR - 欧元",
            FiatCurrency::CNY => "🇨🇳 CNY - 人民币",
            FiatCurrency::GBP => "🇬🇧 GBP - 英镑",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            FiatCurrency::USD,
            FiatCurrency::EUR,
            FiatCurrency::CNY,
            FiatCurrency::GBP,
        ]
    }
}

/// 代币选项（支持原生代币）
#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    ETH,
    BTC,
    SOL,
    USDT,
    USDC,
}

impl Token {
    fn value(&self) -> &'static str {
        match self {
            Token::ETH => "ETH",
            Token::BTC => "BTC",
            Token::SOL => "SOL",
            Token::USDT => "USDT",
            Token::USDC => "USDC",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Token::ETH => "🔷 ETH (Ethereum)",
            Token::BTC => "🟠 BTC (Bitcoin)",
            Token::SOL => "🟣 SOL (Solana)",
            Token::USDT => "💵 USDT (Tether)",
            Token::USDC => "💵 USDC (USD Coin)",
        }
    }

    fn chain(&self) -> &'static str {
        match self {
            Token::ETH | Token::USDT | Token::USDC => "ethereum",
            Token::BTC => "bitcoin",
            Token::SOL => "solana",
        }
    }

    fn all() -> Vec<Self> {
        vec![Token::ETH, Token::BTC, Token::SOL, Token::USDT, Token::USDC]
    }
}

/// Sell Page - 主组件
#[component]
pub fn Sell() -> Element {
    let app_state = use_context::<AppState>();
    let navigator = use_navigator();

    // 检查用户是否已登录
    let is_authenticated = use_memo(move || {
        let user_state = app_state.user.read();
        user_state.is_authenticated
    });

    // 如果未登录，显示登录提示
    if !*is_authenticated.read() {
        return rsx! {
            div { class: "min-h-screen p-4", style: format!("background: {};", Colors::BG_PRIMARY),
                div { class: "container mx-auto max-w-3xl px-4 sm:px-6 flex items-center justify-center h-[70vh]",
                    Card {
                        variant: crate::components::atoms::card::CardVariant::Base,
                        padding: Some("32px".to_string()),
                        children: rsx! {
                            div { class: "text-center",
                                h1 { class: "text-2xl font-bold mb-4", style: format!("color: {};", Colors::TEXT_PRIMARY), "🔒 需要登录" }
                                p { class: "text-sm mb-4", style: format!("color: {};", Colors::TEXT_SECONDARY), "请先登录您的账户，然后再进行法币提现操作。" }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    size: ButtonSize::Large,
                                    onclick: move |_| { navigator.push(Route::Login {}); },
                                    "前往登录"
                                }
                            }
                        }
                    }
                }
            }
        };
    }

    // 当前选中钱包（入口级安全门）
    let current_wallet = use_memo(move || {
        let wallet_state = app_state.wallet.read();
        wallet_state.get_selected_wallet().cloned()
    });

    // 如果未选择钱包，直接显示提示
    if current_wallet.read().is_none() {
        return rsx! {
            div { class: "min-h-screen p-4", style: format!("background: {};", Colors::BG_PRIMARY),
                div { class: "container mx-auto max-w-3xl px-4 sm:px-6 flex items-center justify-center h-[70vh]",
                    Card {
                        variant: crate::components::atoms::card::CardVariant::Base,
                        padding: Some("32px".to_string()),
                        children: rsx! {
                            div { class: "text-center",
                                h1 { class: "text-2xl font-bold mb-4", style: format!("color: {};", Colors::TEXT_PRIMARY), "💰 提现到银行卡" }
                                p { class: "text-sm mb-4", style: format!("color: {};", Colors::TEXT_SECONDARY), "请先在仪表盘选择并解锁一个钱包，然后再进行法币提现操作。" }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    size: ButtonSize::Large,
                                    onclick: move |_| { navigator.push(Route::Dashboard {}); },
                                    "前往仪表盘选择钱包"
                                }
                            }
                        }
                    }
                }
            }
        };
    }

    // 表单状态
    let amount = use_signal(|| "1".to_string());
    let selected_token = use_signal(|| Option::<TokenInfo>::None); // ✅ 改为使用TokenInfo
    let mut selected_currency = use_signal(|| FiatCurrency::USD);
    let mut selected_withdraw_method = use_signal(|| WithdrawMethod::BankCard);
    let recipient_info = use_signal(|| String::new());

    // 报价状态
    let quote = use_signal(|| None::<FiatOfframpQuoteResponse>);
    let is_loading = use_signal(|| false);
    let error_message = use_signal(|| None::<String>);

    // 订单状态
    let order_created = use_signal(|| false);
    let order_id = use_signal(|| None::<String>);

    // 获取报价
    let get_quote = {
        let app_state = app_state.clone();
        move |_| {
            let app_state = app_state.clone();
            let token_opt = selected_token.read().clone();
            if token_opt.is_none() {
                return; // 未选择代币，不执行
            }
            let token_info = token_opt.unwrap();
            let token = token_info.symbol.clone();
            let amount = amount.read().clone();
            let chain = token_info.chain.as_str();
            let currency = selected_currency.read().value();
            let withdraw_method = selected_withdraw_method.read().value();
            let mut is_loading = is_loading;
            let mut error_message = error_message;
            let mut quote = quote;

            spawn(async move {
                is_loading.set(true);
                error_message.set(None);

                let service = FiatOfframpService::new(Arc::new(app_state));
                match service
                    .get_quote(&token, &amount, &chain, currency, withdraw_method)
                    .await
                {
                    Ok(q) => {
                        quote.set(Some(q));
                        is_loading.set(false);
                    }
                    Err(e) => {
                        error_message.set(Some(e));
                        is_loading.set(false);
                    }
                }
            });
        }
    };

    // 创建提现订单
    let create_order = {
        let app_state = app_state.clone();
        move |_| {
            let app_state = app_state.clone();
            let token_opt = selected_token.read().clone();
            if token_opt.is_none() {
                return; // 未选择代币，不执行
            }
            let token_info = token_opt.unwrap();
            let token = token_info.symbol.clone();
            let amount = amount.read().clone();
            let chain = token_info.chain.as_str();
            let currency = selected_currency.read().value();
            let withdraw_method = selected_withdraw_method.read().value();
            let recipient = recipient_info.read().clone();
            let current_quote = quote.read().clone();
            let mut is_loading = is_loading;
            let mut error_message = error_message;
            let mut order_created = order_created;
            let mut order_id = order_id;

            spawn(async move {
                is_loading.set(true);
                error_message.set(None);

                // 验证钱包已解锁
                let wallet_state = app_state.wallet.read();
                if let Some(wallet) = wallet_state.get_selected_wallet() {
                    if let Err(e) = ensure_wallet_unlocked(&app_state, &wallet.id) {
                        error_message.set(Some(format!("钱包未解锁: {}", e)));
                        is_loading.set(false);
                        return;
                    }
                } else {
                    error_message.set(Some("未选择钱包".to_string()));
                    is_loading.set(false);
                    return;
                }

                // 验证收款信息
                if recipient.is_empty() {
                    error_message.set(Some("请输入收款账户信息".to_string()));
                    is_loading.set(false);
                    return;
                }

                let quote_id = current_quote.map(|q| q.quote_id).unwrap_or_default();

                let service = FiatOfframpService::new(Arc::new(app_state));
                match service
                    .create_order(
                        &token,
                        &amount,
                        &chain,
                        currency,
                        withdraw_method,
                        &recipient,
                        Some(&quote_id),
                    )
                    .await
                {
                    Ok(order) => {
                        order_created.set(true);
                        order_id.set(Some(order.order_id.clone()));
                        is_loading.set(false);
                    }
                    Err(e) => {
                        error_message.set(Some(e));
                        is_loading.set(false);
                    }
                }
            });
        }
    };

    // 如果订单已创建，显示成功提示
    if *order_created.read() {
        return rsx! {
            div { class: "min-h-screen p-4", style: format!("background: {};", Colors::BG_PRIMARY),
                div { class: "container mx-auto max-w-3xl px-4 sm:px-6 py-8",
                    Card {
                        variant: crate::components::atoms::card::CardVariant::Base,
                        padding: Some("32px".to_string()),
                        children: rsx! {
                            div { class: "text-center",
                                div { class: "text-6xl mb-4", "✅" }
                                h1 { class: "text-2xl font-bold mb-4", style: format!("color: {};", Colors::TEXT_PRIMARY), "提现订单创建成功！" }
                                p { class: "text-sm mb-6", style: format!("color: {};", Colors::TEXT_SECONDARY), "您的提现订单已提交，系统将自动处理代币兑换和法币提现流程。" }
                                
                                if let Some(id) = (*order_id.read()).clone() {
                                    div { class: "mb-6 p-4 rounded-lg", style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                                        p { class: "text-xs", style: format!("color: {};", Colors::TEXT_SECONDARY), "订单ID：" }
                                        p { class: "text-sm font-mono mt-1", style: format!("color: {};", Colors::TEXT_PRIMARY), "{id}" }
                                    }
                                }

                                div { class: "space-y-3",
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        size: ButtonSize::Large,
                                        onclick: move |_| { navigator.push(Route::Dashboard {}); },
                                        "返回仪表盘"
                                    }
                                    p { class: "text-xs", style: format!("color: {};", Colors::TEXT_SECONDARY), 
                                        match *selected_withdraw_method.read() {
                                            WithdrawMethod::BankCard => "⏰ 预计 1-3 个工作日到账，请留意您的银行账户。",
                                            WithdrawMethod::PayPal => "⚡ PayPal 预计即时到账，请检查您的 PayPal 账户。",
                                            WithdrawMethod::ApplePay => "⚡ Apple Pay 预计即时到账，请检查您绑定的银行卡。",
                                            WithdrawMethod::GooglePay => "⚡ Google Pay 预计即时到账，请检查您绑定的银行卡。",
                                            WithdrawMethod::Alipay => "⚡ 支付宝预计即时到账，请检查支付宝余额。",
                                            WithdrawMethod::WechatPay => "⚡ 微信支付预计即时到账，请检查微信零钱。",
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
    }

    rsx! {
        div { class: "min-h-screen p-4", style: format!("background: {};", Colors::BG_PRIMARY),
            div { class: "container mx-auto max-w-3xl px-4 sm:px-6 py-8",
                // 页面标题
                div { class: "mb-6",
                    button {
                        onclick: move |_| { navigator.push(Route::Dashboard {}); },
                        class: "flex items-center gap-2 mb-4 transition-colors",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "← 返回仪表盘"
                    }
                    h1 { class: "text-3xl font-bold", style: format!("color: {};", Colors::TEXT_PRIMARY), "💰 提现到银行卡" }
                    p { class: "text-sm mt-2", style: format!("color: {};", Colors::TEXT_SECONDARY), 
                        "将加密货币提现为法币，支持 ETH、BTC、SOL 等主流币种。系统将自动完成：代币 → 稳定币 → 法币的两步转换。" 
                    }
                }

                // 表单卡片
                Card {
                    variant: crate::components::atoms::card::CardVariant::Base,
                    padding: Some("24px".to_string()),
                    children: rsx! {
                        div { class: "space-y-6",
                            // 代币选择（✅ 使用TokenSelector从钱包真实余额加载）
                            div {
                                label { class: "block text-sm font-medium mb-2", style: format!("color: {};", Colors::TEXT_PRIMARY), "选择代币" }
                                // ✅ 获取当前选中钱包的第一个账户地址（用于加载余额）
                                TokenSelector {
                                    chain: ChainType::Ethereum, // 默认以太坊链，用户可通过代币选择切换
                                    selected_token: selected_token,
                                    wallet_address: app_state.wallet.read()
                                        .get_selected_wallet()
                                        .and_then(|w| w.accounts.first())
                                        .map(|a| a.address.clone()),
                                }
                            }

                            // 提现数量
                            div {
                                label { class: "block text-sm font-medium mb-2", style: format!("color: {};", Colors::TEXT_PRIMARY), "提现数量" }
                                Input {
                                    input_type: InputType::Text,
                                    placeholder: Some("请输入提现数量".to_string()),
                                    value: Some(amount.read().clone()),
                                    onchange: {
                                        let mut amount = amount;
                                        Some(EventHandler::new(move |e: FormEvent| {
                                            amount.set(e.value());
                                        }))
                                    },
                                }
                                p { class: "text-xs mt-1", style: format!("color: {};", Colors::TEXT_SECONDARY), "系统将自动兑换为稳定币后提现" }
                            }

                            // 目标法币
                            div {
                                label { class: "block text-sm font-medium mb-2", style: format!("color: {};", Colors::TEXT_PRIMARY), "目标法币" }
                                div { class: "grid grid-cols-2 gap-2",
                                    for currency in FiatCurrency::all() {
                                        button {
                                            key: "{currency.value()}",
                                            onclick: move |_| { selected_currency.set(currency); },
                                            class: "p-3 rounded-lg border-2 transition-all",
                                            style: if *selected_currency.read() == currency {
                                                format!("background: {}; border-color: {}; color: {};", Colors::TECH_PRIMARY, Colors::TECH_PRIMARY, Colors::TEXT_PRIMARY)
                                            } else {
                                                format!("background: {}; border-color: {}; color: {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY, Colors::TEXT_SECONDARY)
                                            },
                                            "{currency.label()}"
                                        }
                                    }
                                }
                            }

                            // 提现方式（6个国际标准方式 - 横向卡片布局）
                            div {
                                label { class: "block text-sm font-medium mb-2", style: format!("color: {};", Colors::TEXT_PRIMARY), "提现方式" }
                                div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2",
                                    for method in WithdrawMethod::all() {
                                        button {
                                            key: "{method.value()}",
                                            onclick: move |_| { selected_withdraw_method.set(method); },
                                            class: "p-3 rounded-lg border text-left transition-all hover:scale-105",
                                            style: format!(
                                                "background: {}; border-color: {}; color: {};",
                                                if *selected_withdraw_method.read() == method {
                                                    "rgba(99, 102, 241, 0.15)"
                                                } else {
                                                    Colors::BG_SECONDARY
                                                },
                                                if *selected_withdraw_method.read() == method {
                                                    Colors::TECH_PRIMARY
                                                } else {
                                                    Colors::BORDER_PRIMARY
                                                },
                                                Colors::TEXT_PRIMARY
                                            ),
                                            div {
                                                class: if method.is_recommended() { "font-medium flex items-center gap-2" } else { "font-medium" },
                                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                                span { "{method.label()}" }
                                                if method.is_recommended() {
                                                    span {
                                                        class: "text-xs px-2 py-0.5 rounded",
                                                        style: "background: rgba(99, 102, 241, 0.2); color: rgb(99, 102, 241);",
                                                        "推荐"
                                                    }
                                                }
                                            }
                                            div {
                                                class: "text-xs mt-1",
                                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                                "{method.description()}"
                                            }
                                        }
                                    }
                                }
                            }

                            // 收款账户信息
                            div {
                                label { class: "block text-sm font-medium mb-2", style: format!("color: {};", Colors::TEXT_PRIMARY), "收款账户信息" }
                                Input {
                                    input_type: InputType::Text,
                                    placeholder: Some(match *selected_withdraw_method.read() {
                                        WithdrawMethod::BankCard => "银行卡号 (例: 6222 0000 0000 0000)".to_string(),
                                        WithdrawMethod::PayPal => "PayPal账号 (例: your@email.com)".to_string(),
                                        WithdrawMethod::ApplePay => "Apple ID (例: your@icloud.com)".to_string(),
                                        WithdrawMethod::GooglePay => "Google账号 (例: your@gmail.com)".to_string(),
                                        WithdrawMethod::Alipay => "支付宝账号 (手机号或邮箱)".to_string(),
                                        WithdrawMethod::WechatPay => "微信账号 (微信ID或手机号)".to_string(),
                                    }),
                                    value: Some(recipient_info.read().clone()),
                                    onchange: {
                                        let mut recipient_info = recipient_info;
                                        Some(EventHandler::new(move |e: FormEvent| {
                                            recipient_info.set(e.value());
                                        }))
                                    },
                                }
                                p { class: "text-xs mt-1", style: format!("color: {};", Colors::TEXT_SECONDARY), 
                                    match *selected_withdraw_method.read() {
                                        WithdrawMethod::BankCard => "⚠️ 银行卡提现需1-3工作日，请确保卡号准确",
                                        WithdrawMethod::PayPal => "✅ PayPal即时到账，支持全球200+国家",
                                        WithdrawMethod::ApplePay => "✅ Apple Pay即时到账，需iOS设备绑定",
                                        WithdrawMethod::GooglePay => "✅ Google Pay即时到账，需Android设备绑定",
                                        WithdrawMethod::Alipay => "✅ 支付宝即时到账，中国地区首选",
                                        WithdrawMethod::WechatPay => "✅ 微信支付即时到账，中国地区首选",
                                    }
                                }
                            }

                            // 获取报价按钮
                            Button {
                                variant: ButtonVariant::Primary,
                                size: ButtonSize::Large,
                                disabled: is_loading.read().clone(),
                                onclick: get_quote,
                                if *is_loading.read() { "获取报价中..." } else { "获取报价" }
                            }

                            // 显示报价
                            if let Some(q) = quote.read().as_ref() {
                                div { class: "p-4 rounded-lg", style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                                    h3 { class: "font-semibold mb-3", style: format!("color: {};", Colors::TEXT_PRIMARY), "报价详情（自动两步转换）" }
                                    div { class: "space-y-2 text-sm",
                                        // 第一步：代币→稳定币
                                        div { class: "pb-2", style: format!("border-bottom: 1px solid {};", Colors::BORDER_PRIMARY),
                                            p { class: "text-xs font-semibold mb-2", style: format!("color: {};", Colors::TECH_PRIMARY), "步骤 1: 代币 → 稳定币" }
                                            div { class: "flex justify-between",
                                                span { style: format!("color: {};", Colors::TEXT_SECONDARY), "支付代币:" }
                                                span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{q.token_amount} {q.token_symbol}" }
                                            }
                                            div { class: "flex justify-between",
                                                span { style: format!("color: {};", Colors::TEXT_SECONDARY), "获得稳定币:" }
                                                span { style: format!("color: {};", Colors::PAYMENT_SUCCESS), "{q.stablecoin_amount} {q.stablecoin_symbol}" }
                                            }
                                            div { class: "flex justify-between",
                                                span { style: format!("color: {};", Colors::TEXT_SECONDARY), "兑换率:" }
                                                span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{q.exchange_rate_token_to_stable}" }
                                            }
                                        }
                                        
                                        // 第二步：稳定币→法币
                                        div { class: "pt-2",
                                            p { class: "text-xs font-semibold mb-2", style: format!("color: {};", Colors::TECH_PRIMARY), "步骤 2: 稳定币 → 法币" }
                                            div { class: "flex justify-between",
                                                span { style: format!("color: {};", Colors::TEXT_SECONDARY), "稳定币金额:" }
                                                span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{q.stablecoin_amount} {q.stablecoin_symbol}" }
                                            }
                                            div { class: "flex justify-between",
                                                span { style: format!("color: {};", Colors::TEXT_SECONDARY), "到账法币:" }
                                                span { style: format!("color: {};", Colors::PAYMENT_SUCCESS), "{q.fiat_amount} {q.fiat_currency}" }
                                            }
                                            div { class: "flex justify-between",
                                                span { style: format!("color: {};", Colors::TEXT_SECONDARY), "兑换率:" }
                                                span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{q.exchange_rate_stable_to_fiat}" }
                                            }
                                        }

                                        // 费用汇总
                                        div { class: "pt-2", style: format!("border-top: 1px solid {};", Colors::BORDER_PRIMARY),
                                            div { class: "flex justify-between",
                                                span { style: format!("color: {};", Colors::TEXT_SECONDARY), "总手续费:" }
                                                span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{q.fee_amount} ({q.fee_percentage}%)" }
                                            }
                                            div { class: "flex justify-between",
                                                span { style: format!("color: {};", Colors::TEXT_SECONDARY), "预计到账:" }
                                                span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{q.estimated_arrival}" }
                                            }
                                            div { class: "flex justify-between",
                                                span { style: format!("color: {};", Colors::TEXT_SECONDARY), "报价有效期:" }
                                                span { style: format!("color: {};", Colors::PAYMENT_WARNING), "{q.quote_expires_at}" }
                                            }
                                        }
                                    }

                                    // 创建订单按钮
                                    div { class: "mt-4",
                                        Button {
                                            variant: ButtonVariant::Success,
                                            size: ButtonSize::Large,
                                            disabled: is_loading.read().clone(),
                                            onclick: create_order,
                                            if *is_loading.read() { "创建提现订单中..." } else { "确认提现" }
                                        }
                                    }
                                }
                            }

                            // 错误消息
                            if let Some(err) = error_message.read().as_ref() {
                                ErrorMessage { message: err.clone() }
                            }
                        }
                    }
                }

                // 企业级提示
                div { class: "mt-6 p-4 rounded-lg", style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                    h3 { class: "font-semibold mb-2 text-sm", style: format!("color: {};", Colors::TEXT_PRIMARY), "💡 自动两步提现流程" }
                    ul { class: "text-xs space-y-1", style: format!("color: {};", Colors::TEXT_SECONDARY),
                        li { "1️⃣ 系统自动将您的代币（ETH/BTC/SOL）兑换为稳定币（USDT/USDC）" }
                        li { "2️⃣ 然后通过 5 家顶级支付服务商（MoonPay、Simplex等）提现为法币" }
                        li { "3️⃣ 全程自动化，无需手动操作，1-3 个工作日到账" }
                    }
                }
            }
        }
    }
}
