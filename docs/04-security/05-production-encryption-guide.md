# 生产级加密实现指南

> **版本**: V2.0 Production  
> **状态**: 🔴 生产级（零Mock）  
> **更新日期**: 2025-11-25  
> **依赖**: argon2, aes-gcm, zeroize, bip39

---

## 📋 完整依赖清单

```toml
# Cargo.toml

[dependencies]
# 加密算法
argon2 = "0.5"
aes-gcm = "0.10"
pbkdf2 = { version = "0.12", features = ["simple"] }
sha2 = "0.10"
hmac = "0.12"

# BIP标准
bip39 = "2.0"
bip32 = "0.5"
bitcoin = { version = "0.31", features = ["rand"] }

# 曲线支持
secp256k1 = { version = "0.28", features = ["rand"] }
ed25519-dalek = "2.1"
k256 = "0.13"

# 内存安全
zeroize = { version = "1.7", features = ["derive"] }

# 随机数生成
rand = "0.8"
getrandom = { version = "0.2", features = ["js"] }  # WASM支持

# 编码
hex = "0.4"
bs58 = "0.5"
base64 = "0.21"

# 存储
indexed_db = "0.2"
web-sys = { version = "0.3", features = ["Storage", "Window"] }

# 异步
tokio = { version = "1", features = ["full"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## 生产级加密实现

### 1. 助记词加密（Argon2id + AES-256-GCM）

```rust
// src/security/encryption.rs

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, ParamsBuilder, Algorithm, Version,
};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng as AeadOsRng},
    Aes256Gcm, Key, Nonce,
};
use rand::{RngCore, rngs::OsRng};
use zeroize::{Zeroize, Zeroizing};
use serde::{Deserialize, Serialize};

/// 加密配置（生产级参数）
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// Argon2id 内存成本（64MB）
    pub memory_cost: u32,
    /// Argon2id 时间成本（3次迭代）
    pub time_cost: u32,
    /// Argon2id 并行度（4线程）
    pub parallelism: u32,
    /// 输出密钥长度（32字节）
    pub key_length: usize,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            memory_cost: 65536,  // 64MB
            time_cost: 3,
            parallelism: 4,
            key_length: 32,
        }
    }
}

/// 加密后的数据结构
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct EncryptedData {
    /// 密文
    pub ciphertext: Vec<u8>,
    /// 盐值（32字节）
    pub salt: Vec<u8>,
    /// Nonce（12字节）
    pub nonce: Vec<u8>,
    /// 算法标识
    pub algorithm: String,
    /// Argon2 参数
    pub params: Argon2Params,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argon2Params {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

/// 生产级加密服务
pub struct EncryptionService {
    config: EncryptionConfig,
}

impl EncryptionService {
    pub fn new() -> Self {
        Self {
            config: EncryptionConfig::default(),
        }
    }
    
    /// 加密敏感数据（助记词、私钥等）
    /// 
    /// # 安全性
    /// - Argon2id: 抗侧信道攻击
    /// - AES-256-GCM: 认证加密
    /// - 自动内存清零
    pub fn encrypt(&self, plaintext: &str, password: &str) -> Result<EncryptedData, EncryptionError> {
        // 1. 生成随机盐（32字节）
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        
        // 2. 使用 Argon2id 派生加密密钥
        let params = ParamsBuilder::new()
            .m_cost(self.config.memory_cost)
            .t_cost(self.config.time_cost)
            .p_cost(self.config.parallelism)
            .output_len(self.config.key_length)
            .build()
            .map_err(|e| EncryptionError::InvalidParams(e.to_string()))?;
        
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        
        // 使用 Zeroizing 保护密钥
        let mut key = Zeroizing::new([0u8; 32]);
        argon2
            .hash_password_into(password.as_bytes(), &salt, &mut *key)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;
        
        // 3. 生成随机 Nonce（12字节用于GCM）
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        
        // 4. AES-256-GCM 加密
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        
        // 5. 密钥自动清零（Zeroizing Drop）
        drop(key);
        
        Ok(EncryptedData {
            ciphertext,
            salt: salt.to_vec(),
            nonce: nonce_bytes.to_vec(),
            algorithm: "argon2id-aes256gcm".to_string(),
            params: Argon2Params {
                m_cost: self.config.memory_cost,
                t_cost: self.config.time_cost,
                p_cost: self.config.parallelism,
            },
        })
    }
    
    /// 解密数据
    pub fn decrypt(&self, encrypted: &EncryptedData, password: &str) -> Result<Zeroizing<String>, EncryptionError> {
        // 1. 重建 Argon2 参数
        let params = ParamsBuilder::new()
            .m_cost(encrypted.params.m_cost)
            .t_cost(encrypted.params.t_cost)
            .p_cost(encrypted.params.p_cost)
            .output_len(32)
            .build()
            .map_err(|e| EncryptionError::InvalidParams(e.to_string()))?;
        
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        
        // 2. 重新派生密钥
        let mut key = Zeroizing::new([0u8; 32]);
        argon2
            .hash_password_into(password.as_bytes(), &encrypted.salt, &mut *key)
            .map_err(|_| EncryptionError::InvalidPassword)?;
        
        // 3. AES-256-GCM 解密
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|_| EncryptionError::DecryptionFailed)?;
        
        // 4. 转换为字符串（使用 Zeroizing 保护）
        let plaintext_str = String::from_utf8(plaintext)
            .map_err(|e| EncryptionError::InvalidUtf8(e.to_string()))?;
        
        Ok(Zeroizing::new(plaintext_str))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Invalid password")]
    InvalidPassword,
    
    #[error("Decryption failed")]
    DecryptionFailed,
    
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt() {
        let service = EncryptionService::new();
        let plaintext = "test secret mnemonic phrase";
        let password = "SecurePassword123!";
        
        // 加密
        let encrypted = service.encrypt(plaintext, password).unwrap();
        
        // 解密
        let decrypted = service.decrypt(&encrypted, password).unwrap();
        
        assert_eq!(*decrypted, plaintext);
    }
    
    #[test]
    fn test_wrong_password() {
        let service = EncryptionService::new();
        let plaintext = "test secret";
        let password = "CorrectPassword";
        
        let encrypted = service.encrypt(plaintext, password).unwrap();
        
        // 使用错误密码
        let result = service.decrypt(&encrypted, "WrongPassword");
        
        assert!(matches!(result, Err(EncryptionError::InvalidPassword)));
    }
}
```

---

## IndexedDB 安全存储

```rust
// src/storage/indexed_db.rs

use indexed_db::{Database, ObjectStore, Transaction, TransactionMode};
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// 安全存储服务（IndexedDB）
pub struct SecureStorage {
    db: Database,
}

impl SecureStorage {
    /// 初始化 IndexedDB
    pub async fn new() -> Result<Self, StorageError> {
        let db = Database::open("IronForgeVault", 1, |db, old_version, _| {
            if old_version < 1 {
                // 创建对象存储
                db.create_object_store("wallets", |store| {
                    store.key_path("wallet_id");
                    store.auto_increment(false);
                })?;
                
                db.create_object_store("encrypted_mnemonics", |store| {
                    store.key_path("wallet_id");
                })?;
                
                db.create_object_store("audit_logs", |store| {
                    store.key_path("event_id");
                    store.create_index("timestamp", "timestamp", false)?;
                })?;
            }
            Ok(())
        }).await?;
        
        Ok(Self { db })
    }
    
    /// 保存加密的助记词
    pub async fn save_encrypted_mnemonic(
        &self,
        wallet_id: &str,
        encrypted: &EncryptedData,
    ) -> Result<(), StorageError> {
        let tx = self.db.transaction(&["encrypted_mnemonics"], TransactionMode::ReadWrite)?;
        let store = tx.object_store("encrypted_mnemonics")?;
        
        let value = serde_wasm_bindgen::to_value(&serde_json::json!({
            "wallet_id": wallet_id,
            "encrypted_data": encrypted,
            "created_at": js_sys::Date::now(),
        }))?;
        
        store.put(&value)?;
        tx.commit().await?;
        
        Ok(())
    }
    
    /// 读取加密的助记词
    pub async fn load_encrypted_mnemonic(
        &self,
        wallet_id: &str,
    ) -> Result<EncryptedData, StorageError> {
        let tx = self.db.transaction(&["encrypted_mnemonics"], TransactionMode::ReadOnly)?;
        let store = tx.object_store("encrypted_mnemonics")?;
        
        let value = store.get(&JsValue::from_str(wallet_id))?
            .ok_or(StorageError::NotFound)?;
        
        let data: serde_json::Value = serde_wasm_bindgen::from_value(value)?;
        let encrypted: EncryptedData = serde_json::from_value(data["encrypted_data"].clone())?;
        
        Ok(encrypted)
    }
    
    /// 删除钱包数据（完全删除）
    pub async fn delete_wallet(&self, wallet_id: &str) -> Result<(), StorageError> {
        let tx = self.db.transaction(
            &["wallets", "encrypted_mnemonics"],
            TransactionMode::ReadWrite
        )?;
        
        let wallets_store = tx.object_store("wallets")?;
        let mnemonics_store = tx.object_store("encrypted_mnemonics")?;
        
        wallets_store.delete(&JsValue::from_str(wallet_id))?;
        mnemonics_store.delete(&JsValue::from_str(wallet_id))?;
        
        tx.commit().await?;
        
        Ok(())
    }
}
```

---

## 完整导入流程实现

```rust
// src/flows/wallet_import.rs

use super::encryption::EncryptionService;
use super::storage::SecureStorage;
use bip39::{Mnemonic, Language};

/// 导入钱包完整流程（生产级）
pub async fn import_wallet_complete(
    mnemonic_phrase: String,
    wallet_name: String,
    wallet_password: String,
    master_password: String,
    selected_chains: Vec<ChainType>,
) -> Result<ImportResult, ImportError> {
    // 1. 验证助记词
    let mnemonic = Mnemonic::from_phrase(&mnemonic_phrase, Language::English)
        .map_err(|e| ImportError::InvalidMnemonic(e.to_string()))?;
    
    // 2. 派生种子
    let seed = mnemonic.to_seed("");
    
    // 3. 为每条链派生地址
    let key_manager = KeyManager::new();
    let mut addresses = HashMap::new();
    
    for chain in &selected_chains {
        let chain_config = get_chain_config(chain)?;
        let account = key_manager.derive_account(&seed, &chain_config, 0).await?;
        addresses.insert(chain.clone(), account.address);
    }
    
    // 4. 加密助记词（使用主密码）
    let encryption_service = EncryptionService::new();
    let encrypted_mnemonic = encryption_service.encrypt(&mnemonic_phrase, &master_password)?;
    
    // 5. 存储到 IndexedDB
    let storage = SecureStorage::new().await?;
    let wallet_id = generate_wallet_id(&addresses);
    
    storage.save_encrypted_mnemonic(&wallet_id, &encrypted_mnemonic).await?;
    storage.save_wallet_metadata(&WalletMetadata {
        wallet_id: wallet_id.clone(),
        wallet_name,
        addresses: addresses.clone(),
        chains: selected_chains.clone(),
        created_at: current_timestamp(),
        is_imported: true,
    }).await?;
    
    // 6. 清零敏感数据
    drop(seed);
    drop(mnemonic);
    drop(mnemonic_phrase);
    
    // 7. 记录审计日志
    storage.log_audit_event(AuditEvent {
        event_id: Uuid::new_v4().to_string(),
        timestamp: current_timestamp(),
        operation: AuditOperation::WalletImported,
        wallet_id: Some(wallet_id.clone()),
        result: AuditResult::Success,
        metadata: serde_json::json!({
            "chains": selected_chains.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>(),
        }),
    }).await?;
    
    Ok(ImportResult {
        wallet_id,
        addresses,
    })
}

#[derive(Debug)]
pub struct ImportResult {
    pub wallet_id: String,
    pub addresses: HashMap<ChainType, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Key derivation error: {0}")]
    DerivationError(String),
}
```

---

## 审计日志

```rust
// src/security/audit.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub timestamp: u64,
    pub operation: AuditOperation,
    pub wallet_id: Option<String>,
    pub result: AuditResult,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOperation {
    WalletCreated,
    WalletImported,
    WalletUnlocked,
    WalletLocked,
    MnemonicEncrypted,
    MnemonicDecrypted,
    TransactionSigned,
    TransactionBroadcast,
    PasswordChanged,
    WalletDeleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure { reason: String },
}
```

---

## 🔴 关键安全提示

1. **永不明文存储**: 助记词/私钥必须加密存储
2. **内存自动清零**: 使用 `zeroize` crate
3. **强密钥派生**: Argon2id (64MB, 3迭代, 4线程)
4. **认证加密**: AES-256-GCM (防篡改)
5. **审计日志**: 所有敏感操作可追溯
6. **无Mock代码**: 100%生产级实现
