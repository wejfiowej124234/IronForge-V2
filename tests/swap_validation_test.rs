#![cfg(target_arch = "wasm32")]

//! Swap Validation Tests - 交换验证逻辑测试
//! 企业级单元测试，使用wasm-bindgen-test

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// 测试金额验证逻辑
#[wasm_bindgen_test]
fn test_amount_validation() {
    // 测试有效金额
    assert!(validate_amount("100.0").is_ok());
    assert!(validate_amount("0.001").is_ok());
    assert!(validate_amount("1e10").is_ok());

    // 测试无效金额
    assert!(validate_amount("").is_err());
    assert!(validate_amount("abc").is_err());
    assert!(validate_amount("-100").is_err());
    assert!(validate_amount("0").is_err());

    // 测试边界情况
    assert!(validate_amount("1e15").is_ok());
    assert!(validate_amount("1e16").is_err()); // 过大
}

/// 测试代币选择验证
#[wasm_bindgen_test]
fn test_token_selection_validation() {
    // 测试相同代币
    assert!(validate_token_pair("USDT", "USDT").is_err());
    assert!(validate_token_pair("ETH", "ETH").is_err());

    // 测试不同代币
    assert!(validate_token_pair("USDT", "USDC").is_ok());
    assert!(validate_token_pair("ETH", "USDT").is_ok());

    // 测试空值
    assert!(validate_token_pair("", "USDT").is_err());
    assert!(validate_token_pair("USDT", "").is_err());
}

/// 辅助函数：验证金额
fn validate_amount(amount: &str) -> Result<f64, String> {
    let parsed = amount
        .parse::<f64>()
        .map_err(|_| "无效的金额格式".to_string())?;

    if parsed.is_nan() || parsed.is_infinite() || parsed <= 0.0 {
        return Err("金额必须大于0".to_string());
    }

    if parsed > 1e15 {
        return Err("金额过大".to_string());
    }

    Ok(parsed)
}

/// 辅助函数：验证代币对
fn validate_token_pair(from: &str, to: &str) -> Result<(), String> {
    if from.is_empty() || to.is_empty() {
        return Err("代币符号不能为空".to_string());
    }

    if from == to {
        return Err("不能交换相同的代币".to_string());
    }

    Ok(())
}
