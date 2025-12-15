//! Bridge Service - 跨链桥接服务
//! 集成后端跨链兑换API (/api/swap/cross-chain)
//! 生产级实现，包含历史查询、状态轮询等功能

use crate::services::transaction::TransactionService;
use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 跨链桥接请求（适配后端CrossChainSwapRequest）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRequest {
    pub source_chain: String, // 源链：ethereum, bsc, polygon
    pub source_token: String, // 源代币：ETH, BNB, MATIC
    pub source_amount: f64,   // 源数量
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source_wallet_id: String, // 源钱包ID（可选，后端会自动查找）
    pub target_chain: String, // 目标链：sol, avax, polygon
    pub target_token: String, // 目标代币：SOL, AVAX, MATIC
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target_wallet_id: String, // 目标钱包ID（可选）
}

/// 跨链桥接响应（适配后端CrossChainSwapResponse）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResponse {
    pub swap_id: String,                   // 交换ID（原bridge_id）
    pub status: String,                    // pending, processing, completed, failed
    pub source_amount: f64,                // 源数量
    pub estimated_target_amount: f64,      // 预估目标数量
    pub actual_target_amount: Option<f64>, // 实际目标数量（完成后）
    pub exchange_rate: f64,                // 汇率
    pub fee_usdt: f64,                     // 手续费（USDT）
    pub bridge_protocol: String,           // 桥协议：wormhole, layerzero
    pub estimated_time_minutes: u32,       // 预估时间（分钟）
    pub created_at: String,                // 创建时间
    pub completed_at: Option<String>,      // 完成时间
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

/// Bridge历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeHistoryItem {
    pub swap_id: String,                   // 交换ID
    pub source_chain: String,              // 源链
    pub target_chain: String,              // 目标链
    pub source_token: String,              // 源代币
    pub target_token: String,              // 目标代币
    pub source_amount: f64,                // 源数量
    pub estimated_target_amount: f64,      // 预估目标数量
    pub actual_target_amount: Option<f64>, // 实际目标数量
    pub status: String,                    // pending, processing, completed, failed
    pub fee_usdt: f64,                     // 手续费
    pub bridge_protocol: String,           // 桥协议
    pub created_at: String,                // 创建时间
    pub completed_at: Option<String>,      // 完成时间
}

/// Bridge历史响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeHistoryResponse {
    pub bridges: Vec<BridgeHistoryItem>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Bridge服务
#[derive(Clone)]
pub struct BridgeService {
    api_client: Arc<ApiClient>,
    transaction_service: Arc<TransactionService>,
}

impl BridgeService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            api_client: Arc::new(app_state.get_api_client()),
            transaction_service: Arc::new(TransactionService::new((*app_state).clone())),
        }
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
        use serde::{Deserialize, Serialize};

        // 后端API请求格式（对齐 IronCore/src/api/types.rs）
        #[derive(Debug, Serialize)]
        struct BridgeAssetsRequest {
            from_wallet: String,
            from_chain: String,
            to_chain: String,
            token: String,
            amount: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            client_request_id: Option<String>,
        }

        // 后端API响应格式（对齐后端CrossChainSwapResponse）
        #[derive(Debug, Deserialize)]
        struct BridgeAssetsResponse {
            #[serde(alias = "bridge_id", alias = "swap_id")]
            swap_id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            bridge_tx_id: Option<String>,
            status: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            target_chain: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            amount: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            from_chain: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            token: Option<String>,
            // 企业级实现：从后端响应获取费用和汇率信息
            #[serde(skip_serializing_if = "Option::is_none")]
            estimated_target_amount: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            exchange_rate: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            fee_usdt: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            bridge_protocol: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            estimated_time_minutes: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            created_at: Option<String>,
        }

        // 构建请求
        let request = BridgeAssetsRequest {
            from_wallet: from_wallet.to_string(),
            from_chain: from_chain.to_string(),
            to_chain: to_chain.to_string(),
            token: token.to_string(),
            amount: amount.to_string(),
            client_request_id: None, // 可选，用于幂等性
        };

        // ✅ v1标准端点
        let response: BridgeAssetsResponse = self
            .api_client
            .post("/api/v1/bridge/execute", &request)
            .await
            .map_err(|e| format!("Failed to bridge assets: {}", e))?;

        // 企业级实现：从后端响应获取费用和汇率信息，如果缺失则使用环境变量配置的默认值
        let source_amount = amount.parse::<f64>()
            .unwrap_or_else(|_| {
                tracing::error!(
                    "严重警告：金额解析失败 (amount={})，使用硬编码默认值 0.0。生产环境必须确保金额格式正确",
                    amount
                );
                0.0 // 安全默认值：0.0（仅作为最后保障，生产环境不应使用）
            });

        // 企业级实现：从环境变量读取默认汇率（支持动态调整）
        let default_exchange_rate = std::env::var("BRIDGE_DEFAULT_EXCHANGE_RATE")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite() && v <= 1000.0) // 验证范围：合理值
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的桥接默认汇率，使用硬编码默认值 1.0。生产环境必须配置环境变量 BRIDGE_DEFAULT_EXCHANGE_RATE"
                );
                1.0 // 安全默认值：1.0（仅作为最后保障，生产环境不应使用）
            });

        // 企业级实现：从环境变量读取默认费用（支持动态调整）
        let default_fee_usdt = std::env::var("BRIDGE_DEFAULT_FEE_USDT")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v >= 0.0 && v.is_finite() && v <= 10000.0) // 验证范围：合理值
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的桥接默认费用，使用硬编码默认值 0.0。生产环境必须配置环境变量 BRIDGE_DEFAULT_FEE_USDT"
                );
                0.0 // 安全默认值：0.0（仅作为最后保障，生产环境不应使用）
            });

        // 企业级实现：从环境变量读取默认桥接协议（支持动态调整）
        let default_bridge_protocol = std::env::var("BRIDGE_DEFAULT_PROTOCOL")
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的桥接默认协议，使用硬编码默认值 'unknown'。生产环境必须配置环境变量 BRIDGE_DEFAULT_PROTOCOL"
                );
                "unknown".to_string() // 安全默认值（仅作为最后保障，生产环境不应使用）
            });

        // 企业级实现：从环境变量读取默认预估时间（支持动态调整）
        let default_estimated_time = std::env::var("BRIDGE_DEFAULT_ESTIMATED_TIME_MINUTES")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|&v| v > 0 && v <= 1440) // 验证范围：0-1440分钟（24小时）
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的桥接默认预估时间，使用硬编码默认值 5分钟。生产环境必须配置环境变量 BRIDGE_DEFAULT_ESTIMATED_TIME_MINUTES"
                );
                5 // 安全默认值：5分钟（仅作为最后保障，生产环境不应使用）
            });

        Ok(BridgeResponse {
            swap_id: response.swap_id,
            status: response.status,
            source_amount,
            estimated_target_amount: response.estimated_target_amount.unwrap_or(source_amount),
            actual_target_amount: None,
            exchange_rate: response.exchange_rate.unwrap_or(default_exchange_rate),
            fee_usdt: response.fee_usdt.unwrap_or(default_fee_usdt),
            bridge_protocol: response.bridge_protocol.unwrap_or(default_bridge_protocol),
            estimated_time_minutes: response
                .estimated_time_minutes
                .unwrap_or(default_estimated_time),
            created_at: response
                .created_at
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            completed_at: None,
        })
    }

    /// 获取桥接历史（通过交易历史过滤桥接交易）
    ///
    /// 注意：后端暂无专用的桥接历史端点，此方法通过查询交易历史并过滤桥接相关交易实现
    pub async fn get_history(
        &self,
        page: Option<usize>,
        page_size: Option<usize>,
    ) -> Result<BridgeHistoryResponse, String> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(20);

        // 获取交易历史
        let transactions = self
            .transaction_service
            .history(page as u32)
            .await
            .map_err(|e| format!("Failed to fetch transaction history: {}", e))?;

        // 过滤桥接交易（通过检查交易类型或token字段）
        // 注意：这里假设桥接交易在token字段中包含特定标识，或通过交易类型判断
        let bridge_transactions: Vec<BridgeHistoryItem> = transactions
            .into_iter()
            .filter_map(|tx| {
                // 检查是否为桥接交易（通过token或tx_type判断）
                let is_bridge = tx.tx_type.contains("bridge")
                    || tx.tx_type.contains("cross-chain")
                    || tx.tx_type.contains("swap")
                    || tx.token.contains("bridge")
                    || tx.hash.starts_with("swap_"); // 假设swap_id格式为swap_xxx

                if is_bridge {
                    // 解析数量
                    let amount = tx.amount.parse::<f64>().unwrap_or(0.0);
                    let fee = tx.fee.parse::<f64>().unwrap_or_else(|e| {
                        tracing::warn!(
                            "交易费用解析失败: fee={}, error={}，使用默认值 0.0",
                            tx.fee,
                            e
                        );
                        0.0 // 安全默认值：解析失败时使用0.0
                    });

                    // 尝试从hash中提取swap_id（如果格式为swap_xxx）
                    let swap_id = if tx.hash.starts_with("swap_") {
                        tx.hash.clone()
                    } else {
                        format!("swap_{}", tx.hash)
                    };

                    Some(BridgeHistoryItem {
                        swap_id,
                        source_chain: "unknown".to_string(), // 需要从后端数据中提取
                        target_chain: "unknown".to_string(), // 需要从后端数据中提取
                        source_token: tx.token.clone(),
                        target_token: tx.token.clone(), // 默认相同，实际应从后端数据解析
                        source_amount: amount,
                        estimated_target_amount: amount,
                        actual_target_amount: Some(amount),
                        status: match tx.status.as_str() {
                            "confirmed" | "success" | "completed" => "completed".to_string(),
                            "pending" => "pending".to_string(),
                            "failed" | "error" => "failed".to_string(),
                            _ => "processing".to_string(),
                        },
                        fee_usdt: fee,
                        bridge_protocol: "unknown".to_string(),
                        created_at: {
                            // 将Unix时间戳转换为RFC3339格式
                            let timestamp = tx.timestamp as i64;
                            chrono::DateTime::from_timestamp(timestamp, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
                        },
                        completed_at: if tx.status == "confirmed"
                            || tx.status == "success"
                            || tx.status == "completed"
                        {
                            let timestamp = tx.timestamp as i64;
                            Some(
                                chrono::DateTime::from_timestamp(timestamp, 0)
                                    .map(|dt| dt.to_rfc3339())
                                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                            )
                        } else {
                            None
                        },
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(BridgeHistoryResponse {
            bridges: bridge_transactions,
            total: 0, // 实际总数需要后端支持
            page,
            page_size,
        })
    }

    /// 获取桥接状态（使用后端 /api/swap/{id} 端点）
    pub async fn get_status(&self, swap_id: &str) -> Result<BridgeResponse, String> {
        let url = format!("/api/v1/swap/{}", swap_id);

        self.api_client
            .get::<BridgeResponse>(&url)
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
        swap_id: &str,
        max_attempts: Option<usize>,
        interval_ms: Option<u64>,
    ) -> Result<BridgeResponse, String> {
        let max_attempts = max_attempts.unwrap_or(30);
        let interval_ms = interval_ms.unwrap_or(2000);

        for attempt in 1..=max_attempts {
            match self.get_status(swap_id).await {
                Ok(status) => {
                    // 检查是否已完成或失败
                    match status.status.as_str() {
                        "completed" | "failed" => {
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
        _swap_id: &str,
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
