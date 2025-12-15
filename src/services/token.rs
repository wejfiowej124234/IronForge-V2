//! Token Service - 企业级代币服务
//! 提供代币信息查询、余额查询、代币列表等功能

use crate::services::address_detector::ChainType;
use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 代币信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenInfo {
    /// 代币合约地址（原生代币使用特殊地址）
    pub address: String,
    /// 代币符号（如 USDT, USDC）
    pub symbol: String,
    /// 代币名称（如 Tether USD）
    pub name: String,
    /// 代币精度（小数位数）
    pub decimals: u8,
    /// 所属链
    pub chain: ChainType,
    /// 代币图标URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    /// 是否为原生代币
    pub is_native: bool,
}

/// 代币余额
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenBalance {
    /// 代币信息
    pub token: TokenInfo,
    /// 余额（最小单位，字符串格式避免精度丢失）
    pub balance_raw: String,
    /// 格式化余额（考虑decimals）
    pub balance_formatted: f64,
}

/// 代币服务
#[derive(Clone)]
pub struct TokenService {
    api_client: Arc<ApiClient>,
}

impl TokenService {
    /// 创建TokenService实例
    pub fn new(app_state: AppState) -> Self {
        Self {
            api_client: Arc::new(app_state.get_api_client()),
        }
    }

    /// 获取链上支持的代币列表
    ///
    /// # 参数
    /// - `chain`: 链类型
    ///
    /// # 返回
    /// 代币列表（包含原生代币和常见ERC-20代币）
    pub async fn get_token_list(&self, chain: ChainType) -> Result<Vec<TokenInfo>> {
        // 首先尝试从后端API获取
        match self.get_token_list_from_api(chain).await {
            Ok(tokens) => Ok(tokens),
            Err(e) => {
                log::warn!("从API获取代币列表失败: {}，使用默认列表", e);
                // 降级：使用内置的默认代币列表
                Ok(Self::get_default_token_list(chain))
            }
        }
    }

    /// 从后端API获取代币列表
    async fn get_token_list_from_api(&self, chain: ChainType) -> Result<Vec<TokenInfo>> {
        let chain_str = chain.as_str();
        let path = format!("/api/v1/tokens/list?chain={}", chain_str);

        // deserialize 方法已自动提取 data 字段
        self.api_client
            .get(&path)
            .await
            .map_err(|e| anyhow!("获取代币列表API调用失败: {}", e))
    }

    /// 获取默认代币列表（降级方案，仅在API失败时使用）
    /// ✅ 行业标准：即使API失败，也应显示常用代币列表
    fn get_default_token_list(chain: ChainType) -> Vec<TokenInfo> {
        let mut tokens = Vec::new();

        match chain {
            ChainType::Ethereum => {
                // ETH主网常用代币（Uniswap/1inch标准）
                tokens.extend(vec![
                    TokenInfo {
                        address: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE".to_string(),
                        symbol: "ETH".to_string(),
                        name: "Ethereum".to_string(),
                        decimals: 18,
                        chain,
                        logo_url: None,
                        is_native: true,
                    },
                    TokenInfo {
                        address: "0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(),
                        symbol: "USDT".to_string(),
                        name: "Tether USD".to_string(),
                        decimals: 6,
                        chain,
                        logo_url: None,
                        is_native: false,
                    },
                    TokenInfo {
                        address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
                        symbol: "USDC".to_string(),
                        name: "USD Coin".to_string(),
                        decimals: 6,
                        chain,
                        logo_url: None,
                        is_native: false,
                    },
                    TokenInfo {
                        address: "0x6b175474e89094c44da98b954eedeac495271d0f".to_string(),
                        symbol: "DAI".to_string(),
                        name: "Dai Stablecoin".to_string(),
                        decimals: 18,
                        chain,
                        logo_url: None,
                        is_native: false,
                    },
                    TokenInfo {
                        address: "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599".to_string(),
                        symbol: "WBTC".to_string(),
                        name: "Wrapped Bitcoin".to_string(),
                        decimals: 8,
                        chain,
                        logo_url: None,
                        is_native: false,
                    },
                ]);
            }
            ChainType::BSC => {
                // BSC常用代币
                tokens.extend(vec![
                    TokenInfo {
                        address: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE".to_string(),
                        symbol: "BNB".to_string(),
                        name: "BNB".to_string(),
                        decimals: 18,
                        chain,
                        logo_url: None,
                        is_native: true,
                    },
                    TokenInfo {
                        address: "0x55d398326f99059ff775485246999027b3197955".to_string(),
                        symbol: "USDT".to_string(),
                        name: "Tether USD".to_string(),
                        decimals: 18,
                        chain,
                        logo_url: None,
                        is_native: false,
                    },
                    TokenInfo {
                        address: "0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d".to_string(),
                        symbol: "USDC".to_string(),
                        name: "USD Coin".to_string(),
                        decimals: 18,
                        chain,
                        logo_url: None,
                        is_native: false,
                    },
                ]);
            }
            ChainType::Polygon => {
                // Polygon常用代币
                tokens.extend(vec![
                    TokenInfo {
                        address: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE".to_string(),
                        symbol: "MATIC".to_string(),
                        name: "Polygon".to_string(),
                        decimals: 18,
                        chain,
                        logo_url: None,
                        is_native: true,
                    },
                    TokenInfo {
                        address: "0xc2132d05d31c914a87c6611c10748aeb04b58e8f".to_string(),
                        symbol: "USDT".to_string(),
                        name: "Tether USD".to_string(),
                        decimals: 6,
                        chain,
                        logo_url: None,
                        is_native: false,
                    },
                    TokenInfo {
                        address: "0x2791bca1f2de4661ed88a30c99a7a9449aa84174".to_string(),
                        symbol: "USDC".to_string(),
                        name: "USD Coin".to_string(),
                        decimals: 6,
                        chain,
                        logo_url: None,
                        is_native: false,
                    },
                ]);
            }
            _ => {
                // 其他链：只返回原生代币
                if let Ok(native_symbol) =
                    crate::services::chain_config::ChainConfigManager::new().get_native_token(chain)
                {
                    tokens.push(TokenInfo {
                        address: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE".to_string(),
                        symbol: native_symbol.clone(),
                        name: format!("{} Native Token", native_symbol),
                        decimals: 18,
                        chain,
                        logo_url: None,
                        is_native: true,
                    });
                }
            }
        }

        log::warn!(
            "使用降级代币列表（{}个代币），建议检查后端API连接",
            tokens.len()
        );
        tokens
    }

    /// 获取代币信息
    ///
    /// # 参数
    /// - `chain`: 链类型
    /// - `token_address`: 代币合约地址
    ///
    /// # 返回
    /// 代币信息
    pub async fn get_token_info(&self, chain: ChainType, token_address: &str) -> Result<TokenInfo> {
        // 检查是否为原生代币
        if Self::is_native_token_address(token_address) {
            return Self::get_native_token_info(chain);
        }

        // 尝试从API获取
        match self.get_token_info_from_api(chain, token_address).await {
            Ok(info) => Ok(info),
            Err(e) => {
                log::warn!("从API获取代币信息失败: {}，使用默认信息", e);
                // 降级：从默认列表中查找
                let default_list = Self::get_default_token_list(chain);
                default_list
                    .into_iter()
                    .find(|t| t.address.eq_ignore_ascii_case(token_address))
                    .ok_or_else(|| anyhow!("未找到代币信息: {}", token_address))
            }
        }
    }

    /// 从后端API获取代币信息
    async fn get_token_info_from_api(
        &self,
        chain: ChainType,
        token_address: &str,
    ) -> Result<TokenInfo> {
        let chain_str = chain.as_str();
        let path = format!("/api/v1/tokens/{}/info?chain={}", token_address, chain_str);

        // deserialize 方法已自动提取 data 字段
        self.api_client
            .get(&path)
            .await
            .map_err(|e| anyhow!("获取代币信息API调用失败: {}", e))
    }

    /// 获取原生代币信息
    fn get_native_token_info(chain: ChainType) -> Result<TokenInfo> {
        use crate::services::chain_config::ChainConfigManager;
        let config_manager = ChainConfigManager::new();
        let symbol = config_manager.get_native_token(chain)?;

        Ok(TokenInfo {
            address: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE".to_string(),
            symbol: symbol.clone(),
            name: format!("{} Native Token", symbol),
            decimals: 18,
            chain,
            logo_url: None,
            is_native: true,
        })
    }

    /// 检查是否为原生代币地址
    fn is_native_token_address(address: &str) -> bool {
        address.eq_ignore_ascii_case("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE")
            || address.eq_ignore_ascii_case("native")
            || address.is_empty()
    }

    /// 获取代币余额
    ///
    /// # 参数
    /// - `chain`: 链类型
    /// - `token_address`: 代币合约地址（原生代币使用特殊地址）
    /// - `wallet_address`: 钱包地址
    ///
    /// # 返回
    /// 代币余额
    pub async fn get_token_balance(
        &self,
        chain: ChainType,
        token_address: &str,
        wallet_address: &str,
    ) -> Result<TokenBalance> {
        // 获取代币信息
        let token_info = self.get_token_info(chain, token_address).await?;

        // 如果是原生代币，从账户余额获取
        if token_info.is_native {
            return self.get_native_token_balance(chain, wallet_address).await;
        }

        // ERC-20代币，从API获取
        self.get_erc20_token_balance(chain, token_address, wallet_address, &token_info)
            .await
    }

    /// 获取原生代币余额
    async fn get_native_token_balance(
        &self,
        _chain: ChainType,
        _wallet_address: &str,
    ) -> Result<TokenBalance> {
        // 原生代币余额应该从账户信息中获取
        // 这里返回一个占位符，实际应该从Account.balance获取
        // 在UI层会从Account对象获取余额
        let token_info = Self::get_native_token_info(_chain)?;
        Ok(TokenBalance {
            token: token_info,
            balance_raw: "0".to_string(),
            balance_formatted: 0.0,
        })
    }

    /// 获取ERC-20代币余额
    async fn get_erc20_token_balance(
        &self,
        chain: ChainType,
        token_address: &str,
        wallet_address: &str,
        token_info: &TokenInfo,
    ) -> Result<TokenBalance> {
        #[derive(Debug, Deserialize)]
        struct TokenBalanceData {
            #[serde(default)]
            token: Option<TokenInfo>,
            balance_raw: Option<String>,
            balance: Option<String>, // 兼容旧格式
            balance_formatted: Option<f64>,
        }

        let chain_str = chain.as_str();
        let path = format!(
            "/api/v1/tokens/{}/balance?address={}&chain={}",
            token_address, wallet_address, chain_str
        );

        // deserialize 方法已自动提取 data 字段
        let data: TokenBalanceData = self
            .api_client
            .get(&path)
            .await
            .map_err(|e| anyhow!("获取代币余额API调用失败: {}", e))?;

        let balance_raw = data
            .balance_raw
            .or(data.balance)
            .unwrap_or_else(|| "0".to_string());
        let balance_formatted = data.balance_formatted.unwrap_or_else(|| {
            // 如果没有格式化余额，手动计算
            let balance_u256 = balance_raw.parse::<u128>().unwrap_or(0);
            let divisor = 10u128.pow(token_info.decimals as u32);
            balance_u256 as f64 / divisor as f64
        });

        // 使用响应中的token信息（如果有），否则使用传入的token_info
        let final_token_info = data.token.unwrap_or_else(|| token_info.clone());

        Ok(TokenBalance {
            token: final_token_info,
            balance_raw,
            balance_formatted,
        })
    }

    /// 批量获取代币余额
    ///
    /// # 参数
    /// - `chain`: 链类型
    /// - `wallet_address`: 钱包地址
    /// - `token_addresses`: 代币地址列表
    ///
    /// # 返回
    /// 代币余额列表
    pub async fn get_token_balances_batch(
        &self,
        chain: ChainType,
        wallet_address: &str,
        token_addresses: &[String],
    ) -> Result<Vec<TokenBalance>> {
        #[derive(Debug, Serialize)]
        struct BatchBalanceRequest {
            chain: String,
            wallet_address: String,
            token_addresses: Vec<String>,
        }

        let request = BatchBalanceRequest {
            chain: chain.as_str().to_string(),
            wallet_address: wallet_address.to_string(),
            token_addresses: token_addresses.to_vec(),
        };

        // deserialize 方法已自动提取 data 字段
        match self
            .api_client
            .post("/api/v1/tokens/balances", &request)
            .await
        {
            Ok(Some(response)) => Ok(response),
            Ok(None) => {
                // 降级：逐个查询
                log::warn!("批量查询返回空数据，降级为逐个查询");
                let mut balances = Vec::new();
                for token_address in token_addresses {
                    match self
                        .get_token_balance(chain, token_address, wallet_address)
                        .await
                    {
                        Ok(balance) => balances.push(balance),
                        Err(e) => log::warn!("查询代币余额失败 {}: {}", token_address, e),
                    }
                }
                Ok(balances)
            }
            Err(e) => {
                // API 调用失败，降级：逐个查询
                log::warn!("批量查询API调用失败: {}，降级为逐个查询", e);
                let mut balances = Vec::new();
                for token_address in token_addresses {
                    match self
                        .get_token_balance(chain, token_address, wallet_address)
                        .await
                    {
                        Ok(balance) => balances.push(balance),
                        Err(e) => log::warn!("查询代币余额失败 {}: {}", token_address, e),
                    }
                }
                Ok(balances)
            }
        }
    }
}
