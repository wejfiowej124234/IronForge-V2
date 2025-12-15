//! Buy Page - 法币购买页面
//! 企业级法币充值实现，支持多支付方式，智能服务商选择

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::atoms::input::{Input, InputType};
use crate::components::molecules::ErrorMessage;
use crate::features::wallet::unlock::ensure_wallet_unlocked;
use crate::router::Route;
use crate::services::fiat_onramp::{FiatOnrampService, FiatQuoteResponse};
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use std::sync::Arc;

/// 支付方式选项
#[derive(Debug, Clone, Copy, PartialEq)]
enum PaymentMethod {
    CreditCard,
    BankTransfer,
    PayPal,
}

impl PaymentMethod {
    fn value(&self) -> &'static str {
        match self {
            PaymentMethod::CreditCard => "credit_card",
            PaymentMethod::BankTransfer => "bank_transfer",
            PaymentMethod::PayPal => "paypal",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            PaymentMethod::CreditCard => "💳 信用卡/借记卡",
            PaymentMethod::BankTransfer => "🏦 银行转账",
            PaymentMethod::PayPal => "💰 PayPal",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            PaymentMethod::CreditCard,
            PaymentMethod::BankTransfer,
            PaymentMethod::PayPal,
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

/// 稳定币选项
#[derive(Debug, Clone, Copy, PartialEq)]
enum StableCoin {
    USDT,
    USDC,
}

impl StableCoin {
    fn value(&self) -> &'static str {
        match self {
            StableCoin::USDT => "USDT",
            StableCoin::USDC => "USDC",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            StableCoin::USDT => "💵 USDT (Tether)",
            StableCoin::USDC => "💵 USDC (USD Coin)",
        }
    }

    fn all() -> Vec<Self> {
        vec![StableCoin::USDT, StableCoin::USDC]
    }
}

/// Buy Page - 主组件
#[component]
pub fn Buy() -> Element {
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
                                p { class: "text-sm mb-4", style: format!("color: {};", Colors::TEXT_SECONDARY), "请先登录您的账户，然后再进行法币购买操作。" }
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
                                h1 { class: "text-2xl font-bold mb-4", style: format!("color: {};", Colors::TEXT_PRIMARY), "💳 购买稳定币" }
                                p { class: "text-sm mb-4", style: format!("color: {};", Colors::TEXT_SECONDARY), "请先在仪表盘选择并解锁一个钱包，然后再进行法币购买操作。" }
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
    let amount = use_signal(|| "100".to_string());
    let mut selected_currency = use_signal(|| FiatCurrency::USD);
    let mut selected_token = use_signal(|| StableCoin::USDT);
    let mut selected_payment = use_signal(|| PaymentMethod::CreditCard);
    let wallet_address = use_signal(|| String::new());

    // 报价状态
    let quote = use_signal(|| None::<FiatQuoteResponse>);
    let is_loading = use_signal(|| false);
    let error_message = use_signal(|| None::<String>);

    // 订单状态
    let order_created = use_signal(|| false);
    let payment_url = use_signal(|| None::<String>);

    // 获取报价
    let get_quote = {
        let app_state = app_state.clone();
        move |_| {
            let app_state = app_state.clone();
            let amount = amount.read().clone();
            let currency = selected_currency.read().value();
            let token = selected_token.read().value();
            let payment_method = selected_payment.read().value();
            let mut is_loading = is_loading;
            let mut error_message = error_message;
            let mut quote = quote;

            spawn(async move {
                is_loading.set(true);
                error_message.set(None);

                let service = FiatOnrampService::new(Arc::new(app_state));
                match service.get_quote(&amount, currency, token, payment_method).await {
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

    // 创建订单
    let create_order = {
        let app_state = app_state.clone();
        move |_| {
            let app_state = app_state.clone();
            let amount = amount.read().clone();
            let currency = selected_currency.read().value();
            let token = selected_token.read().value();
            let payment_method = selected_payment.read().value();
            let wallet_addr = wallet_address.read().clone();
            let current_quote = quote.read().clone();
            let mut is_loading = is_loading;
            let mut error_message = error_message;
            let mut order_created = order_created;
            let mut payment_url = payment_url;

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

                // 验证必须先获取报价
                let quote_id = match current_quote {
                    Some(q) => q.quote_id,
                    None => {
                        error_message.set(Some("请先点击【获取报价】按钮获取实时报价".to_string()));
                        is_loading.set(false);
                        tracing::warn!("[Buy] Attempted to create order without getting quote first");
                        return;
                    }
                };

                let wallet_address_opt = if wallet_addr.is_empty() {
                    None
                } else {
                    Some(wallet_addr.as_str())
                };

                tracing::info!("[Buy] Creating order: amount={}, currency={}, token={}, payment_method={}, quote_id={}", 
                    amount, currency, token, payment_method, quote_id);

                let service = FiatOnrampService::new(Arc::new(app_state));
                match service
                    .create_order(
                        &amount,
                        currency,
                        token,
                        payment_method,
                        &quote_id,
                        wallet_address_opt,
                    )
                    .await
                {
                    Ok(order) => {
                        tracing::info!("[Buy] Order created successfully: order_id={}, payment_url={:?}", order.order_id, order.payment_url);
                        order_created.set(true);
                        payment_url.set(order.payment_url.clone());
                        is_loading.set(false);
                    }
                    Err(e) => {
                        tracing::error!("[Buy] Failed to create order: {}", e);
                        error_message.set(Some(format!("创建订单失败：{}", e)));
                        is_loading.set(false);
                    }
                }
            });
        }
    };

    // 如果订单已创建，显示支付引导
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
                                h1 { class: "text-2xl font-bold mb-4", style: format!("color: {};", Colors::TEXT_PRIMARY), "订单创建成功！" }
                                p { class: "text-sm mb-6", style: format!("color: {};", Colors::TEXT_SECONDARY), "您的购买订单已创建，请点击下方按钮前往支付。" }
                                
                                if let Some(url) = (*payment_url.read()).clone() {
                                    div { class: "space-y-4",
                                        // 显示支付URL（调试用）
                                        div { class: "p-3 rounded bg-gray-800 text-xs break-all",
                                            p { class: "text-gray-400 mb-1", "支付链接：" }
                                            p { class: "text-green-400", "{url}" }
                                        }
                                        Button {
                                            variant: ButtonVariant::Primary,
                                            size: ButtonSize::Large,
                                            onclick: move |_| {
                                                tracing::info!("[Buy] Opening payment URL: {}", url);
                                                if let Some(window) = web_sys::window() {
                                                    match window.open_with_url_and_target(&url, "_blank") {
                                                        Ok(_) => tracing::info!("[Buy] Payment window opened successfully"),
                                                        Err(e) => tracing::error!("[Buy] Failed to open payment window: {:?}", e),
                                                    }
                                                } else {
                                                    tracing::error!("[Buy] window object not available");
                                                }
                                            },
                                            "🔗 前往支付页面"
                                        }
                                    }
                                } else {
                                    div { class: "p-4 rounded", style: format!("background: {};", Colors::PAYMENT_WARNING),
                                        p { class: "text-sm font-semibold", "⚠️ 未获取到支付链接" }
                                        p { class: "text-xs mt-2", "支付URL为空，这可能是后端配置问题。请检查浏览器控制台日志或联系技术支持。" }
                                    }
                                }

                                div { class: "mt-6 flex gap-4 justify-center",
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        size: ButtonSize::Medium,
                                        onclick: move |_| { 
                                            // 跳转到订单页面
                                            navigator.push(Route::Orders {});
                                        },
                                        "查看我的订单"
                                    }
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        size: ButtonSize::Medium,
                                        onclick: move |_| { navigator.push(Route::Dashboard {}); },
                                        "返回仪表盘"
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
                    h1 { class: "text-3xl font-bold", style: format!("color: {};", Colors::TEXT_PRIMARY), "💳 购买稳定币" }
                    p { class: "text-sm mt-2", style: format!("color: {};", Colors::TEXT_SECONDARY), 
                        "使用法币购买 USDT 或 USDC，支持多种支付方式。系统将自动选择最优惠的支付服务商。" 
                    }
                }

                // 表单卡片
                Card {
                    variant: crate::components::atoms::card::CardVariant::Base,
                    padding: Some("24px".to_string()),
                    children: rsx! {
                        div { class: "space-y-6",
                            // 金额输入
                            div {
                                label { class: "block text-sm font-medium mb-2", style: format!("color: {};", Colors::TEXT_PRIMARY), "购买金额" }
                                Input {
                                    input_type: InputType::Text,
                                    placeholder: Some("请输入金额（最低 $10）".to_string()),
                                    value: Some(amount.read().clone()),
                                    onchange: {
                                        let mut amount = amount;
                                        Some(EventHandler::new(move |e: FormEvent| {
                                            amount.set(e.value());
                                        }))
                                    },
                                }
                                p { class: "text-xs mt-1", style: format!("color: {};", Colors::TEXT_SECONDARY), "最低购买金额为 $10" }
                            }

                            // 法币货币选择
                            div {
                                label { class: "block text-sm font-medium mb-2", style: format!("color: {};", Colors::TEXT_PRIMARY), "法币货币" }
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

                            // 稳定币选择
                            div {
                                label { class: "block text-sm font-medium mb-2", style: format!("color: {};", Colors::TEXT_PRIMARY), "购买稳定币" }
                                div { class: "grid grid-cols-2 gap-2",
                                    for token in StableCoin::all() {
                                        button {
                                            key: "{token.value()}",
                                            onclick: move |_| { selected_token.set(token); },
                                            class: "p-3 rounded-lg border-2 transition-all",
                                            style: if *selected_token.read() == token {
                                                format!("background: {}; border-color: {}; color: {};", Colors::TECH_PRIMARY, Colors::TECH_PRIMARY, Colors::TEXT_PRIMARY)
                                            } else {
                                                format!("background: {}; border-color: {}; color: {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY, Colors::TEXT_SECONDARY)
                                            },
                                            "{token.label()}"
                                        }
                                    }
                                }
                            }

                            // 支付方式选择
                            div {
                                label { class: "block text-sm font-medium mb-2", style: format!("color: {};", Colors::TEXT_PRIMARY), "支付方式" }
                                div { class: "space-y-2",
                                    for payment in PaymentMethod::all() {
                                        button {
                                            key: "{payment.value()}",
                                            onclick: move |_| { selected_payment.set(payment); },
                                            class: "w-full p-3 rounded-lg border-2 transition-all text-left",
                                            style: if *selected_payment.read() == payment {
                                                format!("background: {}; border-color: {}; color: {};", Colors::TECH_PRIMARY, Colors::TECH_PRIMARY, Colors::TEXT_PRIMARY)
                                            } else {
                                                format!("background: {}; border-color: {}; color: {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY, Colors::TEXT_SECONDARY)
                                            },
                                            "{payment.label()}"
                                        }
                                    }
                                }
                            }

                            // 钱包地址（可选）
                            div {
                                label { class: "block text-sm font-medium mb-2", style: format!("color: {};", Colors::TEXT_PRIMARY), "接收地址（可选）" }
                                Input {
                                    input_type: InputType::Text,
                                    placeholder: Some("留空则使用当前选中钱包地址".to_string()),
                                    value: Some(wallet_address.read().clone()),
                                    onchange: {
                                        let mut wallet_address = wallet_address;
                                        Some(EventHandler::new(move |e: FormEvent| {
                                            wallet_address.set(e.value());
                                        }))
                                    },
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
                                    h3 { class: "font-semibold mb-3", style: format!("color: {};", Colors::TEXT_PRIMARY), "报价详情" }
                                    div { class: "space-y-2 text-sm",
                                        div { class: "flex justify-between",
                                            span { style: format!("color: {};", Colors::TEXT_SECONDARY), "购买金额:" }
                                            span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{q.fiat_amount} {selected_currency.read().value()}" }
                                        }
                                        div { class: "flex justify-between",
                                            span { style: format!("color: {};", Colors::TEXT_SECONDARY), "获得稳定币:" }
                                            span { style: format!("color: {};", Colors::PAYMENT_SUCCESS), "{q.crypto_amount} {selected_token.read().value()}" }
                                        }
                                        div { class: "flex justify-between",
                                            span { style: format!("color: {};", Colors::TEXT_SECONDARY), "汇率:" }
                                            span { style: format!("color: {};", Colors::TEXT_PRIMARY), "{q.exchange_rate}" }
                                        }
                                        div { class: "flex justify-between",
                                            span { style: format!("color: {};", Colors::TEXT_SECONDARY), "手续费:" }
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

                                    // 创建订单按钮
                                    div { class: "mt-4",
                                        Button {
                                            variant: ButtonVariant::Success,
                                            size: ButtonSize::Large,
                                            disabled: is_loading.read().clone(),
                                            onclick: create_order,
                                            if *is_loading.read() { "创建订单中..." } else { "确认购买" }
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
                    h3 { class: "font-semibold mb-2 text-sm", style: format!("color: {};", Colors::TEXT_PRIMARY), "💡 智能服务商选择" }
                    p { class: "text-xs", style: format!("color: {};", Colors::TEXT_SECONDARY), 
                        "系统已接入 MoonPay、Simplex、Transak、Ramp、Banxa 5家顶级支付服务商，自动为您选择手续费最低的服务商，节省交易成本。" 
                    }
                }
            }
        }
    }
}
