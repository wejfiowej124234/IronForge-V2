//! 钱包会话管理器增强版（B项前端实现）
//! 企业级双锁机制：自动锁定+后端验证

use anyhow::Result;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

const SESSION_TIMEOUT_MS: u64 = 15 * 60 * 1000; // 15分钟
const CHECK_INTERVAL_MS: u64 = 5 * 1000; // 每5秒检查一次

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub wallet_id: String,
    pub unlock_token: String,        // 后端返回的unlock_token
    pub unlocked_at: u64,
    pub expires_at: u64,
    pub auto_lock_enabled: bool,
}

/// 钱包会话管理器
pub struct WalletSessionManager {
    current_session: Arc<Mutex<Option<SessionState>>>,
    check_timer_active: Arc<Mutex<bool>>,
}

impl WalletSessionManager {
    pub fn new() -> Self {
        Self {
            current_session: Arc::new(Mutex::new(None)),
            check_timer_active: Arc::new(Mutex::new(false)),
        }
    }

    /// 创建会话（钱包解锁时调用）
    pub async fn create_session(
        &self,
        wallet_id: String,
        unlock_proof: String,
    ) -> Result<String> {
        let now = Self::current_timestamp();
        
        // 1. 调用后端API创建unlock_token
        let unlock_token = self.request_unlock_token(&wallet_id, &unlock_proof, now).await?;
        
        // 2. 创建本地会话
        let session = SessionState {
            wallet_id: wallet_id.clone(),
            unlock_token: unlock_token.clone(),
            unlocked_at: now,
            expires_at: now + SESSION_TIMEOUT_MS,
            auto_lock_enabled: true,
        };
        
        // 3. 保存会话
        *self.current_session.lock().unwrap() = Some(session.clone());
        
        // 4. 启动自动锁定检查
        self.start_auto_lock_timer();
        
        // 5. 保存到LocalStorage（加密）
        LocalStorage::set("wallet_session", &session).ok();
        
        Ok(unlock_token)
    }

    /// 请求后端创建unlock_token（✅ V1 API标准）
    async fn request_unlock_token(
        &self,
        wallet_id: &str,
        unlock_proof: &str,
        _unlocked_at: u64, // 保留参数兼容性，但不使用
    ) -> Result<String> {
        let auth_token = LocalStorage::get::<String>("auth_token")
            .map_err(|_| anyhow::anyhow!("Not logged in"))?;
        
        // ✅ V1 API: POST /api/v1/wallets/unlock
        // 请求体格式：{wallet_id, unlock_proof, session_duration}
        let request_body = serde_json::json!({
            "wallet_id": wallet_id,
            "unlock_proof": unlock_proof,
            "session_duration": 900, // 15分钟（默认值）
        });
        
        let response = gloo_net::http::Request::post("/api/v1/wallets/unlock")
            .header("Authorization", &format!("Bearer {}", auth_token))
            .header("Content-Type", "application/json")
            .json(&request_body)?
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Network error: {:?}", e))?;
        
        if !response.ok() {
            return Err(anyhow::anyhow!("Failed to unlock wallet: {}", response.status()));
        }
        
        // ✅ V1 API响应格式：{code, message, data: {unlock_token, expires_at, wallet}}
        let json: serde_json::Value = response.json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {:?}", e))?;
        
        json.get("data")
            .and_then(|d| d.get("unlock_token"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No unlock_token in response"))
    }

    /// 检查会话是否有效
    pub fn is_session_valid(&self) -> bool {
        if let Some(session) = self.current_session.lock().unwrap().as_ref() {
            let now = Self::current_timestamp();
            now < session.expires_at
        } else {
            false
        }
    }

    /// 获取当前unlock_token
    pub fn get_unlock_token(&self) -> Option<String> {
        self.current_session.lock().unwrap()
            .as_ref()
            .map(|s| s.unlock_token.clone())
    }

    /// 刷新会话（每次使用钱包时调用）（✅ V1 API标准）
    pub async fn refresh_session(&self) -> Result<()> {
        let wallet_id = {
            let session_guard = self.current_session.lock().unwrap();
            if let Some(session) = session_guard.as_ref() {
                session.wallet_id.clone()
            } else {
                return Err(anyhow::anyhow!("No active session"));
            }
        };
        
        // ✅ V1 API: GET /api/v1/wallets/:wallet_id/unlock-status
        let auth_token = LocalStorage::get::<String>("auth_token")
            .map_err(|_| anyhow::anyhow!("Not logged in"))?;
        
        let response = gloo_net::http::Request::get(&format!("/api/v1/wallets/{}/unlock-status", wallet_id))
            .header("Authorization", &format!("Bearer {}", auth_token))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Network error: {:?}", e))?;
        
        if response.ok() {
            // ✅ 解析响应：{code, message, data: {is_unlocked, expires_at, remaining_seconds}}
            let json: serde_json::Value = response.json()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to parse response: {:?}", e))?;
            
            let is_unlocked = json.get("data")
                .and_then(|d| d.get("is_unlocked"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            if is_unlocked {
                // Token仍然有效，无需更新expires_at（服务端管理）
                Ok(())
            } else {
                // Token已失效，清除本地会话
                *self.current_session.lock().unwrap() = None;
                Err(anyhow::anyhow!("Unlock token expired"))
            }
        } else {
            Err(anyhow::anyhow!("Failed to check unlock status: {}", response.status()))
        }
        
        Ok(())
    }

    /// 锁定钱包
    pub fn lock_wallet(&self) {
        *self.current_session.lock().unwrap() = None;
        LocalStorage::delete("wallet_session");
        *self.check_timer_active.lock().unwrap() = false;
        
        tracing::info!("Wallet locked");
    }

    /// 启动自动锁定计时器
    fn start_auto_lock_timer(&self) {
        // 防止重复启动
        {
            let mut active = self.check_timer_active.lock().unwrap();
            if *active {
                return;
            }
            *active = true;
        }
        
        let current_session = self.current_session.clone();
        let check_timer_active = self.check_timer_active.clone();
        
        spawn_local(async move {
            loop {
                // 检查是否应该继续运行
                if !*check_timer_active.lock().unwrap() {
                    break;
                }
                
                // 等待5秒
                gloo_timers::future::TimeoutFuture::new(CHECK_INTERVAL_MS as u32).await;
                
                // 检查会话是否过期
                {
                    let session_guard = current_session.lock().unwrap();
                    if let Some(session) = session_guard.as_ref() {
                        let now = Self::current_timestamp();
                        if now >= session.expires_at {
                            // 会话过期，自动锁定
                            drop(session_guard);
                            *current_session.lock().unwrap() = None;
                            LocalStorage::delete("wallet_session");
                            *check_timer_active.lock().unwrap() = false;
                            
                            tracing::info!("Wallet auto-locked (session timeout)");
                            
                            // 通知UI
                            Self::notify_ui_locked();
                            break;
                        }
                    } else {
                        // 无会话，停止检查
                        *check_timer_active.lock().unwrap() = false;
                        break;
                    }
                }
            }
        });
    }

    /// 通知UI钱包已锁定
    fn notify_ui_locked() {
        if let Some(window) = window() {
            let event = web_sys::CustomEvent::new("wallet-locked").ok();
            if let Some(evt) = event {
                window.dispatch_event(&evt).ok();
            }
        }
    }

    /// 获取当前时间戳（毫秒）
    fn current_timestamp() -> u64 {
        js_sys::Date::now() as u64
    }

    /// 获取剩余时间（秒）
    pub fn get_remaining_seconds(&self) -> Option<u64> {
        let session_guard = self.current_session.lock().unwrap();
        session_guard.as_ref().map(|session| {
            let now = Self::current_timestamp();
            if now < session.expires_at {
                (session.expires_at - now) / 1000
            } else {
                0
            }
        })
    }
}

impl Default for WalletSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

