//! 钱包恢复页面（非托管模式）
//! 通过助记词恢复钱包

use dioxus::prelude::*;
use crate::services::wallet_manager::WalletManager;

#[component]
pub fn WalletRecoverPage() -> Element {
    let mut step = use_signal(|| 1); // 1:输入助记词 2:选择链 3:设置密码 4:完成
    let mut mnemonic_input = use_signal(|| String::new());
    let mut wallet_name = use_signal(|| String::new());
    let mut wallet_password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
    let mut selected_chains = use_signal(|| vec!["ETH".to_string(), "BSC".to_string(), "BTC".to_string()]);
    let mut error = use_signal(|| None::<String>);
    let mut recovering = use_signal(|| false);
    let mut derived_addresses = use_signal(|| None::<std::collections::HashMap<String, String>>);
    
    let mut wallet_manager = use_context::<Signal<WalletManager>>();
    
    // 验证助记词
    let validate_mnemonic = move |_| {
        error.set(None);
        
        let words: Vec<&str> = mnemonic_input().trim().split_whitespace().collect();
        
        // 验证单词数量
        if words.len() != 12 && words.len() != 24 {
            error.set(Some("助记词必须是12个或24个单词".to_string()));
            return;
        }
        
        // 验证助记词有效性（BIP39）
        match bip39::Mnemonic::parse_in(bip39::Language::English, &mnemonic_input()) {
            Ok(_) => {
                step.set(2); // 进入选择链步骤
            }
            Err(e) => {
                error.set(Some(format!("无效的助记词: {:?}", e)));
            }
        }
    };
    
    // 继续到设置密码
    let proceed_to_password = move |_| {
        if selected_chains().is_empty() {
            error.set(Some("请至少选择一条链".to_string()));
            return;
        }
        step.set(3);
    };
    
    // 恢复钱包
    let recover_wallet = move |_| {
        spawn(async move {
            recovering.set(true);
            error.set(None);
            
            // 验证输入
            if wallet_name().trim().is_empty() {
                error.set(Some("请输入钱包名称".to_string()));
                recovering.set(false);
                return;
            }
            
            if wallet_password().len() < 12 {
                error.set(Some("钱包密码至少需要12位".to_string()));
                recovering.set(false);
                return;
            }
            
            if wallet_password() != confirm_password() {
                error.set(Some("两次密码输入不一致".to_string()));
                recovering.set(false);
                return;
            }
            
            // TODO: 实现恢复逻辑
            // 1. 使用助记词派生地址
            // 2. 加密助记词
            // 3. 存储到IndexedDB
            // 4. 发送地址到后端
            
            step.set(4);
            recovering.set(false);
        });
    };
    
    rsx! {
        div { class: "wallet-recover-container",
            h1 { "恢复钱包" }
            
            // 进度条
            div { class: "progress-bar",
                div { class: "progress-step", class: if step() >= 1 { "active" } else { "" }, "1. 输入助记词" }
                div { class: "progress-step", class: if step() >= 2 { "active" } else { "" }, "2. 选择链" }
                div { class: "progress-step", class: if step() >= 3 { "active" } else { "" }, "3. 设置密码" }
                div { class: "progress-step", class: if step() >= 4 { "active" } else { "" }, "4. 完成" }
            }
            
            // Step 1: 输入助记词
            if step() == 1 {
                div { class: "step-content",
                    div { class: "alert alert-info",
                        "⚠️ 请输入您在创建钱包时备份的助记词"
                    }
                    
                    div { class: "form-group",
                        label { "助记词（12或24个单词，用空格分隔）" }
                        textarea {
                            rows: 4,
                            value: "{mnemonic_input}",
                            oninput: move |e| mnemonic_input.set(e.value()),
                            placeholder: "abandon ability able about...",
                            required: true,
                        }
                        small { "提示：输入完整的助记词，单词之间用空格分隔" }
                    }
                    
                    if let Some(err) = error() {
                        div { class: "alert alert-error", "{err}" }
                    }
                    
                    button {
                        class: "btn btn-primary",
                        onclick: validate_mnemonic,
                        "验证助记词"
                    }
                }
            }
            
            // Step 2: 选择链
            else if step() == 2 {
                div { class: "step-content",
                    h3 { "选择要恢复的链" }
                    
                    p { "从您的助记词可以派生多条链的钱包地址" }
                    
                    div { class: "chain-selection",
                        {["ETH", "BSC", "POLYGON", "BTC", "SOL", "TON"].iter().map(|chain| {
                            rsx! {
                                label { class: "checkbox-label",
                                    input {
                                        r#type: "checkbox",
                                        checked: selected_chains().contains(&chain.to_string()),
                                        onchange: move |e| {
                                            let chain_str = chain.to_string();
                                            let mut chains = selected_chains();
                                            if e.checked() {
                                                if !chains.contains(&chain_str) {
                                                    chains.push(chain_str);
                                                }
                                            } else {
                                                chains.retain(|c| c != &chain_str);
                                            }
                                            selected_chains.set(chains);
                                        },
                                    }
                                    span { "{chain}" }
                                }
                            }
                        })}
                    }
                    
                    button {
                        class: "btn btn-primary",
                        onclick: proceed_to_password,
                        "继续"
                    }
                }
            }
            
            // Step 3: 设置密码
            else if step() == 3 {
                div { class: "step-content",
                    h3 { "设置新的钱包密码" }
                    
                    form {
                        onsubmit: recover_wallet,
                        
                        div { class: "form-group",
                            label { "钱包名称" }
                            input {
                                r#type: "text",
                                value: "{wallet_name}",
                                oninput: move |e| wallet_name.set(e.value()),
                                placeholder: "恢复的钱包",
                                required: true,
                            }
                        }
                        
                        div { class: "form-group",
                            label { "新钱包密码" }
                            input {
                                r#type: "password",
                                value: "{wallet_password}",
                                oninput: move |e| wallet_password.set(e.value()),
                                placeholder: "至少12位",
                                required: true,
                                minlength: 12,
                            }
                        }
                        
                        div { class: "form-group",
                            label { "确认密码" }
                            input {
                                r#type: "password",
                                value: "{confirm_password}",
                                oninput: move |e| confirm_password.set(e.value()),
                                required: true,
                            }
                        }
                        
                        if let Some(err) = error() {
                            div { class: "alert alert-error", "{err}" }
                        }
                        
                        button {
                            r#type: "submit",
                            class: "btn btn-primary",
                            disabled: recovering(),
                            if recovering() { "恢复中..." } else { "恢复钱包" }
                        }
                    }
                }
            }
            
            // Step 4: 完成
            else if step() == 4 {
                div { class: "step-content",
                    div { class: "success-message",
                        h2 { "✅ 钱包恢复成功！" }
                        
                        p { "您的钱包已成功恢复并加密存储" }
                        
                        if let Some(addresses) = derived_addresses() {
                            div { class: "wallet-addresses",
                                h3 { "恢复的地址：" }
                                ul {
                                    {addresses.iter().map(|(chain, address)| {
                                        rsx! {
                                            li {
                                                strong { "{chain}: " }
                                                code { "{address}" }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                        
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| {
                                // 跳转到钱包页面
                            },
                            "进入钱包"
                        }
                    }
                }
            }
        }
    }
}

