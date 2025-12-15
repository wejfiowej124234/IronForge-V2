//! Wallet Pages - 钱包相关页面
//! 创建钱包、导入钱包等

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::atoms::input::{Input, InputType};
use crate::components::molecules::ErrorMessage;
use crate::components::route_guard::AuthGuard;
use crate::features::wallet::hooks::use_wallet;
use crate::router::Route;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::events::FormEvent;
use dioxus::prelude::*;

/// Create Wallet Page - 创建钱包页面
#[component]
pub fn CreateWallet() -> Element {
    rsx! {
        AuthGuard {
            CreateWalletContent {}
        }
    }
}

/// 创建钱包内容组件
#[component]
fn CreateWalletContent() -> Element {
    let wallet_name = use_signal(|| String::new());
    let password = use_signal(|| String::new());
    let confirm_password = use_signal(|| String::new());
    let error_message = use_signal(|| Option::<String>::None);
    let is_loading = use_signal(|| false);

    let wallet_controller = use_wallet();
    let navigator = use_navigator();
    let app_state = use_context::<AppState>();

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center",
            style: format!("background: {};", Colors::BG_PRIMARY),

            Card {
                variant: crate::components::atoms::card::CardVariant::Base,
                padding: Some("32px".to_string()),
                children: rsx! {
                    h1 {
                        class: "text-2xl font-bold mb-6",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "创建钱包"
                    }

                    Input {
                        input_type: InputType::Text,
                        label: Some("钱包名称".to_string()),
                        placeholder: Some("请输入钱包名称".to_string()),
                        value: Some(wallet_name.read().clone()),
                        onchange: {
                            let mut wallet_name = wallet_name;
                            Some(EventHandler::new(move |e: FormEvent| {
                                wallet_name.set(e.value());
                            }))
                        },
                    }

                    Input {
                        input_type: InputType::Password,
                        label: Some("密码".to_string()),
                        placeholder: Some("请输入密码".to_string()),
                        value: Some(password.read().clone()),
                        onchange: {
                            let mut password = password;
                            Some(EventHandler::new(move |e: FormEvent| {
                                password.set(e.value());
                            }))
                        },
                    }

                    Input {
                        input_type: InputType::Password,
                        label: Some("确认密码".to_string()),
                        placeholder: Some("请再次输入密码".to_string()),
                        value: Some(confirm_password.read().clone()),
                        onchange: {
                            let mut confirm_password = confirm_password;
                            Some(EventHandler::new(move |e: FormEvent| {
                                confirm_password.set(e.value());
                            }))
                        },
                    }

                    ErrorMessage {
                        message: error_message.read().clone()
                    }

                    div {
                        class: "mt-6 flex gap-4",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            disabled: is_loading(),
                            loading: is_loading(),
                            onclick: {
                                let wallet_name = wallet_name;
                                let password = password;
                                let confirm_password = confirm_password;
                                let mut error_message = error_message;
                                let mut is_loading = is_loading;
                                let wallet_controller = wallet_controller;
                                let navigator = navigator.clone();

                                move |_| {
                                    let name = wallet_name.read().clone();
                                    let pwd = password.read().clone();
                                    let confirm = confirm_password.read().clone();

                                    // 验证输入
                                    if name.trim().is_empty() {
                                        error_message.set(Some("钱包名称不能为空".to_string()));
                                        return;
                                    }
                                    if pwd.len() < 8 {
                                        error_message.set(Some("密码至少需要8个字符".to_string()));
                                        return;
                                    }
                                    if pwd != confirm {
                                        error_message.set(Some("两次输入的密码不一致".to_string()));
                                        return;
                                    }

                                    // 创建钱包
                                    is_loading.set(true);
                                    error_message.set(None);

                                    let wallet_ctrl = wallet_controller;
                                    let nav = navigator.clone();
                                    let mut loading = is_loading;
                                    let mut error_msg = error_message;
                                    let toasts = app_state.toasts;

                                    spawn(async move {
                                        match wallet_ctrl.create_wallet(&name, &pwd).await {
                                            Ok(phrase) => {
                                                loading.set(false);
                                                AppState::show_success(toasts, "钱包创建成功，请备份助记词".to_string());
                                                // 导航到助记词备份页面，传递助记词
                                                nav.push(Route::MnemonicBackup { phrase: phrase.clone() });
                                            }
                                            Err(e) => {
                                                loading.set(false);
                                                let err_msg = format!("创建钱包失败: {}", e);
                                                AppState::show_error(toasts, err_msg.clone());
                                                error_msg.set(Some(err_msg));
                                            }
                                        }
                                    });
                                }
                            },
                            "创建钱包"
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Large,
                            disabled: is_loading(),
                            onclick: move |_| {
                                navigator.go_back();
                            },
                            "取消"
                        }
                    }
                }
            }
        }
    }
}
