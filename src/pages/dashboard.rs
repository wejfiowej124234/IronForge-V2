//! Dashboard Page - 仪表盘页面
//! 显示钱包列表，支持选择钱包和查看资产

#![allow(clippy::clone_on_copy)]

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::molecules::WalletDeleteModal;
use crate::components::route_guard::AuthGuard;
use crate::components::wallet_unlock_modal::WalletUnlockModal;
use crate::features::auth::hooks::use_auth;
use crate::features::wallet::hooks::use_wallet;
use crate::features::wallet::state::Wallet;
use crate::pages::dashboard_balance::BalanceOverview;
use crate::pages::dashboard_transactions::TransactionHistoryPreview;
use crate::router::Route;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;

/// 链ID映射（用于API调用）
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

/// Dashboard Page 组件
#[component]
pub fn Dashboard() -> Element {
    rsx! {
        AuthGuard {
            DashboardContent {}
        }
    }
}

/// Dashboard 内容组件（需要认证）
#[component]
fn DashboardContent() -> Element {
    let app_state = use_context::<AppState>();
    let navigator = use_navigator();
    let auth_controller = use_auth();
    let t = crate::i18n::use_translation();

    // 登录后从后端同步钱包列表
    // 使用use_future确保在组件渲染时立即执行，而不是等待use_effect
    use_future(move || {
        let auth_ctrl = auth_controller;
        let mut app_state = app_state;
        async move {
            // 如果已登录且有token，从后端同步钱包
            let user_state = app_state.user.read();
            let is_authenticated = user_state.is_authenticated;
            let has_token = user_state
                .access_token
                .as_ref()
                .map(|t| !t.is_empty())
                .unwrap_or(false);

            if is_authenticated && has_token {
                // 同步钱包（如果失败，会保留本地钱包）
                if let Err(e) = auth_ctrl.sync_wallets_from_backend().await {
                    #[cfg(debug_assertions)]
                    {
                        use tracing::warn;
                        warn!("Failed to sync wallets from backend: {:?}", e);
                    }
                    // 错误已在sync_wallets_from_backend中处理，这里不需要额外处理
                }
            } else if !is_authenticated {
                // 如果未登录，尝试从本地存储加载钱包（用于离线查看）
                let mut wallet_state = app_state.wallet.write();
                if wallet_state.wallets.is_empty() {
                    // 使用WalletState::load()方法加载钱包
                    use crate::features::wallet::state::WalletState;
                    let local_wallet_state = WalletState::load().await;
                    if !local_wallet_state.wallets.is_empty() {
                        wallet_state.wallets = local_wallet_state.wallets;
                        wallet_state.selected_wallet_id = local_wallet_state.selected_wallet_id;
                        let _ = wallet_state.save();
                    }
                }
            }
        }
    });

    // 钱包自动锁定定时器（每30秒检查一次，5分钟后自动锁定）
    use_effect(move || {
        let wallet_ctrl = use_wallet();
        let app_state_for_timer = app_state;

        spawn(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(30000).await; // 每30秒检查一次

                let wallet_state = app_state_for_timer.wallet.read();
                let unlock_times = app_state_for_timer.wallet_unlock_time.read();
                let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;

                // 检查所有钱包的解锁状态
                for wallet in wallet_state.wallets.iter() {
                    if !wallet.is_locked {
                        if let Some(unlock_time) = unlock_times.get(&wallet.id) {
                            // 超过5分钟（300秒）自动锁定
                            if now - unlock_time > 300 {
                                #[cfg(debug_assertions)]
                                {
                                    use tracing::info;
                                    info!("🔒 钱包 '{}' 自动锁定（已解锁超过5分钟）", wallet.name);
                                }
                                let wallet_id = wallet.id.clone();
                                drop(wallet_state);
                                drop(unlock_times);
                                wallet_ctrl.lock_wallet(Some(&wallet_id));
                                break;
                            }
                        }
                    }
                }
            }
        });
    });

    let user_state = app_state.user.read();
    let wallet_state = app_state.wallet.read();

    // 用户头像
    let avatar_url = user_state.get_avatar_url();

    rsx! {
        div {
            class: "min-h-screen",
            style: format!("background: {};", Colors::BG_PRIMARY),

            div {
                class: "container mx-auto px-4 sm:px-6 lg:px-8 py-4 sm:py-6 lg:py-8",
                // 移动端优化的顶部栏
                div {
                    class: "flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6 sm:mb-8",
                    div {
                        class: "flex items-center gap-3 sm:gap-4",
                        // 用户头像
                        img {
                            src: "{avatar_url}",
                            alt: "Avatar",
                            class: "w-10 h-10 sm:w-12 sm:h-12 rounded-full border-2",
                            style: format!("border-color: {};", Colors::TECH_PRIMARY),
                        }
                        div {
                            h1 {
                                class: "text-xl sm:text-2xl font-bold",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {user_state.email.as_ref().unwrap_or(&"用户".to_string()).clone()}
                            }
                            p {
                                class: "text-xs sm:text-sm",
                                style: format!("color: {};", Colors::TEXT_TERTIARY),
                                "IronForge 钱包"
                            }
                        }
                    }
                }

                // 钱包列表或空状态
                if wallet_state.wallets.is_empty() {
                    Card {
                        variant: crate::components::atoms::card::CardVariant::Base,
                        padding: Some("48px".to_string()),
                        children: rsx! {
                            div {
                                class: "text-center",
                                crate::components::atoms::icon::Icon {
                                    name: "wallet".to_string(),
                                    size: crate::components::atoms::icon::IconSize::XXL,
                                }
                                h2 {
                                    class: "text-2xl font-bold mt-4 mb-2",
                                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                                    "还没有钱包"
                                }
                                p {
                                    class: "text-sm mb-6",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "创建或导入您的第一个钱包"
                                }
                                div {
                                    class: "flex gap-4 justify-center",
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        size: ButtonSize::Large,
                                        onclick: move |_| {
                                            navigator.push(Route::CreateWallet {});
                                        },
                                        {t("dashboard.create_wallet")}
                                    }
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        size: ButtonSize::Large,
                                        onclick: move |_| {
                                            navigator.push(Route::ImportWallet {});
                                        },
                                        "导入/恢复钱包"
                                    }
                                }
                                div {
                                    class: "mt-4 text-center text-sm",
                                    style: format!("color: {};", Colors::TEXT_TERTIARY),
                                    p {
                                        class: "mb-2",
                                        "💡 提示："
                                    }
                                    p {
                                        "• 创建钱包：生成新钱包和助记词"
                                    }
                                    p {
                                        "• 导入/恢复钱包：使用助记词或私钥在新设备上恢复钱包"
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // 选中的钱包余额聚合显示和交易历史
                    if let Some(selected_wallet_id) = &wallet_state.selected_wallet_id {
                        if let Some(selected_wallet) = wallet_state.wallets.iter().find(|w| &w.id == selected_wallet_id) {
                            BalanceOverview {
                                wallet: selected_wallet.clone(),
                            }

                            // 交易历史预览
                            TransactionHistoryPreview {
                                wallet_id: selected_wallet_id.clone(),
                                accounts: selected_wallet.accounts.clone(),
                            }
                        }
                    }

                    // 钱包列表
                    div {
                        class: "mb-6 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4",
                        h2 {
                            class: "text-xl sm:text-2xl font-bold",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            {t("dashboard.my_wallets")}
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Small,
                            class: Some("w-full sm:w-auto".to_string()),
                            onclick: move |_| {
                                navigator.push(Route::CreateWallet {});
                            },
                            {t("dashboard.create_wallet")}
                        }
                    }

                    div {
                        class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4",
                        for wallet in wallet_state.wallets.iter() {
                            WalletCard {
                                wallet: wallet.clone(),
                                is_selected: wallet_state.selected_wallet_id.as_ref() == Some(&wallet.id),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 钱包卡片组件
#[component]
fn WalletCard(wallet: Wallet, is_selected: bool) -> Element {
    let app_state = use_context::<AppState>();
    let navigator = use_navigator();
    let wallet_controller = use_wallet();

    // 删除确认模态框状态
    let mut show_delete_confirm = use_signal(|| false);
    // 解锁钱包模态框状态
    let mut show_unlock_modal = use_signal(|| false);

    // 检查钱包是否在本地存储中
    let is_in_local_storage = wallet_controller.is_wallet_in_local_storage(&wallet.id);
    let is_unlocked = wallet_controller.is_wallet_unlocked(&wallet.id);

    let wallet_id_clone = wallet.id.clone();
    let handle_select_1 = {
        let mut app_state = app_state;
        let wallet_id = wallet_id_clone.clone();
        move |_| {
            let mut wallet_state = app_state.wallet.write();
            wallet_state.selected_wallet_id = Some(wallet_id.clone());
            wallet_state.save().ok();
        }
    };
    let handle_select_2 = {
        let mut app_state = app_state;
        let wallet_id = wallet_id_clone.clone();
        move |_| {
            let mut wallet_state = app_state.wallet.write();
            wallet_state.selected_wallet_id = Some(wallet_id.clone());
            wallet_state.save().ok();
        }
    };

    rsx! {
        Card {
            variant: if is_selected {
                crate::components::atoms::card::CardVariant::Strong
            } else {
                crate::components::atoms::card::CardVariant::Base
            },
            padding: Some("24px".to_string()),
            children: rsx! {
                div {
                    class: "cursor-pointer",
                    onclick: handle_select_1,
                    div {
                        class: "flex justify-between items-start mb-4",
                        div {
                            h3 {
                                class: "text-lg font-semibold",
                                style: format!("color: {};", Colors::TEXT_PRIMARY),
                                {wallet.name.clone()}
                            }
                            p {
                                class: "text-xs mt-1",
                                style: format!("color: {};", Colors::TEXT_TERTIARY),
                                {format!("{} 个账户", wallet.accounts.len())}
                            }
                        }
                        if is_selected {
                            span {
                                class: "text-xs px-2 py-1 rounded",
                                style: format!("background: {}; color: white;", Colors::TECH_PRIMARY),
                                "已选择"
                            }
                        }
                    }

                    // 钱包状态
                    div {
                        class: "flex flex-col gap-2 mb-4",
                        // 恢复状态（新设备检测）
                        if !is_in_local_storage {
                            div {
                                class: "p-2 rounded-lg",
                                style: format!("background: rgba(251, 191, 36, 0.1); border: 1px solid rgba(251, 191, 36, 0.3);"),
                                div {
                                    class: "flex items-center gap-2",
                                    span {
                                        class: "text-xs font-semibold",
                                        style: format!("color: rgb(251, 191, 36);"),
                                        "⚠️ 需要恢复"
                                    }
                                }
                                p {
                                    class: "text-xs mt-1",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "新设备：需要恢复钱包才能签名交易"
                                }
                            }
                        }
                        // 锁定/解锁状态（标签样式，与整体设计统一）
                        if is_in_local_storage {
                            div {
                                class: "inline-flex items-center gap-2 px-2 py-1 rounded-full text-xs",
                                style: if wallet.is_locked || !is_unlocked {
                                    // 锁定：警告色背景
                                    format!("background: rgba(248, 113, 113, 0.12); color: {};", Colors::PAYMENT_WARNING)
                                } else {
                                    // 已解锁：成功色背景
                                    format!("background: rgba(34, 197, 94, 0.12); color: {};", Colors::PAYMENT_SUCCESS)
                                },
                                span {
                                    if wallet.is_locked || !is_unlocked {
                                        "🔒 已锁定 · 仅可查看，不能交易"
                                    } else {
                                        "🔓 已解锁 · 会话约 5 分钟内有效"
                                    }
                                }
                            }
                        }
                    }

                    // 账户预览 - 显示所有账户（4个链：BTC、ETH、Solana、TON）
                    if !wallet.accounts.is_empty() {
                        div {
                            class: "space-y-2",
                            for account in wallet.accounts.iter() {
                                div {
                                    class: "flex justify-between items-center text-xs",
                                    span {
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        {account.chain_label()}
                                    }
                                    span {
                                        class: "font-mono",
                                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                                        {account.short_address()}
                                    }
                                }
                            }
                        }
                    }

                    // 操作按钮
                    div {
                        class: "flex gap-2 mt-4 pt-4",
                        style: format!("border-top: 1px solid {};", Colors::BORDER_PRIMARY),
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Small,
                            class: Some("flex-1".to_string()),
                            onclick: {
                                let wallet_id = wallet.id.clone();
                                move |_| {
                                    navigator.push(Route::WalletDetail { id: wallet_id.clone() });
                                }
                            },
                            "详情"
                        }
                        if !is_in_local_storage {
                            // 新设备：显示"恢复钱包"按钮
                            Button {
                                variant: ButtonVariant::Primary,
                                size: ButtonSize::Small,
                                class: Some("flex-1".to_string()),
                                onclick: {
                                    let _wallet_id = wallet.id.clone();
                                    move |_| {
                                        navigator.push(Route::ImportWallet {});
                                    }
                                },
                                "恢复钱包"
                            }
                        } else if wallet.is_locked || !is_unlocked {
                            // 已在本地但锁定：优先提供"解锁钱包"按钮
                            Button {
                                variant: ButtonVariant::Primary,
                                size: ButtonSize::Small,
                                class: Some("flex-1".to_string()),
                                onclick: move |_| {
                                    show_unlock_modal.set(true);
                                },
                                "解锁钱包"
                            }
                        } else if !is_selected {
                            // 已解锁但未选中：显示"选择"按钮
                            Button {
                                variant: ButtonVariant::Primary,
                                size: ButtonSize::Small,
                                class: Some("flex-1".to_string()),
                                onclick: handle_select_2,
                                "选择"
                            }
                        } else {
                            // 已解锁且已选中：提供手动锁定按钮
                            Button {
                                variant: ButtonVariant::Secondary,
                                size: ButtonSize::Small,
                                class: Some("flex-1".to_string()),
                                onclick: {
                                    let mut app_state = app_state;
                                    let wallet_id = wallet.id.clone();
                                    let wallet_ctrl = wallet_controller.clone();
                                    move |_| {
                                        // 1. 调用钱包控制器锁定本地 KeyManager / 会话
                                        wallet_ctrl.lock_wallet(Some(&wallet_id));

                                        // 2. 清除 AppState 中的解锁时间戳，使 TTL 立即失效
                                        let mut state = app_state.wallet_unlock_time.write();
                                        state.remove(&wallet_id);
                                    }
                                },
                                "锁定钱包"
                            }
                        }
                        // 删除钱包按钮（始终显示）- 弹出主题风格确认弹窗
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Small,
                            class: Some("px-3".to_string()),
                            onclick: {
                                move |_| {
                                    show_delete_confirm.set(true);
                                }
                            },
                            "删除"
                        }
                    }
                }
            }
        }

        // 删除确认弹窗
        WalletDeleteModal {
            open: show_delete_confirm(),
            wallet_id: wallet.id.clone(),
            wallet_name: wallet.name.clone(),
            app_state: Signal::new(app_state.clone()),
            wallet_controller: Signal::new(wallet_controller.clone()),
            on_close: move |_| {
                show_delete_confirm.set(false);
            },
        }

        // 解锁钱包弹窗
        if show_unlock_modal() {
            WalletUnlockModal {
                wallet_id: wallet.id.clone(),
                open: true,
                on_unlock: move |_| {
                    // 解锁成功后关闭弹窗
                    show_unlock_modal.set(false);
                },
                on_close: move |_| {
                    show_unlock_modal.set(false);
                },
            }
        }
    }
}
