use crate::components::molecules::toast::{ToastMessage, ToastType};
use crate::crypto::key_manager::KeyManager;
use crate::features::auth::state::UserState;
use crate::features::settings::state::UserPreferences;
use crate::features::wallet::state::WalletState;
use crate::shared::api::{ApiClient, ApiConfig};
use crate::shared::cache::CacheEntry;
use dioxus::prelude::ReadableExt;
use dioxus::prelude::*;
use gloo_storage::Storage;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
pub struct AppState {
    pub user: Signal<UserState>, // 用户认证状态
    pub wallet: Signal<WalletState>,
    #[allow(dead_code)] // 用户偏好设置，用于未来功能
    pub preferences: Signal<UserPreferences>,
    pub api: Signal<ApiClient>,
    pub key_manager: Signal<Option<KeyManager>>,
    pub last_active: Signal<u64>, // Timestamp for auto-lock (账户锁 - 1小时自动登出)
    pub wallet_unlock_time: Signal<HashMap<String, u64>>, // 每个钱包的解锁时间戳（钱包锁 - 15分钟自动锁）
    pub is_online: Signal<bool>,                          // Network status
    pub cache: Signal<HashMap<String, CacheEntry>>,       // Smart Cache: Key -> Value + timestamp
    pub inflight_requests: Signal<HashSet<String>>,       // Request Deduplication
    #[allow(dead_code)] // 隐私模式，用于未来功能
    pub privacy_mode: Signal<bool>, // Hide amounts when blurred
    pub toasts: Signal<Vec<ToastMessage>>,                // Toast消息列表
    pub language: Signal<String>,                         // 当前语言: "zh", "en", "ja", "ko"
}

impl AppState {
    pub fn new() -> Self {
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;

        // Allow overriding API base URL via LocalStorage key `api_base_url`
        let mut api_cfg = ApiConfig::default();
        if let Ok(saved_url) = gloo_storage::LocalStorage::get::<String>("api_base_url") {
            if !saved_url.trim().is_empty() {
                api_cfg.base_url = saved_url;
            }
        }

        Self {
            user: Signal::new(UserState::load()),
            wallet: Signal::new(WalletState::default()),
            preferences: Signal::new(UserPreferences::load()),
            api: Signal::new(ApiClient::new(api_cfg)),
            key_manager: Signal::new(None),
            last_active: Signal::new(now),
            wallet_unlock_time: Signal::new(HashMap::new()), // 钱包锁时间戳
            is_online: Signal::new(true),                    // Assume online initially
            cache: Signal::new(HashMap::new()),
            inflight_requests: Signal::new(HashSet::new()),
            privacy_mode: Signal::new(false),
            toasts: Signal::new(Vec::new()),
            language: Signal::new(
                gloo_storage::LocalStorage::get::<String>("app_language")
                    .unwrap_or_else(|_| "zh".to_string()),
            ),
        }
    }

    /// Get a cloned copy of the ApiClient with the latest auth token from UserState
    /// Dioxus 0.7 compatible: uses Readable trait
    /// This ensures the ApiClient always has the current authentication token
    pub fn get_api_client(&self) -> ApiClient {
        use wasm_bindgen::prelude::*;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = console)]
            fn log(s: &str);
        }

        let mut api_client = (*self.api.read()).clone();
        let user_state = self.user.read();

        #[cfg(debug_assertions)]
        log(&format!(
            "🔍 AppState.get_api_client(): is_authenticated={}",
            user_state.is_authenticated
        ));

        // Always sync the token from UserState to ensure we have the latest token
        if user_state.is_authenticated {
            if let Some(ref token) = user_state.access_token {
                if !token.is_empty() {
                    api_client.set_bearer_token(token.clone());
                    #[cfg(debug_assertions)]
                    {
                        log(&format!("✅ Token synced (length: {})", token.len()));
                        use tracing::debug;
                        debug!(
                            "API Client: Token synced from UserState (length: {})",
                            token.len()
                        );
                    }
                } else {
                    #[cfg(debug_assertions)]
                    {
                        log("⚠️ UserState has EMPTY token");
                        use tracing::warn;
                        warn!("API Client: UserState has empty token, clearing auth");
                    }
                    api_client.clear_auth();
                }
            } else {
                #[cfg(debug_assertions)]
                {
                    log("⚠️ UserState.access_token is None");
                    use tracing::warn;
                    warn!("API Client: UserState.is_authenticated=true but access_token is None");
                }
                api_client.clear_auth();
            }
        } else {
            // Clear auth if user is not authenticated
            #[cfg(debug_assertions)]
            {
                log("❌ User NOT authenticated");
                use tracing::debug;
                debug!("API Client: User not authenticated, clearing auth");
            }
            api_client.clear_auth();
        }

        api_client
    }

    /// Handle 401 Unauthorized error - clear expired token and update user state
    /// This should be called when an API request returns 401
    ///
    /// ## 重构说明
    /// 此方法现在委托给 `AuthManager::clear_auth()`
    /// 建议直接使用 `crate::features::auth::handle_unauthorized(app_state)`
    pub fn handle_unauthorized(self) {
        use crate::features::auth::AuthManager;
        let auth_manager = AuthManager::new(self);
        auth_manager.clear_auth();
    }

    /// 显示Toast消息（辅助函数）
    pub fn show_toast(
        mut toasts: Signal<Vec<ToastMessage>>,
        message: String,
        toast_type: ToastType,
        duration: Option<u32>,
    ) {
        let mut toasts_guard = toasts.write();
        let id = (js_sys::Date::new_0().get_time() as u64) + toasts_guard.len() as u64;
        toasts_guard.push(ToastMessage {
            id,
            message,
            toast_type,
            duration: duration.unwrap_or(3000), // 默认3秒
        });
    }

    /// 显示成功消息
    pub fn show_success(toasts: Signal<Vec<ToastMessage>>, message: String) {
        Self::show_toast(toasts, message, ToastType::Success, None);
    }

    /// 显示错误消息
    pub fn show_error(toasts: Signal<Vec<ToastMessage>>, message: String) {
        Self::show_toast(toasts, message, ToastType::Error, Some(5000)); // 错误消息显示5秒
    }

    /// 显示警告消息
    #[allow(dead_code)] // 警告提示，用于未来功能
    pub fn show_warning(toasts: Signal<Vec<ToastMessage>>, message: String) {
        Self::show_toast(toasts, message, ToastType::Warning, None);
    }

    /// 显示信息消息
    pub fn show_info(toasts: Signal<Vec<ToastMessage>>, message: String) {
        Self::show_toast(toasts, message, ToastType::Info, None);
    }

    /// 移除Toast消息
    #[allow(dead_code)] // 移除提示，用于未来功能
    pub fn remove_toast(mut toasts: Signal<Vec<ToastMessage>>, id: u64) {
        let mut toasts_guard = toasts.write();
        toasts_guard.retain(|t| t.id != id);
    }
}
