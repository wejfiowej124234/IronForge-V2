//! 跨链桥执行页面（非托管模式）
//! 完整的客户端签名流程

use dioxus::prelude::*;
use crate::services::wallet_manager::{WalletManager, TransactionParams};
use crate::components::molecules::wallet_unlock_modal_enhanced::WalletUnlockModal;

#[component]
pub fn BridgeExecutePage() -> Element {
    let mut source_chain = use_signal(|| "ETH".to_string());
    let mut destination_chain = use_signal(|| "BSC".to_string());
    let mut token = use_signal(|| "USDT".to_string());
    let mut amount = use_signal(|| String::new());
    let mut destination_address = use_signal(|| String::new());
    
    let mut show_unlock_modal = use_signal(|| false);
    let mut step = use_signal(|| 1); // 1:输入 2:确认 3:签名 4:完成
    let mut bridge_quote = use_signal(|| None::<BridgeQuote>);
    let mut signed_tx = use_signal(|| None::<String>);
    let mut bridge_id = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    
    let mut wallet_manager = use_context::<Signal<WalletManager>>();
    
    // 获取报价
    let get_quote = move |_| {
        spawn(async move {
            loading.set(true);
            error.set(None);
            
            // 验证输入
            if amount().is_empty() {
                error.set(Some("请输入金额".to_string()));
                loading.set(false);
                return;
            }
            
            // 调用后端获取报价
            match fetch_bridge_quote(
                &source_chain(),
                &destination_chain(),
                &token(),
                &amount(),
            ).await {
                Ok(quote) => {
                    bridge_quote.set(Some(quote));
                    step.set(2);
                }
                Err(e) => {
                    error.set(Some(format!("获取报价失败: {}", e)));
                }
            }
            
            loading.set(false);
        });
    };
    
    // 确认并签名
    let confirm_and_sign = move |_| {
        // 检查钱包是否已解锁
        if !wallet_manager.read().is_unlocked() {
            show_unlock_modal.set(true);
            return;
        }
        
        sign_bridge_transaction();
    };
    
    // 签名跨链交易
    let sign_bridge_transaction = move || {
        spawn(async move {
            loading.set(true);
            step.set(3);
            
            // 1. 构建源链转账交易（发送到跨链桥合约）
            let bridge_contract = get_bridge_contract_address(&source_chain());
            let tx_params = TransactionParams {
                to: bridge_contract,
                value: amount(),
                nonce: 0, // TODO: 从后端获取
                gas_price: 50_000_000_000, // 50 Gwei
                gas_limit: 100_000,
                chain_id: get_chain_id(&source_chain()),
            };
            
            // 2. 客户端签名
            match wallet_manager.write().sign_transaction(&source_chain(), &tx_params) {
                Ok(signed) => {
                    signed_tx.set(Some(signed.clone()));
                    
                    // 3. 发送到后端执行跨链
                    match execute_bridge_with_backend(&source_chain(), &destination_chain(), &signed).await {
                        Ok(bridge_id_str) => {
                            bridge_id.set(Some(bridge_id_str));
                            step.set(4);
                        }
                        Err(e) => {
                            error.set(Some(format!("执行失败: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    error.set(Some(format!("签名失败: {}", e)));
                }
            }
            
            loading.set(false);
        });
    };
    
    rsx! {
        div { class: "bridge-execute-page",
            h2 { "跨链转账" }
            
            // Step 1: 输入信息
            if step() == 1 {
                div { class: "step-content",
                    div { class: "form-group",
                        label { "源链" }
                        select {
                            value: "{source_chain}",
                            onchange: move |e| source_chain.set(e.value()),
                            option { value: "ETH", "Ethereum" }
                            option { value: "BSC", "BSC" }
                            option { value: "POLYGON", "Polygon" }
                        }
                    }
                    
                    div { class: "form-group",
                        label { "目标链" }
                        select {
                            value: "{destination_chain}",
                            onchange: move |e| destination_chain.set(e.value()),
                            option { value: "ETH", "Ethereum" }
                            option { value: "BSC", "BSC" }
                            option { value: "POLYGON", "Polygon" }
                        }
                    }
                    
                    div { class: "form-group",
                        label { "代币" }
                        select {
                            value: "{token}",
                            onchange: move |e| token.set(e.value()),
                            option { value: "USDT", "USDT" }
                            option { value: "USDC", "USDC" }
                            option { value: "DAI", "DAI" }
                        }
                    }
                    
                    div { class: "form-group",
                        label { "金额" }
                        input {
                            r#type: "text",
                            value: "{amount}",
                            oninput: move |e| amount.set(e.value()),
                            placeholder: "100.00",
                        }
                    }
                    
                    div { class: "form-group",
                        label { "接收地址（目标链）" }
                        input {
                            r#type: "text",
                            value: "{destination_address}",
                            oninput: move |e| destination_address.set(e.value()),
                            placeholder: "0x...",
                        }
                    }
                    
                    if let Some(err) = error() {
                        div { class: "alert alert-error", "{err}" }
                    }
                    
                    button {
                        class: "btn btn-primary",
                        onclick: get_quote,
                        disabled: loading(),
                        if loading() { "获取报价中..." } else { "获取报价" }
                    }
                }
            }
            
            // Step 2: 确认信息
            else if step() == 2 {
                div { class: "step-content",
                    h3 { "确认跨链信息" }
                    
                    if let Some(quote) = bridge_quote() {
                        div { class: "quote-info",
                            div { class: "info-row",
                                span { "源链：" }
                                strong { "{source_chain()}" }
                            }
                            div { class: "info-row",
                                span { "目标链：" }
                                strong { "{destination_chain()}" }
                            }
                            div { class: "info-row",
                                span { "金额：" }
                                strong { "{amount()} {token()}" }
                            }
                            div { class: "info-row",
                                span { "预计到账：" }
                                strong { "{quote.estimated_receive_amount} {token()}" }
                            }
                            div { class: "info-row",
                                span { "跨链费用：" }
                                strong { "${quote.bridge_fee_usd:.2}" }
                            }
                            div { class: "info-row",
                                span { "预计时间：" }
                                strong { "{quote.estimated_time_minutes} 分钟" }
                            }
                        }
                        
                        div { class: "security-notice",
                            "🔐 交易将在您的设备上签名，私钥不会上传"
                        }
                        
                        button {
                            class: "btn btn-primary",
                            onclick: confirm_and_sign,
                            "确认并签名"
                        }
                    }
                }
            }
            
            // Step 3: 签名中
            else if step() == 3 {
                div { class: "step-content",
                    div { class: "loading-spinner" }
                    h3 { "正在签名交易..." }
                    p { "请稍候" }
                }
            }
            
            // Step 4: 完成
            else if step() == 4 {
                div { class: "step-content",
                    div { class: "success-message",
                        h3 { "✅ 跨链交易已提交！" }
                        
                        if let Some(id) = bridge_id() {
                            p { "跨链ID：{id}" }
                            p { "预计15-30分钟到账" }
                            
                            button {
                                class: "btn btn-primary",
                                onclick: move |_| {
                                    // 跳转到交易详情页
                                },
                                "查看详情"
                            }
                        }
                    }
                }
            }
            
            // 解锁弹窗
            if show_unlock_modal() {
                WalletUnlockModal {
                    wallet_id: "current".to_string(),
                    on_unlocked: move |_| {
                        show_unlock_modal.set(false);
                        sign_bridge_transaction();
                    },
                    on_cancel: move |_| show_unlock_modal.set(false),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct BridgeQuote {
    estimated_receive_amount: String,
    bridge_fee_usd: f64,
    estimated_time_minutes: u32,
}

/// 获取跨链报价
async fn fetch_bridge_quote(
    source_chain: &str,
    destination_chain: &str,
    token: &str,
    amount: &str,
) -> Result<BridgeQuote, String> {
    let auth_token = get_auth_token().ok_or("Not logged in")?;
    
    let url = format!(
        "/api/v1/bridge/quote?source_chain={}&destination_chain={}&token_symbol={}&amount={}",
        source_chain, destination_chain, token, amount
    );
    
    let response = gloo_net::http::Request::get(&url)
        .header("Authorization", &format!("Bearer {}", auth_token))
        .send()
        .await
        .map_err(|e| format!("Network error: {:?}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    let json: serde_json::Value = response.json()
        .await
        .map_err(|e| format!("Parse error: {:?}", e))?;
    
    let data = json.get("data").ok_or("No data in response")?;
    
    Ok(BridgeQuote {
        estimated_receive_amount: data["estimated_receive_amount"].as_str().unwrap_or("0").to_string(),
        bridge_fee_usd: data["fee_breakdown"]["bridge_fee_usd"].as_f64().unwrap_or(0.0),
        estimated_time_minutes: data["estimated_time_minutes"].as_u64().unwrap_or(15) as u32,
    })
}

/// 执行跨链（发送已签名交易到后端）
async fn execute_bridge_with_backend(
    source_chain: &str,
    destination_chain: &str,
    signed_tx: &str,
) -> Result<String, String> {
    let auth_token = get_auth_token().ok_or("Not logged in")?;
    
    let request_body = serde_json::json!({
        "source_chain": source_chain,
        "destination_chain": destination_chain,
        "signed_source_tx": signed_tx,
        // 其他参数...
    });
    
    let response = gloo_net::http::Request::post("/api/v1/bridge/execute")
        .header("Authorization", &format!("Bearer {}", auth_token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("Failed to build request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {:?}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    let json: serde_json::Value = response.json()
        .await
        .map_err(|e| format!("Parse error: {:?}", e))?;
    
    json.get("data")
        .and_then(|d| d.get("bridge_id"))
        .and_then(|id| id.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No bridge_id in response".to_string())
}

/// 获取跨链桥合约地址
fn get_bridge_contract_address(chain: &str) -> String {
    match chain {
        "ETH" => "0x1234567890123456789012345678901234567890".to_string(),
        "BSC" => "0x2345678901234567890123456789012345678901".to_string(),
        "POLYGON" => "0x3456789012345678901234567890123456789012".to_string(),
        _ => "0x0000000000000000000000000000000000000000".to_string(),
    }
}

fn get_chain_id(chain: &str) -> u64 {
    match chain {
        "ETH" => 1,
        "BSC" => 56,
        "POLYGON" => 137,
        _ => 1,
    }
}

fn get_auth_token() -> Option<String> {
    web_sys::window()?
        .local_storage().ok()??
        .get_item("auth_token").ok()?
}

