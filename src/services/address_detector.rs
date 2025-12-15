//! Address Detector - 地址链类型检测服务
//! 自动检测地址所属的区块链网络

use anyhow::{anyhow, Result};
use bs58;

/// 支持的链类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum ChainType {
    Ethereum,
    Bitcoin,
    Solana,
    TON,
    BSC,
    Polygon,
}

impl ChainType {
    /// 获取链的显示名称
    pub fn label(&self) -> &'static str {
        match self {
            ChainType::Ethereum => "Ethereum",
            ChainType::Bitcoin => "Bitcoin",
            ChainType::Solana => "Solana",
            ChainType::TON => "TON",
            ChainType::BSC => "BSC",
            ChainType::Polygon => "Polygon",
        }
    }

    /// 转换为字符串（用于API调用）
    pub fn as_str(&self) -> &'static str {
        match self {
            ChainType::Ethereum => "ethereum",
            ChainType::Bitcoin => "bitcoin",
            ChainType::Solana => "solana",
            ChainType::TON => "ton",
            ChainType::BSC => "bsc",
            ChainType::Polygon => "polygon",
        }
    }

    /// 从字符串创建
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" => Some(ChainType::Ethereum),
            "bitcoin" | "btc" => Some(ChainType::Bitcoin),
            "solana" | "sol" => Some(ChainType::Solana),
            "ton" => Some(ChainType::TON),
            "bsc" | "binance" => Some(ChainType::BSC),
            "polygon" | "matic" => Some(ChainType::Polygon),
            _ => None,
        }
    }

    /// 获取链的原生代币符号
    pub fn native_token_symbol(&self) -> &'static str {
        match self {
            ChainType::Ethereum => "ETH",
            ChainType::Bitcoin => "BTC",
            ChainType::Solana => "SOL",
            ChainType::TON => "TON",
            ChainType::BSC => "BNB",
            ChainType::Polygon => "MATIC",
        }
    }
}

/// 地址检测器
pub struct AddressDetector;

impl AddressDetector {
    /// 检测地址所属的链类型
    ///
    /// # 支持的地址格式
    /// - Ethereum/BSC/Polygon: 0x开头，42字符
    /// - Bitcoin: bc1/1/3开头
    /// - TON: 0:或EQ开头
    /// - Solana: Base58编码，32-44字符
    pub fn detect_chain(address: &str) -> Result<ChainType> {
        let address = address.trim();

        if address.is_empty() {
            return Err(anyhow!("地址不能为空"));
        }

        // Ethereum/BSC/Polygon (EVM链)
        // 格式: 0x + 40个十六进制字符 = 42字符
        if address.starts_with("0x") && address.len() == 42 {
            // 验证是否为有效的十六进制
            if address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                // 默认返回Ethereum，实际使用时可以通过RPC进一步区分
                return Ok(ChainType::Ethereum);
            }
        }

        // Bitcoin
        // Legacy: 1开头
        // SegWit: bc1开头 (主网) 或 tb1开头 (测试网)
        // Taproot: bc1p开头
        if address.starts_with("bc1")
            || address.starts_with("tb1")
            || address.starts_with("1")
            || address.starts_with("3")
        {
            return Ok(ChainType::Bitcoin);
        }

        // TON
        // 格式: 0:开头 或 EQ开头
        if address.starts_with("0:") || address.starts_with("EQ") {
            return Ok(ChainType::TON);
        }

        // Solana (Base58编码)
        // 长度通常在32-44字符之间
        if address.len() >= 32 && address.len() <= 44 {
            if let Ok(decoded) = bs58::decode(address).into_vec() {
                // Solana地址解码后通常是32字节
                if decoded.len() == 32 {
                    return Ok(ChainType::Solana);
                }
            }
        }

        Err(anyhow!("无法识别的地址格式: {}", address))
    }

    /// 验证地址格式是否正确
    pub fn validate_address(address: &str, chain: ChainType) -> Result<()> {
        let detected = Self::detect_chain(address)?;

        // EVM链之间可以互相兼容（地址格式相同）
        let evm_chains = [ChainType::Ethereum, ChainType::BSC, ChainType::Polygon];
        let is_evm_chain = evm_chains.contains(&chain);
        let is_evm_detected = evm_chains.contains(&detected);

        if is_evm_chain && is_evm_detected {
            return Ok(()); // EVM链地址格式兼容
        }

        if detected == chain {
            Ok(())
        } else {
            Err(anyhow!(
                "地址格式不匹配: 检测到 {}, 但期望 {}",
                detected.label(),
                chain.label()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ethereum() {
        let addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb6";
        assert_eq!(
            AddressDetector::detect_chain(addr).unwrap(),
            ChainType::Ethereum
        );
    }

    #[test]
    fn test_detect_bitcoin() {
        let addr = "bc1qqp3kzgvpvvp6xc5qsqqcxcppy9pzzsurqzqgrgfzcfsuyc8rsypsjxgll6";
        assert_eq!(
            AddressDetector::detect_chain(addr).unwrap(),
            ChainType::Bitcoin
        );
    }

    #[test]
    fn test_detect_solana() {
        let addr = "2DW3219WuFwqLQqdFmkPa6bFL9pKj4LeG2GG8gDsHcGn";
        assert_eq!(
            AddressDetector::detect_chain(addr).unwrap(),
            ChainType::Solana
        );
    }

    #[test]
    fn test_detect_ton() {
        let addr = "0:60bcb52d2c0e92eab79dc0e5e9d1b6fb1da2b45815e8136f56507ad3d33a081a";
        assert_eq!(AddressDetector::detect_chain(addr).unwrap(), ChainType::TON);
    }
}
