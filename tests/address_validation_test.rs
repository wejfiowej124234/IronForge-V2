//! Address Validation Tests - 地址验证逻辑测试
//! 企业级单元测试，使用wasm-bindgen-test

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// 测试以太坊地址验证
#[wasm_bindgen_test]
fn test_ethereum_address_validation() {
    // 有效地址
    assert!(validate_ethereum_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb").is_ok());
    assert!(validate_ethereum_address("0x0000000000000000000000000000000000000000").is_ok());

    // 无效地址
    assert!(validate_ethereum_address("invalid").is_err());
    assert!(validate_ethereum_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bE").is_err()); // 太短
    assert!(validate_ethereum_address("742d35Cc6634C0532925a3b844Bc9e7595f0bEb").is_err());
    // 缺少0x
}

/// 测试比特币地址验证
#[wasm_bindgen_test]
fn test_bitcoin_address_validation() {
    // 有效地址
    assert!(validate_bitcoin_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa").is_ok());
    assert!(validate_bitcoin_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy").is_ok());

    // 无效地址
    assert!(validate_bitcoin_address("invalid").is_err());
    assert!(validate_bitcoin_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7Divf").is_err()); // 太短
    assert!(validate_bitcoin_address("").is_err());
}

/// 辅助函数：验证以太坊地址
fn validate_ethereum_address(address: &str) -> Result<(), String> {
    if !address.starts_with("0x") {
        return Err("以太坊地址必须以0x开头".to_string());
    }

    if address.len() != 42 {
        return Err("以太坊地址长度必须为42字符".to_string());
    }

    // 检查是否为有效的十六进制
    let hex_part = &address[2..];
    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("以太坊地址包含无效字符".to_string());
    }

    Ok(())
}

/// 辅助函数：验证比特币地址
fn validate_bitcoin_address(address: &str) -> Result<(), String> {
    if address.is_empty() {
        return Err("比特币地址不能为空".to_string());
    }

    if address.len() < 26 || address.len() > 35 {
        return Err("比特币地址长度必须在26-35字符之间".to_string());
    }

    // 基本格式检查（Base58字符）
    let base58_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    if !address.chars().all(|c| base58_chars.contains(c)) {
        return Err("比特币地址包含无效字符".to_string());
    }

    Ok(())
}
