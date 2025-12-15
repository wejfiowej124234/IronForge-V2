//! Import Wallet Page - 导入钱包页面
//! 支持助记词、私钥、Keystore导入，支持4种链恢复

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::atoms::input::{Input, InputType};
use crate::components::molecules::ErrorMessage;
use crate::features::wallet::hooks::use_wallet;
use crate::router::Route;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::events::FormEvent;
use dioxus::prelude::*;

/// 导入方式
#[derive(Clone, Copy, PartialEq)]
enum ImportMethod {
    Mnemonic,
    PrivateKey,
    Keystore,
}

/// Import Wallet Page - 导入钱包页面
#[component]
pub fn ImportWallet() -> Element {
    let import_method = use_signal(|| ImportMethod::Mnemonic);
    let wallet_name = use_signal(|| String::new());
    let password = use_signal(|| String::new());
    let confirm_password = use_signal(|| String::new());

    // 助记词相关
    let mnemonic_phrase = use_signal(|| String::new());

    // 私钥相关
    let private_key = use_signal(|| String::new());

    // Keystore相关
    let keystore_json = use_signal(|| String::new());
    let keystore_password = use_signal(|| String::new());

    // UI状态
    let error_message = use_signal(|| Option::<String>::None);
    let is_loading = use_signal(|| false);

    let wallet_controller = use_wallet();
    let navigator = use_navigator();
    let app_state = use_context::<AppState>();

    // 验证助记词
    let validate_mnemonic = |phrase: &str| -> Result<(), String> {
        let words: Vec<&str> = phrase.trim().split_whitespace().collect();
        if words.len() != 12 && words.len() != 24 {
            return Err("助记词必须是12个或24个单词".to_string());
        }
        // 基本验证：检查是否都是有效的BIP39单词
        // 这里简化处理，实际应该检查BIP39词表
        Ok(())
    };

    // 验证私钥
    let validate_private_key = |key: &str| -> Result<(), String> {
        let trimmed = key.trim();
        // Ethereum私钥：64字符hex（不带0x）
        if trimmed.len() == 64 {
            if hex::decode(trimmed).is_ok() {
                return Ok(());
            }
        }
        // 带0x前缀
        if trimmed.starts_with("0x") && trimmed.len() == 66 {
            if hex::decode(&trimmed[2..]).is_ok() {
                return Ok(());
            }
        }
        Err("无效的私钥格式".to_string())
    };

    // 处理导入
    let handle_import = {
        let wallet_name = wallet_name;
        let password = password;
        let confirm_password = confirm_password;
        let import_method = import_method;
        let wallet_controller = wallet_controller;
        let is_loading = is_loading;
        let error_message = error_message;
        let navigator = navigator.clone();

        move |_| {
            let name = wallet_name.read().trim().to_string();
            let pwd = password.read().clone();
            let confirm_pwd = confirm_password.read().clone();
            let method = *import_method.read();
            let wallet_ctrl = wallet_controller;
            let mut loading = is_loading;
            let mut error = error_message;
            let nav = navigator.clone();
            let toasts = app_state.toasts;

            // 验证输入
            if name.is_empty() {
                error.set(Some("请输入钱包名称".to_string()));
                return;
            }

            if pwd.len() < 8 {
                error.set(Some("密码至少需要8个字符".to_string()));
                return;
            }

            if pwd != confirm_pwd {
                error.set(Some("两次输入的密码不一致".to_string()));
                return;
            }

            loading.set(true);
            error.set(None);

            let mnemonic_phrase = mnemonic_phrase;
            let private_key = private_key;
            let keystore_json = keystore_json;
            let keystore_password = keystore_password;

            spawn(async move {
                let result = match method {
                    ImportMethod::Mnemonic => {
                        let phrase = mnemonic_phrase.read().trim().to_string();
                        if phrase.is_empty() {
                            error.set(Some("请输入助记词".to_string()));
                            loading.set(false);
                            return;
                        }
                        if let Err(e) = validate_mnemonic(&phrase) {
                            error.set(Some(e.clone()));
                            AppState::show_error(toasts, e);
                            loading.set(false);
                            return;
                        }
                        wallet_ctrl.recover_wallet(&name, &phrase, &pwd).await
                    }
                    ImportMethod::PrivateKey => {
                        let key = private_key.read().trim().to_string();
                        if key.is_empty() {
                            error.set(Some("请输入私钥".to_string()));
                            loading.set(false);
                            return;
                        }
                        if let Err(e) = validate_private_key(&key) {
                            error.set(Some(e.clone()));
                            AppState::show_error(toasts, e);
                            loading.set(false);
                            return;
                        }
                        // 实现私钥导入
                        match wallet_ctrl.import_from_private_key(&name, &key, &pwd).await {
                            Ok(_wallet_id) => {
                                loading.set(false);
                                AppState::show_success(toasts, "钱包导入成功".to_string());
                                nav.push(Route::Dashboard {});
                                return;
                            }
                            Err(e) => {
                                loading.set(false);
                                let err_msg = format!("私钥导入失败: {}", e);
                                AppState::show_error(toasts, err_msg.clone());
                                error.set(Some(err_msg));
                                return;
                            }
                        }
                    }
                    ImportMethod::Keystore => {
                        let json = keystore_json.read().trim().to_string();
                        let keystore_pwd = keystore_password.read().clone();
                        if json.is_empty() {
                            error.set(Some("请输入Keystore JSON".to_string()));
                            loading.set(false);
                            return;
                        }
                        if keystore_pwd.is_empty() {
                            error.set(Some("请输入Keystore密码".to_string()));
                            loading.set(false);
                            return;
                        }
                        // 实现Keystore导入
                        match wallet_ctrl
                            .import_from_keystore(&name, &json, &keystore_pwd, &pwd)
                            .await
                        {
                            Ok(_wallet_id) => {
                                loading.set(false);
                                AppState::show_success(toasts, "钱包导入成功".to_string());
                                nav.push(Route::Dashboard {});
                                return;
                            }
                            Err(e) => {
                                loading.set(false);
                                let err_msg = format!("Keystore导入失败: {}", e);
                                AppState::show_error(toasts, err_msg.clone());
                                error.set(Some(err_msg));
                                return;
                            }
                        }
                    }
                };

                match result {
                    Ok(_) => {
                        loading.set(false);
                        AppState::show_success(toasts, "钱包导入成功".to_string());
                        nav.push(Route::Dashboard {});
                    }
                    Err(e) => {
                        loading.set(false);
                        let err_msg = format!("导入失败: {}", e);
                        AppState::show_error(toasts, err_msg.clone());
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
                class: Some("max-w-2xl w-full".to_string()),
                children: rsx! {
                    // 标题
                    h1 {
                        class: "text-2xl font-bold mb-6",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "导入钱包"
                    }

                    // 导入方式选择
                    div {
                        class: "mb-6",
                        label {
                            class: "block text-sm font-medium mb-2",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "导入方式"
                        }
                        div {
                            class: "grid grid-cols-3 gap-2",
                            Button {
                                variant: if *import_method.read() == ImportMethod::Mnemonic {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Secondary
                                },
                                size: ButtonSize::Medium,
                            onclick: {
                                let mut import_method = import_method;
                                move |_| {
                                    import_method.set(ImportMethod::Mnemonic);
                                }
                            },
                                "助记词"
                            }
                            Button {
                                variant: if *import_method.read() == ImportMethod::PrivateKey {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Secondary
                                },
                                size: ButtonSize::Medium,
                            onclick: {
                                let mut import_method = import_method;
                                move |_| {
                                    import_method.set(ImportMethod::PrivateKey);
                                }
                            },
                                "私钥"
                            }
                            Button {
                                variant: if *import_method.read() == ImportMethod::Keystore {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Secondary
                                },
                                size: ButtonSize::Medium,
                            onclick: {
                                let mut import_method = import_method;
                                move |_| {
                                    import_method.set(ImportMethod::Keystore);
                                }
                            },
                                "Keystore"
                            }
                        }
                    }

                    // 钱包名称
                    div {
                        class: "mb-6",
                        Input {
                            input_type: InputType::Text,
                            label: Some("钱包名称".to_string()),
                            placeholder: Some("请输入钱包名称".to_string()),
                            value: Some(wallet_name.read().clone()),
                            onchange: {
                                let mut wallet_name = wallet_name;
                                let mut error_message = error_message;
                                Some(EventHandler::new(move |e: FormEvent| {
                                    wallet_name.set(e.value());
                                    error_message.set(None);
                                }))
                            },
                        }
                    }

                    // 根据导入方式显示不同的输入
                    match *import_method.read() {
                        ImportMethod::Mnemonic => rsx! {
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
                                div {
                                    class: "mt-2 text-xs",
                                    style: format!("color: {};", Colors::TEXT_TERTIARY),
                                    "💡 导入后将自动恢复4种链的地址（ETH, BTC, SOL, TON）"
                                }
                            }
                        },
                        ImportMethod::PrivateKey => rsx! {
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
                                div {
                                    class: "mt-2 p-3 rounded-lg",
                                    style: format!("background: rgba(239, 68, 68, 0.1); border: 1px solid {};", Colors::PAYMENT_ERROR),
                                    p {
                                        class: "text-xs font-semibold mb-1",
                                        style: format!("color: {};", Colors::PAYMENT_ERROR),
                                        "⚠️ 安全警告"
                                    }
                                    p {
                                        class: "text-xs",
                                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                                        "私钥导入仅支持单链钱包，建议使用助记词导入以支持多链"
                                    }
                                }
                            }
                        },
                        ImportMethod::Keystore => rsx! {
                            div {
                                class: "mb-6",
                                Input {
                                    input_type: InputType::Text,
                                    label: Some("Keystore JSON".to_string()),
                                    placeholder: Some("粘贴Keystore JSON内容".to_string()),
                                    value: Some(keystore_json.read().clone()),
                                    onchange: {
                                        let mut keystore_json = keystore_json;
                                        let mut error_message = error_message;
                                        Some(EventHandler::new(move |e: FormEvent| {
                                            keystore_json.set(e.value());
                                            error_message.set(None);
                                        }))
                                    },
                                }
                            }
                            div {
                                class: "mb-6",
                                Input {
                                    input_type: InputType::Password,
                                    label: Some("Keystore密码".to_string()),
                                    placeholder: Some("请输入Keystore密码".to_string()),
                                    value: Some(keystore_password.read().clone()),
                                    onchange: {
                                        let mut keystore_password = keystore_password;
                                        let mut error_message = error_message;
                                        Some(EventHandler::new(move |e: FormEvent| {
                                            keystore_password.set(e.value());
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
                            label: Some("新密码".to_string()),
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
                        class: "flex gap-4",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            class: Some("flex-1".to_string()),
                            disabled: is_loading(),
                            loading: is_loading(),
                            onclick: handle_import,
                            "导入钱包"
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Large,
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
