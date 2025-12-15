//! Authentication Feature - 用户认证功能模块
//!
//! ## 模块结构
//! - `state.rs`: UserState数据结构 + LocalStorage持久化
//! - `hooks.rs`: 登录/注册/登出 hooks
//! - `auth_manager.rs`: 统一认证状态管理器（新增）

pub mod auth_manager;
pub mod hooks;
pub mod state;

pub use auth_manager::{AuthManager, handle_unauthorized, is_unauthorized_error};
pub use state::UserState;
// pub use hooks::use_auth;  // 未使用，暂时注释
