//! Login Page - 登录页面
//! 用户登录账户

#![allow(
    clippy::redundant_closure,
    clippy::redundant_locals,
    clippy::clone_on_copy
)]

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::atoms::input::{Input, InputType};
use crate::components::molecules::ErrorMessage;
use crate::features::auth::hooks::use_auth;
use crate::router::Route;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::events::FormEvent;
use dioxus::prelude::*;

/// Login Page - 登录页面
#[component]
pub fn Login() -> Element {
    let navigator = use_navigator();
    let auth_controller = use_auth();
    let app_state = use_context::<AppState>();

    let email = use_signal(|| String::new());
    let password = use_signal(|| String::new());
    let error_message = use_signal(|| Option::<String>::None);
    let is_loading = use_signal(|| false);

    let handle_login = {
        let email = email;
        let password = password;
        let auth_controller = auth_controller;
        let mut is_loading = is_loading;
        let mut error_message = error_message;
        let navigator = navigator.clone();

        move |_| {
            let email_val = email.read().trim().to_string();
            let pwd = password.read().clone();

            // 验证输入
            if email_val.is_empty() || !email_val.contains('@') {
                error_message.set(Some("请输入有效的邮箱地址".to_string()));
                return;
            }

            if pwd.is_empty() {
                error_message.set(Some("请输入密码".to_string()));
                return;
            }

            is_loading.set(true);
            error_message.set(None);

            let auth_ctrl = auth_controller;
            let mut loading = is_loading;
            let mut error = error_message;
            let nav = navigator.clone();

            spawn(async move {
                match auth_ctrl.login(&email_val, &pwd).await {
                    Ok(_) => {
                        loading.set(false);
                        // 登录成功，显示Toast并跳转到Dashboard
                        AppState::show_success(app_state.toasts, "登录成功".to_string());
                        nav.push(Route::Dashboard {});
                    }
                    Err(e) => {
                        loading.set(false);
                        let err_msg = format!("登录失败: {}", e);
                        AppState::show_error(app_state.toasts, err_msg.clone());
                        error.set(Some(err_msg));
                    }
                }
            });
        }
    };

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center p-4",
            style: format!("background: {};", Colors::BG_PRIMARY),

            Card {
                variant: crate::components::atoms::card::CardVariant::Base,
                padding: Some("32px".to_string()),
                class: Some("max-w-md w-full".to_string()),
                children: rsx! {
                    // Logo和标题
                    div {
                        class: "text-center mb-8",
                        h1 {
                            class: "text-3xl font-bold mb-2",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "登录账户"
                        }
                        p {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "登录您的 IronForge 账户"
                        }
                    }

                    // 邮箱输入
                    div {
                        class: "mb-6",
                        Input {
                            input_type: InputType::Text,
                            label: Some("邮箱".to_string()),
                            placeholder: Some("请输入邮箱地址".to_string()),
                            value: Some(email.read().clone()),
                            onchange: {
                                let mut email = email;
                                let mut error_message = error_message;
                                Some(EventHandler::new(move |e: FormEvent| {
                                    email.set(e.value());
                                    error_message.set(None);
                                }))
                            },
                        }
                    }

                    // 密码输入
                    div {
                        class: "mb-6",
                        Input {
                            input_type: InputType::Password,
                            label: Some("密码".to_string()),
                            placeholder: Some("请输入密码".to_string()),
                            value: Some(password.read().clone()),
                            onchange: {
                                let mut password = password;
                                let mut error_message = error_message;
                                Some(EventHandler::new(move |e: FormEvent| {
                                    password.set(e.value());
                                    error_message.set(None);
                                }))
                            },
                        }
                    }

                    // 错误提示
                    ErrorMessage {
                        message: error_message.read().clone()
                    }

                    // 登录按钮
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Large,
                        class: Some("w-full mb-4".to_string()),
                        disabled: is_loading(),
                        loading: is_loading(),
                        onclick: handle_login,
                        "登录"
                    }

                    // 注册链接
                    div {
                        class: "text-center",
                        span {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "还没有账户？"
                        }
                        button {
                            class: "ml-2 text-sm font-medium",
                            style: format!("color: {};", Colors::TECH_PRIMARY),
                            onclick: move |_| {
                                navigator.push(Route::Register {});
                            },
                            "立即注册"
                        }
                    }
                }
            }
        }
    }
}
