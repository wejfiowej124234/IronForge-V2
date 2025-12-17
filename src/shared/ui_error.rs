//! User-facing error message sanitization.
//!
//! 目标：避免把底层 `ApiError` / HTTP 状态 / 反序列化错误等直接展示给用户，
//! 统一输出更清晰、更可操作的提示。

use std::borrow::Cow;

/// 将错误消息转换为更适合展示给用户的提示。
///
/// - 如果消息中包含明显的底层错误（401/Unauthorized、timeout、network、429、5xx、deserialize 等），会被替换为友好提示。
/// - 如果消息本身已经是可读的中文提示（例如“复制失败，请手动复制”），则保持原样。
pub fn sanitize_user_message(message: impl Into<String>) -> String {
    let message: String = message.into();
    let lower = message.to_lowercase();

    // 先快速判断：如果不包含任何“看起来像底层错误”的关键字，就不动它。
    if !looks_like_raw_error(&lower) {
        return message;
    }

    // 尝试从消息中提取“业务上下文前缀”，例如："导入失败: ..." -> "导入失败"
    let (prefix, has_prefix) = extract_prefix(&message);

    let friendly = if is_auth_error(&lower) {
        "认证失败，请先登录或重新登录"
    } else if is_timeout_error(&lower) {
        "请求超时，请稍后再试"
    } else if is_rate_limit_error(&lower) {
        "请求过于频繁，请稍后再试"
    } else if is_network_error(&lower) {
        "网络异常，请检查网络后重试"
    } else if is_server_error(&lower) {
        "服务暂时不可用，请稍后再试"
    } else if is_client_input_error(&lower) {
        "输入参数有误，请检查后重试"
    } else if is_deserialize_error(&lower) {
        "服务响应异常，请稍后再试"
    } else {
        "操作失败，请稍后再试"
    };

    if has_prefix {
        format!("{}：{}", prefix, friendly)
    } else {
        friendly.to_string()
    }
}

fn looks_like_raw_error(lower: &str) -> bool {
    // ApiError / HTTP / 底层库常见特征
    lower.contains("api error")
        || lower.contains("request failed")
        || lower.contains("response error")
        || lower.contains("unauthorized")
        || lower.contains("timeout")
        || lower.contains("failed to fetch")
        || lower.contains("network")
        || lower.contains("connection")
        || lower.contains("429")
        || lower.contains("rate limit")
        || lower.contains("500")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("internal server error")
        || lower.contains("deserialize")
        || lower.contains("failed to deserialize")
        || lower.contains("invalid type")
        || lower.contains("serde")
        || lower.contains("json")
        || lower.contains("jwt")
        || lower.contains("token")
        || lower.contains("forbidden")
        || lower.contains("403")
        || lower.contains("401")
        || lower.contains("400")
}

fn extract_prefix(message: &str) -> (Cow<'_, str>, bool) {
    // 常见分隔符："失败:" / "失败：" / 以及其它带冒号的结构
    if let Some(idx) = message.find("失败:").or_else(|| message.find("失败：")) {
        let end = idx + "失败".len();
        return (Cow::Borrowed(message[..end].trim()), true);
    }

    // 其它情况：如果存在中文冒号或英文冒号，也可以认为是“前缀: 详情”
    if let Some(idx) = message.find('：').or_else(|| message.find(':')) {
        let left = message[..idx].trim();
        if !left.is_empty() && left.chars().count() <= 24 {
            return (Cow::Borrowed(left), true);
        }
    }

    (Cow::Borrowed(message), false)
}

fn is_auth_error(lower: &str) -> bool {
    lower.contains("unauthorized")
        || lower.contains("401")
        || lower.contains("token")
        || lower.contains("jwt")
        || lower.contains("forbidden")
        || lower.contains("403")
}

fn is_timeout_error(lower: &str) -> bool {
    lower.contains("timeout") || lower.contains("timed out")
}

fn is_rate_limit_error(lower: &str) -> bool {
    lower.contains("rate limit") || lower.contains("429")
}

fn is_network_error(lower: &str) -> bool {
    lower.contains("request failed")
        || lower.contains("failed to fetch")
        || lower.contains("network")
        || lower.contains("connection")
        || lower.contains("dns")
}

fn is_server_error(lower: &str) -> bool {
    lower.contains("500")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("internal server error")
}

fn is_client_input_error(lower: &str) -> bool {
    lower.contains("invalid")
        || lower.contains("validation")
        || lower.contains("bad request")
        || lower.contains("400")
}

fn is_deserialize_error(lower: &str) -> bool {
    lower.contains("deserialize")
        || lower.contains("failed to deserialize")
        || lower.contains("invalid type")
        || lower.contains("serde")
        || lower.contains("json")
}
