use crate::crypto::bip39::generate_mnemonic;
use crate::crypto::encryption::{decrypt, derive_key, encrypt, generate_salt};
use crate::crypto::key_manager::KeyManager;
use crate::crypto::keystore::decrypt_keystore;
use crate::features::wallet::state::{Account, AccountType, Wallet};
use crate::services::wallet::WalletService;
use crate::shared::cache::CacheEntry;
use crate::shared::state::AppState;
use anyhow::{anyhow, Result};
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use uuid::Uuid;

pub fn use_wallet() -> WalletController {
    let app_state = use_context::<AppState>();
    WalletController { app_state }
}

#[derive(Clone, Copy)]
pub struct WalletController {
    app_state: AppState,
}

impl WalletController {
    /// 更新活动时间（账户锁）
    pub fn update_activity(&self) {
        let mut app_state = self.app_state;
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
        *app_state.last_active.write() = now;
    }

    /// 检查账户自动锁定（5分钟无操作）
    #[allow(dead_code)] // 用于自动锁定功能
    pub fn check_auto_lock(&self) {
        let app_state = self.app_state;
        let last_active = *app_state.last_active.read();
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;

        // 5 minutes = 300 seconds
        if now - last_active > 300 {
            // 账户自动锁定 - 仅清除本地状态（不调用后端API）
            let auth_ctrl = crate::features::auth::hooks::use_auth();
            auth_ctrl.logout_local().ok();
        }
    }

    /// 创建新钱包（多钱包系统）
    /// 注意：此函数只生成助记词，不创建钱包
    /// 钱包将在助记词验证通过后创建（调用 finalize_wallet_creation）
    pub async fn create_wallet(&self, name: &str, password: &str) -> Result<String> {
        // 只生成助记词，不创建钱包
        // 钱包将在助记词验证通过后创建

        // Input Sanitization
        let name = name.trim();
        if name.is_empty() {
            return Err(anyhow!("Wallet name cannot be empty"));
        }
        if password.len() < 8 {
            return Err(anyhow!("Password must be at least 8 characters"));
        }

        // 1. Generate Wallet ID
        let wallet_id = Uuid::new_v4().to_string();

        // 2. Generate Mnemonic
        let mnemonic = generate_mnemonic(12)?;
        let phrase = mnemonic.as_str().to_string();

        // 3. Derive Seed
        let seed = mnemonic.to_seed("");

        // 4. Encrypt Seed
        let salt = generate_salt();
        let key = derive_key(password, &salt)?;
        let encrypted_seed = encrypt(&key, &seed)?;

        // 5. Save to Storage (临时保存，等待验证通过后创建钱包)
        // 使用临时key，验证通过后会移动到正式key
        let temp_salt_key = format!("wallet_pending_{}_salt", wallet_id);
        let temp_seed_key = format!("wallet_pending_{}_seed", wallet_id);
        let temp_name_key = format!("wallet_pending_{}_name", wallet_id);
        let temp_password_key = format!("wallet_pending_{}_password", wallet_id);
        let temp_mnemonic_key = format!("wallet_pending_{}_mnemonic", wallet_id);
        LocalStorage::set(&temp_salt_key, hex::encode(salt))?;
        LocalStorage::set(&temp_seed_key, hex::encode(encrypted_seed))?;
        LocalStorage::set(&temp_name_key, name)?;
        LocalStorage::set(&temp_password_key, password)?;
        LocalStorage::set(&temp_mnemonic_key, &phrase)?; // 保存助记词以便后续使用

        // 保存 wallet_id 以便后续使用
        LocalStorage::set("wallet_pending_id", &wallet_id)?;

        // 6. 不创建钱包，只返回助记词
        // 钱包将在助记词验证通过后创建（调用 finalize_wallet_creation）

        Ok(phrase)
    }

    /// 完成钱包创建（在助记词验证通过后调用）
    /// 此函数会从临时存储中读取钱包数据，创建钱包并保存到本地和数据库
    pub async fn finalize_wallet_creation(&self) -> Result<()> {
        let mut app_state = self.app_state;

        // 1. 从临时存储中读取钱包数据
        let wallet_id: String = LocalStorage::get("wallet_pending_id")
            .map_err(|_| anyhow!("未找到待创建的钱包数据，请重新创建钱包"))?;

        let temp_salt_key = format!("wallet_pending_{}_salt", wallet_id);
        let temp_seed_key = format!("wallet_pending_{}_seed", wallet_id);
        let temp_name_key = format!("wallet_pending_{}_name", wallet_id);
        let temp_password_key = format!("wallet_pending_{}_password", wallet_id);
        let temp_mnemonic_key = format!("wallet_pending_{}_mnemonic", wallet_id);

        let salt_hex: String =
            LocalStorage::get(&temp_salt_key).map_err(|_| anyhow!("未找到钱包盐值"))?;
        let encrypted_seed_hex: String =
            LocalStorage::get(&temp_seed_key).map_err(|_| anyhow!("未找到钱包种子"))?;
        let name: String =
            LocalStorage::get(&temp_name_key).map_err(|_| anyhow!("未找到钱包名称"))?;
        let password: String =
            LocalStorage::get(&temp_password_key).map_err(|_| anyhow!("未找到钱包密码"))?;
        let mnemonic_phrase: String =
            LocalStorage::get(&temp_mnemonic_key).map_err(|_| anyhow!("未找到助记词"))?;

        // 2. 解密种子
        let salt = hex::decode(salt_hex)?;
        let encrypted_seed = hex::decode(encrypted_seed_hex)?;
        let key = derive_key(&password, &salt)?;
        let seed = decrypt(&key, &encrypted_seed)?;

        // 3. 创建钱包对象
        let mut wallet = Wallet::new(wallet_id.clone(), name.clone());

        // 4. 创建 KeyManager 并派生账户（✅ 同时提取公钥）
        let key_manager = KeyManager::new(seed.to_vec());

        // Ethereum
        let eth_priv = key_manager.derive_eth_private_key(0)?;
        let eth_addr = key_manager.get_eth_address(&eth_priv)?;
        let eth_pubkey = {
            use k256::ecdsa::{SigningKey, VerifyingKey};
            let signing_key = SigningKey::from_slice(&hex::decode(&eth_priv)?)?;
            let verifying_key = VerifyingKey::from(&signing_key);
            let pub_bytes = verifying_key.to_encoded_point(false).as_bytes().to_vec();
            hex::encode(&pub_bytes) // ✅ 完整的65字节未压缩公钥（包含0x04前缀）
        };
        wallet.accounts.push(Account {
            address: eth_addr,
            chain: "ethereum".to_string(),
            public_key: eth_pubkey,
            derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
            account_type: AccountType::Derived,
            balance: "0".to_string(),
        });

        // Bitcoin
        let btc_priv = key_manager.derive_btc_private_key(0)?;
        let btc_addr = key_manager.get_btc_address(&btc_priv)?;
        let btc_pubkey = {
            use k256::ecdsa::{SigningKey, VerifyingKey};
            let signing_key = SigningKey::from_slice(&hex::decode(&btc_priv)?)?;
            let verifying_key = VerifyingKey::from(&signing_key);
            let pub_bytes = verifying_key.to_encoded_point(true).as_bytes().to_vec();
            hex::encode(&pub_bytes) // 压缩格式公钥
        };
        wallet.accounts.push(Account {
            address: btc_addr,
            chain: "bitcoin".to_string(),
            public_key: btc_pubkey,
            derivation_path: Some("m/84'/0'/0'/0/0".to_string()),
            account_type: AccountType::Derived,
            balance: "0".to_string(),
        });

        // Solana
        let sol_priv = key_manager.derive_sol_private_key(0)?;
        let sol_addr = key_manager.get_sol_address(&sol_priv)?;
        let sol_pubkey = key_manager.get_sol_public_key(&sol_priv)?;
        wallet.accounts.push(Account {
            address: sol_addr,
            chain: "solana".to_string(),
            public_key: sol_pubkey,
            derivation_path: Some("m/44'/501'/0'/0'/0".to_string()),
            account_type: AccountType::Derived,
            balance: "0".to_string(),
        });

        // TON
        let ton_priv = key_manager.derive_ton_private_key(0)?;
        let ton_addr = key_manager.get_ton_address(&ton_priv)?;
        let ton_pubkey = key_manager.get_ton_public_key(&ton_priv)?;
        wallet.accounts.push(Account {
            address: ton_addr,
            chain: "ton".to_string(),
            public_key: ton_pubkey,
            derivation_path: Some("m/44'/607'/0'/0'/0".to_string()),
            account_type: AccountType::Derived,
            balance: "0".to_string(),
        });

        wallet.selected_account_index = Some(0);
        wallet.is_locked = true;

        // 5. 将临时数据移动到正式存储
        let salt_key = format!("wallet_{}_salt", wallet_id);
        let seed_key = format!("wallet_{}_seed", wallet_id);
        LocalStorage::set(&salt_key, hex::encode(salt))?;
        LocalStorage::set(&seed_key, hex::encode(encrypted_seed))?;

        // 6. 清理临时数据
        LocalStorage::delete(&temp_salt_key);
        LocalStorage::delete(&temp_seed_key);
        LocalStorage::delete(&temp_name_key);
        LocalStorage::delete(&temp_password_key);
        LocalStorage::delete(&temp_mnemonic_key);
        LocalStorage::delete("wallet_pending_id");

        // 7. 添加到钱包列表（本地）
        {
            let mut wallet_state = app_state.wallet.write();
            wallet_state.add_wallet(wallet.clone());

            // 如果是第一个钱包，自动选择
            if wallet_state.wallets.len() == 1 {
                wallet_state.selected_wallet_id = Some(wallet_id.clone());
            }

            wallet_state.save()?;
        }

        // 8. 保存到后端数据库
        if app_state.user.read().is_authenticated {
            // 确保 API 客户端有最新的认证 token
            let user_state = app_state.user.read();
            if let Some(ref token) = user_state.access_token {
                app_state.api.write().set_bearer_token(token.clone());
            } else {
                return Err(anyhow!("用户已登录但缺少访问令牌，请重新登录"));
            }
            drop(user_state);

            let wallet_service = WalletService::new(app_state);

            // 使用批量创建API（✅ 直接使用account中已保存的公钥）
            use crate::services::wallet::{BatchCreateWalletsRequest, WalletRegistrationInfo};

            let wallets: Vec<WalletRegistrationInfo> = wallet
                .accounts
                .iter()
                .map(|account| {
                    let chain_str = match account.chain.as_str() {
                        "ethereum" => "ETH",
                        "bitcoin" => "BTC",
                        "solana" => "SOL",
                        "ton" => "TON",
                        _ => account.chain.as_str(),
                    };

                    WalletRegistrationInfo {
                        chain: chain_str.to_uppercase(),
                        address: account.address.clone(),
                        public_key: account.public_key.clone(), // ✅ 直接使用已保存的公钥
                        derivation_path: account.derivation_path.clone(),
                        name: Some(name.to_string()), // ✅ 使用相同的钱包名称（不加链后缀），便于前端合并
                    }
                })
                .collect();

            let batch_request = BatchCreateWalletsRequest { wallets };

            match wallet_service.batch_create_wallets(batch_request).await {
                Ok(response) => {
                    let saved_count = response.wallets.len();
                    let failed_count = response.failed.len();

                    tracing::info!(
                        "✅ Batch wallet creation: {} succeeded, {} failed",
                        saved_count,
                        failed_count
                    );

                    for wallet_result in &response.wallets {
                        tracing::info!(
                            "  ✅ Wallet saved: {} - {}",
                            wallet_result.chain,
                            wallet_result.address
                        );
                    }

                    if !response.failed.is_empty() {
                        // 检查是否是外键约束错误（数据库重建导致）
                        let has_fk_error = response.failed.iter().any(|e| {
                            e.error.contains("foreign key constraint")
                                || e.error.contains("fk_wallets_tenant")
                                || e.error.contains("fk_wallets_user")
                        });

                        if has_fk_error {
                            tracing::error!("🚨 检测到数据库不一致错误（后端数据库可能已重建）");
                            tracing::error!("📝 请执行以下操作：");
                            tracing::error!("   1. 点击右上角【Logout】登出");
                            tracing::error!(
                                "   2. 清除浏览器缓存（F12 → Application → Local Storage → 清除）"
                            );
                            tracing::error!("   3. 重新注册账号");

                            // 自动清理本地存储（可选，取消注释启用）
                            // use gloo_storage::{LocalStorage, Storage};
                            // LocalStorage::delete("user_state");
                            // tracing::warn!("⚠️ 已自动清理本地登录状态，请刷新页面后重新注册");

                            return Err(anyhow::anyhow!(
                                "数据库不一致：后端数据库可能已重建。请登出后重新注册账号。\n\
                                 原因：您的登录凭证对应的用户记录在数据库中不存在。\n\
                                 解决方案：1) 点击Logout 2) 清除浏览器缓存 3) 重新注册"
                            ));
                        }

                        for err in &response.failed {
                            tracing::warn!(
                                "  ⚠️ Failed to save: {} - {} ({})",
                                err.chain,
                                err.address,
                                err.error
                            );
                        }

                        // ✅ 修复：即使部分失败，也不阻止用户继续（钱包已在本地创建）
                        // 用户可以稍后手动同步或重新创建
                        tracing::warn!(
                            "⚠️ 部分钱包保存失败（{}/{} 成功），但本地钱包已创建成功，您可以继续使用",
                            saved_count,
                            wallet.accounts.len()
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("❌ 后端保存失败: {}", e);

                    // 检查是否是401认证错误
                    let error_msg = e.to_string().to_lowercase();
                    if error_msg.contains("unauthorized") || error_msg.contains("401") {
                        tracing::warn!("⚠️ 认证已过期，请重新登录");

                        // 清理认证状态
                        app_state.handle_unauthorized();

                        // 跳转到登录页
                        use crate::router::Route;
                        let nav = use_navigator();
                        nav.push(Route::Login {});

                        return Err(anyhow!("认证已过期，请重新登录后再创建钱包"));
                    } else {
                        // 其他错误：网络错误等，不阻止用户（钱包已在本地创建）
                        tracing::warn!("⚠️ 钱包已在本地创建成功，但未同步到服务器。您可以继续使用，稍后会自动同步");
                        // 不返回错误，允许用户继续
                    }
                }
            }
        } else {
            return Err(anyhow!(
                "请先登录账户后再创建钱包。钱包需要保存到服务器，否则退出登录后钱包将丢失。"
            ));
        }

        // Update activity
        self.update_activity();

        Ok(())
    }

    /// 检查钱包是否在本地存储中（用于检测新设备）
    pub fn is_wallet_in_local_storage(&self, wallet_id: &str) -> bool {
        let seed_key = format!("wallet_{}_seed", wallet_id);
        let priv_key = format!("wallet_{}_private_key", wallet_id);

        // 检查是否有seed或private_key
        LocalStorage::get::<String>(&seed_key).is_ok()
            || LocalStorage::get::<String>(&priv_key).is_ok()
    }

    /// 解锁钱包（用于交易签名）
    #[allow(dead_code)] // 用于钱包解锁功能
    pub async fn unlock_wallet(&self, wallet_id: &str, password: &str) -> Result<()> {
        let mut app_state = self.app_state;

        // 1. 检查钱包是否在本地存储中
        let salt_key = format!("wallet_{}_salt", wallet_id);
        let seed_key = format!("wallet_{}_seed", wallet_id);
        let priv_key = format!("wallet_{}_private_key", wallet_id);

        // 检查是否有seed或private_key
        let has_seed = LocalStorage::get::<String>(&seed_key).is_ok();
        let has_priv = LocalStorage::get::<String>(&priv_key).is_ok();

        if !has_seed && !has_priv {
            return Err(anyhow!(
                "WALLET_NOT_IN_LOCAL_STORAGE: Wallet not found in local storage. \
                This appears to be a new device. Please recover your wallet using your mnemonic phrase or private key."
            ));
        }

        // 2. Load Salt and Encrypted Seed/Private Key
        let (salt_hex, encrypted_data_hex) = if has_seed {
            let salt: String =
                LocalStorage::get(&salt_key).map_err(|_| anyhow!("Failed to load wallet salt"))?;
            let seed: String =
                LocalStorage::get(&seed_key).map_err(|_| anyhow!("Failed to load wallet seed"))?;
            (salt, seed)
        } else {
            // For private key imports, we still need to decrypt and use it
            let salt: String =
                LocalStorage::get(&salt_key).map_err(|_| anyhow!("Failed to load wallet salt"))?;
            let private_key: String = LocalStorage::get(&priv_key)
                .map_err(|_| anyhow!("Failed to load wallet private key"))?;
            (salt, private_key)
        };

        let salt = hex::decode(salt_hex)?;
        let encrypted_data = hex::decode(encrypted_data_hex)?;

        // 3. Derive Key
        let key = derive_key(password, &salt)?;

        // 4. Decrypt Seed or Private Key
        let seed = decrypt(&key, &encrypted_data)
            .map_err(|_| anyhow!("Invalid password or corrupted wallet data"))?;

        // 5. Initialize KeyManager (works for both seed and private key imports)
        let key_manager = KeyManager::new(seed);

        // 5. Update Wallet State (unlock this wallet)
        let mut wallet_state = app_state.wallet.write();
        if let Some(wallet) = wallet_state.get_wallet_mut(wallet_id) {
            wallet.is_locked = false;
            wallet_state.save()?;
        }

        // 6. Set KeyManager in global state (for current wallet)
        *app_state.key_manager.write() = Some(key_manager);

        // 7. Record unlock time (for auto-lock after 5 minutes)
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
        app_state
            .wallet_unlock_time
            .write()
            .insert(wallet_id.to_string(), now);

        self.update_activity();

        Ok(())
    }

    /// 锁定钱包（清除内存中的密钥）
    pub fn lock_wallet(&self, wallet_id: Option<&str>) {
        let mut app_state = self.app_state;

        // 如果指定了钱包ID，锁定该钱包；否则锁定当前选中的钱包
        if let Some(id) = wallet_id {
            let mut wallet_state = app_state.wallet.write();
            if let Some(wallet) = wallet_state.get_wallet_mut(id) {
                wallet.is_locked = true;
                wallet_state.save().ok();
            }
            app_state.wallet_unlock_time.write().remove(id);
        } else {
            // 锁定当前选中的钱包
            let selected_id = {
                let wallet_state = app_state.wallet.read();
                wallet_state.selected_wallet_id.clone()
            };
            if let Some(selected_id) = selected_id {
                let mut wallet_state = app_state.wallet.write();
                if let Some(wallet) = wallet_state.get_wallet_mut(&selected_id) {
                    wallet.is_locked = true;
                    wallet_state.save().ok();
                }
                app_state.wallet_unlock_time.write().remove(&selected_id);
            }
        }

        // 清除KeyManager
        *app_state.key_manager.write() = None;
    }

    /// 检查钱包是否需要解锁（用于交易签名）
    pub fn is_wallet_unlocked(&self, wallet_id: &str) -> bool {
        let app_state = self.app_state;
        let wallet_state = app_state.wallet.read();

        // 检查钱包是否存在且未锁定
        if let Some(wallet) = wallet_state.get_wallet(wallet_id) {
            if wallet.is_locked {
                return false;
            }

            // 检查解锁时间是否过期（5分钟）
            let unlock_time = app_state.wallet_unlock_time.read().get(wallet_id).copied();
            if let Some(timestamp) = unlock_time {
                let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
                if now - timestamp > 300 {
                    // 自动锁定
                    drop(wallet_state);
                    self.lock_wallet(Some(wallet_id));
                    return false;
                }
            }

            return true;
        }

        false
    }

    /// 恢复钱包（导入助记词）
    pub async fn recover_wallet(
        &self,
        name: &str,
        mnemonic_phrase: &str,
        password: &str,
    ) -> Result<String> {
        let mut app_state = self.app_state;

        // Input Sanitization
        let name = name.trim();
        let mnemonic_phrase = mnemonic_phrase.trim().to_lowercase();
        let mnemonic_phrase = mnemonic_phrase
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        if name.is_empty() {
            return Err(anyhow!("Wallet name cannot be empty"));
        }
        if password.len() < 8 {
            return Err(anyhow!("Password must be at least 8 characters"));
        }

        // 1. Generate Wallet ID
        let wallet_id = Uuid::new_v4().to_string();

        // 2. Validate Mnemonic
        use bip39::{Language, Mnemonic};
        let mnemonic = Mnemonic::parse_in(Language::English, &mnemonic_phrase)
            .map_err(|e| anyhow!("Invalid mnemonic phrase: {}", e))?;

        // 3. Derive Seed
        let seed = mnemonic.to_seed("");

        // 4. Encrypt Seed
        let salt = generate_salt();
        let key = derive_key(password, &salt)?;
        let encrypted_seed = encrypt(&key, &seed)?;

        // 5. Save to Storage
        let salt_key = format!("wallet_{}_salt", wallet_id);
        let seed_key = format!("wallet_{}_seed", wallet_id);
        LocalStorage::set(&salt_key, hex::encode(salt))?;
        LocalStorage::set(&seed_key, hex::encode(encrypted_seed))?;

        // 6. Create Wallet Object
        let mut wallet = Wallet::new(wallet_id.clone(), name.to_string());

        // 7. Create KeyManager and derive accounts (✅ 同时提取公钥)
        let key_manager = KeyManager::new(seed.to_vec());

        // Ethereum
        let eth_priv = key_manager.derive_eth_private_key(0)?;
        let eth_addr = key_manager.get_eth_address(&eth_priv)?;
        let eth_pubkey = {
            use k256::ecdsa::{SigningKey, VerifyingKey};
            let signing_key = SigningKey::from_slice(&hex::decode(&eth_priv)?)?;
            let verifying_key = VerifyingKey::from(&signing_key);
            let pub_bytes = verifying_key.to_encoded_point(false).as_bytes().to_vec();
            hex::encode(&pub_bytes) // ✅ 完整的65字节未压缩公钥（包含0x04前缀）
        };
        wallet.accounts.push(Account {
            address: eth_addr,
            chain: "ethereum".to_string(),
            public_key: eth_pubkey,
            derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
            account_type: AccountType::Derived,
            balance: "0".to_string(),
        });

        // Bitcoin
        let btc_priv = key_manager.derive_btc_private_key(0)?;
        let btc_addr = key_manager.get_btc_address(&btc_priv)?;
        let btc_pubkey = {
            use k256::ecdsa::{SigningKey, VerifyingKey};
            let signing_key = SigningKey::from_slice(&hex::decode(&btc_priv)?)?;
            let verifying_key = VerifyingKey::from(&signing_key);
            let pub_bytes = verifying_key.to_encoded_point(true).as_bytes().to_vec();
            hex::encode(&pub_bytes) // 压缩格式公钥
        };
        wallet.accounts.push(Account {
            address: btc_addr,
            chain: "bitcoin".to_string(),
            public_key: btc_pubkey,
            derivation_path: Some("m/84'/0'/0'/0/0".to_string()),
            account_type: AccountType::Derived,
            balance: "0".to_string(),
        });

        // Solana
        let sol_priv = key_manager.derive_sol_private_key(0)?;
        let sol_addr = key_manager.get_sol_address(&sol_priv)?;
        let sol_pubkey = key_manager.get_sol_public_key(&sol_priv)?;
        wallet.accounts.push(Account {
            address: sol_addr,
            chain: "solana".to_string(),
            public_key: sol_pubkey,
            derivation_path: Some("m/44'/501'/0'/0'/0".to_string()),
            account_type: AccountType::Derived,
            balance: "0".to_string(),
        });

        // TON
        let ton_priv = key_manager.derive_ton_private_key(0)?;
        let ton_addr = key_manager.get_ton_address(&ton_priv)?;
        let ton_pubkey = key_manager.get_ton_public_key(&ton_priv)?;
        wallet.accounts.push(Account {
            address: ton_addr,
            chain: "ton".to_string(),
            public_key: ton_pubkey,
            derivation_path: Some("m/44'/607'/0'/0'/0".to_string()),
            account_type: AccountType::Derived,
            balance: "0".to_string(),
        });

        wallet.selected_account_index = Some(0);
        wallet.is_locked = true;

        // 8. Add wallet to wallet list
        let mut wallet_state = app_state.wallet.write();
        wallet_state.add_wallet(wallet);
        wallet_state.save()?;

        self.update_activity();

        Ok(wallet_id)
    }

    /// 从私钥导入钱包（仅支持Ethereum）
    pub async fn import_from_private_key(
        &self,
        name: &str,
        private_key: &str,
        password: &str,
    ) -> Result<String> {
        let mut app_state = self.app_state;

        // Input Sanitization
        let name = name.trim();
        let private_key = private_key.trim().trim_start_matches("0x").to_string();

        if name.is_empty() {
            return Err(anyhow!("Wallet name cannot be empty"));
        }
        if password.len() < 8 {
            return Err(anyhow!("Password must be at least 8 characters"));
        }
        if private_key.is_empty() {
            return Err(anyhow!("Private key cannot be empty"));
        }

        // 验证私钥格式（64个十六进制字符）
        if private_key.len() != 64 {
            return Err(anyhow!(
                "Invalid private key format (must be 64 hex characters)"
            ));
        }

        // 1. Generate Wallet ID
        let wallet_id = Uuid::new_v4().to_string();

        // 2. 从私钥获取地址
        use crate::crypto::key_manager::KeyManager;
        let key_manager = KeyManager::new(vec![]); // 空seed，因为我们只使用私钥
        let eth_address = key_manager.get_eth_address(&private_key)?;

        // 3. 加密私钥（存储私钥而不是seed）
        let salt = generate_salt();
        let key = derive_key(password, &salt)?;
        let encrypted_private_key = encrypt(&key, &hex::decode(&private_key)?)?;

        // 4. Save to Storage
        let salt_key = format!("wallet_{}_salt", wallet_id);
        let priv_key = format!("wallet_{}_private_key", wallet_id);
        LocalStorage::set(&salt_key, hex::encode(salt))?;
        LocalStorage::set(&priv_key, hex::encode(encrypted_private_key))?;

        // 5. Create Wallet Object (✅ 从私钥提取公钥)
        let private_key_bytes = hex::decode(&private_key)?;
        let public_key = {
            use k256::ecdsa::{SigningKey, VerifyingKey};
            let signing_key = SigningKey::from_slice(&private_key_bytes)?;
            let verifying_key = VerifyingKey::from(&signing_key);
            let pub_bytes = verifying_key.to_encoded_point(false).as_bytes().to_vec();
            hex::encode(&pub_bytes) // ✅ 完整的65字节未压缩公钥（包含0x04前缀）
        };

        let mut wallet = Wallet::new(wallet_id.clone(), name.to_string());
        wallet.accounts.push(Account {
            address: eth_address,
            chain: "ethereum".to_string(),
            public_key,
            derivation_path: None, // 导入的私钥没有派生路径
            account_type: AccountType::Imported,
            balance: "0".to_string(),
        });

        wallet.selected_account_index = Some(0);
        wallet.is_locked = true;

        // 6. Add wallet to wallet list
        let mut wallet_state = app_state.wallet.write();
        wallet_state.add_wallet(wallet);
        wallet_state.save()?;

        self.update_activity();

        Ok(wallet_id)
    }

    /// 从Keystore导入钱包
    pub async fn import_from_keystore(
        &self,
        name: &str,
        keystore_json: &str,
        keystore_password: &str,
        wallet_password: &str,
    ) -> Result<String> {
        // Input Sanitization
        let name = name.trim();

        if name.is_empty() {
            return Err(anyhow!("Wallet name cannot be empty"));
        }
        if wallet_password.len() < 8 {
            return Err(anyhow!("Wallet password must be at least 8 characters"));
        }
        if keystore_json.is_empty() {
            return Err(anyhow!("Keystore JSON cannot be empty"));
        }
        if keystore_password.is_empty() {
            return Err(anyhow!("Keystore password cannot be empty"));
        }

        // 1. Parse Keystore JSON
        let keystore: serde_json::Value = serde_json::from_str(keystore_json)
            .map_err(|e| anyhow!("Invalid Keystore JSON: {}", e))?;

        // 2. 验证Keystore格式
        let _crypto = keystore
            .get("crypto")
            .ok_or_else(|| anyhow!("Missing 'crypto' field in Keystore JSON"))?;

        // 3. 检查Keystore版本
        let version = keystore
            .get("version")
            .and_then(|v| v.as_u64())
            .unwrap_or(3);

        if version != 3 {
            return Err(anyhow!(
                "Unsupported Keystore version: {}. Only version 3 is supported.",
                version
            ));
        }

        // 4. 解密Keystore获取私钥
        let private_key_hex = decrypt_keystore(keystore_json, keystore_password)
            .map_err(|e| anyhow!("Failed to decrypt keystore: {}", e))?;

        // 5. 使用私钥导入逻辑（复用现有代码）
        self.import_from_private_key(name, &private_key_hex, wallet_password)
            .await

        // ⚠️ Keystore导入功能说明
        //
        // 当前状态：基础框架已实现，但完整解密逻辑需要外部库支持
        //
        // 完整实现需要集成Keystore解析库（推荐使用 eth-keystore-rs 或类似库）：
        //
        // 实现步骤：
        // 1. 解析JSON结构：version, id, address, crypto (cipher, cipherparams, kdf, kdfparams, mac)
        // 2. 根据kdf类型（scrypt/pbkdf2）派生密钥：
        //    - scrypt: 使用 n, r, p, salt 参数
        //    - pbkdf2: 使用 c, dklen, prf, salt 参数
        // 3. 使用派生密钥和cipherparams解密私钥：
        //    - AES-128-CTR: 使用 iv 和派生密钥
        //    - AES-128-CBC: 使用 iv 和派生密钥
        // 4. 验证MAC（使用HMAC-SHA3-256或HMAC-SHA256）
        // 5. 从解密后的私钥恢复钱包（使用现有的私钥导入逻辑）
        //
        // 依赖建议：
        // - eth-keystore-rs: 完整的Keystore解析和加密/解密
        // - scrypt: scrypt密钥派生
        // - aes-gcm 或 aes: AES加密/解密
        // - hmac: MAC验证
        //
        // 当前实现：返回明确的错误提示，引导用户使用其他导入方式
    }

    /// 获取余额（兼容旧代码）
    #[allow(dead_code)] // 用于余额查询功能
    pub async fn get_balance(&self) -> Result<String> {
        let mut app_state = self.app_state;
        let wallet_state = app_state.wallet.read();

        if let Some(wallet) = wallet_state.get_selected_wallet() {
            if let Some(idx) = wallet.selected_account_index {
                if let Some(account) = wallet.accounts.get(idx) {
                    let cache_key = format!("{}:{}", account.chain, account.address);
                    let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;

                    // 1. Check Cache
                    if let Some(entry) = app_state.cache.read().get(&cache_key) {
                        if now - entry.stored_at < 30 {
                            if let Some(val) = entry.as_str() {
                                return Ok(val.to_string());
                            }
                        }
                    }

                    // 2. Request Deduplication
                    if app_state.inflight_requests.read().contains(&cache_key) {
                        if let Some(entry) = app_state.cache.read().get(&cache_key) {
                            if let Some(val) = entry.as_str() {
                                return Ok(val.to_string());
                            }
                        }
                        return Err(anyhow!("Request already in progress"));
                    }

                    app_state
                        .inflight_requests
                        .write()
                        .insert(cache_key.clone());

                    // 3. Fetch New Data
                    let adapter_result =
                        crate::blockchain::registry::ChainRegistry::get_adapter(&account.chain);

                    let balance_result = match adapter_result {
                        Ok(adapter) => adapter.get_balance(&account.address).await,
                        Err(e) => Err(e),
                    };

                    app_state.inflight_requests.write().remove(&cache_key);

                    let balance = balance_result?;

                    // 4. Update Cache
                    app_state
                        .cache
                        .write()
                        .insert(cache_key, CacheEntry::from_string(balance.clone(), now));

                    return Ok(balance);
                }
            }
        }
        Err(anyhow!("No account selected"))
    }

    /// 删除钱包（从用户账号绑定的钱包删除）
    /// 删除钱包及其所有存储数据（salt、seed/private_key等）
    /// 同时从后端删除钱包记录
    pub async fn delete_wallet(&self, wallet_id: &str) -> Result<()> {
        let mut app_state = self.app_state;

        // 1. 获取钱包信息（用于后端删除）
        let wallet_name = {
            let wallet_state = app_state.wallet.read();
            if let Some(wallet) = wallet_state.get_wallet(wallet_id) {
                wallet.name.clone()
            } else {
                return Err(anyhow!("Wallet not found"));
            }
        };

        // 2. 从后端删除钱包（如果已登录）
        if app_state.user.read().is_authenticated {
            // 确保 API 客户端有最新的认证 token
            let user_state = app_state.user.read();
            if let Some(ref token) = user_state.access_token {
                app_state.api.write().set_bearer_token(token.clone());
            }
            drop(user_state);

            // 从后端删除所有链的钱包记录
            // 注意：后端每个链的钱包都有不同的ID，我们需要通过钱包名称来查找并删除
            use crate::services::wallet::WalletService;
            let wallet_service = WalletService::new(app_state);

            // 获取所有后端钱包，找到匹配的钱包并删除
            if let Ok(backend_wallets) = wallet_service.list_wallets().await {
                for backend_wallet in backend_wallets {
                    // 检查是否是同一个钱包（通过名称匹配）
                    let backend_base_name = backend_wallet
                        .name
                        .split(" (")
                        .next()
                        .unwrap_or(&backend_wallet.name)
                        .to_string();

                    if backend_base_name == wallet_name {
                        // 删除后端钱包记录
                        // 将 String ID 转换为 Uuid
                        if let Ok(uuid) = uuid::Uuid::parse_str(&backend_wallet.id) {
                            if let Err(e) = wallet_service.delete_wallet(uuid).await {
                                tracing::warn!(
                                    "Failed to delete backend wallet {}: {}",
                                    backend_wallet.id,
                                    e
                                );
                                // 继续删除其他链的钱包，不因为一个失败而停止
                            }
                        } else {
                            tracing::warn!("Invalid wallet ID format: {}", backend_wallet.id);
                        }
                    }
                }
            }
        }

        // 3. 从钱包状态中移除钱包
        let mut wallet_state = app_state.wallet.write();
        if !wallet_state.remove_wallet(wallet_id) {
            return Err(anyhow!("Wallet not found in local state"));
        }
        wallet_state.save()?;
        drop(wallet_state);

        // 4. 清理LocalStorage中的钱包数据
        let salt_key = format!("wallet_{}_salt", wallet_id);
        let seed_key = format!("wallet_{}_seed", wallet_id);
        let priv_key = format!("wallet_{}_private_key", wallet_id);

        // 尝试删除所有可能存在的key
        // 使用delete方法删除LocalStorage中的项（gloo-storage 0.3 API）
        LocalStorage::delete(&salt_key);
        LocalStorage::delete(&seed_key);
        LocalStorage::delete(&priv_key);

        self.update_activity();

        Ok(())
    }
}
