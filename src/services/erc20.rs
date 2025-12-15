//! ERC-20 Token Transfer - 企业级ERC-20代币转账服务
//! 提供ERC-20代币转账的编码和签名功能

use anyhow::{anyhow, Result};
use hex;

/// ERC-20 transfer函数选择器
/// function transfer(address to, uint256 amount) returns (bool)
/// 函数签名: transfer(address,uint256)
/// 选择器: 0xa9059cbb (前4字节)
const ERC20_TRANSFER_SELECTOR: &[u8] = &[0xa9, 0x05, 0x9c, 0xbb];

/// ERC-20代币转账编码器
pub struct Erc20Encoder;

impl Erc20Encoder {
    /// 编码ERC-20 transfer函数调用
    ///
    /// # 参数
    /// - `to`: 接收地址（20字节，0x开头）
    /// - `amount`: 转账金额（最小单位，考虑decimals）
    ///
    /// # 返回
    /// 编码后的calldata（十六进制字符串）
    pub fn encode_transfer(to: &str, amount: &str) -> Result<String> {
        // 1. 函数选择器（4字节）
        let mut calldata = ERC20_TRANSFER_SELECTOR.to_vec();

        // 2. 编码接收地址（32字节，右对齐）
        let to_address = Self::parse_address(to)?;
        let mut to_padded = vec![0u8; 12]; // 前12字节填充0
        to_padded.extend_from_slice(&to_address);
        calldata.extend_from_slice(&to_padded);

        // 3. 编码金额（32字节，大端序）
        let _amount_u256 = Self::parse_amount(amount)?;
        let amount_bytes = Self::u256_to_bytes(amount);
        calldata.extend_from_slice(&amount_bytes);

        // 4. 转换为十六进制字符串
        Ok(format!("0x{}", hex::encode(calldata)))
    }

    /// 解析地址（去除0x前缀，验证长度）
    fn parse_address(address: &str) -> Result<[u8; 20]> {
        let addr_clean = address.trim_start_matches("0x");
        if addr_clean.len() != 40 {
            return Err(anyhow!("地址长度无效: {}", address));
        }

        let bytes = hex::decode(addr_clean).map_err(|e| anyhow!("地址格式无效: {}", e))?;

        if bytes.len() != 20 {
            return Err(anyhow!("地址字节长度无效: {}", bytes.len()));
        }

        let mut result = [0u8; 20];
        result.copy_from_slice(&bytes);
        Ok(result)
    }

    /// 解析金额（字符串转u256）
    fn parse_amount(amount: &str) -> Result<[u8; 32]> {
        // 移除可能的0x前缀
        let amount_clean = amount.trim_start_matches("0x");

        // 尝试解析为u128（对于大多数代币足够）
        let amount_u128 = amount_clean
            .parse::<u128>()
            .map_err(|e| anyhow!("金额格式无效: {}", e))?;

        // 转换为32字节大端序
        Ok(Self::u128_to_u256_bytes(amount_u128))
    }

    /// 将u128转换为32字节大端序
    fn u128_to_u256_bytes(value: u128) -> [u8; 32] {
        let mut result = [0u8; 32];
        let bytes = value.to_be_bytes();
        // u128是16字节，放在后16字节
        result[16..].copy_from_slice(&bytes);
        result
    }

    /// 将u256（字符串）转换为32字节
    fn u256_to_bytes(value: &str) -> [u8; 32] {
        // 简化实现：假设value是u128范围内的字符串
        if let Ok(u128_val) = value.parse::<u128>() {
            return Self::u128_to_u256_bytes(u128_val);
        }

        // 如果是十六进制
        if let Some(hex_val) = value.strip_prefix("0x") {
            if let Ok(hex_bytes) = hex::decode(hex_val) {
                if hex_bytes.len() <= 32 {
                    let mut result = [0u8; 32];
                    let start = 32 - hex_bytes.len();
                    result[start..].copy_from_slice(&hex_bytes);
                    return result;
                }
            }
        }

        // 默认返回0
        [0u8; 32]
    }

    /// 计算代币金额（考虑decimals）
    ///
    /// # 参数
    /// - `amount`: 用户输入的金额（格式化后的，如1.5）
    /// - `decimals`: 代币精度（如6表示USDT，18表示ETH）
    ///
    /// # 返回
    /// 最小单位的金额（字符串）
    pub fn calculate_token_amount(amount: f64, decimals: u8) -> Result<String> {
        if amount < 0.0 {
            return Err(anyhow!("金额不能为负数"));
        }

        if !amount.is_finite() {
            return Err(anyhow!("金额必须是有效数字"));
        }

        // 使用字符串操作避免浮点数精度问题
        let amount_str = format!("{:.18}", amount);
        let parts: Vec<&str> = amount_str.split('.').collect();

        if parts.len() == 1 {
            // 整数部分
            let integer_part = parts[0]
                .parse::<u128>()
                .map_err(|e| anyhow!("解析整数部分失败: {}", e))?;
            let multiplier = 10u128.pow(decimals as u32);
            Ok((integer_part * multiplier).to_string())
        } else {
            // 有小数部分
            let integer_part = parts[0]
                .parse::<u128>()
                .map_err(|e| anyhow!("解析整数部分失败: {}", e))?;
            let decimal_part = parts[1];

            // 确保小数部分不超过decimals位
            let decimal_str = if decimal_part.len() > decimals as usize {
                &decimal_part[..decimals as usize]
            } else {
                decimal_part
            };

            // 补齐到decimals位
            let decimal_padded = format!("{:0<width$}", decimal_str, width = decimals as usize);
            let decimal_amount = decimal_padded
                .parse::<u128>()
                .map_err(|e| anyhow!("解析小数部分失败: {}", e))?;

            let multiplier = 10u128.pow(decimals as u32);
            Ok((integer_part * multiplier + decimal_amount).to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_transfer() {
        let to = "0x742d35Cc6634C0532925a3b844Bc9e8Ef5bEd1e1";
        let amount = "1000000"; // 1 USDT (6 decimals)

        let calldata = Erc20Encoder::encode_transfer(to, amount).unwrap();

        // 验证格式
        assert!(calldata.starts_with("0x"));
        assert_eq!(calldata.len(), 2 + 8 + 64 + 64); // 0x + selector(8 hex chars) + address(64) + amount(64)
    }

    #[test]
    fn test_calculate_token_amount() {
        // USDT (6 decimals)
        let result = Erc20Encoder::calculate_token_amount(1.5, 6).unwrap();
        assert_eq!(result, "1500000");

        // ETH (18 decimals)
        let result = Erc20Encoder::calculate_token_amount(1.5, 18).unwrap();
        assert_eq!(result, "1500000000000000000");
    }
}
