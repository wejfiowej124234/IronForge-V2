//! Cache Service - 前端缓存服务
//! 提供内存缓存和IndexedDB持久化缓存功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 获取当前 Unix 时间戳（秒）- WebAssembly 兼容
fn now_timestamp() -> u64 {
    js_sys::Date::new_0().get_time() as u64 / 1000
}

/// 缓存项
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheItem<T> {
    value: T,
    expires_at: u64, // Unix timestamp in seconds
}

/// 内存缓存管理器
pub struct MemoryCache {
    data: HashMap<String, String>, // key -> serialized value
    default_ttl: Duration,
}

impl MemoryCache {
    /// 创建新的缓存管理器
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            default_ttl,
        }
    }

    /// 获取缓存值
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        if let Some(serialized) = self.data.get(key) {
            if let Ok(item) = serde_json::from_str::<CacheItem<T>>(serialized) {
                let now = now_timestamp();

                if now < item.expires_at {
                    return Some(item.value);
                } else {
                    // 过期，从内存中移除
                    // 注意：这里不能直接修改，需要返回None
                }
            }
        }
        None
    }

    /// 设置缓存值
    pub fn set<T: Serialize>(&mut self, key: String, value: T, ttl: Option<Duration>) {
        let ttl = ttl.unwrap_or(self.default_ttl);
        // WebAssembly 兼容：使用 js_sys::Date 获取当前时间
        let expires_at = now_timestamp() + ttl.as_secs();

        let item = CacheItem { value, expires_at };
        if let Ok(serialized) = serde_json::to_string(&item) {
            self.data.insert(key, serialized);
        }
    }

    /// 删除缓存
    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// 清理过期项
    pub fn cleanup(&mut self) {
        // WebAssembly 兼容：使用 js_sys::Date 获取当前时间
        let now = now_timestamp();

        self.data.retain(|_, serialized| {
            if let Ok(item) = serde_json::from_str::<CacheItem<serde_json::Value>>(serialized) {
                now < item.expires_at
            } else {
                false
            }
        });
    }

    /// 清除所有以指定前缀开头的缓存键
    pub fn remove_by_prefix(&mut self, prefix: &str) {
        let keys_to_remove: Vec<String> = self
            .data
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        for key in keys_to_remove {
            self.data.remove(&key);
        }
    }
}

/// 缓存键生成器
pub struct CacheKey;

impl CacheKey {
    /// 生成报价缓存键
    pub fn quote(from: &str, to: &str, amount: &str) -> String {
        format!("quote:{}:{}:{}", from, to, amount)
    }

    /// 生成代币余额缓存键
    pub fn balance(chain: &str, address: &str, token: &str) -> String {
        format!("balance:{}:{}:{}", chain, address, token)
    }

    /// 生成订单列表缓存键
    pub fn orders(order_type: &str, status: Option<&str>) -> String {
        if let Some(status) = status {
            format!("orders:{}:{}", order_type, status)
        } else {
            format!("orders:{}", order_type)
        }
    }

    /// 生成交易历史缓存键
    pub fn history(chain: &str, address: &str) -> String {
        format!("history:{}:{}", chain, address)
    }
}
