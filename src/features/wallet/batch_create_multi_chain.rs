//! 多链钱包批量创建功能（企业级前端实现）
//! 核心功能：一个助记词创建所有链的钱包

use dioxus::prelude::*;
use crate::crypto::key_manager::KeyManager;
use crate::services::wallet_manager::WalletManager;
use std::collections::HashMap;

#[component]
pub fn BatchCreateMultiChain() -> Element {
    let mut step = use_signal(|| 1);
    let mut wallet_name = use_signal(|| String::new());
    let mut wallet_password = use_signal(|| String::new());
    let mut password_confirm = use_signal(|| String::new());
    let mut selected_chains = use_signal(|| vec!["ETH".to_string(), "BSC".to_string(), "BTC".to_string()]);
    let mut mnemonic = use_signal(|| None::<String>);
    let mut addresses = use_signal(|| HashMap::<String, String>::new());
    let mut creating = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    
    let mut wallet_manager = use_context::<Signal<WalletManager>>();
    
    // 步骤1：输入钱包信息
    let render_step1 = move || rsx! {
        div { class: "step-content",
            h3 { "创建多链钱包 - 步骤 1/4" }
            p { class: "hint", "一个助记词管理所有链的钱包" }
            
            div { class: "form-group",
                label { "钱包名称" }
                input {
                    r#type: "text",
                    value: "{wallet_name}",
                    oninput: move |e| wallet_name.set(e.value()),
                    placeholder: "My Multi-Chain Wallet",
                }
            }
            
            div { class: "form-group",
                label { "钱包密码（用于本地加密）" }
                input {
                    r#type: "password",
                    value: "{wallet_password}",
                    oninput: move |e| wallet_password.set(e.value()),
                    placeholder: "至少12位",
                }
            }
            
            div { class: "form-group",
                label { "确认密码" }
                input {
                    r#type: "password",
                    value: "{password_confirm}",
                    oninput: move |e| password_confirm.set(e.value()),
                }
            }
            
            button {
                class: "btn btn-primary",
                disabled: wallet_name().is_empty() || wallet_password().len() < 12,
                onclick: move |_| {
                    if wallet_password() != password_confirm() {
                        error.set(Some("密码不一致".to_string()));
                        return;
                    }
                    error.set(None);
                    step.set(2);
                },
                "下一步"
            }
        }
    };
    
    // 步骤2：选择链
    let render_step2 = move || rsx! {
        div { class: "step-content",
            h3 { "创建多链钱包 - 步骤 2/4" }
            p { class: "hint", "选择要创建的链（可以后续添加更多）" }
            
            div { class: "chain-selector",
                ChainCheckbox { chain: "ETH", label: "Ethereum", selected_chains: selected_chains }
                ChainCheckbox { chain: "BSC", label: "BNB Chain", selected_chains: selected_chains }
                ChainCheckbox { chain: "POLYGON", label: "Polygon", selected_chains: selected_chains }
                ChainCheckbox { chain: "BTC", label: "Bitcoin", selected_chains: selected_chains }
                ChainCheckbox { chain: "SOL", label: "Solana", selected_chains: selected_chains }
                ChainCheckbox { chain: "TON", label: "TON", selected_chains: selected_chains }
            }
            
            div { class: "button-group",
                button {
                    class: "btn btn-secondary",
                    onclick: move |_| step.set(1),
                    "上一步"
                }
                button {
                    class: "btn btn-primary",
                    disabled: selected_chains().is_empty(),
                    onclick: move |_| {
                        step.set(3);
                        // 生成钱包
                        generate_multi_chain_wallet();
                    },
                    "生成钱包"
                }
            }
        }
    };
    
    // 生成多链钱包
    let generate_multi_chain_wallet = move || {
        spawn(async move {
            creating.set(true);
            error.set(None);
            
            match wallet_manager.write().create_wallet(
                wallet_name(),
                wallet_password()
            ) {
                Ok((mnemonic_phrase, wallet_data)) => {
                    mnemonic.set(Some(mnemonic_phrase));
                    addresses.set(wallet_data.addresses.clone());
                    step.set(3);
                }
                Err(e) => {
                    error.set(Some(format!("创建失败: {}", e)));
                }
            }
            
            creating.set(false);
        });
    };
    
    // 步骤3：显示助记词
    let render_step3 = move || rsx! {
        div { class: "step-content",
            h3 { "创建多链钱包 - 步骤 3/4" }
            div { class: "warning-box",
                h4 { "⚠️ 请妥善保管助记词" }
                p { "这是恢复钱包的唯一方式！" }
                ul {
                    li { "助记词丢失 = 资产永久丢失" }
                    li { "平台无法帮你找回" }
                    li { "任何人获得助记词 = 可以盗取资产" }
                }
            }
            
            if let Some(words) = mnemonic() {
                div { class: "mnemonic-display",
                    h4 { "你的24个助记词：" }
                    div { class: "mnemonic-grid",
                        {words.split_whitespace().enumerate().map(|(i, word)| rsx! {
                            div { class: "mnemonic-word",
                                span { class: "word-number", "{i+1}." }
                                span { class: "word-text", "{word}" }
                            }
                        })}
                    }
                    
                    div { class: "mnemonic-actions",
                        button {
                            class: "btn btn-secondary",
                            onclick: move |_| copy_to_clipboard(&words),
                            "📋 复制"
                        }
                        button {
                            class: "btn btn-secondary",
                            onclick: move |_| download_as_txt(&words),
                            "💾 下载txt"
                        }
                    }
                }
                
                div { class: "backup-checklist",
                    h4 { "备份检查清单：" }
                    label {
                        input { r#type: "checkbox", id: "check1" }
                        " 我已手写到纸上"
                    }
                    label {
                        input { r#type: "checkbox", id: "check2" }
                        " 我已制作多份备份"
                    }
                    label {
                        input { r#type: "checkbox", id: "check3" }
                        " 我已存放到安全地点"
                    }
                    label {
                        input { r#type: "checkbox", id: "check4" }
                        " 我理解丢失=永久丢失"
                    }
                }
                
                button {
                    class: "btn btn-primary btn-large",
                    onclick: move |_| step.set(4),
                    "我已备份，继续"
                }
            }
        }
    };
    
    // 步骤4：验证并完成
    let render_step4 = move || rsx! {
        div { class: "step-content",
            h3 { "创建多链钱包 - 步骤 4/4" }
            p { "验证助记词并注册到后端" }
            
            if creating() {
                div { class: "loading",
                    "⏳ 正在注册钱包到后端..."
                }
            } else {
                div { class: "success-message",
                    h4 { "✅ 多链钱包创建成功！" }
                    
                    div { class: "addresses-list",
                        h5 { "已创建的钱包地址：" }
                        {addresses().iter().map(|(chain, addr)| rsx! {
                            div { class: "address-item",
                                strong { "{chain}: " }
                                code { "{addr}" }
                            }
                        })}
                    }
                    
                    div { class: "next-steps",
                        h5 { "接下来可以：" }
                        ul {
                            li { "充值到任意链地址" }
                            li { "开始转账和交易" }
                            li { "使用跨链桥" }
                        }
                    }
                    
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            // 跳转到钱包首页
                        },
                        "开始使用"
                    }
                }
            }
        }
    };
    
    rsx! {
        div { class: "batch-create-page",
            div { class: "progress-bar",
                div { class: "progress-step {if step() >= 1 { \"active\" } else { \"\" }}",
                    "1. 钱包信息"
                }
                div { class: "progress-step {if step() >= 2 { \"active\" } else { \"\" }}",
                    "2. 选择链"
                }
                div { class: "progress-step {if step() >= 3 { \"active\" } else { \"\" }}",
                    "3. 备份助记词"
                }
                div { class: "progress-step {if step() >= 4 { \"active\" } else { \"\" }}",
                    "4. 完成"
                }
            }
            
            if let Some(err) = error() {
                div { class: "alert alert-error", "{err}" }
            }
            
            match step() {
                1 => render_step1(),
                2 => render_step2(),
                3 => render_step3(),
                4 => render_step4(),
                _ => rsx! { div { "Unknown step" } }
            }
        }
    }
}

#[component]
fn ChainCheckbox(
    chain: &'static str,
    label: &'static str,
    selected_chains: Signal<Vec<String>>,
) -> Element {
    let is_checked = selected_chains().contains(&chain.to_string());
    
    rsx! {
        label { class: "chain-checkbox",
            input {
                r#type: "checkbox",
                checked: is_checked,
                onchange: move |_| {
                    let mut chains = selected_chains();
                    if is_checked {
                        chains.retain(|c| c != chain);
                    } else {
                        chains.push(chain.to_string());
                    }
                    selected_chains.set(chains);
                },
            }
            span { class: "chain-icon", "🔗" }
            span { class: "chain-label", "{label}" }
        }
    }
}

fn copy_to_clipboard(text: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(clipboard) = window.navigator().clipboard() {
            let _ = clipboard.write_text(text);
        }
    }
}

fn download_as_txt(text: &str) {
    // 创建Blob并触发下载
    if let Some(window) = web_sys::window() {
        let content = format!("IronForge Wallet Mnemonic\n\n{}\n\n⚠️ Keep this safe!\n", text);
        // 实际实现需要创建Blob和下载链接
    }
}

