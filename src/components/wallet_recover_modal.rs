//! Wallet Recover Modal - 钱包恢复模态框
//! 用于新设备场景，提示用户恢复钱包

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::input::{Input, InputType};
use crate::components::atoms::modal::Modal;
use crate::components::molecules::ErrorMessage;
use crate::features::wallet::hooks::use_wallet;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::events::FormEvent;
use dioxus::prelude::*;

/// 恢复方式
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)] // 在组件内部使用
enum RecoverMethod {
    Mnemonic,
    PrivateKey,
}

/// 钱包恢复模态框
/// 用于新设备场景，当检测到钱包不在本地存储时显示
#[component]
pub fn WalletRecoverModal(
    wallet_id: String,
    wallet_name: String,
    open: bool,
    on_recovered: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let recover_method = use_signal(|| RecoverMethod::Mnemonic);
    let password = use_signal(|| String::new());
    let confirm_password = use_signal(|| String::new());

    // 助记词相关
    let mnemonic_phrase = use_signal(|| String::new());

    // 私钥相关
    let private_key = use_signal(|| String::new());

    // UI状态
    let error_message = use_signal(|| Option::<String>::None);
    let is_loading = use_signal(|| false);

    let wallet_controller = use_wallet();
    let navigator = use_navigator();
    let app_state = use_context::<AppState>();

    let handle_recover = {
        let wallet_id = wallet_id.clone();
        let wallet_name = wallet_name.clone();
        let password = password;
        let confirm_password = confirm_password;
        let recover_method = recover_method;
        let mnemonic_phrase = mnemonic_phrase;
        let private_key = private_key;
        let wallet_controller = wallet_controller;
        let mut is_loading = is_loading;
        let mut error_message = error_message;
        let on_recovered = on_recovered;
        let navigator = navigator.clone();
        let toasts = app_state.toasts;

        move |_| {
            let pwd = password.read().clone();
            let confirm_pwd = confirm_password.read().clone();
            let method = *recover_method.read();

            // 验证输入
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

            let wallet_ctrl = wallet_controller;
            let wallet_id_clone = wallet_id.clone();
            let wallet_name_clone = wallet_name.clone();
            let mut loading = is_loading;
            let mut error = error_message;
            let on_recovered_handler = on_recovered;
            let _nav = navigator.clone();

            spawn(async move {
                let result = match method {
                    RecoverMethod::Mnemonic => {
                        let phrase = mnemonic_phrase.read().trim().to_string();
                        if phrase.is_empty() {
                            error.set(Some("请输入助记词".to_string()));
                            loading.set(false);
                            return;
                        }
                        wallet_ctrl
                            .recover_wallet(&wallet_name_clone, &phrase, &pwd)
                            .await
                    }
                    RecoverMethod::PrivateKey => {
                        let key = private_key.read().trim().to_string();
                        if key.is_empty() {
                            error.set(Some("请输入私钥".to_string()));
                            loading.set(false);
                            return;
                        }
                        wallet_ctrl
                            .import_from_private_key(&wallet_name_clone, &key, &pwd)
                            .await
                    }
                };

                match result {
                    Ok(_) => {
                        loading.set(false);
                        AppState::show_success(
                            toasts,
                            "钱包恢复成功！现在可以解锁并签名交易了。".to_string(),
                        );
                        on_recovered_handler.call(wallet_id_clone);
                    }
                    Err(e) => {
                        loading.set(false);
                        let err_msg = format!("恢复失败: {}", e);
                        AppState::show_error(toasts, err_msg.clone());
                        error.set(Some(err_msg));
                    }
                }
            });
        }
    };

    rsx! {
        Modal {
            open: open,
            onclose: {
                let mut password = password;
                let mut confirm_password = confirm_password;
                let mut mnemonic_phrase = mnemonic_phrase;
                let mut private_key = private_key;
                let mut error_message = error_message;
                let on_close = on_close;
                move |_| {
                    password.set(String::new());
                    confirm_password.set(String::new());
                    mnemonic_phrase.set(String::new());
                    private_key.set(String::new());
                    error_message.set(None);
                    on_close.call(());
                }
            },
            children: rsx! {
                div {
                    class: "p-6 max-w-md",
                    h2 {
                        class: "text-xl font-bold mb-2",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "恢复钱包"
                    }
                    p {
                        class: "text-sm mb-6",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "检测到这是新设备，钱包数据不在本地存储中。请输入助记词或私钥来恢复钱包，以便在此设备上签名交易。"
                    }

                    // 重要提示
                    div {
                        class: "mb-6 p-4 rounded-lg",
                        style: format!("background: rgba(59, 130, 246, 0.1); border: 1px solid {};", Colors::TECH_PRIMARY),
                        div {
                            class: "flex items-start gap-2",
                            span {
                                class: "text-lg",
                                "💡"
                            }
                            div {
                                p {
                                    class: "text-xs font-semibold mb-1",
                                    style: format!("color: {};", Colors::TECH_PRIMARY),
                                    "新设备恢复说明"
                                }
                                p {
                                    class: "text-xs",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "恢复后，您的私钥将加密存储在此设备的本地浏览器中。只有您可以使用钱包密码解锁。"
                                }
                            }
                        }
                    }

                    // 恢复方式选择
                    div {
                        class: "mb-6",
                        label {
                            class: "block text-sm font-medium mb-2",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "恢复方式"
                        }
                        div {
                            class: "grid grid-cols-2 gap-2",
                            Button {
                                variant: if *recover_method.read() == RecoverMethod::Mnemonic {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Secondary
                                },
                                size: ButtonSize::Medium,
                                onclick: {
                                    let mut recover_method = recover_method;
                                    move |_| {
                                        recover_method.set(RecoverMethod::Mnemonic);
                                    }
                                },
                                "助记词"
                            }
                            Button {
                                variant: if *recover_method.read() == RecoverMethod::PrivateKey {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Secondary
                                },
                                size: ButtonSize::Medium,
                                onclick: {
                                    let mut recover_method = recover_method;
                                    move |_| {
                                        recover_method.set(RecoverMethod::PrivateKey);
                                    }
                                },
                                "私钥"
                            }
                        }
                    }

                    // 根据恢复方式显示不同的输入
                    match *recover_method.read() {
                        RecoverMethod::Mnemonic => rsx! {
                            div {
                                class: "mb-6",
                                Input {
                                    input_type: InputType::Text,
                                    label: Some("助记词".to_string()),
                                    placeholder: Some("请输入12或24个助记词，用空格分隔".to_string()),
                                    value: Some(mnemonic_phrase.read().clone()),
                                    onchange: {
                                        let mut mnemonic_phrase = mnemonic_phrase;
                                        let mut error_message = error_message;
                                        Some(EventHandler::new(move |e: FormEvent| {
                                            mnemonic_phrase.set(e.value());
                                            error_message.set(None);
                                        }))
                                    },
                                }
                            }
                        },
                        RecoverMethod::PrivateKey => rsx! {
                            div {
                                class: "mb-6",
                                Input {
                                    input_type: InputType::Password,
                                    label: Some("私钥".to_string()),
                                    placeholder: Some("请输入私钥（64字符hex，可带0x前缀）".to_string()),
                                    value: Some(private_key.read().clone()),
                                    onchange: {
                                        let mut private_key = private_key;
                                        let mut error_message = error_message;
                                        Some(EventHandler::new(move |e: FormEvent| {
                                            private_key.set(e.value());
                                            error_message.set(None);
                                        }))
                                    },
                                }
                            }
                        },
                    }

                    // 新密码设置
                    div {
                        class: "mb-6",
                        Input {
                            input_type: InputType::Password,
                            label: Some("钱包密码".to_string()),
                            placeholder: Some("请设置钱包密码（至少8个字符）".to_string()),
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

                    // 操作按钮
                    div {
                        class: "flex gap-4 mt-6",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            class: Some("flex-1".to_string()),
                            disabled: is_loading(),
                            loading: is_loading(),
                            onclick: handle_recover,
                            "恢复钱包"
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Large,
                            class: Some("flex-1".to_string()),
                            disabled: is_loading(),
                            onclick: {
                                let mut password = password;
                                let mut confirm_password = confirm_password;
                                let mut mnemonic_phrase = mnemonic_phrase;
                                let mut private_key = private_key;
                                let mut error_message = error_message;
                                let on_close = on_close;
                                move |_| {
                                    password.set(String::new());
                                    confirm_password.set(String::new());
                                    mnemonic_phrase.set(String::new());
                                    private_key.set(String::new());
                                    error_message.set(None);
                                    on_close.call(());
                                }
                            },
                            "取消"
                        }
                    }
                }
            }
        }
    }
}
