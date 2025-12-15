//! Token Selector - 企业级代币选择器组件
//! 提供代币选择、搜索、余额显示等功能

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::input::{Input, InputType};
use crate::components::atoms::modal::Modal;
use crate::services::address_detector::ChainType;
use crate::services::token::{TokenInfo, TokenService};
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;

/// 代币选择器组件
#[component]
pub fn TokenSelector(
    /// 当前选择的链
    chain: ChainType,
    /// 当前选择的代币（Signal）
    selected_token: Signal<Option<TokenInfo>>,
    /// 钱包地址（用于显示余额）
    wallet_address: Option<String>,
) -> Element {
    let app_state = use_context::<AppState>();
    let show_modal = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let tokens = use_signal(Vec::<TokenInfo>::new);
    let loading = use_signal(|| false);
    let error = use_signal(|| Option::<String>::None);
    let token_balances = use_signal(std::collections::HashMap::<String, f64>::new);

    // ✅ 克隆 wallet_address 用于多处使用（因为 Option<String> 不实现 Copy）
    let has_wallet = wallet_address.is_some();

    // ✅ 智能代币加载：从钱包中获取有余额的代币，而不是硬编码所有代币
    // 如果没有钱包地址，则fallback到加载默认代币列表
    // 🔧 修复：明确追踪 chain 和 wallet_address 的变化
    use_effect(move || {
        let app_state_clone = app_state;
        let chain_clone = chain; // 读取chain值，触发追踪
        let wallet_opt_clone = wallet_address.clone();
        let mut tokens_mut = tokens;
        let mut loading_mut = loading;
        let mut error_mut = error;
        let mut balances_mut = token_balances;

        spawn(async move {
            loading_mut.set(true);
            error_mut.set(None);

            let token_service = TokenService::new(app_state_clone);

            // ✅ 智能策略：优先从钱包余额中获取代币
            if let Some(ref wallet_addr) = wallet_opt_clone {
                // 1. 获取钱包账户信息（包含原生代币余额）
                let wallet_state = app_state_clone.wallet.read();
                let mut tokens_with_balance = Vec::new();
                let mut balances_map = std::collections::HashMap::new();

                // 2. 添加当前链的原生代币（如果有余额）
                if let Some(wallet) = wallet_state.get_selected_wallet() {
                    if let Some(account) = wallet
                        .accounts
                        .iter()
                        .find(|acc| acc.address.to_lowercase() == wallet_addr.to_lowercase())
                    {
                        // 原生代币始终显示
                        let native_token = TokenInfo {
                            address: "0x0000000000000000000000000000000000000000".to_string(),
                            symbol: chain_clone.native_token_symbol().to_string(),
                            name: format!("{} Native Token", chain_clone.label()),
                            decimals: 18,
                            chain: chain_clone,
                            logo_url: None,
                            is_native: true,
                        };

                        // 解析余额（从字符串转换为f64）
                        let balance = account.balance.parse::<f64>().unwrap_or(0.0);
                        if balance > 0.0 {
                            balances_map.insert(native_token.address.clone(), balance);
                        }
                        tokens_with_balance.push(native_token);
                    }
                }

                // 3. 获取所有ERC-20代币并过滤有余额的
                match token_service.get_token_list(chain_clone).await {
                    Ok(all_tokens) => {
                        for token in all_tokens {
                            if !token.is_native {
                                // 查询ERC-20代币余额
                                if let Ok(balance_info) = token_service
                                    .get_token_balance(chain_clone, &token.address, wallet_addr)
                                    .await
                                {
                                    // ✅ 只添加有余额的代币（大于0.0001）
                                    if balance_info.balance_formatted > 0.0001 {
                                        balances_map.insert(
                                            token.address.clone(),
                                            balance_info.balance_formatted,
                                        );
                                        tokens_with_balance.push(token);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error_mut.set(Some(format!("加载代币列表失败: {}", e)));
                    }
                }

                tokens_mut.set(tokens_with_balance);
                balances_mut.set(balances_map);
            } else {
                // ✅ 没有钱包地址：显示所有可交换代币（用于交换页面的To选择器）
                #[cfg(debug_assertions)]
                {
                    use tracing::info;
                    info!("TokenSelector - Loading all tokens (no wallet address provided)");
                }

                match token_service.get_token_list(chain_clone).await {
                    Ok(token_list) => {
                        #[cfg(debug_assertions)]
                        {
                            use tracing::info;
                            info!(
                                "TokenSelector - Loaded {} tokens for selection",
                                token_list.len()
                            );
                        }
                        tokens_mut.set(token_list);
                    }
                    Err(e) => {
                        error_mut.set(Some(format!("加载代币列表失败: {}", e)));

                        #[cfg(debug_assertions)]
                        {
                            use tracing::error;
                            error!("TokenSelector - API error: {}", e);
                        }
                    }
                }
            }

            loading_mut.set(false);
        });
    });

    // ✅ 余额加载已合并到上面的智能代币加载中

    // 过滤代币列表
    let filtered_tokens = use_memo(move || {
        let query = search_query.read().to_lowercase();
        tokens
            .read()
            .iter()
            .filter(|token| {
                query.is_empty()
                    || token.symbol.to_lowercase().contains(&query)
                    || token.name.to_lowercase().contains(&query)
                    || token.address.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    // 当前选择的代币显示
    let selected_token_display = if let Some(token) = selected_token.read().as_ref() {
        format!("{} ({})", token.symbol, token.name)
    } else {
        "选择代币".to_string()
    };

    rsx! {
        div {
            class: "mb-6",
            label {
                class: "block text-sm font-medium mb-2",
                style: format!("color: {};", Colors::TEXT_SECONDARY),
                "选择代币"
            }
            Button {
                variant: ButtonVariant::Secondary,
                size: ButtonSize::Medium,
                class: Some("w-full justify-between".to_string()),
                onclick: {
                    let mut show_modal_mut = show_modal;
                    move |_| {
                        show_modal_mut.set(true);
                    }
                },
                div {
                    class: "flex items-center justify-between w-full",
                    span {
                        {selected_token_display}
                    }
                    span {
                        class: "ml-2",
                        "▼"
                    }
                }
            }

            // 显示当前代币余额（如果有）
            if let Some(token) = selected_token.read().as_ref() {
                if let Some(balance) = token_balances.read().get(&token.address) {
                    div {
                        class: "mt-2 text-sm",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        {format!("余额: {:.6} {}", balance, token.symbol)}
                    }
                }
            }
        }

        // 代币选择模态框
        if show_modal() {
            Modal {
                open: true,
                onclose: {
                    let mut show_modal_mut = show_modal;
                    EventHandler::new(move |_| {
                        show_modal_mut.set(false);
                    })
                },
                title: Some("选择代币".to_string()),
                children: rsx! {
                    div {
                        class: "flex flex-col",
                        style: "height: 600px; max-height: 80vh;",

                        // 🔍 搜索框 - 根据场景调整文案
                        div {
                            class: "sticky top-0 z-10 pb-4 mb-2",
                            style: format!("background: {};", Colors::BG_PRIMARY),

                            Input {
                                input_type: InputType::Text,
                                placeholder: Some(if has_wallet {
                                    "🔍 搜索钱包中的代币...".to_string()
                                } else {
                                    "🔍 搜索代币名称或粘贴合约地址".to_string()
                                }),
                                value: Some(search_query.read().clone()),
                                onchange: {
                                    let mut search_query_mut = search_query;
                                    Some(EventHandler::new(move |e: dioxus::html::FormEvent| {
                                        search_query_mut.set(e.value());
                                    }))
                                },
                            }

                            // 搜索结果统计
                            if !search_query.read().is_empty() {
                                div {
                                    class: "mt-2 flex items-center justify-between text-xs",
                                    div {
                                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                                        "找到 {filtered_tokens.read().len()} 个代币"
                                    }
                                    if !filtered_tokens.read().is_empty() {
                                        button {
                                            class: "text-xs font-medium hover:underline",
                                            style: format!("color: {};", Colors::TECH_PRIMARY),
                                            onclick: move |_| search_query.set(String::new()),
                                            "清除搜索"
                                        }
                                    }
                                }
                            }
                        }

                        // 🏷️ 热门代币快捷选择 - 仅在没有搜索时显示
                        if search_query.read().is_empty() && !has_wallet {
                            div {
                                class: "pb-4 mb-4 border-b",
                                style: format!("border-color: {};", Colors::BORDER_PRIMARY),

                                div {
                                    class: "flex items-center justify-between mb-3",
                                    div {
                                        class: "text-sm font-bold flex items-center gap-2",
                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                        span { class: "text-base", "🔥" }
                                        span { "热门代币" }
                                    }
                                    div {
                                        class: "text-xs",
                                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                                        "共 {tokens.read().len()} 个可用"
                                    }
                                }

                                div {
                                    class: "flex flex-wrap gap-2",
                                    // 热门代币快捷按钮
                                    for symbol in ["ETH", "USDT", "USDC", "DAI", "WBTC"] {
                                        button {
                                            class: "px-4 py-2 rounded-xl text-sm font-semibold transition-all hover:scale-105 hover:shadow-lg",
                                            style: format!(
                                                "background: {}; color: {}; border: 2px solid {};",
                                                "rgba(99, 102, 241, 0.1)",
                                                Colors::TECH_PRIMARY,
                                                "rgba(99, 102, 241, 0.3)"
                                            ),
                                            onclick: {
                                                let symbol_str = symbol.to_string();
                                                let mut search_mut = search_query;
                                                move |_| {
                                                    search_mut.set(symbol_str.clone());
                                                }
                                            },
                                            {symbol}
                                        }
                                    }
                                }
                            }
                        }



                        // ⚠️ 加载/错误状态
                        if loading() {
                            div {
                                class: "flex-1 flex items-center justify-center py-12",
                                div {
                                    class: "text-center",
                                    div {
                                        class: "animate-spin rounded-full h-8 w-8 border-b-2 mx-auto mb-2",
                                        style: format!("border-color: {};", Colors::TECH_PRIMARY),
                                    }
                                    p {
                                        class: "text-sm",
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        "加载代币中..."
                                    }
                                }
                            }
                        } else if let Some(err) = error.read().as_ref() {
                            div {
                                class: "p-4 rounded-lg text-center",
                                style: format!("background: rgba(239, 68, 68, 0.1); color: {};", Colors::PAYMENT_ERROR),
                                div { class: "text-2xl mb-2", "⚠️" }
                                div { class: "text-sm font-medium mb-1", "加载失败" }
                                div { class: "text-xs", {err.clone()} }
                            }
                        }

                        // 📋 代币列表 - Uniswap风格
                        if !loading() && error.read().is_none() {
                            div {
                                class: "flex-1 overflow-y-auto custom-scrollbar",
                                style: "max-height: 360px; padding-right: 4px;",

                                // 无结果提示 - 根据场景调整文案
                                if filtered_tokens.read().is_empty() {
                                    div {
                                        class: "flex flex-col items-center justify-center py-16",
                                        div {
                                            class: "text-6xl mb-4 opacity-50",
                                            if has_wallet { "💰" } else { "🔍" }
                                        }
                                        p {
                                            class: "text-base font-semibold mb-2",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            if has_wallet {
                                                "钱包中暂无此代币"
                                            } else {
                                                "未找到匹配的代币"
                                            }
                                        }
                                        p {
                                            class: "text-sm text-center px-4",
                                            style: format!("color: {};", Colors::TEXT_TERTIARY),
                                            if has_wallet {
                                                "您的钱包中还没有这个代币的余额"
                                            } else {
                                                "尝试搜索其他名称或直接粘贴代币合约地址"
                                            }
                                        }
                                        button {
                                            class: "mt-4 px-4 py-2 rounded-lg text-sm font-medium transition-all hover:scale-105",
                                            style: format!("background: {}; color: white;", Colors::TECH_PRIMARY),
                                            onclick: move |_| search_query.set(String::new()),
                                            "清除搜索"
                                        }
                                    }
                                }

                                // 代币列表项
                                for token in filtered_tokens.read().iter() {
                                    div {
                                        class: "flex items-center justify-between p-4 mb-2 cursor-pointer transition-all rounded-xl border-2",
                                        style: format!(
                                            "background: {}; border-color: {};",
                                            if selected_token.read().as_ref().map(|t| t.address == token.address).unwrap_or(false) {
                                                "rgba(99, 102, 241, 0.15)"
                                            } else {
                                                "transparent"
                                            },
                                            if selected_token.read().as_ref().map(|t| t.address == token.address).unwrap_or(false) {
                                                Colors::TECH_PRIMARY
                                            } else {
                                                "rgba(99, 102, 241, 0.2)"
                                            }
                                        ),
                                        onclick: {
                                            let mut selected_token_mut = selected_token;
                                            let mut show_modal_mut = show_modal;
                                            let token_clone = token.clone();
                                            move |_| {
                                                selected_token_mut.set(Some(token_clone.clone()));
                                                show_modal_mut.set(false);
                                            }
                                        },

                                        // 左侧：图标 + 信息
                                        div {
                                            class: "flex items-center gap-3 flex-1",

                                            // 代币图标
                                            div {
                                                class: "relative",
                                                if let Some(logo_url) = &token.logo_url {
                                                    img {
                                                        src: logo_url.clone(),
                                                        alt: token.symbol.clone(),
                                                        class: "w-12 h-12 rounded-full shadow-md",
                                                    }
                                                } else {
                                                    div {
                                                        class: "w-12 h-12 rounded-full flex items-center justify-center font-bold text-white text-xl shadow-md",
                                                        style: format!(
                                                            "background: linear-gradient(135deg, {} 0%, {} 100%);",
                                                            Colors::TECH_PRIMARY,
                                                            Colors::TECH_SECONDARY
                                                        ),
                                                        {token.symbol.chars().next().unwrap_or('?').to_string()}
                                                    }
                                                }
                                                // 原生代币标记
                                                if token.is_native {
                                                    div {
                                                        class: "absolute -bottom-1 -right-1 w-4 h-4 rounded-full flex items-center justify-center text-[10px]",
                                                        style: format!("background: {}; color: white;", Colors::TECH_PRIMARY),
                                                        "⭐"
                                                    }
                                                }
                                            }

                                            // 代币信息
                                            div {
                                                class: "flex-1",
                                                div {
                                                    class: "flex items-center gap-2",
                                                    span {
                                                        class: "font-bold text-lg",
                                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                                        {token.symbol.clone()}
                                                    }
                                                    if token.is_native {
                                                        span {
                                                            class: "text-[10px] px-1.5 py-0.5 rounded",
                                                            style: format!("background: {}; color: white;", Colors::TECH_PRIMARY),
                                                            "原生"
                                                        }
                                                    }
                                                }
                                                div {
                                                    class: "text-xs mt-0.5",
                                                    style: format!("color: {};", Colors::TEXT_TERTIARY),
                                                    {token.name.clone()}
                                                }
                                            }
                                        }

                                        // 右侧：余额信息
                                        div {
                                            class: "text-right",
                                            if has_wallet {
                                                if let Some(balance) = token_balances.read().get(&token.address) {
                                                    div {
                                                        class: "font-semibold text-sm",
                                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                                        {format!("{:.6}", balance)}
                                                    }
                                                    div {
                                                        class: "text-xs",
                                                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                                                        {token.symbol.clone()}
                                                    }
                                                } else {
                                                    div {
                                                        class: "text-xs",
                                                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                                                        "—"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 📌 底部提示 - 根据场景显示不同内容
                        if !loading() && error.read().is_none() && search_query.read().is_empty() {
                            div {
                                class: "pt-4 mt-2 border-t",
                                style: format!("background: {}; border-color: {};",
                                    Colors::BG_PRIMARY, Colors::BORDER_PRIMARY),

                                if has_wallet {
                                    // 有钱包场景：显示余额提示
                                    div {
                                        class: "flex items-center justify-center gap-2 p-3 rounded-xl",
                                        style: format!("background: {}; border: 2px solid {};",
                                            "rgba(99, 102, 241, 0.05)",
                                            "rgba(99, 102, 241, 0.2)"
                                        ),
                                        span { class: "text-base", "💡" }
                                        p {
                                            class: "text-xs",
                                            style: format!("color: {};", Colors::TEXT_TERTIARY),
                                            "只显示有余额的代币 · 共 {filtered_tokens.read().len()} 个"
                                        }
                                    }
                                } else {
                                    // 无钱包场景：显示导入按钮
                                    button {
                                        class: "w-full flex items-center justify-center gap-2 p-3 rounded-xl transition-all hover:scale-[1.02] hover:shadow-lg active:scale-95",
                                        style: format!("background: {}; color: white; border: 2px solid {};",
                                            Colors::TECH_PRIMARY,
                                            Colors::TECH_PRIMARY
                                        ),
                                        span { class: "text-lg", "➕" }
                                        span {
                                            class: "text-sm font-bold",
                                            "导入自定义代币"
                                        }
                                    }

                                    p {
                                        class: "text-xs text-center mt-2 opacity-60",
                                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                                        "粘贴 ERC-20 代币合约地址"
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
