//! Lazy Loader Service - 懒加载服务
//! 提供组件和资源的懒加载功能

use dioxus::prelude::*;
use std::sync::Arc;

/// 懒加载组件包装器
pub struct LazyLoader;

impl LazyLoader {
    /// 懒加载组件（仅在需要时加载）
    pub fn lazy_component<T: 'static + Clone>(
        condition: Signal<bool>,
        loader: impl Fn() -> T + 'static,
    ) -> Signal<Option<T>> {
        let loaded = use_signal(|| Option::<T>::None);

        use_effect({
            let mut loaded_sig = loaded;
            let condition_sig = condition;
            move || {
                if *condition_sig.read() && loaded_sig.read().is_none() {
                    let value = loader();
                    loaded_sig.set(Some(value));
                }
            }
        });

        loaded
    }

    /// 延迟执行函数（防抖）
    pub fn debounce<F: Fn() + 'static>(delay_ms: u32, callback: F) -> impl Fn() {
        let callback = Arc::new(callback);
        move || {
            let callback_clone = callback.clone();
            spawn(async move {
                gloo_timers::future::TimeoutFuture::new(delay_ms).await;
                callback_clone();
            });
        }
    }

    /// 节流函数（限制执行频率）
    pub fn throttle<F: Fn() + 'static>(delay_ms: u32, callback: F) -> impl Fn() {
        let callback = Arc::new(callback);
        let last_run = Arc::new(std::sync::Mutex::new(None));

        move || {
            let now = js_sys::Date::now() as u64;
            let mut last = last_run.lock().unwrap();

            if let Some(last_time) = *last {
                if now - last_time < delay_ms as u64 {
                    return; // 还在节流期内，不执行
                }
            }

            *last = Some(now);
            let callback_clone = callback.clone();
            spawn(async move {
                callback_clone();
            });
        }
    }
}
