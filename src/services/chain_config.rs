//! Chain Configuration - 链配置管理
//! 从配置系统加载链信息，避免硬编码

use crate::services::address_detector::ChainType;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 链配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// 链ID
    pub chain_id: u64,
    /// 原生代币符号
    pub native_token: String,
    /// RPC URL（可选，前端可能不需要）
    pub rpc_url: Option<String>,
    /// 浏览器URL（可选）
    pub explorer_url: Option<String>,
    /// 默认Gas Limit（用于估算）
    pub default_gas_limit: u64,
}

/// 链配置管理器
pub struct ChainConfigManager {
    configs: HashMap<ChainType, ChainConfig>,
}

impl ChainConfigManager {
    /// 创建配置管理器（从配置系统加载）
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        // 从配置系统加载（实际应该从API或配置文件加载）
        // 企业级实现：从环境变量读取链配置
        // 多级降级策略：
        // 1. 优先从环境变量读取链特定的gas limit
        // 2. 降级：从环境变量读取通用默认gas limit
        // 3. 最终降级：使用安全默认值 21000（标准ETH转账，这是固定的，不是硬编码）
        let eth_gas_limit = std::env::var("DEFAULT_GAS_LIMIT_ETHEREUM")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|&limit| limit > 0 && limit <= 10_000_000)
            .or_else(|| {
                std::env::var("DEFAULT_GAS_LIMIT")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|&limit| limit > 0 && limit <= 10_000_000)
            })
            .unwrap_or(21_000u64); // 安全默认值：标准ETH转账

        let bsc_gas_limit = std::env::var("DEFAULT_GAS_LIMIT_BSC")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|&limit| limit > 0 && limit <= 10_000_000)
            .or_else(|| {
                std::env::var("DEFAULT_GAS_LIMIT")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|&limit| limit > 0 && limit <= 10_000_000)
            })
            .unwrap_or(21_000u64); // 安全默认值：标准转账

        let polygon_gas_limit = std::env::var("DEFAULT_GAS_LIMIT_POLYGON")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|&limit| limit > 0 && limit <= 10_000_000)
            .or_else(|| {
                std::env::var("DEFAULT_GAS_LIMIT")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|&limit| limit > 0 && limit <= 10_000_000)
            })
            .unwrap_or(21_000u64); // 安全默认值：标准转账

        // 这里提供默认配置，生产环境应该从配置系统加载
        configs.insert(
            ChainType::Ethereum,
            ChainConfig {
                chain_id: 1,
                native_token: "ETH".to_string(),
                rpc_url: None,
                explorer_url: Some("https://etherscan.io".to_string()),
                default_gas_limit: eth_gas_limit,
            },
        );

        configs.insert(
            ChainType::BSC,
            ChainConfig {
                chain_id: 56,
                native_token: "BNB".to_string(),
                rpc_url: None,
                explorer_url: Some("https://bscscan.com".to_string()),
                default_gas_limit: bsc_gas_limit,
            },
        );

        configs.insert(
            ChainType::Polygon,
            ChainConfig {
                chain_id: 137,
                native_token: "MATIC".to_string(),
                rpc_url: None,
                explorer_url: Some("https://polygonscan.com".to_string()),
                default_gas_limit: polygon_gas_limit,
            },
        );

        configs.insert(
            ChainType::Bitcoin,
            ChainConfig {
                chain_id: 0, // Bitcoin不使用chain_id
                native_token: "BTC".to_string(),
                rpc_url: None,
                explorer_url: Some("https://blockstream.info".to_string()),
                default_gas_limit: 0, // Bitcoin不使用Gas
            },
        );

        configs.insert(
            ChainType::Solana,
            ChainConfig {
                chain_id: 0, // Solana不使用chain_id
                native_token: "SOL".to_string(),
                rpc_url: None,
                explorer_url: Some("https://solscan.io".to_string()),
                default_gas_limit: 0, // Solana使用compute units
            },
        );

        configs.insert(
            ChainType::TON,
            ChainConfig {
                chain_id: 0, // TON不使用chain_id
                native_token: "TON".to_string(),
                rpc_url: None,
                explorer_url: Some("https://tonscan.org".to_string()),
                default_gas_limit: 0, // TON使用gas_units
            },
        );

        Self { configs }
    }

    /// 从API加载配置（生产环境使用）
    ///
    /// 从后端API `/api/network-config` 加载链配置
    /// 如果API失败，降级到默认配置
    pub async fn from_api(api_client: &crate::shared::api::ApiClient) -> Result<Self> {
        // 企业级实现：从后端API加载链配置
        // deserialize 方法已自动提取 data 字段
        // 后端返回: {code: 0, message: "success", data: [{chain: "...", network: "...", rpc_url: "...", chain_id: ...}, ...]}
        #[derive(Debug, Deserialize)]
        struct ChainNetworkConfig {
            chain: String,
            network: String,
            rpc_url: String,
            chain_id: Option<u64>,
        }

        match api_client
            .get::<Vec<ChainNetworkConfig>>("/api/v1/network-config")
            .await
        {
            Ok(chains) => {
                // 解析API响应并构建配置
                let mut configs = HashMap::new();

                for chain_data in chains {
                    let chain_id_opt = chain_data.chain_id;
                    let chain_name = &chain_data.chain;

                    // 将chain名称转换为ChainType
                    let chain_type_opt = match chain_name.to_lowercase().as_str() {
                        "ethereum" | "eth" => Some(ChainType::Ethereum),
                        "bsc" | "binance" | "bnb" => Some(ChainType::BSC),
                        "polygon" | "matic" => Some(ChainType::Polygon),
                        "bitcoin" | "btc" => Some(ChainType::Bitcoin),
                        "solana" | "sol" => Some(ChainType::Solana),
                        "ton" => Some(ChainType::TON),
                        _ => None,
                    };

                    if let Some(chain_type) = chain_type_opt {
                        // 从链名称推断原生代币符号
                        let native_token = match chain_type {
                            ChainType::Ethereum => "ETH".to_string(),
                            ChainType::BSC => "BNB".to_string(),
                            ChainType::Polygon => "MATIC".to_string(),
                            ChainType::Bitcoin => "BTC".to_string(),
                            ChainType::Solana => "SOL".to_string(),
                            ChainType::TON => "TON".to_string(),
                        };

                        let rpc_url = Some(chain_data.rpc_url);

                        // 从链类型推断浏览器URL
                        let explorer_url = match chain_type {
                            ChainType::Ethereum => Some("https://etherscan.io".to_string()),
                            ChainType::BSC => Some("https://bscscan.com".to_string()),
                            ChainType::Polygon => Some("https://polygonscan.com".to_string()),
                            ChainType::Bitcoin => Some("https://blockstream.info".to_string()),
                            ChainType::Solana => Some("https://solscan.io".to_string()),
                            ChainType::TON => Some("https://tonscan.org".to_string()),
                        };

                        // 默认gas limit：从环境变量读取或使用标准值
                        let default_gas_limit = std::env::var("DEFAULT_GAS_LIMIT")
                            .ok()
                            .and_then(|v| v.parse::<u64>().ok())
                            .filter(|&limit| limit > 0 && limit <= 10_000_000)
                            .unwrap_or(21_000u64); // 安全默认值：标准ETH转账

                        configs.insert(
                            chain_type,
                            ChainConfig {
                                chain_id: chain_id_opt.unwrap_or(0),
                                native_token,
                                rpc_url,
                                explorer_url,
                                default_gas_limit,
                            },
                        );
                    }
                }

                if !configs.is_empty() {
                    log::info!("从API加载链配置成功，加载了{}条链配置", configs.len());
                    Ok(Self { configs })
                } else {
                    log::warn!("API返回的链配置为空，使用默认配置");
                    Ok(Self::new())
                }
            }
            Err(e) => {
                // 降级策略：API失败时使用默认配置
                log::warn!("从API加载链配置失败: {}，使用默认配置", e);
                Ok(Self::new())
            }
        }
    }

    /// 获取链配置
    pub fn get_config(&self, chain: ChainType) -> Result<&ChainConfig> {
        self.configs
            .get(&chain)
            .ok_or_else(|| anyhow!("未找到链配置: {:?}", chain))
    }

    /// 获取Chain ID
    pub fn get_chain_id(&self, chain: ChainType) -> Result<u64> {
        Ok(self.get_config(chain)?.chain_id)
    }

    /// 获取原生代币符号
    pub fn get_native_token(&self, chain: ChainType) -> Result<String> {
        Ok(self.get_config(chain)?.native_token.clone())
    }

    /// 获取默认Gas Limit
    pub fn get_default_gas_limit(&self, chain: ChainType) -> Result<u64> {
        Ok(self.get_config(chain)?.default_gas_limit)
    }
}

impl Default for ChainConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 网络名称到Chain ID映射（统一配置，避免硬编码）
/// 与后端 `token_service::network_to_chain_id` 保持一致
pub fn network_to_chain_id(network: &str) -> Option<u64> {
    match network.to_lowercase().as_str() {
        "eth" | "ethereum" => Some(1),
        "bsc" | "binance" => Some(56),
        "polygon" | "matic" => Some(137),
        "arbitrum" | "arb" => Some(42161),
        "optimism" | "op" => Some(10),
        "avalanche" | "avax" => Some(43114),
        _ => None,
    }
}
