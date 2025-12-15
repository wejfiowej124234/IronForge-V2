// Token Detection Service
// Auto-discover ERC20/SPL tokens with metadata querying and whitelist filtering

use crate::shared::error::AppError;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use wasm_bindgen_futures::spawn_local;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenMetadata {
    pub chain: String,
    pub contract_address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub logo_uri: Option<String>,
    pub verified: bool,
    pub balance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenListResponse {
    pub tokens: Vec<TokenMetadata>,
    pub total: usize,
}

#[derive(Clone, Copy)]
pub struct TokenDetectionService {
    app_state: AppState,
}

impl TokenDetectionService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Auto-detect tokens owned by an address
    ///
    /// # Arguments
    /// * `chain` - Blockchain (eth, bsc, polygon, solana)
    /// * `address` - Wallet address
    /// * `min_balance` - Minimum balance threshold (optional)
    ///
    /// # Returns
    /// List of detected tokens with metadata
    pub async fn detect_tokens(
        &self,
        chain: &str,
        address: &str,
        min_balance: Option<f64>,
    ) -> Result<Vec<TokenMetadata>, AppError> {
        let api = self.app_state.get_api_client();

        let mut url = format!("/api/v1/tokens/detect?chain={}&address={}", chain, address);

        if let Some(min_bal) = min_balance {
            url.push_str(&format!("&min_balance={}", min_bal));
        }

        let response: TokenListResponse = api.get(&url).await?;

        // Apply whitelist filter
        let filtered = self.filter_by_whitelist(response.tokens);

        Ok(filtered)
    }

    /// Get token metadata by contract address
    ///
    /// # Arguments
    /// * `chain` - Blockchain
    /// * `contract_address` - Token contract address
    ///
    /// # Returns
    /// Token metadata including name, symbol, decimals
    #[allow(dead_code)] // 用于代币检测功能
    pub async fn get_token_metadata(
        &self,
        chain: &str,
        contract_address: &str,
    ) -> Result<TokenMetadata, AppError> {
        let api = self.app_state.get_api_client();

        let url = format!(
            "/api/v1/tokens/metadata?chain={}&address={}",
            chain, contract_address
        );

        let metadata: TokenMetadata = api.get(&url).await?;

        Ok(metadata)
    }

    /// Import custom token by contract address
    ///
    /// # Arguments
    /// * `chain` - Blockchain
    /// * `contract_address` - Token contract address
    ///
    /// # Returns
    /// Token metadata if valid
    #[allow(dead_code)] // 用于代币检测功能
    pub async fn import_token(
        &self,
        chain: &str,
        contract_address: &str,
    ) -> Result<TokenMetadata, AppError> {
        // Validate contract address format
        self.validate_contract_address(chain, contract_address)?;

        // Fetch metadata
        let metadata = self.get_token_metadata(chain, contract_address).await?;

        // Verify it's a valid token contract
        if metadata.name.is_empty() || metadata.symbol.is_empty() {
            return Err(AppError::Validation(
                "Invalid token contract: missing name or symbol".into(),
            ));
        }

        Ok(metadata)
    }

    /// Get user's token balances
    ///
    /// # Arguments
    /// * `chain` - Blockchain
    /// * `address` - Wallet address
    /// * `token_addresses` - Array of token contract addresses
    ///
    /// # Returns
    /// Tokens with balance information
    #[allow(dead_code)] // 用于代币检测功能
    pub async fn get_token_balances(
        &self,
        chain: &str,
        address: &str,
        token_addresses: &[String],
    ) -> Result<Vec<TokenMetadata>, AppError> {
        let api = self.app_state.get_api_client();

        let payload = serde_json::json!({
            "chain": chain,
            "address": address,
            "tokens": token_addresses,
        });

        let response: TokenListResponse = api.post("/api/v1/tokens/balances", &payload).await?;

        Ok(response.tokens)
    }

    /// Search tokens by name or symbol
    ///
    /// # Arguments
    /// * `query` - Search query
    /// * `chain` - Optional chain filter
    ///
    /// # Returns
    /// Matching tokens
    #[allow(dead_code)] // 用于代币检测功能
    pub async fn search_tokens(
        &self,
        query: &str,
        chain: Option<&str>,
    ) -> Result<Vec<TokenMetadata>, AppError> {
        let api = self.app_state.get_api_client();

        let mut url = format!("/api/v1/tokens/search?q={}", query);
        if let Some(ch) = chain {
            url.push_str(&format!("&chain={}", ch));
        }

        let response: TokenListResponse = api.get(&url).await?;

        Ok(response.tokens)
    }

    /// Validate contract address format
    #[allow(dead_code)] // 内部使用
    fn validate_contract_address(&self, chain: &str, address: &str) -> Result<(), AppError> {
        match chain.to_lowercase().as_str() {
            "eth" | "bsc" | "polygon" | "avalanche" => {
                // EVM chains: 0x + 40 hex chars
                if !address.starts_with("0x") || address.len() != 42 {
                    return Err(AppError::Validation(
                        "Invalid EVM contract address format".into(),
                    ));
                }
            }
            "solana" => {
                // Solana: Base58, typically 32-44 chars
                if address.len() < 32 || address.len() > 44 {
                    return Err(AppError::Validation(
                        "Invalid Solana contract address format".into(),
                    ));
                }
            }
            _ => {
                return Err(AppError::Validation(format!(
                    "Unsupported chain: {}",
                    chain
                )));
            }
        }
        Ok(())
    }

    /// Filter tokens by whitelist (security measure)
    fn filter_by_whitelist(&self, tokens: Vec<TokenMetadata>) -> Vec<TokenMetadata> {
        // Whitelist of known legitimate tokens
        let whitelist: HashSet<&str> = [
            // Ethereum mainnet
            "0xdac17f958d2ee523a2206206994597c13d831ec7", // USDT
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", // USDC
            "0x6b175474e89094c44da98b954eedeac495271d0f", // DAI
            "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599", // WBTC
            "0x514910771af9ca656af840dff83e8264ecf986ca", // LINK
                                                          // Add more trusted tokens here
        ]
        .iter()
        .copied()
        .collect();

        tokens
            .into_iter()
            .filter(|token| {
                // Always include verified tokens
                token.verified ||
                // Or if in whitelist
                whitelist.contains(token.contract_address.to_lowercase().as_str())
            })
            .collect()
    }

    /// Get popular tokens for a chain
    #[allow(dead_code)] // 用于代币检测功能
    pub async fn get_popular_tokens(&self, chain: &str) -> Result<Vec<TokenMetadata>, AppError> {
        let api = self.app_state.get_api_client();
        let url = format!("/api/v1/tokens/popular?chain={}", chain);
        let response: TokenListResponse = api.get(&url).await?;
        Ok(response.tokens)
    }
}

/// Hook for using token detection service
/// 获取代币检测服务实例
///
/// 注意：此函数当前未使用，但保留用于未来扩展
#[allow(dead_code)]
pub fn use_token_detection() -> TokenDetectionService {
    let app_state = use_context::<AppState>();
    TokenDetectionService::new(app_state)
}

/// Hook for auto-detecting tokens on component mount
/// 自动检测代币（响应式）
///
/// 注意：此函数当前未使用，但保留用于未来扩展
#[allow(dead_code)]
pub fn use_auto_detect_tokens(chain: &str, address: &str) -> Signal<Option<Vec<TokenMetadata>>> {
    let tokens = use_signal(|| None);
    let service = use_token_detection();
    let chain_owned = chain.to_string();
    let address_owned = address.to_string();

    use_effect(move || {
        let mut tokens = tokens;
        let chain_clone = chain_owned.clone();
        let address_clone = address_owned.clone();
        spawn_local(async move {
            match service
                .detect_tokens(&chain_clone, &address_clone, Some(0.0001))
                .await
            {
                Ok(detected) => {
                    tracing::info!("Auto-detected {} tokens", detected.len());
                    tokens.set(Some(detected));
                }
                Err(e) => {
                    tracing::error!("Token detection failed: {:?}", e);
                }
            }
        });
    });

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evm_address_validation() {
        let app_state = AppState::new();
        let service = TokenDetectionService::new(app_state);

        // Valid EVM address
        assert!(service
            .validate_contract_address("eth", "0x1234567890123456789012345678901234567890")
            .is_ok());

        // Invalid: missing 0x
        assert!(service
            .validate_contract_address("eth", "1234567890123456789012345678901234567890")
            .is_err());

        // Invalid: wrong length
        assert!(service
            .validate_contract_address("eth", "0x123456")
            .is_err());
    }

    #[test]
    fn test_whitelist_filtering() {
        let app_state = AppState::new();
        let service = TokenDetectionService::new(app_state);

        let tokens = vec![
            TokenMetadata {
                chain: "eth".to_string(),
                contract_address: "0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(),
                name: "Tether".to_string(),
                symbol: "USDT".to_string(),
                decimals: 6,
                logo_uri: None,
                verified: false,
                balance: Some("1000".to_string()),
            },
            TokenMetadata {
                chain: "eth".to_string(),
                contract_address: "0xscamtoken".to_string(),
                name: "Scam Token".to_string(),
                symbol: "SCAM".to_string(),
                decimals: 18,
                logo_uri: None,
                verified: false,
                balance: Some("1000000".to_string()),
            },
        ];

        let filtered = service.filter_by_whitelist(tokens);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].symbol, "USDT");
    }
}
