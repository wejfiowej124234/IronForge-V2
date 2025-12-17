//! Register Page - 注册页面
//! 用户注册新账户

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
use crate::shared::error::{ApiError, AppError};
use crate::shared::state::AppState;
use dioxus::events::FormEvent;
use dioxus::prelude::*;

fn friendly_register_error(err: &anyhow::Error) -> String {
    if let Some(app_err) = err.downcast_ref::<AppError>() {
        match app_err {
            AppError::Api(ApiError::Timeout) => "请求超时，请稍后再试".to_string(),
            AppError::Api(ApiError::RequestFailed(_)) => {
                "无法连接服务器，请检查网络或稍后再试".to_string()
            }
            // 注册场景通常不会是 Unauthorized，这里给一个保守提示
            AppError::Api(ApiError::Unauthorized) => "注册失败，请稍后再试".to_string(),
            AppError::Api(ApiError::ResponseError(msg)) => {
                let lower = msg.to_lowercase();
                if lower.contains("already")
                    || lower.contains("exists")
                    || lower.contains("duplicate")
                    || lower.contains("conflict")
                    || lower.contains("409")
                {
                    "该邮箱已注册，请直接登录".to_string()
                } else if lower.contains("match") && lower.contains("password") {
                    "两次输入的密码不一致".to_string()
                } else {
                    "注册失败，请稍后再试".to_string()
                }
            }
            _ => "注册失败，请稍后再试".to_string(),
        }
    } else {
        let msg = err.to_string().to_lowercase();
        if msg.contains("timeout") {
            "请求超时，请稍后再试".to_string()
        } else if msg.contains("network")
            || msg.contains("connection")
            || msg.contains("request failed")
        {
            "无法连接服务器，请检查网络或稍后再试".to_string()
        } else if msg.contains("already")
            || msg.contains("exists")
            || msg.contains("duplicate")
            || msg.contains("conflict")
            || msg.contains("409")
        {
            "该邮箱已注册，请直接登录".to_string()
        } else {
            "注册失败，请稍后再试".to_string()
        }
    }
}

/// Register Page - 注册页面
#[component]
pub fn Register() -> Element {
    let navigator = use_navigator();
    let auth_controller = use_auth();
    let app_state = use_context::<AppState>();

    let email = use_signal(|| String::new());
    let password = use_signal(|| String::new());
    let confirm_password = use_signal(|| String::new());
    let error_message = use_signal(|| Option::<String>::None);
    let is_loading = use_signal(|| false);

    let handle_register = {
        let email = email;
        let password = password;
        let confirm_password = confirm_password;
        let auth_controller = auth_controller;
        let mut is_loading = is_loading;
        let mut error_message = error_message;
        let navigator = navigator.clone();

        move |_| {
            let email_val = email.read().trim().to_string();
            let pwd = password.read().clone();
            let confirm_pwd = confirm_password.read().clone();

            // 验证输入
            if email_val.is_empty() || !email_val.contains('@') {
                error_message.set(Some("请输入有效的邮箱地址".to_string()));
                return;
            }

            if pwd.len() < 8 {
                error_message.set(Some("密码至少需要8个字符".to_string()));
                return;
            }

            if pwd != confirm_pwd {
                error_message.set(Some("两次输入的密码不一致".to_string()));
                return;
            }

            is_loading.set(true);
            error_message.set(None);

            let auth_ctrl = auth_controller;
            let mut loading = is_loading;
            let mut error = error_message;
            let nav = navigator.clone();

            spawn(async move {
                match auth_ctrl.register(&email_val, &pwd, &confirm_pwd).await {
                    Ok(_) => {
                        loading.set(false);
                        // 注册成功，显示Toast并跳转到Dashboard
                        AppState::show_success(app_state.toasts, "注册成功".to_string());
                        nav.push(Route::Dashboard {});
                    }
                    Err(e) => {
                        loading.set(false);
                        let err_msg = friendly_register_error(&e);
                        #[cfg(debug_assertions)]
                        {
                            use tracing::warn;
                            warn!("Register failed (raw): {:#}", e);
                        }
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
                            "创建账户"
                        }
                        p {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "注册您的 IronForge 账户"
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
                            placeholder: Some("至少8个字符".to_string()),
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

                    // 确认密码
                    div {
                        class: "mb-6",
                        Input {
                            input_type: InputType::Password,
                            label: Some("确认密码".to_string()),
                            placeholder: Some("请再次输入密码".to_string()),
                            value: Some(confirm_password.read().clone()),
                            onchange: {
                                let mut confirm_password = confirm_password;
                                let mut error_message = error_message;
                                Some(EventHandler::new(move |e: FormEvent| {
                                    confirm_password.set(e.value());
                                    error_message.set(None);
                                }))
                            },
                        }
                    }

                    // 错误提示
                    ErrorMessage {
                        message: error_message.read().clone()
                    }

                    // 注册按钮
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Large,
                        class: Some("w-full mb-4".to_string()),
                        disabled: is_loading(),
                        loading: is_loading(),
                        onclick: handle_register,
                        "注册"
                    }

                    // 登录链接
                    div {
                        class: "text-center",
                        span {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "已有账户？"
                        }
                        button {
                            class: "ml-2 text-sm font-medium",
                            style: format!("color: {};", Colors::TECH_PRIMARY),
                            onclick: move |_| {
                                navigator.push(Route::Login {});
                            },
                            "立即登录"
                        }
                    }
                }
            }
        }
    }
}
