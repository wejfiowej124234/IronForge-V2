//! Error Handling Tests - 错误处理逻辑测试
//! 企业级单元测试，使用wasm-bindgen-test

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// 测试错误消息格式化
#[wasm_bindgen_test]
fn test_error_message_formatting() {
    // 测试用户友好的错误消息格式化
    let technical_error = "NetworkError: Failed to fetch";
    let user_friendly = format_user_friendly_error(technical_error);

    assert!(user_friendly.contains("网络"));
    assert!(!user_friendly.contains("NetworkError"));
}

/// 测试错误级别判断
#[wasm_bindgen_test]
fn test_error_level_determination() {
    // 测试根据错误类型判断错误级别
    assert_eq!(determine_error_level("网络错误"), "warning");
    assert_eq!(determine_error_level("余额不足"), "error");
    assert_eq!(determine_error_level("系统错误"), "critical");
}

/// 辅助函数：格式化用户友好的错误消息
fn format_user_friendly_error(error: &str) -> String {
    if error.contains("NetworkError") || error.contains("Failed to fetch") {
        "网络连接失败，请检查网络设置后重试".to_string()
    } else if error.contains("余额不足") || error.contains("insufficient") {
        "余额不足，请减少交换数量或先充值".to_string()
    } else {
        format!("操作失败：{}", error)
    }
}

/// 辅助函数：判断错误级别
fn determine_error_level(error: &str) -> &str {
    if error.contains("系统") || error.contains("critical") {
        "critical"
    } else if error.contains("余额") || error.contains("验证失败") {
        "error"
    } else {
        "warning"
    }
}
