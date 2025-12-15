use anyhow::Result;
use async_trait::async_trait;
use gloo_storage::{LocalStorage, Storage};

#[async_trait]
#[allow(dead_code)] // 存储适配器接口，用于未来功能
pub trait StorageAdapter {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn remove(&self, key: &str) -> Result<()>;
}

#[allow(dead_code)] // 加密存储，用于未来功能
pub struct EncryptedStorage<S: StorageAdapter> {
    adapter: S,
    key: [u8; 32],
}

#[allow(dead_code)] // 加密存储实现，用于未来功能
impl<S: StorageAdapter> EncryptedStorage<S> {
    #[allow(dead_code)]
    pub fn new(adapter: S, key: [u8; 32]) -> Self {
        Self { adapter, key }
    }

    #[allow(dead_code)]
    pub async fn save(&self, key: &str, data: &[u8]) -> Result<()> {
        let encrypted = crate::crypto::encryption::encrypt(&self.key, data)?;
        self.adapter.set(key, &encrypted).await
    }

    #[allow(dead_code)]
    pub async fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(encrypted) = self.adapter.get(key).await? {
            let decrypted = crate::crypto::encryption::decrypt(&self.key, &encrypted)?;
            Ok(Some(decrypted))
        } else {
            Ok(None)
        }
    }
}

#[allow(dead_code)] // 为未来功能准备
pub struct LocalStorageAdapter;

#[async_trait]
impl StorageAdapter for LocalStorageAdapter {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let val: Result<String, _> = LocalStorage::get(key);
        match val {
            Ok(s) => {
                let bytes =
                    hex::decode(s).map_err(|e| anyhow::anyhow!("Hex decode failed: {}", e))?;
                Ok(Some(bytes))
            }
            Err(_) => Ok(None),
        }
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let s = hex::encode(value);
        LocalStorage::set(key, s).map_err(|e| anyhow::anyhow!("LocalStorage set failed: {}", e))
    }

    async fn remove(&self, key: &str) -> Result<()> {
        LocalStorage::delete(key);
        Ok(())
    }
}
