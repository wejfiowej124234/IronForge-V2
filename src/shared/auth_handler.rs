//! Authentication Error Handler - 统一的认证错误处理
//!
//! 当API返回401 Unauthorized时，自动清理状态并跳转到登录页

use crate::shared::error::ApiError;
use crate::shared::state::AppState;
use dioxus::prelude::*;

/// 检查错误是否为401 Unauthorized
pub fn is_unauthorized_error(error: &ApiError) -> bool {
    matches!(error, ApiError::Unauthorized)
        || error.to_string().to_lowercase().contains("unauthorized")
        || error.to_string().contains("401")
}

/// 处理401错误：记录日志但不自动登出
///
/// # 设计理念
/// 401 错误有多种原因：
/// 1. Token 真的过期（1小时后）
/// 2. 后端 JWT 密钥配置变化
/// 3. Token 格式错误
/// 4. 后端重启导致 session 失效
///
/// **不应该盲目自动登出**，而是应该：
/// - 让用户看到明确的错误提示
/// - 由用户决定是否重新登录
/// - 避免「刚登录就被踢出」的糟糕体验
///
/// # Example
/// ```rust
/// match api_client.get::<T>(&url).await {
///     Ok(data) => Ok(data),
///     Err(e) if is_unauthorized_error(&e) => {
///         handle_unauthorized_and_redirect(app_state);
///         Err("认证已过期，请重新登录".to_string())
///     }
///     Err(e) => Err(format!("请求失败: {}", e))
/// }
/// ```
pub fn handle_unauthorized_and_redirect(app_state: AppState) {
    // 对于 401，我们只做「静默记录」，不再给已登录用户打扰性的警告
    let user_state = app_state.user.read();
    let has_valid_session = user_state.is_authenticated && user_state.access_token.is_some();

    if has_valid_session {
        // 用户已登录但收到 401，通常是：
        // - 该功能在后台未开通 / 权限不足
        // - token 真的过期
        // 交给具体页面用自己的文案提示，这里只做 debug 日志
        tracing::debug!(
            "用户已登录但收到401（可能是接口权限/功能未开通/后端配置变化），交由页面自行处理提示"
        );
    } else {
        tracing::debug!("401 错误且用户未登录");
    }

    // 仍然不自动登出和跳转，让用户自己决定
    // app_state.handle_unauthorized();
    // navigator().push(Route::Login {});
}

/// 将ApiError转换为用户友好的错误消息
///
/// 如果是401错误，自动处理并返回友好提示
pub fn handle_api_error_with_auth(error: ApiError, app_state: AppState, context: &str) -> String {
    if is_unauthorized_error(&error) {
        handle_unauthorized_and_redirect(app_state);
        "认证已过期，请重新登录".to_string()
    } else {
        let error_msg = error.to_string().to_lowercase();

        // 通用错误转换
        if error_msg.contains("network") || error_msg.contains("connection") {
            format!("{}：网络错误，请稍后重试", context)
        } else if error_msg.contains("timeout") {
            format!("{}：请求超时，请稍后重试", context)
        } else if error_msg.contains("invalid") || error_msg.contains("validation") {
            format!("{}：输入参数无效，请检查后重试", context)
        } else {
            format!("{}失败：{}", context, error)
        }
    }
}
