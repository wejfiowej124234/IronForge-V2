#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// 业务逻辑模块 - 全部保留
mod blockchain;
mod components;
mod crypto;
mod features;
mod i18n;
mod pages;
mod router;
mod services;
mod shared;

// 业务逻辑导入
use components::molecules::ToastContainer;
use features::wallet::state::WalletState;
#[allow(unused_imports)]
use gloo_storage::Storage;
use shared::state::AppState;

fn main() {
    // Init panic hook for better error messages in console
    console_error_panic_hook::set_once();

    // Init logger
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");
    tracing::info!("IronForge - Starting application");
    launch(App);
}

/// 主应用组件
/// 只包含业务逻辑初始化，不包含UI路由
fn App() -> Element {
    // Initialize global state
    use_context_provider(AppState::new);
    let app_state = use_context::<AppState>();

    // Hydrate API bearer token from UserState on startup
    use_effect(move || {
        let mut api_sig = app_state.api;
        let user_state = app_state.user.read();
        // 从 UserState 恢复 token（如果用户已登录）
        if user_state.is_authenticated {
            if let Some(ref token) = user_state.access_token {
                api_sig.write().set_bearer_token(token.clone());
                tracing::info!("Auth token hydrated from UserState");
            }
        }
    });

    // Auto-Lock Timer - 1小时无操作自动锁定（与JWT token过期时间一致）
    use_effect(move || {
        let mut app_state_clone = app_state;
        spawn(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(30000).await; // Check every 30 seconds
                                                                      // 检查账户自动锁定
                let last_active = *app_state_clone.last_active.read();
                let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
                if now - last_active > 3600 {
                    // 1小时无操作，自动登出（与JWT token过期时间一致）
                    if let Ok(mut user_state) = app_state_clone.user.try_write() {
                        user_state.logout().ok();
                    }
                    app_state_clone.api.write().clear_auth();
                }
            }
        });
    });

    // Activity Listener - 监听用户活动（更新账户活动时间）
    use_effect(move || {
        let app_state_clone = app_state;
        if let Some(window) = web_sys::window() {
            let mut app_state_clone = app_state_clone;
            let on_activity = Closure::wrap(Box::new(move || {
                // 更新活动时间
                let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
                *app_state_clone.last_active.write() = now;
            }) as Box<dyn FnMut()>);

            let _ = window.add_event_listener_with_callback(
                "mousemove",
                on_activity.as_ref().unchecked_ref::<js_sys::Function>(),
            );
            let _ = window.add_event_listener_with_callback(
                "keydown",
                on_activity.as_ref().unchecked_ref::<js_sys::Function>(),
            );
            let _ = window.add_event_listener_with_callback(
                "click",
                on_activity.as_ref().unchecked_ref::<js_sys::Function>(),
            );
            let _ = window.add_event_listener_with_callback(
                "touchstart",
                on_activity.as_ref().unchecked_ref::<js_sys::Function>(),
            );

            on_activity.forget();
        }
    });

    // Network Status Listener - 监听网络状态
    use_effect(move || {
        let mut is_online_signal = app_state.is_online;
        if let Some(window) = web_sys::window() {
            let on_online = Closure::wrap(Box::new(move || {
                *is_online_signal.write() = true;
                tracing::info!("Network is Online");
            }) as Box<dyn FnMut()>);

            let on_offline = Closure::wrap(Box::new(move || {
                *is_online_signal.write() = false;
                tracing::warn!("Network is Offline");
            }) as Box<dyn FnMut()>);

            let _ = window.add_event_listener_with_callback(
                "online",
                on_online.as_ref().unchecked_ref::<js_sys::Function>(),
            );
            let _ = window.add_event_listener_with_callback(
                "offline",
                on_offline.as_ref().unchecked_ref::<js_sys::Function>(),
            );

            on_online.forget();
            on_offline.forget();
        }
    });

    // Async load wallet state (多钱包系统)
    use_future(move || async move {
        let wallet = WalletState::load().await;
        let mut wallet_signal = app_state.wallet;
        *wallet_signal.write() = wallet;
    });

    // 使用路由系统和Toast容器
    rsx! {
        router::AppRouter {}
        ToastContainer {
            messages: app_state.toasts
        }
    }
}
