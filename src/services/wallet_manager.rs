//! 非托管钱包管理器（企业级实现）
//! 核心功能：助记词生成、加密存储、钱包解锁、签名管理

use crate::crypto::key_manager::KeyManager;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use bip39::{Language, Mnemonic};
use pbkdf2::pbkdf2_hmac;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use web_sys::{window, Storage};
use zeroize::{Zeroize, ZeroizeOnDrop};

const PBKDF2_ITERATIONS: u32 = 600_000; // OWASP 2023标准
const SESSION_TIMEOUT_MS: u64 = 15 * 60 * 1000; // 15分钟

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// 数据结构
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// 加密的助记词
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMnemonic {
    pub ciphertext: String, // Base64编码
    pub salt: String,       // Base64编码
    pub nonce: String,      // Base64编码（GCM IV）
    pub algorithm: String,  // "AES-256-GCM"
    pub iterations: u32,    // PBKDF2迭代次数
}

/// 钱包数据（存储在IndexedDB）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub id: String,
    pub name: String,
    pub encrypted_mnemonic: EncryptedMnemonic,
    pub addresses: HashMap<String, String>, // chain -> address
    pub public_keys: HashMap<String, String>, // chain -> pubkey
    pub derivation_paths: HashMap<String, String>, // chain -> path
    pub created_at: u64,
    pub version: u32,
}

/// 会话密钥（内存中，自动清零）
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SessionKey {
    #[zeroize(skip)]
    wallet_id: String,
    master_key: Vec<u8>, // 从助记词派生的主密钥
    unlocked_at: u64,
    expires_at: u64,
}

/// 钱包管理器
pub struct WalletManager {
    session_key: Option<SessionKey>,
}

impl WalletManager {
    pub fn new() -> Self {
        Self { session_key: None }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 钱包创建
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// 创建新钱包（24个单词）
    pub fn create_wallet(
        &mut self,
        wallet_name: String,
        wallet_password: String,
    ) -> Result<(String, WalletData)> {
        // 1. 生成24个单词的助记词
        let mnemonic = Mnemonic::generate_in(Language::English, 24)
            .map_err(|e| anyhow!("Failed to generate mnemonic: {}", e))?;

        let mnemonic_phrase = mnemonic.to_string();

        // 2. 派生多链地址
        let (addresses, public_keys, derivation_paths) = self.derive_addresses(&mnemonic)?;

        // 3. 加密助记词
        let encrypted_mnemonic = self.encrypt_mnemonic(&mnemonic_phrase, &wallet_password)?;

        // 4. 创建钱包数据
        let wallet_id = self.generate_wallet_id(&addresses);
        let wallet_data = WalletData {
            id: wallet_id.clone(),
            name: wallet_name,
            encrypted_mnemonic,
            addresses: addresses.clone(),
            public_keys,
            derivation_paths,
            created_at: self.current_timestamp(),
            version: 2,
        };

        // 5. 存储到IndexedDB
        self.save_wallet_to_storage(&wallet_data)?;

        // 6. 派生主密钥并缓存（解锁状态）
        let master_key = self.derive_master_key(&mnemonic_phrase)?;
        self.session_key = Some(SessionKey {
            wallet_id: wallet_id.clone(),
            master_key,
            unlocked_at: self.current_timestamp(),
            expires_at: self.current_timestamp() + SESSION_TIMEOUT_MS,
        });

        Ok((mnemonic_phrase, wallet_data))
    }

    /// 派生多链地址
    fn derive_addresses(
        &self,
        mnemonic: &Mnemonic,
    ) -> Result<(
        HashMap<String, String>,
        HashMap<String, String>,
        HashMap<String, String>,
    )> {
        let seed = mnemonic.to_seed("");
        let key_manager = KeyManager::new(seed.to_vec());

        let mut addresses = HashMap::new();
        let mut public_keys = HashMap::new();
        let mut derivation_paths = HashMap::new();

        // EVM链（ETH, BSC, Polygon）- 使用secp256k1
        let eth_private_key = key_manager.derive_eth_private_key(0)?;
        let eth_address = key_manager.get_eth_address(&eth_private_key)?;
        
        // 从私钥派生公钥（用于后端记录，不涉及签名）
        use k256::ecdsa::SigningKey;
        let key_bytes = hex::decode(&eth_private_key)?;
        let signing_key = SigningKey::from_bytes(key_bytes.as_slice().into())?;
        let verifying_key = k256::ecdsa::VerifyingKey::from(&signing_key);
        let eth_pubkey = hex::encode(verifying_key.to_encoded_point(false).as_bytes());
        
        addresses.insert("ETH".to_string(), eth_address.clone());
        addresses.insert("BSC".to_string(), eth_address.clone());
        addresses.insert("POLYGON".to_string(), eth_address);
        public_keys.insert("ETH".to_string(), eth_pubkey.clone());
        public_keys.insert("BSC".to_string(), eth_pubkey.clone());
        public_keys.insert("POLYGON".to_string(), eth_pubkey);
        derivation_paths.insert("ETH".to_string(), "m/44'/60'/0'/0/0".to_string());
        derivation_paths.insert("BSC".to_string(), "m/44'/60'/0'/0/0".to_string());
        derivation_paths.insert("POLYGON".to_string(), "m/44'/60'/0'/0/0".to_string());

        // Bitcoin
        let btc_private_key = key_manager.derive_btc_private_key(0)?;
        let btc_address = key_manager.get_btc_address(&btc_private_key)?;
        
        let btc_key_bytes = hex::decode(&btc_private_key)?;
        let btc_signing_key = SigningKey::from_bytes(btc_key_bytes.as_slice().into())?;
        let btc_verifying_key = k256::ecdsa::VerifyingKey::from(&btc_signing_key);
        let btc_pubkey = hex::encode(btc_verifying_key.to_encoded_point(true).as_bytes()); // 压缩格式
        
        addresses.insert("BTC".to_string(), btc_address);
        public_keys.insert("BTC".to_string(), btc_pubkey);
        derivation_paths.insert("BTC".to_string(), "m/84'/0'/0'/0/0".to_string());

        // Solana - ✅ 企业级实现：使用真实的 Ed25519 公钥
        let sol_private_key = key_manager.derive_sol_private_key(0)?;
        let sol_address = key_manager.get_sol_address(&sol_private_key)?;
        
        // ✅ 获取真实的 hex 编码公钥（而非地址）
        let sol_pubkey = key_manager.get_sol_public_key(&sol_private_key)?;
        
        addresses.insert("SOL".to_string(), sol_address);
        public_keys.insert("SOL".to_string(), sol_pubkey);
        derivation_paths.insert("SOL".to_string(), "m/44'/501'/0'/0'".to_string());

        // TON - ✅ 企业级实现：使用真实的 Ed25519 公钥
        let ton_private_key = key_manager.derive_ton_private_key(0)?;
        let ton_address = key_manager.get_ton_address(&ton_private_key)?;
        
        // ✅ 获取真实的 hex 编码公钥（而非地址）
        let ton_pubkey = key_manager.get_ton_public_key(&ton_private_key)?;
        
        addresses.insert("TON".to_string(), ton_address);
        public_keys.insert("TON".to_string(), ton_pubkey);
        derivation_paths.insert("TON".to_string(), "m/44'/607'/0'/0'/0'/0'".to_string());

        Ok((addresses, public_keys, derivation_paths))
    }

    /// 加密助记词
    fn encrypt_mnemonic(&self, mnemonic: &str, password: &str) -> Result<EncryptedMnemonic> {
        // 1. 生成随机盐
        let mut salt = [0u8; 32];
        use rand::RngCore;
        OsRng.fill_bytes(&mut salt);

        // 2. 使用PBKDF2派生加密密钥
        let mut encryption_key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            &salt,
            PBKDF2_ITERATIONS,
            &mut encryption_key,
        );

        // 3. 生成随机nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 4. AES-256-GCM加密
        let cipher = Aes256Gcm::new_from_slice(&encryption_key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

        let ciphertext = cipher
            .encrypt(nonce, mnemonic.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // 5. 清零密钥
        encryption_key.zeroize();

        Ok(EncryptedMnemonic {
            ciphertext: BASE64.encode(&ciphertext),
            salt: BASE64.encode(&salt),
            nonce: BASE64.encode(&nonce_bytes),
            algorithm: "AES-256-GCM".to_string(),
            iterations: PBKDF2_ITERATIONS,
        })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 钱包解锁
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// 解锁钱包
    pub fn unlock_wallet(&mut self, wallet_id: String, wallet_password: String) -> Result<()> {
        // 1. 从存储加载钱包
        let wallet_data = self.load_wallet_from_storage(&wallet_id)?;

        // 2. 解密助记词
        let mnemonic = self.decrypt_mnemonic(&wallet_data.encrypted_mnemonic, &wallet_password)?;

        // 3. 派生主密钥
        let master_key = self.derive_master_key(&mnemonic)?;

        // 4. 创建会话密钥
        self.session_key = Some(SessionKey {
            wallet_id,
            master_key,
            unlocked_at: self.current_timestamp(),
            expires_at: self.current_timestamp() + SESSION_TIMEOUT_MS,
        });

        Ok(())
    }

    /// 解密助记词
    fn decrypt_mnemonic(&self, encrypted: &EncryptedMnemonic, password: &str) -> Result<String> {
        // 1. 解码Base64
        let ciphertext = BASE64
            .decode(&encrypted.ciphertext)
            .map_err(|e| anyhow!("Failed to decode ciphertext: {}", e))?;
        let salt = BASE64
            .decode(&encrypted.salt)
            .map_err(|e| anyhow!("Failed to decode salt: {}", e))?;
        let nonce_bytes = BASE64
            .decode(&encrypted.nonce)
            .map_err(|e| anyhow!("Failed to decode nonce: {}", e))?;

        // 2. 使用PBKDF2派生解密密钥
        let mut decryption_key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            &salt,
            encrypted.iterations,
            &mut decryption_key,
        );

        // 3. AES-256-GCM解密
        let cipher = Aes256Gcm::new_from_slice(&decryption_key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| anyhow!("Decryption failed - invalid password"))?;

        // 4. 清零密钥
        decryption_key.zeroize();

        String::from_utf8(plaintext).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    }

    /// 检查钱包是否已解锁
    pub fn is_unlocked(&self) -> bool {
        if let Some(ref session) = self.session_key {
            let now = self.current_timestamp();
            now < session.expires_at
        } else {
            false
        }
    }

    /// 锁定钱包
    pub fn lock_wallet(&mut self) {
        self.session_key = None;
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 签名
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// 签名交易（自动检查会话）
    pub fn sign_transaction(
        &mut self,
        chain: &str,
        tx_params: &TransactionParams,
    ) -> Result<String> {
        // 1. 检查会话
        if !self.is_unlocked() {
            return Err(anyhow!("Wallet is locked"));
        }

        // 2. 刷新会话
        self.refresh_session();

        // 3. 获取私钥
        let private_key = self.derive_private_key_for_chain(chain)?;

        // 4. 签名
        let signed_tx = match chain {
            "ETH" | "BSC" | "POLYGON" => {
                use crate::crypto::tx_signer::EthereumTxSigner;
                EthereumTxSigner::sign_transaction(
                    &private_key,
                    &tx_params.to,
                    &tx_params.value,
                    tx_params.nonce,
                    tx_params.gas_price,
                    tx_params.gas_limit,
                    tx_params.chain_id,
                )?
            }
            _ => return Err(anyhow!("Unsupported chain: {}", chain)),
        };

        Ok(signed_tx)
    }

    /// 派生链的私钥
    fn derive_private_key_for_chain(&self, chain: &str) -> Result<String> {
        let session = self
            .session_key
            .as_ref()
            .ok_or_else(|| anyhow!("Wallet is locked"))?;

        // 从主密钥重建KeyManager
        let key_manager = KeyManager::new(session.master_key.clone());

        match chain {
            "ETH" | "BSC" | "POLYGON" => key_manager.derive_eth_private_key(0),
            "BTC" => key_manager.derive_btc_private_key(0),
            "SOL" => key_manager.derive_sol_private_key(0),
            "TON" => key_manager.derive_ton_private_key(0),
            _ => Err(anyhow!("Unsupported chain: {}", chain)),
        }
    }

    /// 刷新会话（重置过期时间）
    fn refresh_session(&mut self) {
        let new_expires_at = self.current_timestamp() + SESSION_TIMEOUT_MS;
        if let Some(ref mut session) = self.session_key {
            session.expires_at = new_expires_at;
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 存储管理
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// 保存钱包到LocalStorage
    fn save_wallet_to_storage(&self, wallet: &WalletData) -> Result<()> {
        let storage = self.get_local_storage()?;
        let json = serde_json::to_string(wallet)
            .map_err(|e| anyhow!("Failed to serialize wallet: {}", e))?;

        storage
            .set_item(&format!("wallet_{}", wallet.id), &json)
            .map_err(|_| anyhow!("Failed to save wallet to storage"))?;

        Ok(())
    }

    /// 从LocalStorage加载钱包
    fn load_wallet_from_storage(&self, wallet_id: &str) -> Result<WalletData> {
        let storage = self.get_local_storage()?;
        let json = storage
            .get_item(&format!("wallet_{}", wallet_id))
            .map_err(|_| anyhow!("Failed to load wallet from storage"))?
            .ok_or_else(|| anyhow!("Wallet not found: {}", wallet_id))?;

        serde_json::from_str(&json).map_err(|e| anyhow!("Failed to deserialize wallet: {}", e))
    }

    /// 获取LocalStorage
    fn get_local_storage(&self) -> Result<Storage> {
        window()
            .ok_or_else(|| anyhow!("No window object"))?
            .local_storage()
            .map_err(|_| anyhow!("Failed to get localStorage"))?
            .ok_or_else(|| anyhow!("localStorage is not available"))
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 工具方法
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// 派生主密钥
    fn derive_master_key(&self, mnemonic: &str) -> Result<Vec<u8>> {
        let mnemonic_obj = Mnemonic::parse_in(Language::English, mnemonic)
            .map_err(|e| anyhow!("Invalid mnemonic: {}", e))?;
        let seed = mnemonic_obj.to_seed("");
        Ok(seed[..32].to_vec()) // 使用前32字节作为主密钥
    }

    /// 生成钱包ID
    fn generate_wallet_id(&self, addresses: &HashMap<String, String>) -> String {
        use sha2::Digest;
        let mut hasher = Sha256::new();

        // 按链名称排序确保一致性
        let mut sorted: Vec<_> = addresses.iter().collect();
        sorted.sort_by_key(|(chain, _)| *chain);

        for (chain, address) in sorted {
            hasher.update(format!("{}:{}", chain, address).as_bytes());
        }

        let hash = hasher.finalize();
        format!("{:x}", hash)[..16].to_string()
    }

    /// 获取当前时间戳（毫秒）
    fn current_timestamp(&self) -> u64 {
        js_sys::Date::now() as u64
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// 辅助结构
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct TransactionParams {
    pub to: String,
    pub value: String,
    pub nonce: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
    pub chain_id: u64,
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}
