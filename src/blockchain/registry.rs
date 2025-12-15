use crate::blockchain::bitcoin::BitcoinAdapter;
use crate::blockchain::ethereum::EthereumAdapter;
use crate::blockchain::solana::SolanaAdapter;
use crate::blockchain::ton::TonAdapter;
use crate::blockchain::traits::ChainAdapter;
use anyhow::Result;

/// 链注册表
/// 为未来扩展准备的链适配器注册表
#[allow(dead_code)] // 为未来扩展准备
pub struct ChainRegistry;

impl ChainRegistry {
    pub fn get_adapter(chain: &str) -> Result<Box<dyn ChainAdapter>> {
        match chain.to_lowercase().as_str() {
            "ethereum" | "eth" => Ok(Box::new(EthereumAdapter::new(
                vec![
                    "https://cloudflare-eth.com".to_string(),
                    "https://eth.llamarpc.com".to_string(),
                ],
                1,
            ))),
            "sepolia" => Ok(Box::new(EthereumAdapter::new(
                vec!["https://rpc.sepolia.org".to_string()],
                11155111,
            ))),
            "bitcoin" | "btc" => Ok(Box::new(BitcoinAdapter::new(
                vec![
                    "https://blockstream.info/api".to_string(),
                    "https://mempool.space/api".to_string(),
                ],
                "mainnet".to_string(),
            ))),
            "bitcoin_testnet" | "btc_test" => Ok(Box::new(BitcoinAdapter::new(
                vec!["https://blockstream.info/testnet/api".to_string()],
                "testnet".to_string(),
            ))),
            "solana" | "sol" => Ok(Box::new(SolanaAdapter::new(
                vec!["https://api.mainnet-beta.solana.com".to_string()],
                "mainnet-beta".to_string(),
            ))),
            "solana_devnet" => Ok(Box::new(SolanaAdapter::new(
                vec!["https://api.devnet.solana.com".to_string()],
                "devnet".to_string(),
            ))),
            "ton" => Ok(Box::new(TonAdapter::new(
                vec!["https://toncenter.com/api/v2/jsonRPC".to_string()],
                1,
            ))),
            _ => Err(anyhow::anyhow!("Unsupported chain: {}", chain)),
        }
    }
}
