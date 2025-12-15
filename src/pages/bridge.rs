//! Bridge Page - 跨链桥接页面
//! 生产级Bridge实现，使用后端跨链兑换API
//! 包含状态轮询、历史查询等完整功能

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::{Card, CardVariant};
use crate::components::atoms::input::{Input, InputType};
use crate::components::atoms::select::{Select, SelectOption};
use crate::components::molecules::error_message::ErrorMessage;
use crate::components::molecules::ChainSelector;
use crate::services::bridge::{BridgeHistoryItem, BridgeResponse, BridgeService};
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use std::sync::Arc;

/// Bridge Page
#[component]
pub fn Bridge() -> Element {
    let app_state = use_context::<AppState>();

    // 表单状态
    let mut from_chain = use_signal(|| "ethereum".to_string());
    let mut to_chain = use_signal(|| "polygon".to_string());
    let token = use_signal(|| "ETH".to_string());
    let amount = use_signal(|| String::new());

    // UI状态
    let error_message = use_signal(|| Option::<String>::None);
    let is_loading = use_signal(|| false);
    let bridge_response = use_signal(|| Option::<BridgeResponse>::None);
    let is_polling = use_signal(|| false);
    let bridge_history = use_signal(Vec::<BridgeHistoryItem>::new);
    let mut show_history = use_signal(|| false);

    // 执行桥接
    let execute_bridge = move || {
        let amount_val = amount.read().clone();
        let from = from_chain.read().clone();
        let to = to_chain.read().clone();
        let token_val = token.read().clone();
        let wallet_state = app_state.wallet.read();
        let wallet_opt = wallet_state.get_selected_wallet().cloned();
        let app_state_clone = app_state;
        let mut loading = is_loading;
        let mut err = error_message;
        let mut response_sig = bridge_response;

        if amount_val.is_empty() || amount_val.parse::<f64>().unwrap_or(0.0) <= 0.0 {
            err.set(Some("请输入有效的桥接数量".to_string()));
            return;
        }

        if from == to {
            err.set(Some("源链和目标链不能相同".to_string()));
            return;
        }

        let wallet_id = match wallet_opt {
            Some(w) => w.id.to_string(),
            None => {
                err.set(Some("请先选择钱包".to_string()));
                return;
            }
        };

        spawn(async move {
            loading.set(true);
            err.set(None);

            let bridge_service = BridgeService::new(Arc::new(app_state_clone));
            match bridge_service
                .bridge_assets(&wallet_id, &from, &to, &token_val, &amount_val)
                .await
            {
                Ok(resp) => {
                    log::info!(
                        "Bridge执行成功: swap_id={}, status={}",
                        resp.swap_id,
                        resp.status
                    );
                    response_sig.set(Some(resp.clone()));

                    // 如果状态是pending或processing，开始轮询
                    if resp.status == "pending" || resp.status == "processing" {
                        let swap_id = resp.swap_id.clone();
                        let mut polling = is_polling;
                        let mut response_sig_poll = response_sig;
                        let bridge_service_poll = bridge_service.clone();

                        spawn(async move {
                            polling.set(true);
                            match bridge_service_poll
                                .poll_status(&swap_id, Some(30), Some(2000))
                                .await
                            {
                                Ok(final_status) => {
                                    log::info!(
                                        "Bridge轮询完成: swap_id={}, final_status={}",
                                        swap_id,
                                        final_status.status
                                    );
                                    response_sig_poll.set(Some(final_status));
                                }
                                Err(e) => {
                                    log::warn!("Bridge轮询失败: swap_id={}, error={}", swap_id, e);
                                    // 不设置错误，保持当前状态
                                }
                            }
                            polling.set(false);
                        });
                    }
                }
                Err(e) => {
                    // 优化错误提示
                    let error_msg = if e.contains("Failed to bridge assets") {
                        format!("桥接失败: {}", e.replace("Failed to bridge assets: ", ""))
                    } else if e.contains("Invalid amount") {
                        "请输入有效的数量".to_string()
                    } else if e.contains("network") || e.contains("connection") {
                        "网络连接失败，请检查网络后重试".to_string()
                    } else {
                        format!("桥接失败: {}", e)
                    };
                    err.set(Some(error_msg));
                }
            }
            loading.set(false);
        });
    };

    // 格式化响应显示数据
    let bridge_response_display = bridge_response.read().as_ref().map(|resp| {
        (
            format!("${:.2}", resp.fee_usdt),
            format!("{:.6}", resp.estimated_target_amount),
            format!("{}分钟", resp.estimated_time_minutes),
            format!("{:.6}", resp.source_amount),
            format!("{:.6}", resp.exchange_rate),
        )
    });

    rsx! {
        div {
            class: "min-h-screen",
            style: format!("background: {};", Colors::BG_PRIMARY),

            div {
                class: "container mx-auto max-w-2xl px-4 sm:px-6",

                // 页面标题 - 响应式优化
                div {
                    class: "mb-4 sm:mb-6",
                    h1 {
                        class: "text-xl sm:text-2xl font-bold mb-2",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "🌉 跨链桥接"
                    }
                    p {
                        class: "text-xs sm:text-sm",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "在不同区块链之间转移资产"
                    }
                }

                Card {
                    variant: CardVariant::Base,
                    padding: Some("24px".to_string()),
                    children: rsx! {
                        div {
                            class: "space-y-4",

                            // 从链
                            div {
                                label {
                                    class: "block text-sm font-medium mb-2",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    "从链"
                                }
                                ChainSelector {
                                    selected_chain: from_chain,
                                }
                            }

                            // 交换按钮
                            div {
                            class: "flex justify-center my-2",
                            button {
                                class: "p-2 rounded-full",
                                style: format!("background: {};", Colors::BG_SECONDARY),
                                onclick: move |_| {
                                    let from = from_chain.read().clone();
                                    let to = to_chain.read().clone();
                                    from_chain.set(to);
                                    to_chain.set(from);
                                },
                                "⇅"
                            }
                            }

                            // 到链
                            Select {
                                label: Some("到链".to_string()),
                                value: Some(to_chain.read().clone()),
                                options: vec![
                                    SelectOption::new("ethereum", "Ethereum"),
                                    SelectOption::new("polygon", "Polygon"),
                                    SelectOption::new("bsc", "BSC"),
                                    SelectOption::new("arbitrum", "Arbitrum"),
                                    SelectOption::new("solana", "Solana"),
                                    SelectOption::new("ton", "TON"),
                                ],
                                onchange: {
                                    let mut to_chain_sig = to_chain;
                                    move |e: FormEvent| to_chain_sig.set(e.value())
                                },
                            }

                            // 代币
                            Input {
                                input_type: InputType::Text,
                                label: Some("代币".to_string()),
                                value: Some(token.read().clone()),
                                placeholder: Some("ETH".to_string()),
                                onchange: {
                                    let mut token_sig = token;
                                    move |e: FormEvent| token_sig.set(e.value())
                                },
                            }

                            // 数量输入
                            Input {
                                input_type: InputType::Number,
                                label: Some("数量".to_string()),
                                value: Some(amount.read().clone()),
                                placeholder: Some("0.0".to_string()),
                                onchange: {
                                    let mut amount_sig = amount;
                                    move |e: FormEvent| amount_sig.set(e.value())
                                },
                            }
                        }
                    }
                }

                // 桥接响应显示
                if let Some((fee_str, target_amount_str, time_str, source_amount_str, exchange_rate_str)) = bridge_response_display.as_ref() {
                    if let Some(resp) = bridge_response.read().as_ref() {
                        Card {
                            variant: CardVariant::Base,
                            padding: Some("24px".to_string()),
                            children: rsx! {
                                h3 {
                                    class: "text-lg font-semibold mb-4",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    "桥接信息"
                                }
                                div {
                                    class: "space-y-3",
                                    div {
                                        class: "flex justify-between items-center",
                                        span {
                                            class: "text-sm",
                                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                                            "交换ID"
                                        }
                                        span {
                                            class: "text-sm font-mono",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            "{resp.swap_id}"
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
                                            class: "text-sm font-semibold",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            "{resp.status}"
                                        }
                                    }
                                    div {
                                        class: "flex justify-between items-center",
                                        span {
                                            class: "text-sm",
                                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                                            "源数量"
                                        }
                                        span {
                                            class: "text-sm",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            "{source_amount_str}"
                                        }
                                    }
                                    div {
                                        class: "flex justify-between items-center",
                                        span {
                                            class: "text-sm",
                                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                                            "预估目标数量"
                                        }
                                        span {
                                            class: "text-sm font-semibold",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            "{target_amount_str}"
                                        }
                                    }
                                    div {
                                        class: "flex justify-between items-center",
                                        span {
                                            class: "text-sm",
                                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                                            "预估时间"
                                        }
                                        span {
                                            class: "text-sm",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            "{time_str}"
                                        }
                                    }
                                    div {
                                        class: "flex justify-between items-center",
                                        span {
                                            class: "text-sm",
                                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                                            "手续费"
                                        }
                                        span {
                                            class: "text-sm font-semibold",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            "{fee_str}"
                                        }
                                    }
                                    div {
                                        class: "flex justify-between items-center",
                                        span {
                                            class: "text-sm",
                                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                                            "桥协议"
                                        }
                                        span {
                                            class: "text-sm",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            "{resp.bridge_protocol}"
                                        }
                                    }
                                    div {
                                        class: "flex justify-between items-center",
                                        span {
                                            class: "text-sm",
                                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                                            "汇率"
                                        }
                                        span {
                                            class: "text-sm",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            "{exchange_rate_str}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 错误消息
                ErrorMessage {
                    message: error_message.read().clone(),
                }

                // 执行按钮和状态轮询指示
                div {
                    class: "space-y-2",
                    if *is_polling.read() {
                        div {
                            class: "flex items-center justify-center gap-2 p-3 rounded-lg",
                            style: format!("background: rgba(99, 102, 241, 0.1); border: 1px solid {};", Colors::TECH_PRIMARY),
                            span {
                                class: "inline-block w-4 h-4 border-2 border-[#6366F1] border-t-transparent rounded-full animate-spin",
                            }
                            span {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                "正在轮询桥接状态..."
                            }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Large,
                        onclick: move |_| execute_bridge(),
                        disabled: *is_loading.read() || *is_polling.read(),
                        loading: *is_loading.read(),
                        class: Some("w-full mt-4".to_string()),
                        if *is_loading.read() { "执行中..." } else { "执行桥接" }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Medium,
                        onclick: move |_| {
                            let app_state_clone = app_state;
                            let mut history_sig = bridge_history;
                            let mut show_history_sig = show_history;
                            let mut err = error_message;

                            spawn(async move {
                                let bridge_service = BridgeService::new(Arc::new(app_state_clone));
                                match bridge_service.get_history(Some(1), Some(20)).await {
                                    Ok(response) => {
                                        history_sig.set(response.bridges);
                                        show_history_sig.set(true);
                                    }
                                    Err(e) => {
                                        err.set(Some(format!("获取历史记录失败: {}", e)));
                                    }
                                }
                            });
                        },
                        class: Some("w-full".to_string()),
                        "查看历史记录"
                    }
                }

                // 历史记录显示
                if *show_history.read() {
                    Card {
                        variant: CardVariant::Base,
                        padding: Some("24px".to_string()),
                        children: rsx! {
                            div {
                                class: "flex justify-between items-center mb-4",
                                h3 {
                                    class: "text-lg font-semibold",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    "桥接历史"
                                }
                                button {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    onclick: move |_| show_history.set(false),
                                    "关闭"
                                }
                            }
                            if bridge_history.read().is_empty() {
                                p {
                                    class: "text-sm text-center py-4",
                                    style: format!("color: {};", Colors::TEXT_TERTIARY),
                                    "暂无历史记录"
                                }
                            } else {
                                div {
                                    class: "space-y-3",
                                    for item in bridge_history.read().iter() {
                                        div {
                                            class: "p-3 rounded-lg border",
                                            style: format!("background: {}; border-color: {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                                            div {
                                                class: "flex justify-between items-start mb-2",
                                                div {
                                                    span {
                                                        class: "text-xs font-mono",
                                                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                                                        "{item.swap_id}"
                                                    }
                                                }
                                                span {
                                                    class: "text-xs px-2 py-1 rounded",
                                                    style: format!(
                                                        "color: {}; background: {};",
                                                        match item.status.as_str() {
                                                            "completed" => Colors::PAYMENT_SUCCESS,
                                                            "failed" => Colors::PAYMENT_ERROR,
                                                            "pending" => Colors::PAYMENT_WARNING,
                                                            _ => Colors::TEXT_SECONDARY,
                                                        },
                                                        match item.status.as_str() {
                                                            "completed" => "rgba(16, 185, 129, 0.1)",
                                                            "failed" => "rgba(239, 68, 68, 0.1)",
                                                            "pending" => "rgba(245, 158, 11, 0.1)",
                                                            _ => "rgba(255, 255, 255, 0.05)",
                                                        }
                                                    ),
                                                    {item.status.clone()}
                                                }
                                            }
                                            div {
                                                class: "text-sm",
                                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                                "{item.source_chain} → {item.target_chain}"
                                            }
                                            div {
                                                class: "text-sm font-semibold",
                                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                                {
                                                    let source_amt = format!("{:.6}", item.source_amount);
                                                    let target_amt = format!("{:.6}", item.estimated_target_amount);
                                                    format!("{} {} → {} {}", source_amt, item.source_token, target_amt, item.target_token)
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
