//! 钱包解锁弹窗（非托管模式）
//! 企业级实现：会话管理+自动锁定

use dioxus::prelude::*;
use crate::services::wallet_manager::WalletManager;

#[component]
pub fn WalletUnlockModal(
    wallet_id: String,
    on_unlocked: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut password = use_signal(|| String::new());
    let mut error = use_signal(|| None::<String>);
    let mut unlocking = use_signal(|| false);
    
    let mut wallet_manager = use_context::<Signal<WalletManager>>();
    
    let unlock = move |_| {
        spawn(async move {
            unlocking.set(true);
            error.set(None);
            
            if password().len() < 12 {
                error.set(Some("密码至少12位".to_string()));
                unlocking.set(false);
                return;
            }
            
            // 解锁钱包
            match wallet_manager.write().unlock_wallet(
                wallet_id.clone(),
                password(),
            ) {
                Ok(()) => {
                    // 清空密码输入
                    password.set(String::new());
                    // 触发回调
                    on_unlocked.call(());
                }
                Err(e) => {
                    error.set(Some(format!("解锁失败: {}", e)));
                }
            }
            
            unlocking.set(false);
        });
    };
    
    rsx! {
        div { class: "modal-overlay",
            div { class: "modal wallet-unlock-modal",
                div { class: "modal-header",
                    h3 { "🔒 解锁钱包" }
                    button {
                        class: "close-btn",
                        onclick: move |_| on_cancel.call(()),
                        "×"
                    }
                }
                
                div { class: "modal-body",
                    div { class: "info-box",
                        p { "需要输入钱包密码以签名交易" }
                        p { class: "small-text", "会话将在15分钟后自动过期" }
                    }
                    
                    div { class: "form-group",
                        label { "钱包密码" }
                        input {
                            r#type: "password",
                            value: "{password}",
                            oninput: move |e| password.set(e.value()),
                            placeholder: "输入钱包密码",
                            autofocus: true,
                            onkeypress: move |e| {
                                if e.key() == "Enter" {
                                    unlock.call(());
                                }
                            },
                        }
                    }
                    
                    if let Some(err) = error() {
                        div { class: "alert alert-error", "{err}" }
                    }
                    
                    div { class: "security-notice",
                        "🔐 密码不会上传到服务器，仅在本地解密助记词"
                    }
                }
                
                div { class: "modal-footer",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| on_cancel.call(()),
                        "取消"
                    }
                    
                    button {
                        class: "btn btn-primary",
                        onclick: unlock,
                        disabled: unlocking(),
                        if unlocking() { "解锁中..." } else { "解锁钱包" }
                    }
                }
            }
        }
    }
}

/// 钱包锁定状态指示器
#[component]
pub fn WalletLockIndicator() -> Element {
    let wallet_manager = use_context::<Signal<WalletManager>>();
    let is_unlocked = wallet_manager.read().is_unlocked();
    
    rsx! {
        div { class: "wallet-lock-indicator",
            if is_unlocked {
                span { class: "status unlocked",
                    "🔓 已解锁"
                }
            } else {
                span { class: "status locked",
                    "🔒 已锁定"
                }
            }
        }
    }
}

/// 自动锁定计时器组件
#[component]
pub fn AutoLockTimer() -> Element {
    let mut remaining_seconds = use_signal(|| 0u64);
    let wallet_manager = use_context::<Signal<WalletManager>>();
    
    use_effect(move || {
        spawn(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(1000).await;
                
                if wallet_manager.read().is_unlocked() {
                    // 计算剩余时间（简化实现）
                    // 实际应该从session_key.expires_at计算
                    remaining_seconds.set(remaining_seconds().saturating_sub(1));
                    
                    if remaining_seconds() == 0 {
                        // 自动锁定
                        wallet_manager.write().lock_wallet();
                    }
                } else {
                    break;
                }
            }
        });
    });
    
    rsx! {
        if wallet_manager.read().is_unlocked() {
            div { class: "auto-lock-timer",
                "🕐 钱包将在 {remaining_seconds() / 60} 分钟后自动锁定"
            }
        }
    }
}

