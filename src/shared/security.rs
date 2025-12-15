//! Security utilities for input validation and sanitization
//! UI 安全工具：输入验证和清理

/// Sanitize string input to prevent XSS attacks
/// 清理字符串输入以防止 XSS 攻击
pub fn sanitize_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}

/// Validate address format (basic validation)
/// 验证地址格式（基本验证）
pub fn validate_address(address: &str, chain: Option<&str>) -> bool {
    if address.is_empty() || address.len() > 200 {
        return false;
    }

    // Basic format checks based on chain
    match chain {
        Some("ethereum") | Some("polygon") | Some("bsc") => {
            // Ethereum-style addresses: 0x followed by 40 hex characters
            address.starts_with("0x")
                && address.len() == 42
                && address[2..].chars().all(|c| c.is_ascii_hexdigit())
        }
        Some("bitcoin") => {
            // Bitcoin addresses: bech32 (bc1...) or legacy/base58
            address.starts_with("bc1")
                || address.starts_with("tb1")
                || address.len() >= 26 && address.len() <= 35
        }
        Some("solana") => {
            // Solana addresses: base58, typically 32-44 characters
            address.len() >= 32 && address.len() <= 44
        }
        _ => {
            // Generic validation: alphanumeric and common address characters
            address
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':')
        }
    }
}

/// Validate amount input
/// 验证金额输入
#[allow(dead_code)] // 安全工具函数，用于未来 UI 开发
pub fn validate_amount(amount: &str) -> bool {
    if amount.is_empty() {
        return false;
    }

    // Allow decimal numbers with optional decimal point
    let parts: Vec<&str> = amount.split('.').collect();
    if parts.len() > 2 {
        return false;
    }

    parts
        .iter()
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

/// Sanitize and validate numeric input
/// 清理并验证数字输入
#[allow(dead_code)] // 安全工具函数，用于未来 UI 开发
pub fn sanitize_numeric(input: &str) -> Option<String> {
    let cleaned: String = input
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

/// Validate mnemonic phrase
/// 验证助记词短语
#[allow(dead_code)] // 安全工具函数，用于未来 UI 开发
pub fn validate_mnemonic(mnemonic: &str) -> bool {
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    // BIP39 standard: 12, 15, 18, 21, or 24 words
    matches!(words.len(), 12 | 15 | 18 | 21 | 24)
}

/// Escape special characters for display
/// 转义特殊字符用于显示
pub fn escape_for_display(input: &str) -> String {
    sanitize_html(input)
}

/// Validate URL for safe navigation
/// 验证 URL 用于安全导航
#[allow(dead_code)] // 安全工具函数，用于未来 UI 开发
pub fn validate_url(url: &str) -> bool {
    if url.is_empty() || url.len() > 2048 {
        return false;
    }

    // Only allow http/https URLs
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return false;
    }

    // Basic URL format validation
    // For WASM environment, use web_sys::Url
    use web_sys::Url;
    Url::new(url).is_ok()
}

/// Sanitize QR code data to prevent injection
/// 清理二维码数据以防止注入
pub fn sanitize_qr_data(data: &str) -> String {
    // QR codes should only contain printable ASCII characters
    data.chars()
        .filter(|c| c.is_ascii() && !c.is_control())
        .take(1000) // Limit length
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html() {
        assert_eq!(
            sanitize_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;"
        );
    }

    #[test]
    fn test_validate_address() {
        assert!(validate_address(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
            Some("ethereum")
        ));
        assert!(!validate_address("<script>", Some("ethereum")));
    }

    #[test]
    fn test_validate_amount() {
        assert!(validate_amount("100.5"));
        assert!(validate_amount("0.001"));
        assert!(!validate_amount("abc"));
        assert!(!validate_amount(""));
    }
}
