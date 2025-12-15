//! 前端安全存储管理器
//! 
//! P4级修复：加固前端安全存储
//! 确保敏感数据在浏览器中的安全性

use gloo_storage::{LocalStorage, SessionStorage, Storage};
use serde::{Deserialize, Serialize};
use web_sys::window;

/// 安全存储管理器
pub struct SecureStorageManager;

impl SecureStorageManager {
    /// 存储敏感数据（使用SessionStorage，浏览器关闭后自动清除）
    ///
    /// # 非托管钱包安全原则
    /// 1. ❌ 私钥、助记词永不明文存储（仅在内存中临时使用，或加密后存储）
    /// 2. ✅ 加密的钱包seed使用LocalStorage（AES-256-GCM加密）
    /// 3. ✅ 公钥可以存储（公开信息，用于查询余额、交易历史）
    /// 4. ✅ 地址、派生路径可以存储（元数据）
    /// 5. 🔐 临时会话数据使用SessionStorage（浏览器关闭后自动清除）
    pub fn store_session_data<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
        SessionStorage::set(key, value).map_err(|e| format!("Failed to store: {}", e))
    }
    
    /// 读取会话数据
    pub fn get_session_data<T: for<'de> Deserialize<'de>>(key: &str) -> Result<T, String> {
        SessionStorage::get(key).map_err(|e| format!("Failed to get: {}", e))
    }
    
    /// 清除会话数据
    pub fn clear_session_data(key: &str) {
        SessionStorage::delete(key);
    }
    
    /// 存储公开数据（使用LocalStorage）
    pub fn store_public_data<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
        LocalStorage::set(key, value).map_err(|e| format!("Failed to store: {}", e))
    }
    
    /// 读取公开数据
    pub fn get_public_data<T: for<'de> Deserialize<'de>>(key: &str) -> Result<T, String> {
        LocalStorage::get(key).map_err(|e| format!("Failed to get: {}", e))
    }
    
    /// 检查是否在隐私模式
    pub fn is_incognito_mode() -> bool {
        // 检测隐私模式的启发式方法
        if let Some(window) = window() {
            if let Ok(storage) = window.local_storage() {
                return storage.is_none();
            }
        }
        false
    }
    
    /// 清除所有敏感数据
    pub fn clear_all_sensitive_data() {
        // 清除SessionStorage
        SessionStorage::clear();
        
        // 清除特定的LocalStorage键
        let sensitive_keys = vec![
            "wallet_unlock_proof",
            "temp_private_key",
            "temp_mnemonic",
        ];
        
        for key in sensitive_keys {
            LocalStorage::delete(key);
        }
        
        tracing::info!("All sensitive data cleared from storage");
    }
    
    /// 安全存储配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StorageConfig {
        pub use_session_storage: bool,
        pub auto_clear_on_close: bool,
        pub max_session_duration: u64, // 秒
    }
    
    impl Default for StorageConfig {
        fn default() -> Self {
            Self {
                use_session_storage: true,
                auto_clear_on_close: true,
                max_session_duration: 900, // 15分钟
            }
        }
    }
}

/// 内存中的临时存储（用于敏感数据）
pub struct MemoryStore<T> {
    data: Option<T>,
}

impl<T> MemoryStore<T> {
    pub fn new() -> Self {
        Self { data: None }
    }
    
    pub fn set(&mut self, value: T) {
        self.data = Some(value);
    }
    
    pub fn get(&self) -> Option<&T> {
        self.data.as_ref()
    }
    
    pub fn clear(&mut self) {
        self.data = None;
    }
}

impl<T> Default for MemoryStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for MemoryStore<T> {
    fn drop(&mut self) {
        // 自动清除
        self.data = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store() {
        let mut store = MemoryStore::new();
        assert!(store.get().is_none());
        
        store.set("test_data".to_string());
        assert_eq!(store.get(), Some(&"test_data".to_string()));
        
        store.clear();
        assert!(store.get().is_none());
    }
}

