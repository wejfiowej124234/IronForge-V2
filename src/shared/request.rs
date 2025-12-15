use std::collections::{HashMap, HashSet};
use std::future::Future;

use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::shared::cache::{self, CacheEntry};
use crate::shared::error::ApiError;
use crate::shared::state::AppState;

#[derive(Clone, Copy, Debug)]
pub struct CachePolicy {
    pub ttl_secs: u64,
    pub dedupe_wait_ms: u32,
    pub stale_while_revalidate: bool,
}

impl CachePolicy {
    #[allow(dead_code)]
    pub const fn disabled() -> Self {
        Self {
            ttl_secs: 0,
            dedupe_wait_ms: 0,
            stale_while_revalidate: false,
        }
    }

    pub const fn short() -> Self {
        Self {
            ttl_secs: 10,
            dedupe_wait_ms: 600,
            stale_while_revalidate: true,
        }
    }

    pub const fn medium() -> Self {
        Self {
            ttl_secs: 30,
            dedupe_wait_ms: 800,
            stale_while_revalidate: true,
        }
    }

    pub const fn long() -> Self {
        Self {
            ttl_secs: 120,
            dedupe_wait_ms: 1000,
            stale_while_revalidate: true,
        }
    }

    pub fn is_cache_enabled(&self) -> bool {
        self.ttl_secs > 0
    }
}

#[derive(Clone, Copy)]
pub struct SmartRequestContext {
    cache: Signal<HashMap<String, CacheEntry>>,
    inflight: Signal<HashSet<String>>,
}

impl SmartRequestContext {
    pub fn new(app_state: AppState) -> Self {
        Self {
            cache: app_state.cache,
            inflight: app_state.inflight_requests,
        }
    }

    pub async fn run<T, F, Fut>(
        &self,
        key: &str,
        policy: CachePolicy,
        fetcher: F,
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
        F: FnOnce() -> Fut + 'static,
        Fut: 'static + Future<Output = Result<Value, ApiError>>,
    {
        let now = cache::now_secs();
        let mut stale: Option<CacheEntry> = None;
        let mut fetcher_opt = Some(fetcher);

        if policy.is_cache_enabled() {
            if let Some(entry) = self.cache.read().get(key) {
                if now.saturating_sub(entry.stored_at) <= policy.ttl_secs {
                    return Self::deserialize(entry.value.clone());
                }
                stale = Some(entry.clone());
            }
        }

        if policy.stale_while_revalidate {
            if let Some(entry) = stale.clone() {
                if self.mark_inflight(key) {
                    if let Some(fetcher_fn) = fetcher_opt.take() {
                        self.spawn_revalidation(key.to_string(), policy, fetcher_fn);
                    }
                }
                return Self::deserialize(entry.value);
            }
        }

        // Try to become the in-flight owner
        let mut owns_request = self.mark_inflight(key);

        if !owns_request {
            if let Some(entry) = stale {
                return Self::deserialize(entry.value);
            }

            if policy.dedupe_wait_ms > 0 {
                if let Some(value) = self.wait_for_refresh(key, policy.dedupe_wait_ms).await {
                    return Self::deserialize(value);
                }
            }

            owns_request = self.mark_inflight(key);
        }

        if owns_request {
            let fetcher_fn = fetcher_opt
                .take()
                .expect("fetcher already consumed when executing request");
            let result: Result<Value, ApiError> = fetcher_fn().await;
            self.unmark_inflight(key);

            match result {
                Ok(value) => {
                    if policy.is_cache_enabled() {
                        self.write_cache(key.to_string(), value.clone(), cache::now_secs());
                    }
                    Self::deserialize(value)
                }
                Err(err) => Err(err),
            }
        } else {
            Err(ApiError::RequestFailed(
                "request deduplication timeout".into(),
            ))
        }
    }

    fn spawn_revalidation<F, Fut>(&self, key: String, policy: CachePolicy, fetcher: F)
    where
        F: FnOnce() -> Fut + 'static,
        Fut: 'static + Future<Output = Result<Value, ApiError>>,
    {
        let ctx = *self;
        spawn(async move {
            let result = fetcher().await;
            match result {
                Ok(value) => {
                    if policy.is_cache_enabled() {
                        ctx.write_cache(key.clone(), value.clone(), cache::now_secs());
                    }
                }
                Err(err) => {
                    log::warn!(
                        target: "smart_request",
                        "stale refresh failed for {}: {}",
                        key,
                        err
                    );
                }
            }
            ctx.unmark_inflight(&key);
        });
    }

    fn mark_inflight(&self, key: &str) -> bool {
        let mut inflight = self.inflight;
        let mut guard = inflight.write();
        guard.insert(key.to_string())
    }

    fn unmark_inflight(&self, key: &str) {
        let mut inflight = self.inflight;
        let mut guard = inflight.write();
        guard.remove(key);
    }

    fn write_cache(&self, key: String, value: Value, stored_at: u64) {
        let mut cache = self.cache;
        let mut guard = cache.write();
        guard.insert(key, CacheEntry::new(value, stored_at));
    }

    async fn wait_for_refresh(&self, key: &str, wait_ms: u32) -> Option<Value> {
        let mut waited = 0;
        while waited < wait_ms {
            if !self.inflight.read().contains(key) {
                if let Some(entry) = self.cache.read().get(key) {
                    return Some(entry.value.clone());
                }
                break;
            }
            TimeoutFuture::new(50).await;
            waited += 50;
        }
        None
    }

    fn deserialize<T: DeserializeOwned>(value: Value) -> Result<T, ApiError> {
        // 处理统一响应格式: { code, message, data }
        // 如果响应包含 "data" 字段，则提取 data 字段的内容
        if let Some(data) = value.get("data") {
            // 检查 code 字段，如果 code != 0，则返回错误
            if let Some(code) = value.get("code").and_then(|c| c.as_i64()) {
                if code != 0 {
                    let message = value
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error")
                        .to_string();
                    return Err(ApiError::ResponseError(message));
                }
            }
            // 从 data 字段反序列化
            serde_json::from_value(data.clone()).map_err(|e| {
                ApiError::ResponseError(format!("Failed to deserialize data field: {}", e))
            })
        } else {
            // 如果没有 data 字段，直接反序列化整个响应（向后兼容）
            serde_json::from_value(value).map_err(|e| ApiError::ResponseError(e.to_string()))
        }
    }
}
