//! 发送交易页面（非托管模式）
//! 完整的客户端签名流程

use dioxus::prelude::*;
use crate::services::wallet_manager::{WalletManager, TransactionParams};
use crate::components::molecules::wallet_unlock_modal_enhanced::WalletUnlockModal;

#[component]
pub fn SendTransactionPage(chain: String) -> Element {
    let mut to_address = use_signal(|| String::new());
    let mut amount = use_signal(|| String::new());
    let mut gas_price = use_signal(|| 50u64);
    let mut gas_limit = use_signal(|| 21000u64);
    let mut show_unlock_modal = use_signal(|| false);
    let mut pending_tx = use_signal(|| None::<TransactionParams>);
    let mut error = use_signal(|| None::<String>);
    let mut signing = use_signal(|| false);
    let mut tx_hash = use_signal(|| None::<String>);
    
    let mut wallet_manager = use_context::<Signal<WalletManager>>();
    
    // 准备交易
    let prepare_transaction = move |_| {
        error.set(None);
        
        // 验证输入
        if to_address().is_empty() {
            error.set(Some("请输入接收地址".to_string()));
            return;
        }
        
        if amount().is_empty() {
            error.set(Some("请输入金额".to_string()));
            return;
        }
        
        // 检查钱包是否已解锁
        if !wallet_manager.read().is_unlocked() {
            // 保存交易参数，显示解锁弹窗
            let chain_id = match chain.as_str() {
                "ETH" => 1,
                "BSC" => 56,
                "POLYGON" => 137,
                _ => 1,
            };
            
            pending_tx.set(Some(TransactionParams {
                to: to_address(),
                value: amount(),
                nonce: 0, // TODO: 从后端获取
                gas_price: gas_price(),
                gas_limit: gas_limit(),
                chain_id,
            }));
            
            show_unlock_modal.set(true);
            return;
        }
        
        // 已解锁，直接签名
        sign_and_send_transaction();
    };
    
    // 解锁后的回调
    let on_unlocked = move |_| {
        show_unlock_modal.set(false);
        sign_and_send_transaction();
    };
    
    // 签名并发送交易
    let sign_and_send_transaction = move || {
        spawn(async move {
            signing.set(true);
            error.set(None);
            
            if let Some(tx_params) = pending_tx() {
                // 1. 客户端签名
                match wallet_manager.write().sign_transaction(&chain, &tx_params) {
                    Ok(signed_tx) => {
                        // 2. 发送到后端广播
                        match send_signed_transaction(&chain, &signed_tx).await {
                            Ok(hash) => {
                                tx_hash.set(Some(hash));
                                pending_tx.set(None);
                            }
                            Err(e) => {
                                error.set(Some(format!("广播失败: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("签名失败: {}", e)));
                    }
                }
            }
            
            signing.set(false);
        });
    };
    
    rsx! {
        div { class: "send-transaction-page",
            h2 { "发送 {chain}" }
            
            if let Some(hash) = tx_hash() {
                // 成功显示
                div { class: "success-message",
                    h3 { "✅ 交易已发送！" }
                    p { "交易哈希：" }
                    code { "{hash}" }
                    
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            // 返回钱包首页
                        },
                        "完成"
                    }
                }
            } else {
                // 交易表单
                form {
                    onsubmit: prepare_transaction,
                    
                    div { class: "form-group",
                        label { "接收地址" }
                        input {
                            r#type: "text",
                            value: "{to_address}",
                            oninput: move |e| to_address.set(e.value()),
                            placeholder: "0x...",
                            required: true,
                        }
                    }
                    
                    div { class: "form-group",
                        label { "金额（{chain}）" }
                        input {
                            r#type: "text",
                            value: "{amount}",
                            oninput: move |e| amount.set(e.value()),
                            placeholder: "0.1",
                            required: true,
                        }
                    }
                    
                    div { class: "form-group",
                        label { "Gas Price (Gwei)" }
                        input {
                            r#type: "number",
                            value: "{gas_price}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u64>() {
                                    gas_price.set(val);
                                }
                            },
                        }
                    }
                    
                    div { class: "form-group",
                        label { "Gas Limit" }
                        input {
                            r#type: "number",
                            value: "{gas_limit}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u64>() {
                                    gas_limit.set(val);
                                }
                            },
                        }
                    }
                    
                    if let Some(err) = error() {
                        div { class: "alert alert-error", "{err}" }
                    }
                    
                    div { class: "security-notice",
                        "🔐 交易将在您的设备上签名，私钥不会上传"
                    }
                    
                    button {
                        r#type: "submit",
                        class: "btn btn-primary",
                        disabled: signing(),
                        if signing() { "签名中..." } else { "发送交易" }
                    }
                }
            }
            
            // 解锁弹窗
            if show_unlock_modal() {
                WalletUnlockModal {
                    wallet_id: "current".to_string(),
                    on_unlocked: on_unlocked,
                    on_cancel: move |_| show_unlock_modal.set(false),
                }
            }
        }
    }
}

/// 发送已签名交易到后端
async fn send_signed_transaction(chain: &str, signed_tx: &str) -> Result<String, String> {
    let auth_token = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .and_then(|s| s)
        .and_then(|storage| storage.get_item("auth_token").ok())
        .flatten()
        .ok_or_else(|| "Not logged in".to_string())?;
    
    let request_body = serde_json::json!({
        "chain": chain,
        "from": "0x...", // TODO: 获取当前钱包地址
        "to": "0x...",
        "amount": "0",
        "signed_tx": signed_tx,
    });
    
    let client = gloo_net::http::Request::post("/api/v1/transactions")
        .header("Authorization", &format!("Bearer {}", auth_token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("Failed to build request: {:?}", e))?;
    
    let response = client.send()
        .await
        .map_err(|e| format!("Network error: {:?}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    let json: serde_json::Value = response.json()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))?;
    
    json.get("data")
        .and_then(|d| d.get("tx_hash"))
        .and_then(|h| h.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No tx_hash in response".to_string())
}

