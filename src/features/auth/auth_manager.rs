//! 🔐 统一认证状态管理器 (Authentication Manager)
//!
//! ## 职责
//! 1. Token生命周期管理（创建、刷新、过期、清理）
//! 2. 认证状态同步（UserState ↔ ApiClient）
//! 3. 401错误统一处理
//! 4. Token有效性验证
//!
//! ## 架构位置
//! ```
//! IronForge/src/features/auth/
//! ├── mod.rs
//! ├── state.rs           # UserState数据结构
//! ├── hooks.rs           # 登录/注册/登出hooks
//! └── auth_manager.rs    # ← 本文件：统一认证管理器
//! ```

use crate::features::auth::state::UserState;
use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use tracing::{debug, info, warn};
use web_sys::js_sys::Date;

/// 认证管理器 - 单例模式
///
/// ## 设计原则
/// - **单一职责**：只负责认证状态管理
/// - **中心化**：所有认证相关操作都通过此管理器
/// - **原子性**：状态更新保证原子性（Signal内部可变性）
#[derive(Clone, Copy)]
pub struct AuthManager {
    app_state: AppState,
}

impl AuthManager {
    /// 创建认证管理器实例
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// 📝 设置认证Token（登录/注册成功后调用）
    ///
    /// ## 执行步骤
    /// 1. 更新UserState（包含token_created_at时间戳）
    /// 2. 同步到ApiClient
    /// 3. 持久化到LocalStorage
    ///
    /// ## 示例
    /// ```rust
    /// auth_manager.set_token("jwt_token_here".to_string()).await;
    /// ```
    pub async fn set_token(mut self, token: String) {
        let now = Self::current_timestamp();
        
        // 1. 更新UserState
        {
            let mut user_state = self.app_state.user.write();
            user_state.is_authenticated = true;
            user_state.access_token = Some(token.clone());
            user_state.token_created_at = Some(now);
            let _ = user_state.save();
        }

        // 2. 等待Signal propagation（100ms确保所有依赖Signal的组件都收到更新）
        TimeoutFuture::new(100).await;

        // 3. 同步到ApiClient
        {
            let mut api = self.app_state.api.write();
            api.set_bearer_token(token);
        }

        info!("✅ Token设置成功，时间戳: {}", now);
    }

    /// 🔄 刷新Token（即将过期时调用）
    ///
    /// ## Token刷新策略
    /// - **提前刷新**：在过期前5分钟开始尝试刷新
    /// - **优雅降级**：刷新失败则清理状态，引导用户重新登录
    ///
    /// ## 返回值
    /// - `Ok(true)`: 刷新成功
    /// - `Ok(false)`: Token仍然有效，无需刷新
    /// - `Err(_)`: 刷新失败
    pub async fn refresh_token_if_needed(&self) -> Result<bool, String> {
        let should_refresh = {
            let user_state = self.app_state.user.read();
            if let Some(created_at) = user_state.token_created_at {
                let now = Self::current_timestamp();
                let age_seconds = (now - created_at) / 1000;
                // 55分钟后刷新（token有效期1小时）
                age_seconds >= 3300
            } else {
                false
            }
        };

        if !should_refresh {
            return Ok(false);
        }

        // TODO: 调用后端refresh_token API
        // let api = self.app_state.api.read();
        // let response = api.post::<RefreshTokenResp>("/api/v1/auth/refresh", &()).await?;
        // self.set_token(response.access_token).await;

        warn!("⚠️ Token刷新功能待实现");
        Ok(false)
    }

    /// ❌ 清理认证状态（登出/Token过期/401错误）
    ///
    /// ## 清理内容
    /// 1. 清空UserState（包括token_created_at）
    /// 2. 清空ApiClient的Bearer Token
    /// 3. 清理LocalStorage
    ///
    /// ## 调用时机
    /// - 用户主动登出
    /// - 收到401 Unauthorized响应
    /// - Token过期检测
    pub fn clear_auth(mut self) {
        // 1. 清理UserState（Signal的write()利用内部可变性）
        {
            let mut user_state = self.app_state.user.write();
            user_state.is_authenticated = false;
            user_state.access_token = None;
            user_state.token_created_at = None;
            user_state.email = None;
            let _ = user_state.save();
        }

        // 2. 清理ApiClient
        {
            let mut api = self.app_state.api.write();
            api.clear_auth();
        }

        info!("🧹 认证状态已清理");
    }

    /// ✅ 检查Token是否有效
    ///
    /// ## 验证规则
    /// 1. Token存在
    /// 2. 未过期（< 3600秒）
    /// 3. 格式有效（可选）
    ///
    /// ## 返回值
    /// - `Ok(true)`: Token有效
    /// - `Ok(false)`: Token无效或过期
    /// - `Err(_)`: 验证过程出错
    pub fn validate_token(&self) -> Result<bool, String> {
        let user_state = self.app_state.user.read();

        // 1. 检查Token是否存在
        if user_state.access_token.is_none() {
            debug!("❌ Token不存在");
            return Ok(false);
        }

        // 2. 检查Token是否过期
        if let Some(created_at) = user_state.token_created_at {
            let now = Self::current_timestamp();
            let age_seconds = (now - created_at) / 1000;

            if age_seconds >= 3600 {
                warn!("⏰ Token已过期（{}秒）", age_seconds);
                return Ok(false);
            }

            debug!("✅ Token有效（剩余{}秒）", 3600 - age_seconds);
            Ok(true)
        } else {
            // 旧Token没有created_at，视为有效（向后兼容）
            warn!("⚠️ Token缺少created_at字段，视为有效");
            Ok(true)
        }
    }

    /// 🔄 同步状态：UserState → ApiClient
    ///
    /// ## 使用场景
    /// - 应用启动时从LocalStorage恢复状态
    /// - 手动触发状态同步
    pub async fn sync_to_api_client(mut self) {
        let token_opt = {
            let user_state = self.app_state.user.read();
            user_state.access_token.clone()
        };

        if let Some(token) = token_opt {
            // 先验证Token是否有效
            let is_valid = self.validate_token().unwrap_or(false);
            
            if is_valid {
                let mut api = self.app_state.api.write();
                api.set_bearer_token(token);
                info!("🔄 状态已同步到ApiClient");
            } else {
                warn!("⚠️ Token无效，清理状态");
                self.clear_auth();
            }
        } else {
            debug!("ℹ️ 无Token需要同步");
        }
    }

    /// 📊 获取Token剩余有效时间（秒）
    pub fn get_token_remaining_seconds(&self) -> Option<u64> {
        let user_state = self.app_state.user.read();
        if let Some(created_at) = user_state.token_created_at {
            let now = Self::current_timestamp();
            let age_seconds = (now - created_at) / 1000;
            if age_seconds < 3600 {
                Some(3600 - age_seconds)
            } else {
                Some(0)
            }
        } else {
            None
        }
    }

    /// 🔐 检查是否已认证
    pub fn is_authenticated(&self) -> bool {
        let user_state = self.app_state.user.read();
        user_state.is_authenticated && user_state.access_token.is_some()
    }

    /// ⏰ 获取当前时间戳（毫秒）
    fn current_timestamp() -> u64 {
        Date::new_0().get_time() as u64
    }
}

/// 🎯 401错误处理器 - 全局拦截器
///
/// ## 使用方式
/// 在每个API service中调用：
/// ```rust
/// match api.get::<T>(&path).await {
///     Err(e) if is_unauthorized_error(&e) => {
///         handle_unauthorized(app_state).await;
///         Err(e.into())
///     }
///     result => result.map_err(Into::into)
/// }
/// ```
pub async fn handle_unauthorized(app_state: AppState) {
    warn!("🚨 收到401错误，清理认证状态");
    let auth_manager = AuthManager::new(app_state);
    auth_manager.clear_auth();
    
    // 可选：导航到登录页
    // let nav = use_navigator();
    // nav.push("/login");
}

/// 🔍 判断是否为401错误
pub fn is_unauthorized_error(error: &crate::shared::error::AppError) -> bool {
    // 检查错误消息中是否包含 "401" 或 "Unauthorized"
    let msg = format!("{:?}", error).to_lowercase();
    msg.contains("401") || msg.contains("unauthorized")
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: 添加单元测试
    // - test_set_token()
    // - test_validate_token()
    // - test_token_expiry()
    // - test_clear_auth()
}
