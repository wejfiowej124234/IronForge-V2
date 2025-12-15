//! Input Validation - 输入验证模块
//! 提供完整的输入验证功能，确保数据安全

use crate::services::address_detector::{AddressDetector, ChainType};
use anyhow::{anyhow, Result};

/// 输入验证器
pub struct PaymentValidator;

impl PaymentValidator {
    /// 验证金额
    ///
    /// # 参数
    /// - `amount`: 金额字符串
    ///
    /// # 返回
    /// 解析后的金额（f64）
    pub fn validate_amount(amount: &str) -> Result<f64> {
        if amount.trim().is_empty() {
            return Err(anyhow!("金额不能为空"));
        }

        let amount_val: f64 = amount
            .parse()
            .map_err(|e| anyhow!("金额格式错误: {} (输入: {})", e, amount))?;

        if amount_val <= 0.0 {
            return Err(anyhow!("金额必须大于0"));
        }

        if !amount_val.is_finite() {
            return Err(anyhow!("金额必须是有效数字"));
        }

        if amount_val > 1_000_000_000.0 {
            return Err(anyhow!("金额过大，请检查输入（最大: 1,000,000,000）"));
        }

        Ok(amount_val)
    }

    /// 验证地址
    ///
    /// # 参数
    /// - `address`: 地址字符串
    /// - `chain`: 链类型（可选，如果提供则验证地址格式是否匹配）
    ///
    /// # 返回
    /// 检测到的链类型
    pub fn validate_address(address: &str, expected_chain: Option<ChainType>) -> Result<ChainType> {
        if address.trim().is_empty() {
            return Err(anyhow!("地址不能为空"));
        }

        let detected_chain = AddressDetector::detect_chain(address)?;

        if let Some(expected) = expected_chain {
            AddressDetector::validate_address(address, expected)?;
        }

        Ok(detected_chain)
    }

    /// 验证地址格式（不检测链类型）
    pub fn validate_address_format(address: &str, chain: ChainType) -> Result<()> {
        AddressDetector::validate_address(address, chain)
    }

    /// 验证金额范围
    pub fn validate_amount_range(amount: f64, min: f64, max: f64) -> Result<()> {
        if amount < min {
            return Err(anyhow!("金额不能小于 {}", min));
        }
        if amount > max {
            return Err(anyhow!("金额不能大于 {}", max));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_amount_valid() {
        assert!(PaymentValidator::validate_amount("100.5").is_ok());
        assert!(PaymentValidator::validate_amount("0.001").is_ok());
    }

    #[test]
    fn test_validate_amount_invalid() {
        assert!(PaymentValidator::validate_amount("").is_err());
        assert!(PaymentValidator::validate_amount("0").is_err());
        assert!(PaymentValidator::validate_amount("-10").is_err());
        assert!(PaymentValidator::validate_amount("abc").is_err());
    }

    #[test]
    fn test_validate_address() {
        let eth_addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb6";
        assert!(PaymentValidator::validate_address(eth_addr, None).is_ok());
        assert!(PaymentValidator::validate_address(eth_addr, Some(ChainType::Ethereum)).is_ok());
    }
}
