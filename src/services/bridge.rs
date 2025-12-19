//! Bridge Service - 跨链桥接服务
//!
//! ✅ 对齐后端 v1 Bridge API：
//! - POST /api/v1/bridge/execute
//! - GET  /api/v1/bridge/:id/status
//! - GET  /api/v1/bridge/history

use crate::crypto::tx_signer::EthereumTxSigner;
use crate::features::wallet::unlock::ensure_wallet_unlocked;
use crate::services::address_detector::ChainType;
use crate::services::chain_config::ChainConfigManager;
use crate::services::gas_limit::GasLimitService;
use crate::services::transaction::TransactionService;
use crate::shared::api::ApiClient;
use crate::shared::api_endpoints;
use crate::shared::state::AppState;
use anyhow::anyhow;
use dioxus::prelude::ReadableExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 后端桥接费用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeFeeInfo {
    pub bridge_fee_usd: f64,
    pub source_gas_fee_usd: f64,
    pub destination_gas_fee_usd: f64,
    pub total_fee_usd: f64,
}

/// 后端桥接执行响应（/api/v1/bridge/execute）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResponse {
    pub bridge_id: String,
    pub status: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub amount: String,
    pub estimated_arrival_time: String,
    pub fee_info: BridgeFeeInfo,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Route-based quote (Phase A)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BridgeRouteQuoteRequest {
    from_chain: String,
    to_chain: String,
    token: String,
    amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    destination_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BridgeRouteQuoteResponse {
    bridge_fee: f64,
    source_gas_fee: Option<f64>,
    target_gas_fee: Option<f64>,
    bridge_protocol: String,
    estimated_time_seconds: Option<u64>,
    route: BridgeRoute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BridgeRoute {
    provider: String,
    source_chain: String,
    destination_chain: String,
    token_symbol: String,
    amount: String,
    message_fee_wei: String,
    steps: Vec<BridgeRouteStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BridgeRouteStep {
    kind: String,
    chain: String,
    to: String,
    value_wei: String,
    data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignedRouteStep {
    kind: String,
    chain: String,
    signed_tx: String,
}

/// Bridge报价（用于兼容旧代码，实际使用SwapQuote）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BridgeQuote {
    pub route_id: String,
    pub from_amount: String,
    pub to_amount: String,
    pub estimated_time_seconds: u32,
    pub estimated_fee_usd: f64,
    pub bridges: Vec<String>,
    pub steps: u32,
}

/// Bridge历史记录（/api/v1/bridge/history）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeHistoryItem {
    pub bridge_id: String,
    pub status: String,
    pub source_chain: String,
    pub source_address: String,
    pub destination_chain: String,
    pub destination_address: String,
    pub token_symbol: String,
    pub amount: String,
    pub bridge_provider: Option<String>,
    pub fee_paid_usd: Option<f64>,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// Bridge历史响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeHistoryResponse {
    pub bridges: Vec<BridgeHistoryItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Bridge状态响应（/api/v1/bridge/:id/status）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatusResponse {
    pub bridge_id: String,
    pub status: String,
    pub source_tx_hash: Option<String>,
    pub source_confirmations: u32,
    pub destination_tx_hash: Option<String>,
    pub destination_confirmations: u32,
    pub progress_percentage: u8,
    pub estimated_completion_time: Option<String>,

    /// Phase A: route-based execution hashes
    #[serde(default)]
    pub approve_tx_hash: Option<String>,
    #[serde(default)]
    pub swap_tx_hash: Option<String>,
    #[serde(default)]
    pub route_step_hashes: Option<Vec<RouteStepHash>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStepHash {
    pub kind: String,
    pub tx_hash: String,
}

/// Bridge服务
#[derive(Clone)]
pub struct BridgeService {
    app_state: AppState,
    api_client: Arc<ApiClient>,
    transaction_service: TransactionService,
}

impl BridgeService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            app_state,
            api_client: Arc::new(app_state.get_api_client()),
            transaction_service: TransactionService::new(app_state),
        }
    }

    fn parse_amount_to_wei_u128(amount: &str) -> Result<u128, String> {
        // 字符串到 wei（18 decimals），避免f64精度问题
        let s = amount.trim();
        if s.is_empty() {
            return Err("amount is empty".to_string());
        }

        let mut parts = s.split('.');
        let int_part = parts.next().unwrap_or("0");
        let frac_part = parts.next().unwrap_or("0");
        if parts.next().is_some() {
            return Err("invalid amount format".to_string());
        }

        let int_u128 = int_part
            .parse::<u128>()
            .map_err(|_| "invalid integer part".to_string())?;

        let mut frac = frac_part.to_string();
        if frac.len() > 18 {
            frac.truncate(18);
        }
        while frac.len() < 18 {
            frac.push('0');
        }
        let frac_u128 = if frac.is_empty() {
            0u128
        } else {
            frac.parse::<u128>()
                .map_err(|_| "invalid fractional part".to_string())?
        };

        Ok(int_u128
            .checked_mul(1_000_000_000_000_000_000u128)
            .and_then(|x| x.checked_add(frac_u128))
            .ok_or_else(|| "amount overflow".to_string())?)
    }

    /// 执行跨链桥接✅使用统一端点
    pub async fn bridge_assets(
        &self,
        from_wallet: &str,
        from_chain: &str,
        to_chain: &str,
        token: &str,
        amount: &str,
    ) -> Result<BridgeResponse, String> {
        // 1) 钱包锁检查
        ensure_wallet_unlocked(&self.app_state, from_wallet)
            .map_err(|e| format!("Wallet locked: {}", e))?;

        // 2) 获取选中钱包与源/目标账户
        let wallet_state = self.app_state.wallet.read();
        let wallet = wallet_state
            .get_wallet(from_wallet)
            .ok_or_else(|| "Wallet not found".to_string())?;

        let source_account_index = wallet
            .accounts
            .iter()
            .position(|a| a.chain.eq_ignore_ascii_case(from_chain))
            .ok_or_else(|| format!("No source account for chain {}", from_chain))?;

        let source_account = &wallet.accounts[source_account_index];

        let destination_account = wallet
            .accounts
            .iter()
            .find(|a| a.chain.eq_ignore_ascii_case(to_chain))
            .ok_or_else(|| format!("No destination account for chain {}", to_chain))?;

        // 3) 当前仅支持 EVM->EVM（签名与广播）
        let from_chain_type = ChainType::from_str(from_chain)
            .ok_or_else(|| format!("Unsupported source chain: {}", from_chain))?;
        let to_chain_type = ChainType::from_str(to_chain)
            .ok_or_else(|| format!("Unsupported destination chain: {}", to_chain))?;

        let is_evm = matches!(
            from_chain_type,
            ChainType::Ethereum | ChainType::BSC | ChainType::Polygon
        ) && matches!(to_chain_type, ChainType::Ethereum | ChainType::BSC | ChainType::Polygon);

        if !is_evm {
            return Err(
                "Bridge currently supports EVM↔EVM only (ethereum/bsc/polygon)".to_string(),
            );
        }

        // 4) 构建并签名一笔源链原生代币转账（真实链上交互）
        // 注意：这里默认发送到目标链同钱包地址（同一私钥在不同EVM链地址相同），
        // 作为跨链桥执行的最小真实链交互载体。
        let cfg = ChainConfigManager::new();
        let chain_id = cfg
            .get_chain_id(from_chain_type)
            .map_err(|e| format!("Failed to get chain_id: {}", e))?;

        let nonce = self
            .transaction_service
            .get_nonce(&source_account.address, chain_id)
            .await
            .map_err(|e| format!("Failed to get nonce: {}", e))?;

        let gas_limit_service = GasLimitService::new(self.app_state);
        let gas_est = gas_limit_service
            .estimate_full(
                chain_id,
                &source_account.address,
                &destination_account.address,
                amount,
                None,
            )
            .await
            .map_err(|e| format!("Failed to estimate gas: {}", e))?;

        let gas_price = gas_est
            .gas_price
            .parse::<u64>()
            .map_err(|_| format!("Invalid gas_price returned: {}", gas_est.gas_price))?;

        let value_wei = Self::parse_amount_to_wei_u128(amount)?;

        // 派生私钥
        let key_manager = self
            .app_state
            .key_manager
            .read()
            .clone()
            .ok_or_else(|| "Wallet not unlocked (missing key manager)".to_string())?;

        let private_key_hex = key_manager
            .derive_eth_private_key(source_account_index as u32)
            .map_err(|e| format!("Failed to derive private key: {}", e))?;

        let signed_tx = EthereumTxSigner::sign_transaction(
            &private_key_hex,
            &destination_account.address,
            &value_wei.to_string(),
            nonce,
            gas_price,
            gas_est.gas_limit,
            chain_id,
        )
        .map_err(|e| format!("Failed to sign tx: {}", e))?;

        // 5) 调用后端 Bridge Execute
        #[derive(Debug, Serialize)]
        struct ExecuteBridgeRequest {
            source_chain: String,
            source_address: String,
            destination_chain: String,
            destination_address: String,
            token_symbol: String,
            amount: String,
            signed_source_tx: String,
            bridge_provider: Option<String>,
            idempotency_key: Option<String>,
        }

        let request = ExecuteBridgeRequest {
            source_chain: from_chain.to_string(),
            source_address: source_account.address.clone(),
            destination_chain: to_chain.to_string(),
            destination_address: destination_account.address.clone(),
            token_symbol: token.to_string(),
            amount: amount.to_string(),
            signed_source_tx: signed_tx,
            bridge_provider: None,
            idempotency_key: None,
        };

        let response: BridgeResponse = self
            .api_client
            .post(api_endpoints::bridge::EXECUTE, &request)
            .await
            .map_err(|e| format!("Failed to execute bridge: {}", e))?;

        Ok(response)
    }

    /// 执行跨链桥接：允许指定目标链的外部 destination_address。
    ///
    /// - 仍然只支持 EVM↔EVM（ethereum/bsc/polygon）。
    /// - signed_source_tx 仍使用“源链自转/同钱包地址”作为最小真实链上交互载体，
    ///   但后端会以 request.destination_address 作为目标链最终收款地址。
    pub async fn bridge_assets_to_address(
        &self,
        from_wallet: &str,
        from_chain: &str,
        to_chain: &str,
        token: &str,
        amount: &str,
        destination_address: &str,
    ) -> Result<BridgeResponse, String> {
        // 1) 钱包锁检查
        ensure_wallet_unlocked(&self.app_state, from_wallet)
            .map_err(|e| format!("Wallet locked: {}", e))?;

        // 2) 获取选中钱包与源账户
        let wallet_state = self.app_state.wallet.read();
        let wallet = wallet_state
            .get_wallet(from_wallet)
            .ok_or_else(|| "Wallet not found".to_string())?;

        let source_account_index = wallet
            .accounts
            .iter()
            .position(|a| a.chain.eq_ignore_ascii_case(from_chain))
            .ok_or_else(|| format!("No source account for chain {}", from_chain))?;

        let source_account = &wallet.accounts[source_account_index];

        // 3) 当前仅支持 EVM->EVM（签名与广播）
        let from_chain_type = ChainType::from_str(from_chain)
            .ok_or_else(|| format!("Unsupported source chain: {}", from_chain))?;
        let to_chain_type = ChainType::from_str(to_chain)
            .ok_or_else(|| format!("Unsupported destination chain: {}", to_chain))?;

        let is_evm = matches!(
            from_chain_type,
            ChainType::Ethereum | ChainType::BSC | ChainType::Polygon
        ) && matches!(to_chain_type, ChainType::Ethereum | ChainType::BSC | ChainType::Polygon);

        if !is_evm {
            return Err(
                "Bridge currently supports EVM↔EVM only (ethereum/bsc/polygon)".to_string(),
            );
        }

        // 4) Phase A: 先从后端获取 route-based quote（approve + swap steps）
        let cfg = ChainConfigManager::new();
        let chain_id = cfg
            .get_chain_id(from_chain_type)
            .map_err(|e| format!("Failed to get chain_id: {}", e))?;

        let amount_f64 = amount
            .parse::<f64>()
            .map_err(|_| "Invalid amount".to_string())?;

        let quote_req = BridgeRouteQuoteRequest {
            from_chain: from_chain.to_string(),
            to_chain: to_chain.to_string(),
            token: token.to_string(),
            amount: amount_f64,
            destination_address: Some(destination_address.to_string()),
            source_address: Some(source_account.address.clone()),
        };

        let quote: BridgeRouteQuoteResponse = self
            .api_client
            .post("/api/v1/bridge/quote", &quote_req)
            .await
            .map_err(|e| format!("Failed to quote bridge route: {}", e))?;

        if quote.route.steps.is_empty() {
            return Err("Bridge route quote returned no steps".to_string());
        }

        // 5) 派生私钥（用于签名 approve/swap steps）
        let key_manager = self
            .app_state
            .key_manager
            .read()
            .clone()
            .ok_or_else(|| "Wallet not unlocked (missing key manager)".to_string())?;

        let private_key_hex = key_manager
            .derive_eth_private_key(source_account_index as u32)
            .map_err(|e| format!("Failed to derive private key: {}", e))?;

        // 6) 计算 nonce 并签名每一个 step
        let base_nonce = self
            .transaction_service
            .get_nonce(&source_account.address, chain_id)
            .await
            .map_err(|e| format!("Failed to get nonce: {}", e))?;

        let gas_limit_service = GasLimitService::new(self.app_state);

        let mut signed_steps: Vec<SignedRouteStep> = Vec::with_capacity(quote.route.steps.len());
        for (i, step) in quote.route.steps.iter().enumerate() {
            // Phase A: steps should be on the source chain
            if !step.chain.eq_ignore_ascii_case(from_chain) {
                return Err(format!(
                    "Unexpected route step chain: {} (expected {})",
                    step.chain, from_chain
                ));
            }

            // Estimate gas with calldata so approve/swap doesn't use 21k
            let gas_est = gas_limit_service
                .estimate_full(
                    chain_id,
                    &source_account.address,
                    &step.to,
                    &step.value_wei,
                    Some(step.data.as_str()),
                )
                .await
                .map_err(|e| format!("Failed to estimate gas for step {}: {}", step.kind, e))?;

            let gas_price = gas_est
                .gas_price
                .parse::<u64>()
                .map_err(|_| format!("Invalid gas_price returned: {}", gas_est.gas_price))?;

            let nonce = base_nonce + i as u64;

            let signed_tx = EthereumTxSigner::sign_transaction_with_data(
                &private_key_hex,
                &step.to,
                &step.value_wei,
                &step.data,
                nonce,
                gas_price,
                gas_est.gas_limit,
                chain_id,
            )
            .map_err(|e| format!("Failed to sign {} step: {}", step.kind, e))?;

            signed_steps.push(SignedRouteStep {
                kind: step.kind.clone(),
                chain: step.chain.clone(),
                signed_tx,
            });
        }

        // 7) 调用后端 Bridge Execute（route_steps）
        #[derive(Debug, Serialize)]
        struct ExecuteBridgeRequest {
            source_chain: String,
            source_address: String,
            destination_chain: String,
            destination_address: String,
            token_symbol: String,
            amount: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            signed_source_tx: Option<String>,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            route_steps: Vec<SignedRouteStep>,
            bridge_provider: Option<String>,
            idempotency_key: Option<String>,
        }

        let request = ExecuteBridgeRequest {
            source_chain: from_chain.to_string(),
            source_address: source_account.address.clone(),
            destination_chain: to_chain.to_string(),
            destination_address: destination_address.to_string(),
            token_symbol: token.to_string(),
            amount: amount.to_string(),
            signed_source_tx: None,
            route_steps: signed_steps,
            bridge_provider: None,
            idempotency_key: None,
        };

        let response: BridgeResponse = self
            .api_client
            .post(api_endpoints::bridge::EXECUTE, &request)
            .await
            .map_err(|e| format!("Failed to execute bridge: {}", e))?;

        Ok(response)
    }

    /// 获取桥接历史（通过交易历史过滤桥接交易）
    ///
    /// 注意：后端暂无专用的桥接历史端点，此方法通过查询交易历史并过滤桥接相关交易实现
    pub async fn get_history(
        &self,
        page: Option<usize>,
        page_size: Option<usize>,
    ) -> Result<BridgeHistoryResponse, String> {
        let page = page.unwrap_or(1) as i64;
        let page_size = page_size.unwrap_or(20) as i64;
        let url = format!(
            "{}?page={}&page_size={}",
            api_endpoints::bridge::HISTORY,
            page,
            page_size
        );

        self.api_client
            .get::<BridgeHistoryResponse>(&url)
            .await
            .map_err(|e| format!("Failed to fetch bridge history: {}", e))
    }

    /// 获取桥接状态（/api/v1/bridge/:id/status）
    pub async fn get_status(&self, bridge_id: &str) -> Result<BridgeStatusResponse, String> {
        let url = api_endpoints::bridge::status(bridge_id);

        self.api_client
            .get::<BridgeStatusResponse>(&url)
            .await
            .map_err(|e| format!("Failed to get bridge status: {}", e))
    }

    /// 轮询桥接状态直到完成或失败
    ///
    /// # 参数
    /// - `swap_id`: 交换ID
    /// - `max_attempts`: 最大轮询次数（默认30次）
    /// - `interval_ms`: 轮询间隔（毫秒，默认2000ms）
    ///
    /// # 返回
    /// - `Ok(BridgeResponse)`: 桥接完成或失败时的最终状态
    /// - `Err(String)`: 轮询超时或发生错误
    pub async fn poll_status(
        &self,
        bridge_id: &str,
        max_attempts: Option<usize>,
        interval_ms: Option<u64>,
    ) -> Result<BridgeStatusResponse, String> {
        let max_attempts = max_attempts.unwrap_or(30);
        let interval_ms = interval_ms.unwrap_or(2000);

        for attempt in 1..=max_attempts {
            match self.get_status(bridge_id).await {
                Ok(status) => {
                    // 检查是否已完成或失败
                    match status.status.as_str() {
                        "DestinationConfirmed" | "Failed" | "Cancelled" => {
                            return Ok(status);
                        }
                        _ => {
                            // 继续轮询
                            if attempt < max_attempts {
                                // 使用wasm定时器等待
                                gloo_timers::future::TimeoutFuture::new(interval_ms as u32).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    if attempt == max_attempts {
                        return Err(format!(
                            "Failed to poll bridge status after {} attempts: {}",
                            max_attempts, e
                        ));
                    }
                    // 等待后重试
                    gloo_timers::future::TimeoutFuture::new(interval_ms as u32).await;
                }
            }
        }

        Err(format!(
            "Bridge status polling timeout after {} attempts",
            max_attempts
        ))
    }

    /// 执行桥接交易（已集成到bridge_assets中，此方法保留用于未来扩展）
    #[allow(dead_code)] // 用于桥接执行功能
    pub async fn execute(
        &self,
        _bridge_id: &str,
        _route_id: &str,
        _signed_tx: &str,
        _from_chain: &str,
    ) -> Result<BridgeExecuteResponse, String> {
        // 注意：后端的跨链兑换是异步处理的，不需要单独的execute步骤
        // bridge_assets 调用后，后端会自动处理交易
        // 此方法保留用于未来可能的同步执行需求
        Err("Bridge execution is handled automatically by backend".to_string())
    }
}

/// Bridge执行请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 用于桥接执行功能
pub struct BridgeExecuteRequest {
    pub bridge_id: String,
    pub route_id: String,
    pub signed_tx: String,
    pub from_chain: String,
}

/// Bridge执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 用于桥接执行功能
pub struct BridgeExecuteResponse {
    pub bridge_id: String,
    pub source_tx_hash: Option<String>,
    pub status: String,
    pub message: Option<String>,
}
