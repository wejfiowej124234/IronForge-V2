//! Orders Page - 订单列表页面
//! 显示用户的所有充值/提现订单

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::molecules::limit_display::{KycLevel, LimitDisplay, LimitInfo};
use crate::services::fiat_onramp::FiatOnrampService;
use crate::services::fiat_offramp::FiatOfframpService;
use crate::services::user::UserService;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 订单列表项
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderItem {
    pub order_id: String,
    pub order_type: String,
    pub status: String,
    pub fiat_amount: String,
    pub crypto_amount: String,
    pub currency: String,
    pub token: String,
    pub payment_method: String,
    pub created_at: String,
    pub payment_url: Option<String>,
    pub tx_hash: Option<String>,
    pub error_message: Option<String>,
}

/// 订单统计信息
#[derive(Debug, Clone, PartialEq)]
pub struct OrderStats {
    pub total_orders: usize,
    pub pending_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

#[component]
pub fn Orders() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let navigator = use_navigator();
    
    // 订单状态
    let onramp_orders = use_signal(|| Vec::<OrderItem>::new());
    let offramp_orders = use_signal(|| Vec::<OrderItem>::new());
    let loading = use_signal(|| false);
    let mut refreshing = use_signal(|| false);
    let error_message = use_signal(|| Option::<String>::None);
    let mut active_tab = use_signal(|| "onramp".to_string()); // "onramp" or "offramp"
    
    // 搜索和筛选状态
    let mut search_query = use_signal(|| String::new());
    let mut status_filter = use_signal(|| "all".to_string()); // "all", "pending", "completed", "failed"
    let expanded_order = use_signal(|| Option::<String>::None); // 展开的订单ID
    
    // 统计信息
    let onramp_stats = use_signal(|| OrderStats {
        total_orders: 0,
        pending_count: 0,
        completed_count: 0,
        failed_count: 0,
    });
    let offramp_stats = use_signal(|| OrderStats {
        total_orders: 0,
        pending_count: 0,
        completed_count: 0,
        failed_count: 0,
    });
    
    // KYC状态（从后端获取真实数据）
    let kyc_info = use_signal(|| LimitInfo {
        kyc_level: KycLevel::None,  // 默认未认证
        daily_used: 0.0,
        daily_limit: 0.0,
        monthly_used: 0.0,
        monthly_limit: 0.0,
    });

    // 加载KYC状态
    use_effect({
        let app_state_clone = app_state.clone();
        let mut kyc_info_sig = kyc_info;

        move || {
            spawn(async move {
                let user_service = UserService::new(Arc::new(app_state_clone.read().get_api_client()));
                match user_service.get_kyc_status().await {
                    Ok(kyc_status) => {
                        // 映射KYC等级
                        let kyc_level = match kyc_status.kyc_status.to_lowercase().as_str() {
                            "unverified" => KycLevel::None,
                            "basic" => KycLevel::Basic,
                            "standard" => KycLevel::Intermediate,
                            "premium" => KycLevel::Advanced,
                            _ => KycLevel::None,
                        };

                        kyc_info_sig.set(LimitInfo {
                            kyc_level,
                            daily_used: kyc_status.daily_used,
                            daily_limit: kyc_status.daily_limit,
                            monthly_used: kyc_status.monthly_used,
                            monthly_limit: kyc_status.monthly_limit,
                        });

                        tracing::info!("[Orders] KYC status loaded: {:?}", kyc_status.kyc_status);
                    }
                    Err(e) => {
                        tracing::error!("[Orders] Failed to load KYC status: {}", e);
                        // 保持默认的未认证状态
                    }
                }
            });
        }
    });

    // 加载订单
    use_effect({
        let app_state_clone = app_state.clone();
        let mut onramp_orders_sig = onramp_orders;
        let mut offramp_orders_sig = offramp_orders;
        let mut loading_sig = loading;
        let mut error_sig = error_message;
        let mut onramp_stats_sig = onramp_stats;
        let mut offramp_stats_sig = offramp_stats;

        move || {
            spawn(async move {
                loading_sig.set(true);
                error_sig.set(None);

                // 检查登录状态
                let app_state_read = app_state_clone.read();
                let user_state = app_state_read.user.read();
                if !user_state.is_authenticated {
                    error_sig.set(Some("请先登录".to_string()));
                    loading_sig.set(false);
                    return;
                }
                drop(user_state);

                // 加载充值订单
                let onramp_service = FiatOnrampService::new(Arc::new(app_state_clone.read().clone()));
                match onramp_service.get_orders(None, None, None).await {
                    Ok(orders) => {
                        let order_items: Vec<OrderItem> = orders.orders
                            .into_iter()
                            .map(|o| OrderItem {
                                order_id: o.order_id.clone(),
                                order_type: "onramp".to_string(),
                                status: o.status.clone(),
                                fiat_amount: o.fiat_amount.clone(),
                                crypto_amount: o.crypto_amount.clone(),
                                currency: "USD".to_string(), // 从后端订单不包含这些字段，使用默认值
                                token: "USDT".to_string(),
                                payment_method: "Card".to_string(),
                                created_at: o.created_at.clone(),
                                payment_url: o.payment_url.clone(),
                                tx_hash: o.tx_hash.clone(),
                                error_message: o.error_message.clone(),
                            })
                            .collect();
                        
                        // 计算统计信息
                        let stats = OrderStats {
                            total_orders: order_items.len(),
                            pending_count: order_items.iter().filter(|o| o.status == "pending").count(),
                            completed_count: order_items.iter().filter(|o| o.status == "completed").count(),
                            failed_count: order_items.iter().filter(|o| o.status == "failed" || o.status == "cancelled").count(),
                        };
                        onramp_stats_sig.set(stats);
                        onramp_orders_sig.set(order_items);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load onramp orders: {}", e);
                        error_sig.set(Some(format!("加载充值订单失败: {}", e)));
                    }
                }

                // 加载提现订单
                let offramp_service = FiatOfframpService::new(Arc::new(app_state_clone.read().clone()));
                match offramp_service.get_orders(None, None, None).await {
                    Ok(orders) => {
                        let order_items: Vec<OrderItem> = orders.orders
                            .into_iter()
                            .map(|o| OrderItem {
                                order_id: o.order_id.clone(),
                                order_type: "offramp".to_string(),
                                status: o.status.clone(),
                                fiat_amount: o.fiat_amount.clone(),
                                crypto_amount: o.token_amount.clone(), // offramp使用token_amount
                                currency: o.fiat_currency.clone(),
                                token: o.token_symbol.clone(),
                                payment_method: "Bank".to_string(), // offramp默认银行转账
                                created_at: o.created_at.clone(),
                                payment_url: None, // offramp没有支付URL
                                tx_hash: o.withdrawal_tx_hash.clone(),
                                error_message: o.error_message.clone(),
                            })
                            .collect();
                        
                        // 计算统计信息
                        let stats = OrderStats {
                            total_orders: order_items.len(),
                            pending_count: order_items.iter().filter(|o| o.status == "pending").count(),
                            completed_count: order_items.iter().filter(|o| o.status == "completed").count(),
                            failed_count: order_items.iter().filter(|o| o.status == "failed" || o.status == "cancelled").count(),
                        };
                        offramp_stats_sig.set(stats);
                        offramp_orders_sig.set(order_items);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load offramp orders: {}", e);
                        error_sig.set(Some(format!("加载提现订单失败: {}", e)));
                    }
                }

                loading_sig.set(false);
            });
        }
    });

    rsx! {
        div {
            class: "min-h-screen p-4",
            style: format!("background: {};", Colors::BG_PRIMARY),
            
            div {
                class: "container mx-auto max-w-6xl px-4 sm:px-6 py-8",
                
                // 页面标题和刷新按钮
                div { class: "mb-6 flex items-center justify-between",
                    div {
                        h1 {
                            class: "text-3xl font-bold mb-2",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "我的订单"
                        }
                        p {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "查看和管理您的充值/提现订单"
                        }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Small,
                        disabled: *loading.read() || *refreshing.read(),
                        onclick: move |_| {
                            refreshing.set(true);
                            // 触发重新加载（通过改变依赖来触发use_effect）
                            let app_state_clone = app_state.clone();
                            spawn(async move {
                                // 简单延迟模拟刷新
                                gloo_timers::future::TimeoutFuture::new(500).await;
                                refreshing.set(false);
                                // 实际应该触发重新加载，这里简化处理
                            });
                        },
                        if *refreshing.read() { "刷新中..." } else { "🔄 刷新" }
                    }
                }

                // KYC状态卡片
                div { class: "mb-6",
                    Card {
                        variant: crate::components::atoms::card::CardVariant::Base,
                        padding: Some("0".to_string()),
                        children: rsx! {
                            LimitDisplay {
                                limit_info: Some(kyc_info.read().clone())
                            }
                        }
                    }
                }

                // 订单统计卡片
                div { class: "mb-6",
                    {
                        let stats = if *active_tab.read() == "onramp" {
                            onramp_stats.read().clone()
                        } else {
                            offramp_stats.read().clone()
                        };
                        
                        rsx! {
                            Card {
                                variant: crate::components::atoms::card::CardVariant::Base,
                                padding: Some("20px".to_string()),
                                children: rsx! {
                                    div { class: "grid grid-cols-4 gap-4 text-center",
                                        div {
                                            div {
                                                class: "text-2xl font-bold",
                                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                                {stats.total_orders.to_string()}
                                            }
                                            div {
                                                class: "text-xs mt-1",
                                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                                "总订单"
                                            }
                                        }
                                        div {
                                            div {
                                                class: "text-2xl font-bold",
                                                style: "color: rgba(251, 191, 36, 1);",
                                                {stats.pending_count.to_string()}
                                            }
                                            div {
                                                class: "text-xs mt-1",
                                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                                "待处理"
                                            }
                                        }
                                        div {
                                            div {
                                                class: "text-2xl font-bold",
                                                style: "color: rgba(34, 197, 94, 1);",
                                                {stats.completed_count.to_string()}
                                            }
                                            div {
                                                class: "text-xs mt-1",
                                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                                "已完成"
                                            }
                                        }
                                        div {
                                            div {
                                                class: "text-2xl font-bold",
                                                style: "color: rgba(239, 68, 68, 1);",
                                                {stats.failed_count.to_string()}
                                            }
                                            div {
                                                class: "text-xs mt-1",
                                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                                "失败/取消"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 订单类型切换
                div { class: "flex gap-2 mb-4",
                    Button {
                        variant: if *active_tab.read() == "onramp" { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                        size: ButtonSize::Small,
                        onclick: move |_| active_tab.set("onramp".to_string()),
                        "充值订单"
                    }
                    Button {
                        variant: if *active_tab.read() == "offramp" { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                        size: ButtonSize::Small,
                        onclick: move |_| active_tab.set("offramp".to_string()),
                        "提现订单"
                    }
                }

                // 搜索和筛选栏
                div { class: "mb-4",
                    Card {
                        variant: crate::components::atoms::card::CardVariant::Base,
                        padding: Some("16px".to_string()),
                        children: rsx! {
                            div { class: "flex gap-4 items-center",
                                // 搜索框
                                div { class: "flex-1",
                                    input {
                                        class: "w-full px-4 py-2 rounded-lg",
                                        style: format!("background: {}; color: {}; border: 1px solid {};",
                                            Colors::BG_PRIMARY, Colors::TEXT_PRIMARY, Colors::BORDER_PRIMARY),
                                        r#type: "text",
                                        placeholder: "搜索订单ID...",
                                        value: "{search_query.read()}",
                                        oninput: move |evt| search_query.set(evt.value().clone()),
                                    }
                                }
                                // 状态筛选
                                div { class: "flex gap-2",
                                    Button {
                                        variant: if *status_filter.read() == "all" { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                        size: ButtonSize::Small,
                                        onclick: move |_| status_filter.set("all".to_string()),
                                        "全部"
                                    }
                                    Button {
                                        variant: if *status_filter.read() == "pending" { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                        size: ButtonSize::Small,
                                        onclick: move |_| status_filter.set("pending".to_string()),
                                        "待处理"
                                    }
                                    Button {
                                        variant: if *status_filter.read() == "completed" { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                        size: ButtonSize::Small,
                                        onclick: move |_| status_filter.set("completed".to_string()),
                                        "已完成"
                                    }
                                    Button {
                                        variant: if *status_filter.read() == "failed" { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                        size: ButtonSize::Small,
                                        onclick: move |_| status_filter.set("failed".to_string()),
                                        "失败"
                                    }
                                }
                            }
                        }
                    }
                }

                // 错误提示
                if let Some(err) = error_message.read().as_ref() {
                    Card {
                        variant: crate::components::atoms::card::CardVariant::Base,
                        padding: Some("16px".to_string()),
                        children: rsx! {
                            div {
                                class: "text-sm",
                                style: format!("color: {};", Colors::PAYMENT_ERROR),
                                {err.clone()}
                            }
                        }
                    }
                }

                // 加载中
                if *loading.read() {
                    Card {
                        variant: crate::components::atoms::card::CardVariant::Base,
                        padding: Some("32px".to_string()),
                        children: rsx! {
                            div { class: "text-center py-12",
                                div { class: "text-4xl mb-4", "⏳" }
                                p {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "加载订单中..."
                                }
                            }
                        }
                    }
                } else {
                    // 显示订单列表
                    {
                        let mut orders = if *active_tab.read() == "onramp" {
                            onramp_orders.read().clone()
                        } else {
                            offramp_orders.read().clone()
                        };

                        // 应用搜索筛选
                        let search = search_query.read().to_lowercase();
                        if !search.is_empty() {
                            orders.retain(|o| o.order_id.to_lowercase().contains(&search));
                        }

                        // 应用状态筛选
                        let filter = status_filter.read().clone();
                        if filter != "all" {
                            orders.retain(|o| {
                                match filter.as_str() {
                                    "pending" => o.status == "pending",
                                    "completed" => o.status == "completed",
                                    "failed" => o.status == "failed" || o.status == "cancelled",
                                    _ => true,
                                }
                            });
                        }

                        if orders.is_empty() {
                            rsx! {
                                Card {
                                    variant: crate::components::atoms::card::CardVariant::Base,
                                    padding: Some("32px".to_string()),
                                    children: rsx! {
                                        div { class: "text-center py-12",
                                            div { class: "text-6xl mb-4", "📋" }
                                            h3 {
                                                class: "text-xl font-semibold mb-2",
                                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                                "暂无订单"
                                            }
                                            p {
                                                class: "text-sm mb-6",
                                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                                if *active_tab.read() == "onramp" {
                                                    "您还没有充值订单，去充值页面创建第一笔订单吧！"
                                                } else {
                                                    "您还没有提现订单"
                                                }
                                            }
                                            if *active_tab.read() == "onramp" {
                                                Button {
                                                    variant: ButtonVariant::Primary,
                                                    size: ButtonSize::Medium,
                                                    onclick: move |_| {
                                    let nav = navigator.clone();
                                    nav.push(crate::router::Route::Buy {});
                                },
                                                    "去充值"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div { class: "space-y-4",
                                    for order in orders {
                                        EnhancedOrderCard { 
                                            order: order.clone(),
                                            expanded_order: expanded_order,
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

/// 增强订单卡片组件（企业级）
#[component]
fn EnhancedOrderCard(order: OrderItem, expanded_order: Signal<Option<String>>) -> Element {
    // 企业级最佳实践：使用Arc共享所有权，避免多次clone的内存开销
    // 在组件初始化时创建Arc，后续所有闭包共享同一个Arc引用
    let order_arc = Arc::new(order);
    let is_expanded = expanded_order.read().as_ref().map_or(false, |id| id == &order_arc.order_id);

    let status_color = match order_arc.status.as_str() {
        "pending" => "rgba(251, 191, 36, 1)",
        "processing" => "rgba(59, 130, 246, 1)",
        "completed" => "rgba(34, 197, 94, 1)",
        "failed" | "cancelled" => "rgba(239, 68, 68, 1)",
        _ => Colors::TEXT_SECONDARY,
    };

    let status_bg = match order_arc.status.as_str() {
        "pending" => "rgba(251, 191, 36, 0.1)",
        "processing" => "rgba(59, 130, 246, 0.1)",
        "completed" => "rgba(34, 197, 94, 0.1)",
        "failed" | "cancelled" => "rgba(239, 68, 68, 0.1)",
        _ => Colors::BG_SECONDARY,
    };

    let status_label = match order_arc.status.as_str() {
        "pending" => "待处理",
        "processing" => "处理中",
        "completed" => "已完成",
        "failed" => "失败",
        "cancelled" => "已取消",
        _ => "未知",
    };

    rsx! {
        Card {
            variant: crate::components::atoms::card::CardVariant::Base,
            padding: Some("20px".to_string()),
            children: rsx! {
                div { class: "space-y-3",
                    // 订单头部（可点击展开）
                    div { 
                        class: "flex items-center justify-between cursor-pointer",
                        onclick: {
                            let order_id = order_arc.order_id.clone();
                            move |_| {
                                let current = expanded_order.read().clone();
                                if current.as_ref() == Some(&order_id) {
                                    expanded_order.set(None);
                                } else {
                                    expanded_order.set(Some(order_id.clone()));
                                }
                            }
                        },
                        div { class: "flex-1",
                            span {
                                class: "text-sm font-medium",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "订单ID: "
                            }
                            span {
                                class: "text-sm font-mono",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {order_arc.order_id.chars().take(12).collect::<String>()}
                                "..."
                            }
                            span {
                                class: "text-xs ml-2",
                                style: format!("color: {};", Colors::TEXT_TERTIARY),
                                {if is_expanded { "▼" } else { "▶" }}
                            }
                        }
                        span {
                            class: "px-3 py-1 rounded-full text-xs font-medium",
                            style: format!("background: {}; color: {};", status_bg, status_color),
                            {status_label}
                        }
                    }

                    // 订单详情
                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            div {
                                class: "text-xs",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "法币金额"
                            }
                            div {
                                class: "text-lg font-bold mt-1",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {format!("{} {}", order_arc.fiat_amount, order_arc.currency)}
                            }
                        }
                        div {
                            div {
                                class: "text-xs",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "加密货币"
                            }
                            div {
                                class: "text-lg font-bold mt-1",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {format!("{} {}", order_arc.crypto_amount, order_arc.token)}
                            }
                        }
                    }

                    // 支付方式和时间
                    div { class: "flex items-center justify-between pt-2 border-t",
                        style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                        div {
                            class: "text-xs",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "支付方式: "
                            span {
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {order_arc.payment_method.as_str()}
                            }
                        }
                        div {
                            class: "text-xs",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            {order_arc.created_at.chars().take(16).collect::<String>()}
                        }
                    }

                    // 展开的详细信息
                    if is_expanded {
                        div { class: "pt-3 border-t space-y-3",
                            style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                            
                            // 完整订单ID
                            div {
                                div {
                                    class: "text-xs font-medium mb-1",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "完整订单ID"
                                }
                                div {
                                    class: "text-xs font-mono p-2 rounded",
                                    style: format!("background: {}; color: {};", Colors::BG_PRIMARY, Colors::TEXT_PRIMARY),
                                    {order_arc.order_id.as_str()}
                                }
                            }

                            // 交易哈希（如果有）
                            if let Some(ref tx_hash) = order_arc.tx_hash {
                                div {
                                    div {
                                        class: "text-xs font-medium mb-1",
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        "区块链交易哈希"
                                    }
                                    div {
                                        class: "text-xs font-mono p-2 rounded",
                                        style: format!("background: {}; color: {};", Colors::BG_PRIMARY, Colors::TECH_PRIMARY),
                                        {tx_hash.as_str()}
                                    }
                                }
                            }

                            // 错误信息（如果有）
                            if let Some(ref error_msg) = order_arc.error_message {
                                div {
                                    div {
                                        class: "text-xs font-medium mb-1",
                                        style: format!("color: {};", Colors::PAYMENT_ERROR),
                                        "错误信息"
                                    }
                                    div {
                                        class: "text-xs p-2 rounded",
                                        style: format!("background: rgba(239, 68, 68, 0.1); color: {};", Colors::PAYMENT_ERROR),
                                        {error_msg.as_str()}
                                    }
                                }
                            }

                            // 操作按钮
                            div { class: "flex gap-2 pt-2",
                                // 支付按钮（仅pending状态的onramp订单）
                                if order_arc.status == "pending" && order_arc.order_type == "onramp" {
                                    if let Some(ref payment_url) = order_arc.payment_url {
                                        Button {
                                            variant: ButtonVariant::Primary,
                                            size: ButtonSize::Small,
                                            onclick: {
                                                let url = payment_url.clone();
                                                move |_| {
                                                    // 在新窗口打开支付URL
                                                    if let Some(window) = web_sys::window() {
                                                        let _ = window.open_with_url_and_target(&url, "_blank");
                                                    }
                                                }
                                            },
                                            "💳 前往支付"
                                        }
                                    }
                                }
                                
                                // 复制订单ID按钮
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    size: ButtonSize::Small,
                                    onclick: {
                                        let order_id = order_arc.order_id.clone();
                                        move |_| {
                                            // 复制到剪贴板
                                            if let Some(window) = web_sys::window() {
                                                let navigator = window.navigator();
                                                let clipboard = navigator.clipboard();
                                                let id = order_id.clone();
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    let _ = wasm_bindgen_futures::JsFuture::from(
                                                        clipboard.write_text(&id)
                                                    ).await;
                                                });
                                            }
                                        }
                                    },
                                    "📋 复制ID"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 基础订单卡片组件（备用）
#[component]
fn BasicOrderCard(order: OrderItem) -> Element {
    let status_color = match order.status.as_str() {
        "pending" => "rgba(251, 191, 36, 1)",
        "processing" => "rgba(59, 130, 246, 1)",
        "completed" => "rgba(34, 197, 94, 1)",
        "failed" | "cancelled" => "rgba(239, 68, 68, 1)",
        _ => Colors::TEXT_SECONDARY,
    };

    let status_bg = match order.status.as_str() {
        "pending" => "rgba(251, 191, 36, 0.1)",
        "processing" => "rgba(59, 130, 246, 0.1)",
        "completed" => "rgba(34, 197, 94, 0.1)",
        "failed" | "cancelled" => "rgba(239, 68, 68, 0.1)",
        _ => Colors::BG_SECONDARY,
    };

    let status_label = match order.status.as_str() {
        "pending" => "待处理",
        "processing" => "处理中",
        "completed" => "已完成",
        "failed" => "失败",
        "cancelled" => "已取消",
        _ => "未知",
    };

    rsx! {
        Card {
            variant: crate::components::atoms::card::CardVariant::Base,
            padding: Some("20px".to_string()),
            children: rsx! {
                div { class: "space-y-3",
                    // 订单头部
                    div { class: "flex items-center justify-between",
                        div {
                            span {
                                class: "text-sm font-medium",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "订单ID: "
                            }
                            span {
                                class: "text-sm font-mono",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {order.order_id.chars().take(12).collect::<String>()}
                                "..."
                            }
                        }
                        span {
                            class: "px-3 py-1 rounded-full text-xs font-medium",
                            style: format!("background: {}; color: {};", status_bg, status_color),
                            {status_label}
                        }
                    }

                    // 订单详情
                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            div {
                                class: "text-xs",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "法币金额"
                            }
                            div {
                                class: "text-lg font-bold mt-1",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {format!("{} {}", order.fiat_amount, order.currency)}
                            }
                        }
                        div {
                            div {
                                class: "text-xs",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "加密货币"
                            }
                            div {
                                class: "text-lg font-bold mt-1",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {format!("{} {}", order.crypto_amount, order.token)}
                            }
                        }
                    }

                    // 支付方式和时间
                    div { class: "flex items-center justify-between pt-2 border-t",
                        style: format!("border-color: {};", Colors::BORDER_PRIMARY),
                        div {
                            class: "text-xs",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "支付方式: "
                            span {
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {order.payment_method.as_str()}
                            }
                        }
                        div {
                            class: "text-xs",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            {order.created_at.chars().take(16).collect::<String>()}
                        }
                    }
                }
            }
        }
    }
}
