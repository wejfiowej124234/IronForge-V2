//! 钱包创建页面（非托管模式）
//! 企业级实现：完整的用户引导流程

use dioxus::prelude::*;
use crate::services::wallet_manager::{WalletManager, WalletData};
use crate::features::auth::state::UserState; // ✅ 导入UserState获取access_token

#[component]
pub fn WalletCreatePage() -> Element {
    let mut step = use_signal(|| 1); // 1:输入信息 2:显示助记词 3:验证备份 4:完成
    let mut wallet_name = use_signal(|| String::new());
    let mut wallet_password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
    let mut mnemonic = use_signal(|| String::new());
    let mut wallet_data = use_signal(|| None::<WalletData>);
    let mut verification_words = use_signal(|| Vec::new());
    let mut user_input = use_signal(|| String::new());
    let mut error = use_signal(|| None::<String>);
    let mut creating = use_signal(|| false);
    
    let mut wallet_manager = use_context::<Signal<WalletManager>>();
    let user_state = use_context::<Signal<UserState>>();  // ✅ 在组件顶层获取
    
    // 创建钱包
    let create_wallet = move |_| {
        spawn(async move {
            creating.set(true);
            error.set(None);
            
            // 验证输入
            if wallet_name().trim().is_empty() {
                error.set(Some("请输入钱包名称".to_string()));
                creating.set(false);
                return;
            }
            
            if wallet_password().len() < 12 {
                error.set(Some("钱包密码至少需要12位".to_string()));
                creating.set(false);
                return;
            }
            
            if wallet_password() != confirm_password() {
                error.set(Some("两次密码输入不一致".to_string()));
                creating.set(false);
                return;
            }
            
            // 创建钱包
            match wallet_manager.write().create_wallet(
                wallet_name(),
                wallet_password(),
            ) {
                Ok((mnemonic_phrase, data)) => {
                    mnemonic.set(mnemonic_phrase);
                    wallet_data.set(Some(data));
                    step.set(2); // 进入助记词显示步骤
                }
                Err(e) => {
                    error.set(Some(crate::shared::ui_error::sanitize_user_message(
                        format!("创建失败: {}", e),
                    )));
                }
            }
            
            creating.set(false);
        });
    };
    
    // 进入备份验证
    let start_verification = move |_| {
        // 随机选择3个单词让用户输入验证
        let words: Vec<&str> = mnemonic().split_whitespace().collect();
        let mut indices = vec![3, 8, 15]; // 选择第4、9、16个单词
        verification_words.set(
            indices.iter()
                .map(|&i| (i, words.get(i).unwrap_or(&"").to_string()))
                .collect()
        );
        step.set(3);
    };
    
    // 验证备份
    let verify_backup = move |_| {
        let words: Vec<&str> = mnemonic().split_whitespace().collect();
        let input_words: Vec<&str> = user_input().split_whitespace().collect();
        
        let mut correct = true;
        for (i, (index, _)) in verification_words().iter().enumerate() {
            if input_words.get(i) != Some(&words.get(*index).unwrap_or(&"")) {
                correct = false;
                break;
            }
        }
        
        if correct {
            step.set(4);
            // 发送地址到后端（✅ user_state已在顶层获取）
            spawn(async move {
                if let Some(data) = wallet_data() {
                    register_wallet_with_backend(data, user_state).await;
                }
            });
        } else {
            error.set(Some("验证失败，请重新输入".to_string()));
        }
    };
    
    rsx! {
        div { class: "wallet-create-container",
            // 进度条
            div { class: "progress-bar",
                div { class: "progress-step", class: if step() >= 1 { "active" } else { "" }, "1. 设置密码" }
                div { class: "progress-step", class: if step() >= 2 { "active" } else { "" }, "2. 备份助记词" }
                div { class: "progress-step", class: if step() >= 3 { "active" } else { "" }, "3. 验证备份" }
                div { class: "progress-step", class: if step() >= 4 { "active" } else { "" }, "4. 完成" }
            }
            
            // Step 1: 输入信息
            if step() == 1 {
                div { class: "step-content",
                    h2 { "创建新钱包" }
                    
                    div { class: "info-box non-custodial-info",
                        h4 { "🔒 非托管钱包安全说明" }
                        p { 
                            "您将完全控制您的资产，私钥和助记词仅保存在您的设备上（加密存储）。"
                        }
                        ul {
                            li { "✅ 私钥由您掌握，任何人（包括我们）都无法访问" }
                            li { "✅ 助记词是恢复钱包的唯一方式，请妥善备份" }
                            li { "⚠️ 如果丢失助记词和密码，资产将永久无法找回" }
                        }
                    }
                    
                    form {
                        onsubmit: create_wallet,
                        
                        div { class: "form-group",
                            label { "钱包名称" }
                            input {
                                r#type: "text",
                                value: "{wallet_name}",
                                oninput: move |e| wallet_name.set(e.value()),
                                placeholder: "我的钱包",
                                required: true,
                            }
                        }
                        
                        div { class: "form-group",
                            label { "钱包密码（用于解锁钱包和签名交易）" }
                            input {
                                r#type: "password",
                                value: "{wallet_password}",
                                oninput: move |e| wallet_password.set(e.value()),
                                placeholder: "至少12位，包含大小写字母、数字、特殊字符",
                                required: true,
                                minlength: 12,
                            }
                            small { "⚠️ 钱包密码无法重置，请务必记住" }
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
                            disabled: creating(),
                            if creating() { "创建中..." } else { "创建钱包" }
                        }
                    }
                }
            }
            
            // Step 2: 显示助记词
            else if step() == 2 {
                div { class: "step-content",
                    h2 { "备份助记词" }
                    
                    div { class: "alert alert-danger",
                        "⚠️ 这是恢复钱包的唯一方式！请妥善保管！"
                    }
                    
                    div { class: "security-tips",
                        h3 { "安全提示：" }
                        ul {
                            li { "✍️ 请用纸笔抄录这24个单词" }
                            li { "🔒 存放到安全地方（保险柜）" }
                            li { "❌ 不要截图或拍照" }
                            li { "❌ 不要通过网络传输" }
                            li { "✅ 制作多份备份存放在不同地点" }
                        }
                    }
                    
                    div { class: "mnemonic-words",
                        {mnemonic().split_whitespace().enumerate().map(|(i, word)| {
                            rsx! {
                                div { class: "mnemonic-word",
                                    span { class: "word-index", "{i + 1}." }
                                    span { class: "word-text", "{word}" }
                                }
                            }
                        })}
                    }
                    
                    div { class: "actions",
                        button {
                            class: "btn btn-secondary",
                            onclick: move |_| {
                                // 复制到剪贴板
                                if let Some(window) = web_sys::window() {
                                    if let Some(navigator) = window.navigator().clipboard() {
                                        let _ = navigator.write_text(&mnemonic());
                                    }
                                }
                            },
                            "📋 复制到剪贴板"
                        }
                        
                        button {
                            class: "btn btn-primary",
                            onclick: start_verification,
                            "我已抄录，继续"
                        }
                    }
                }
            }
            
            // Step 3: 验证备份
            else if step() == 3 {
                div { class: "step-content",
                    h2 { "验证备份" }
                    
                    p { "请输入以下单词以验证您已正确备份：" }
                    
                    div { class: "verification-prompts",
                        {verification_words().iter().map(|(index, _)| {
                            rsx! {
                                div { class: "prompt",
                                    "第 {index + 1} 个单词："
                                }
                            }
                        })}
                    }
                    
                    input {
                        r#type: "text",
                        value: "{user_input}",
                        oninput: move |e| user_input.set(e.value()),
                        placeholder: "输入单词，用空格分隔",
                    }
                    
                    if let Some(err) = error() {
                        div { class: "alert alert-error", "{err}" }
                    }
                    
                    button {
                        class: "btn btn-primary",
                        onclick: verify_backup,
                        "验证"
                    }
                }
            }
            
            // Step 4: 完成
            else if step() == 4 {
                div { class: "step-content",
                    div { class: "success-message",
                        h2 { "✅ 钱包创建成功！" }
                        
                        p { "您的多链钱包已创建并绑定到账户。" }
                        
                        if let Some(data) = wallet_data() {
                            div { class: "wallet-info",
                                h3 { "钱包地址：" }
                                ul {
                                    {data.addresses.iter().map(|(chain, address)| {
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
                                // navigator().push("/wallet");
                            },
                            "进入钱包"
                        }
                    }
                }
            }
        }
    }
}

/// 注册钱包地址到后端
async fn register_wallet_with_backend(wallet_data: WalletData, user_state: Signal<UserState>) {
    // 🔍 调试：检查public_keys是否为空
    tracing::info!("WalletData public_keys count: {}", wallet_data.public_keys.len());
    for (chain, pubkey) in &wallet_data.public_keys {
        tracing::info!("Chain: {}, PubKey length: {}", chain, pubkey.len());
    }
    
    // 🔍 调试：检查addresses和public_keys的一致性
    for (chain, _) in &wallet_data.addresses {
        if !wallet_data.public_keys.contains_key(chain) {
            tracing::error!("❌ CRITICAL: Chain {} has address but NO public_key!", chain);
        }
    }
    
    // 构建批量创建请求（✅ V1 API标准：严格匹配后端WalletRegistrationInfo结构）
    use crate::services::wallet::{BatchCreateWalletsRequest, WalletRegistrationInfo, WalletService};
    
    let wallets: Vec<WalletRegistrationInfo> = wallet_data.addresses.iter().filter_map(|(chain, address)| {
        // ✅ 修复：使用filter_map过滤掉缺失或空的公钥
        let pubkey = wallet_data.public_keys.get(chain)?;
        if pubkey.is_empty() {
            tracing::error!("❌ Empty public_key for chain: {}", chain);
            return None;
        }
        
        tracing::info!("✅ Preparing wallet: chain={}, addr={}, pubkey_len={}", 
            chain, address, pubkey.len());
        
        Some(WalletRegistrationInfo {
            chain: chain.clone(),
            address: address.clone(),
            public_key: pubkey.clone(),
            derivation_path: wallet_data.derivation_paths.get(chain).cloned(),
            name: Some(format!("{} - {}", wallet_data.name, chain)),
        })
    }).collect();
    
    // ✅ 验证：确保至少有一个有效钱包
    if wallets.is_empty() {
        tracing::error!("❌ CRITICAL: No valid wallets to register - all chains missing public_keys!");
        return;
    }
    
    tracing::info!("✅ Total valid wallets: {}", wallets.len());
    
    // 使用WalletService批量创建钱包
    let app_state_ctx = app_state.clone();
    let wallet_service = WalletService::new(*app_state_ctx);
    
    let request = BatchCreateWalletsRequest { wallets };
    
    match wallet_service.batch_create_wallets(request).await {
        Ok(response) => {
            tracing::info!("✅ Wallets registered successfully: {} created, {} failed", 
                response.wallets.len(), response.failed.len());
            
            for wallet in &response.wallets {
                tracing::info!("  Created: {} - {}", wallet.chain, wallet.address);
            }
            
            for error in &response.failed {
                tracing::error!("  Failed: {} - {} ({})", error.chain, error.address, error.error);
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to register wallets: {}", e);
        }
    }
}

/// 获取曲线类型
fn get_curve_type(chain: &str) -> &'static str {
    match chain {
        "ETH" | "BSC" | "POLYGON" | "BTC" => "secp256k1",
        "SOL" | "TON" => "ed25519",
        _ => "unknown",
    }
}

