//! Wallet Unlock Modal - 钱包解锁模态框
//! 用于交易签名前解锁钱包

#![allow(
    clippy::clone_on_copy,
    clippy::redundant_closure,
    clippy::redundant_locals,
    clippy::type_complexity
)]

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::input::{Input, InputType};
use crate::components::atoms::modal::Modal;
use crate::components::molecules::ErrorMessage;
use crate::features::wallet::hooks::use_wallet;
use crate::shared::design_tokens::Colors;
use dioxus::events::FormEvent;
use dioxus::prelude::*;

/// 钱包解锁模态框
#[component]
pub fn WalletUnlockModal(
    wallet_id: String,
    open: bool,
    on_unlock: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let password = use_signal(|| String::new());
    let error_message = use_signal(|| Option::<String>::None);
    let is_loading = use_signal(|| false);

    let wallet_controller = use_wallet();

    let handle_unlock = {
        let password = password;
        let mut error_message = error_message;
        let mut is_loading = is_loading;
        let wallet_controller = wallet_controller;
        let wallet_id = wallet_id.clone();
        let on_unlock = on_unlock;

        move |_| {
            let pwd = password.read().clone();
            let wallet_id_clone = wallet_id.clone();

            if pwd.is_empty() {
                error_message.set(Some("请输入钱包密码".to_string()));
                return;
            }

            is_loading.set(true);
            error_message.set(None);

            let wallet_ctrl = wallet_controller;
            let mut loading = is_loading;
            let mut error = error_message;
            let mut pwd_sig = password;
            let on_unlock_handler = on_unlock;

            spawn(async move {
                match wallet_ctrl.unlock_wallet(&wallet_id_clone, &pwd).await {
                    Ok(_) => {
                        loading.set(false);
                        pwd_sig.set(String::new());
                        on_unlock_handler.call(wallet_id_clone);
                    }
                    Err(e) => {
                        loading.set(false);
                        let error_msg = e.to_string();
                        // 检查是否是新设备需要恢复
                        if error_msg.contains("WALLET_NOT_IN_LOCAL_STORAGE")
                            || error_msg.contains("not found in local storage")
                        {
                            error.set(Some(
                                "钱包未在本地找到。这是新设备，需要恢复钱包才能解锁。请点击'恢复钱包'按钮。".to_string()
                            ));
                        } else {
                            error.set(Some(format!("解锁失败: {}", e)));
                        }
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
                let mut error_message = error_message;
                let on_close = on_close;
                move |_| {
                    password.set(String::new());
                    error_message.set(None);
                    on_close.call(());
                }
            },
            children: rsx! {
                div {
                    class: "p-6",
                    h2 {
                        class: "text-xl font-bold mb-4",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "解锁钱包"
                    }
                    p {
                        class: "text-sm mb-6",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "请输入钱包密码以解锁，用于交易签名"
                    }

                    Input {
                        input_type: InputType::Password,
                        label: Some("钱包密码".to_string()),
                        placeholder: Some("请输入钱包密码".to_string()),
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

                    ErrorMessage {
                        message: error_message.read().clone()
                    }

                    div {
                        class: "flex gap-4 mt-6",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            class: Some("flex-1".to_string()),
                            disabled: is_loading(),
                            loading: is_loading(),
                            onclick: handle_unlock,
                            "解锁"
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Large,
                            class: Some("flex-1".to_string()),
                            disabled: is_loading(),
                            onclick: {
                                let mut password = password;
                                let mut error_message = error_message;
                                let on_close = on_close;
                                move |_| {
                                    password.set(String::new());
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
