//! Authentication Hooks - 认证相关的Hook

use crate::features::auth::state::UserState;
use crate::services::auth::AuthService;
use crate::shared::state::AppState;
use anyhow::Result;
use dioxus::prelude::*;

pub fn use_auth() -> AuthController {
    let app_state = use_context::<AppState>();
    AuthController { app_state }
}

#[derive(Clone, Copy)]
pub struct AuthController {
    pub app_state: AppState,
}

impl AuthController {
    /// 注册新用户
    pub async fn register(
        &self,
        email: &str,
        password: &str,
        confirm_password: &str,
    ) -> Result<()> {
        let mut app_state = self.app_state;
        let auth_service = AuthService::new(app_state);
        let response = auth_service
            .register_email(email, password, confirm_password)
            .await?;

        // 更新用户状态
        {
            let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
            let mut user_state = app_state.user.write();
            user_state.is_authenticated = true;
            user_state.user_id = Some(response.user.id.clone());
            user_state.email = Some(response.user.email.clone());
            user_state.access_token = Some(response.access_token.clone());
            user_state.token_created_at = Some(now); // 记录token创建时间
            user_state.created_at = Some(response.user.created_at.clone());

            // 保存状态
            user_state.save()?;
        } // Drop user_state borrow here

        // 更新API客户端的Bearer Token
        app_state
            .api
            .write()
            .set_bearer_token(response.access_token);

        Ok(())
    }

    /// 用户登录
    pub async fn login(&self, email: &str, password: &str) -> Result<()> {
        let mut app_state = self.app_state;
        let auth_service = AuthService::new(app_state);
        let response = auth_service.login_email(email, password).await?;

        // 更新用户状态
        // 企业级实现：登录成功后，统一更新 UserState 并持久化
        {
            let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
            let mut user_state = app_state.user.write();
            user_state.is_authenticated = true;
            user_state.user_id = Some(response.user.id.clone());
            user_state.email = Some(response.user.email.clone());
            user_state.username = None;
            user_state.access_token = Some(response.access_token.clone());
            user_state.token_created_at = Some(now); // 记录token创建时间
            user_state.created_at = Some(response.user.created_at.clone());
            let _ = user_state.save();
        } // Drop user_state borrow here

        // 验证token确实被保存（防止LocalStorage失败）
        {
            let user_state = app_state.user.read();
            if user_state.access_token.is_none()
                || user_state
                    .access_token
                    .as_ref()
                    .map(|t| t.is_empty())
                    .unwrap_or(true)
            {
                #[cfg(debug_assertions)]
                {
                    use tracing::error;
                    error!("❌ Token保存失败！可能是LocalStorage被禁用或浏览器隐私模式");
                }
                return Err(anyhow::anyhow!(
                    "Token保存失败，请检查浏览器LocalStorage设置"
                ));
            }
        }

        // 更新API客户端的Bearer Token
        app_state
            .api
            .write()
            .set_bearer_token(response.access_token);

        // 更新活动时间
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
        *app_state.last_active.write() = now;

        // 登录成功后，从后端获取用户的钱包列表
        self.sync_wallets_from_backend().await?;

        Ok(())
    }

    /// 从后端同步钱包列表到本地状态
    pub async fn sync_wallets_from_backend(&self) -> Result<()> {
        use crate::services::wallet::WalletService;
        let mut app_state = self.app_state;

        // 等待Signal更新完成，避免竞态条件
        gloo_timers::future::TimeoutFuture::new(100).await;

        // 检查是否已登录
        if !app_state.user.read().is_authenticated {
            return Ok(()); // 未登录时跳过
        }

        // 确保API客户端有最新的认证token
        let user_state = app_state.user.read();
        if let Some(ref token) = user_state.access_token {
            if !token.is_empty() {
                app_state.api.write().set_bearer_token(token.clone());
                #[cfg(debug_assertions)]
                {
                    use tracing::debug;
                    debug!(
                        "🔄 同步钱包: Token已同步到API客户端 (length: {})",
                        token.len()
                    );
                }
            } else {
                #[cfg(debug_assertions)]
                {
                    use tracing::warn;
                    warn!("⚠️ 同步钱包: Token为空，跳过同步");
                }
                return Ok(()); // token为空，跳过同步
            }
        } else {
            #[cfg(debug_assertions)]
            {
                use tracing::warn;
                warn!("⚠️ 同步钱包: 没有token，跳过同步");
            }
            return Ok(()); // 没有token，跳过同步
        }
        drop(user_state);

        // 从后端获取钱包列表
        let backend_wallets_result = {
            let wallet_service = WalletService::new(app_state);
            wallet_service.list_wallets().await
        }; // wallet_service 在这里被释放

        match backend_wallets_result {
            Ok(backend_wallets) => {
                // 将后端钱包转换为前端钱包格式
                use crate::features::wallet::state::{Account, AccountType, Wallet};
                let backend_wallet_count = backend_wallets.len(); // 保存长度用于日志

                #[cfg(debug_assertions)]
                {
                    use tracing::info;
                    info!(
                        "🔄 开始同步钱包: 后端返回 {} 个单链钱包记录",
                        backend_wallet_count
                    );
                }

                let mut wallet_state = app_state.wallet.write();

                // ✅ 行业最佳实践：三层防护策略
                //
                // 第一层：检测数据库重建（后端返回空 + 本地有钱包）
                // 第二层：自动重新同步本地钱包到后端
                // 第三层：即使同步失败，本地钱包仍然可用
                //
                if backend_wallet_count == 0 && !wallet_state.wallets.is_empty() {
                    #[cfg(debug_assertions)]
                    {
                        use tracing::warn;
                        warn!(
                            "⚠️ 检测到数据库可能已重建：后端返回0个钱包，但本地有 {} 个钱包",
                            wallet_state.wallets.len()
                        );
                        warn!("🔄 自动触发：重新同步本地钱包到后端");
                    }

                    // 自动重新同步：将本地钱包推送到后端
                    drop(wallet_state); // 释放锁，允许re_sync修改

                    match self.re_sync_local_wallets_to_backend().await {
                        Ok(synced_count) => {
                            #[cfg(debug_assertions)]
                            {
                                use tracing::info;
                                info!(
                                    "✅ 自动同步成功：已将 {} 个本地钱包重新注册到后端",
                                    synced_count
                                );
                                info!("🔄 重新从后端加载钱包列表（避免递归使用循环重试）");
                            }
                            // ✅ 修复递归问题：使用Box::pin包装递归调用
                            return Box::pin(self.sync_wallets_from_backend()).await;
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            {
                                use tracing::error;
                                error!("❌ 自动同步失败: {}，保留本地钱包（仍可正常使用）", e);
                            }
                            // 即使同步失败，本地钱包仍然可用
                            // 不清空本地钱包，直接返回
                            return Ok(());
                        }
                    }
                }

                // 清空现有钱包列表（从后端同步）
                wallet_state.wallets.clear();

                // 转换后端钱包为前端格式
                // 注意：后端每个钱包只支持一个链，但前端钱包有多个账户
                // 我们需要将相同名称的钱包合并（去除链后缀）
                // 重要：需要查找本地存储中已有的钱包ID（通过钱包名称）
                // 因为本地存储的加密种子使用的是前端生成的wallet_id
                use gloo_storage::{LocalStorage, Storage};
                use std::collections::HashMap;
                let mut wallet_map: HashMap<String, Wallet> = HashMap::new();

                // 首先，从本地存储中查找所有已有的钱包，建立名称到ID的映射
                let mut name_to_id_map: HashMap<String, String> = HashMap::new();
                {
                    // 尝试从本地存储中加载钱包状态
                    use crate::features::wallet::state::WalletState;
                    if let Ok(local_wallet_state) = LocalStorage::get::<WalletState>("wallet_state")
                    {
                        for local_wallet in local_wallet_state.wallets.iter() {
                            // 检查这个本地钱包是否在本地存储中有加密种子
                            let seed_key = format!("wallet_{}_seed", local_wallet.id);
                            if LocalStorage::get::<String>(&seed_key).is_ok() {
                                name_to_id_map
                                    .insert(local_wallet.name.clone(), local_wallet.id.clone());
                            }
                        }
                    }
                }

                for backend_wallet in backend_wallets {
                    // ✅ 使用group_id作为合并键（如果有），否则使用名称
                    let merge_key = if let Some(ref gid) = backend_wallet.group_id {
                        gid.clone()
                    } else {
                        backend_wallet.name.clone()
                    };

                    #[cfg(debug_assertions)]
                    {
                        use tracing::info;
                        info!(
                            "  处理后端钱包: '{}' (链: {}, group_id: {:?})",
                            backend_wallet.name, backend_wallet.chain, backend_wallet.group_id
                        );
                    }

                    // 查找或创建钱包（使用group_id或名称作为key）
                    // 优先使用本地存储中已有的钱包ID（如果存在）
                    // 如果不存在，为这个钱包组创建一个新的钱包ID
                    let wallet = wallet_map.entry(merge_key.clone()).or_insert_with(|| {
                        // 尝试从本地存储中查找已有的钱包ID
                        let id = if let Some(existing_id) = name_to_id_map.get(&backend_wallet.name)
                        {
                            existing_id.clone()
                        } else {
                            // 如果本地存储中没有，生成新的钱包ID
                            // 注意：不使用后端的ID，因为后端每个链都有独立的ID
                            use uuid::Uuid;
                            Uuid::new_v4().to_string()
                        };
                        Wallet::new(id, backend_wallet.name.clone())
                    });

                    // 添加账户（✅ 使用后端返回的公钥）
                    // 标准化链名称
                    let chain_upper = backend_wallet.chain.to_uppercase();
                    let chain_name = match chain_upper.as_str() {
                        "ETH" => "ethereum".to_string(),
                        "BTC" => "bitcoin".to_string(),
                        "SOL" => "solana".to_string(),
                        "TON" => "ton".to_string(),
                        _ => backend_wallet.chain.to_lowercase(),
                    };

                    // 根据链推断派生路径
                    let derivation_path = match chain_name.as_str() {
                        "ethereum" => Some("m/44'/60'/0'/0/0".to_string()),
                        "bitcoin" => Some("m/84'/0'/0'/0/0".to_string()),
                        "solana" => Some("m/44'/501'/0'/0'/0".to_string()),
                        "ton" => Some("m/44'/607'/0'/0'/0".to_string()),
                        _ => None,
                    };

                    wallet.accounts.push(Account {
                        address: backend_wallet.address.clone(),
                        chain: chain_name,
                        public_key: backend_wallet.public_key.clone(), // ✅ 使用后端返回的公钥
                        derivation_path,                               // 推断的派生路径
                        account_type: AccountType::Derived,
                        balance: "0".to_string(), // 余额需要单独获取
                    });
                }

                // 将合并后的钱包添加到状态
                wallet_state.wallets = wallet_map.into_values().collect();

                #[cfg(debug_assertions)]
                {
                    use tracing::info;
                    info!(
                        "✅ 钱包合并完成: {} 个钱包（后端返回 {} 个单链钱包）",
                        wallet_state.wallets.len(),
                        backend_wallet_count
                    );
                    for wallet in &wallet_state.wallets {
                        info!(
                            "  📦 钱包: {} - {} 个账户",
                            wallet.name,
                            wallet.accounts.len()
                        );
                        for account in &wallet.accounts {
                            info!("    └─ {}: {}", account.chain, &account.address[..8]);
                        }
                    }
                }

                // 如果没有选中的钱包且有钱包，选择第一个
                if wallet_state.selected_wallet_id.is_none() && !wallet_state.wallets.is_empty() {
                    wallet_state.selected_wallet_id = Some(wallet_state.wallets[0].id.clone());
                }

                // 保存到本地存储
                wallet_state.save()?;

                Ok(())
            }
            Err(e) => {
                // 如果获取失败，保留本地钱包列表，不阻止登录
                // 检查是否是401错误（token过期）
                let error_msg = e.to_string().to_lowercase();
                let is_unauthorized =
                    error_msg.contains("401") || error_msg.contains("unauthorized");

                #[cfg(debug_assertions)]
                {
                    use tracing::warn;
                    if is_unauthorized {
                        warn!(
                            "Failed to sync wallets from backend: Token may be expired or invalid"
                        );
                    } else {
                        warn!("Failed to sync wallets from backend: {:?}", e);
                    }
                }

                // 如果本地有钱包，保留它们；如果没有，尝试从本地存储加载
                let mut wallet_state = app_state.wallet.write();
                if wallet_state.wallets.is_empty() {
                    // 尝试从本地存储加载钱包
                    // 使用WalletState::load()方法，它是async的，但这里在async上下文中
                    use crate::features::wallet::state::WalletState;
                    let local_wallet_state = WalletState::load().await;
                    if !local_wallet_state.wallets.is_empty() {
                        wallet_state.wallets = local_wallet_state.wallets;
                        wallet_state.selected_wallet_id = local_wallet_state.selected_wallet_id;
                        let _ = wallet_state.save();
                    }
                }

                Ok(())
            }
        }
    }

    /// 用户登出
    /// 清除本地状态并调用后端API撤销Token
    pub async fn logout(&self) -> Result<()> {
        let mut app_state = self.app_state;

        // 1. 调用后端API撤销Token（如果已登录）
        if app_state.user.read().is_authenticated {
            let auth_service = crate::services::auth::AuthService::new(app_state);
            // 尝试调用后端登出API（忽略错误，确保本地状态被清除）
            let _ = auth_service.logout().await;
        }

        // 2. 清除本地状态
        {
            let mut user_state = app_state.user.write();
            user_state.logout()?;
        } // Drop user_state borrow here

        // 3. 清除API Token
        app_state.api.write().clear_auth();

        // 4. 清除钱包状态（登出后需要重新登录）
        {
            let mut wallet_state = app_state.wallet.write();
            *wallet_state = crate::features::wallet::state::WalletState::default();
        } // Drop wallet_state borrow here

        Ok(())
    }

    /// 用户登出（同步版本，用于自动锁定等场景）
    /// 仅清除本地状态，不调用后端API
    pub fn logout_local(&self) -> Result<()> {
        let mut app_state = self.app_state;
        {
            let mut user_state = app_state.user.write();
            user_state.logout()?;
        } // Drop user_state borrow here

        app_state.api.write().clear_auth();

        {
            let mut wallet_state = app_state.wallet.write();
            *wallet_state = crate::features::wallet::state::WalletState::default();
        } // Drop wallet_state borrow here

        Ok(())
    }

    /// 检查用户是否已登录
    ///
    /// 注意：此方法当前未使用，但保留用于未来扩展
    #[allow(dead_code)]
    pub fn is_authenticated(&self) -> bool {
        self.app_state.user.read().is_authenticated
    }

    /// 获取用户信息
    ///
    /// 注意：此方法当前未使用，但保留用于未来扩展
    #[allow(dead_code)]
    pub fn get_user(&self) -> UserState {
        self.app_state.user.read().clone()
    }

    /// 更新用户头像
    ///
    /// 注意：此方法当前未使用，但保留用于未来扩展
    #[allow(dead_code)]
    pub fn set_avatar(&self, avatar_url: String) -> Result<()> {
        let mut app_state = self.app_state;
        {
            let mut user_state = app_state.user.write();
            user_state.avatar_url = Some(avatar_url);
            user_state.save()?;
        } // Drop user_state borrow here
        Ok(())
    }

    /// 更新活动时间（用于账户自动锁定）
    ///
    /// 注意：此方法当前未使用，但保留用于未来扩展
    #[allow(dead_code)]
    pub fn update_activity(&self) {
        let mut app_state = self.app_state;
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
        *app_state.last_active.write() = now;
    }

    /// 检查账户自动锁定（5分钟无操作）
    ///
    /// 注意：此方法当前未使用，但保留用于未来扩展
    #[allow(dead_code)]
    pub fn check_auto_lock(&self) {
        let app_state = self.app_state;
        let last_active = *app_state.last_active.read();
        let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;

        // 5分钟 = 300秒
        if now - last_active > 300 {
            // 自动锁定仅清除本地状态，不调用后端API
            self.logout_local().ok();
        }
    }

    /// 🔄 重新同步本地钱包到后端（数据库重建后的自动修复）
    ///
    /// 行业最佳实践：
    /// 1. 从IndexedDB读取所有本地钱包
    /// 2. 提取公开信息（地址、公钥、名称）
    /// 3. 批量注册到后端
    /// 4. 返回同步成功的钱包数量
    pub async fn re_sync_local_wallets_to_backend(&self) -> Result<usize> {
        use crate::features::wallet::state::WalletState;
        use crate::services::wallet::{
            BatchCreateWalletsRequest, WalletRegistrationInfo, WalletService,
        };
        use gloo_storage::{LocalStorage, Storage};

        // 1. 从LocalStorage加载本地钱包状态
        let local_wallet_state = LocalStorage::get::<WalletState>("wallet_state")
            .map_err(|e| anyhow::anyhow!("无法读取本地钱包状态: {}", e))?;

        if local_wallet_state.wallets.is_empty() {
            return Ok(0);
        }

        #[cfg(debug_assertions)]
        {
            use tracing::info;
            info!(
                "🔍 发现 {} 个本地钱包需要重新同步",
                local_wallet_state.wallets.len()
            );
        }

        // 2. 将本地钱包转换为后端注册格式
        let mut wallet_registrations = Vec::new();

        for local_wallet in local_wallet_state.wallets.iter() {
            // 跳过没有账户的钱包
            if local_wallet.accounts.is_empty() {
                continue;
            }

            // 为每个账户创建注册请求
            for account in local_wallet.accounts.iter() {
                let chain_str = match account.chain.as_str() {
                    "ethereum" => "ETH",
                    "bitcoin" => "BTC",
                    "solana" => "SOL",
                    "ton" => "TON",
                    _ => continue, // 跳过未知链
                };

                wallet_registrations.push(WalletRegistrationInfo {
                    chain: chain_str.to_uppercase(),
                    address: account.address.clone(),
                    public_key: account.public_key.clone(),
                    derivation_path: account.derivation_path.clone(),
                    name: Some(local_wallet.name.clone()),
                });
            }
        }

        if wallet_registrations.is_empty() {
            return Ok(0);
        }

        #[cfg(debug_assertions)]
        {
            use tracing::info;
            info!(
                "📤 准备批量注册 {} 个账户到后端",
                wallet_registrations.len()
            );
        }

        // 3. 批量注册到后端
        let app_state = self.app_state;
        let wallet_service = WalletService::new(app_state);
        let batch_request = BatchCreateWalletsRequest {
            wallets: wallet_registrations,
        };

        match wallet_service.batch_create_wallets(batch_request).await {
            Ok(response) => {
                let success_count = response.wallets.len();
                let failed_count = response.failed.len();

                #[cfg(debug_assertions)]
                {
                    use tracing::info;
                    info!(
                        "✅ 批量注册完成: {} 成功, {} 失败",
                        success_count, failed_count
                    );

                    if !response.failed.is_empty() {
                        use tracing::warn;
                        for err in response.failed.iter() {
                            warn!("  ⚠️ 失败: {} - {} ({})", err.chain, err.address, err.error);
                        }
                    }
                }

                Ok(success_count)
            }
            Err(e) => Err(anyhow::anyhow!("批量注册失败: {}", e)),
        }
    }
}
