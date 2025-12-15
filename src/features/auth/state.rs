//! User Authentication State - 用户认证状态
//! 管理用户账户信息、头像、登录状态等

use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserState {
    pub is_authenticated: bool,
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>, // 用户头像URL或base64
    pub access_token: Option<String>,
    pub created_at: Option<String>,
    #[serde(default)]
    pub token_created_at: Option<u64>, // Token创建时间戳（秒），用于判断是否过期
}

impl Default for UserState {
    fn default() -> Self {
        Self {
            is_authenticated: false,
            user_id: None,
            email: None,
            username: None,
            avatar_url: None,
            access_token: None,
            created_at: None,
            token_created_at: None,
        }
    }
}

impl UserState {
    /// 加载用户状态（从LocalStorage）
    /// 自动检查token是否过期（1小时），过期则清理
    pub fn load() -> Self {
        if let Ok(mut stored) = LocalStorage::get::<UserState>("user_state") {
            // 检查token是否过期（JWT token过期时间为3600秒=1小时）
            if let Some(token_time) = stored.token_created_at {
                let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
                let token_age = now.saturating_sub(token_time);
                
                // Token已过期（1小时=3600秒）
                if token_age >= 3600 {
                    #[cfg(debug_assertions)]
                    {
                        use tracing::warn;
                        warn!("⚠️ Token已过期（{}s），自动清理", token_age);
                    }
                    // 清理过期token
                    stored.is_authenticated = false;
                    stored.access_token = None;
                    stored.token_created_at = None;
                    let _ = stored.save();
                }
            } else if stored.is_authenticated && stored.access_token.is_some() {
                // 旧数据没有token_created_at字段，保守处理：清理token
                #[cfg(debug_assertions)]
                {
                    use tracing::warn;
                    warn!("⚠️ 检测到旧token格式（无创建时间），自动清理");
                }
                stored.is_authenticated = false;
                stored.access_token = None;
                let _ = stored.save();
            }
            stored
        } else {
            Self::default()
        }
    }

    /// 保存用户状态（到LocalStorage）
    pub fn save(&self) -> Result<(), gloo_storage::errors::StorageError> {
        LocalStorage::set("user_state", self)
    }

    /// 登出（清除状态）
    pub fn logout(&mut self) -> Result<(), gloo_storage::errors::StorageError> {
        *self = Self::default();
        LocalStorage::delete("user_state");
        Ok(())
    }

    /// 生成默认头像（基于邮箱或用户ID）
    pub fn generate_default_avatar(&self) -> String {
        // 使用邮箱或用户ID生成头像
        let seed = self
            .email
            .as_ref()
            .map(|e| e.to_string())
            .or_else(|| self.user_id.clone())
            .unwrap_or_else(|| "default".to_string());

        // 简单的头像生成：使用首字母或图标
        // 这里可以集成头像生成服务，如使用identicon或gravatar
        use base64::{engine::general_purpose, Engine as _};
        let initial = seed
            .chars()
            .next()
            .unwrap_or('U')
            .to_uppercase()
            .to_string();
        let color = "#6366F1";
        let svg_content = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><circle cx="50" cy="50" r="50" fill="{}"/><text x="50" y="70" font-size="50" fill="white" text-anchor="middle" font-family="Arial">{}</text></svg>"#,
            color, initial
        );
        format!(
            "data:image/svg+xml;base64,{}",
            general_purpose::STANDARD.encode(svg_content.as_bytes())
        )
    }

    /// 获取头像URL（如果有自定义头像则返回，否则返回默认头像）
    pub fn get_avatar_url(&self) -> String {
        self.avatar_url
            .clone()
            .unwrap_or_else(|| self.generate_default_avatar())
    }
}
