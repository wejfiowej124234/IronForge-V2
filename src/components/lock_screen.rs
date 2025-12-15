//! Lock Screen Component - 钱包解锁屏幕
//! 全屏遮罩，要求用户输入密码解锁钱包

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::atoms::input::{Input, InputType};
use crate::features::wallet::hooks::use_wallet;
use crate::shared::design_tokens::Colors;
use dioxus::events::FormEvent;
use dioxus::prelude::*;

/// Lock Screen Component - 钱包解锁屏幕
///
/// 当钱包锁定时显示，要求用户输入密码解锁
#[component]
pub fn LockScreen() -> Element {
    let password = use_signal(|| String::new());
    let error_message = use_signal(|| Option::<String>::None);
    let is_loading = use_signal(|| false);
    let remember_password = use_signal(|| false);

    let wallet_controller = use_wallet();
    let app_state = use_context::<crate::shared::state::AppState>();

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center",
            style: format!(
                "background: {}; backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px);",
                format!("{}CC", Colors::BG_PRIMARY) // 添加透明度
            ),

            Card {
                variant: crate::components::atoms::card::CardVariant::Base,
                padding: Some("32px".to_string()),
                class: Some("max-w-md w-full mx-4".to_string()),
                children: rsx! {
                    // Logo 和标题
                    div {
                        class: "text-center mb-6",
                        div {
                            class: "text-4xl mb-4",
                            "🔒"
                        }
                        h1 {
                            class: "text-2xl font-bold mb-2",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "钱包已锁定"
                        }
                        p {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "请输入密码以解锁钱包"
                        }
                    }

                    // 安全提示
                    div {
                        class: "mb-6 p-3 rounded-lg",
                        style: format!("background: rgba(99, 102, 241, 0.1); border: 1px solid {};", Colors::TECH_PRIMARY),
                        p {
                            class: "text-xs",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "💡 钱包将在5分钟无操作后自动锁定，以保护您的资产安全"
                        }
                    }

                    // 密码输入
                    form {
                        onsubmit: {
                            let password = password;
                            let mut error_message = error_message;
                            let mut is_loading = is_loading;
                            let remember_password = remember_password;
                            let wallet_controller = wallet_controller;
                            let app_state = app_state;

                            move |e: FormEvent| {
                                e.stop_propagation();
                                let pwd = password.read().clone();

                                if !pwd.is_empty() {
                                    is_loading.set(true);
                                    error_message.set(None);

                                    let wallet_state = app_state.wallet.read();
                                    let wallet_id = wallet_state.selected_wallet_id.clone();
                                    drop(wallet_state);

                                    if let Some(wallet_id) = wallet_id {
                                        let wallet_id_clone = wallet_id.clone();
                                        let wallet_ctrl = wallet_controller;
                                        let mut loading = is_loading;
                                        let mut error_msg = error_message;
                                        let mut pwd_sig = password;
                                        let remember = remember_password;

                                        spawn(async move {
                                            match wallet_ctrl.unlock_wallet(&wallet_id_clone, &pwd).await {
                                                Ok(_) => {
                                                    loading.set(false);
                                                    pwd_sig.set(String::new());

                                                    // 如果选择了"记住密码"，设置记住时间
                                                    if remember() {
                                                        // 可以在这里实现"记住密码5分钟"的逻辑
                                                        // 暂时只是解锁
                                                    }
                                                }
                                                Err(e) => {
                                                    loading.set(false);
                                                    error_msg.set(Some(format!("解锁失败: {}", e)));
                                                    pwd_sig.set(String::new());
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                        },
                        Input {
                            input_type: InputType::Password,
                            label: Some("密码".to_string()),
                            placeholder: Some("请输入钱包密码".to_string()),
                            value: Some(password.read().clone()),
                            error: error_message.read().clone(),
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

                    // 记住密码选项
                    div {
                        class: "mb-6",
                        label {
                            class: "flex items-center gap-3 cursor-pointer",
                            input {
                                r#type: "checkbox",
                                checked: remember_password(),
                                onchange: {
                                    let mut remember_password = remember_password;
                                    move |_e: FormEvent| {
                                        // 切换checkbox状态
                                        remember_password.set(!remember_password());
                                    }
                                },
                                class: "w-5 h-5 rounded",
                                style: format!("accent-color: {};", Colors::TECH_PRIMARY),
                            }
                            span {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "记住密码5分钟"
                            }
                        }
                    }

                    // 解锁按钮
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Large,
                        class: Some("w-full".to_string()),
                        disabled: password.read().is_empty() || *is_loading.read(),
                        loading: *is_loading.read(),
                        onclick: {
                            let password = password;
                            let mut error_message = error_message;
                            let is_loading = is_loading;
                            let remember_password = remember_password;
                            let wallet_controller = wallet_controller;
                            let app_state = app_state;

                            move |_| {
                                let pwd = password.read().clone();

                                if pwd.is_empty() {
                                    error_message.set(Some("请输入密码".to_string()));
                                    return;
                                }

                                let wallet_state = app_state.wallet.read();
                                let wallet_id = wallet_state.selected_wallet_id.clone();
                                drop(wallet_state);

                                if let Some(wallet_id) = wallet_id {
                                    let wallet_id_clone = wallet_id.clone();
                                    let wallet_ctrl = wallet_controller;
                                    let mut loading = is_loading;
                                    let mut error_msg = error_message;
                                    let mut pwd_sig = password;
                                    let remember = remember_password;

                                    loading.set(true);
                                    error_msg.set(None);

                                    spawn(async move {
                                        match wallet_ctrl.unlock_wallet(&wallet_id_clone, &pwd).await {
                                            Ok(_) => {
                                                loading.set(false);
                                                pwd_sig.set(String::new());

                                                // 如果选择了"记住密码"，设置记住时间
                                                if remember() {
                                                    // 可以在这里实现"记住密码5分钟"的逻辑
                                                    // 暂时只是解锁
                                                }
                                            }
                                            Err(e) => {
                                                loading.set(false);
                                                error_msg.set(Some(format!("解锁失败: {}", e)));
                                                pwd_sig.set(String::new());
                                            }
                                        }
                                    });
                                }
                            }
                        },
                        "解锁钱包"
                    }

                    // 帮助文本
                    div {
                        class: "mt-4 text-center",
                        p {
                            class: "text-xs",
                            style: format!("color: {};", Colors::TEXT_TERTIARY),
                            "忘记密码？您可以使用助记词恢复钱包"
                        }
                    }
                }
            }
        }
    }
}
