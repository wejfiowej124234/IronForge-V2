//! Bridge Fee Service - 跨链桥费用实时查询服务
//! 集成后端API，实时查询跨链桥费用

use crate::services::address_detector::ChainType;
use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use anyhow::{anyhow, Result};

/// 跨链桥费用报价（企业级实现）
#[derive(Debug, Clone)]
pub struct BridgeFeeQuote {
    /// 跨链桥协议费用（ETH单位）
    /// 注意：这是跨链桥协议收取的费用，与Gas费用和平台服务费完全独立
    pub bridge_fee: f64,
    /// 源链Gas费用：区块链网络收取的交易执行费用（ETH单位）
    /// 注意：这是区块链网络费用，由源链网络（矿工/验证者）收取，与平台服务费完全独立
    pub source_gas_fee: f64,
    /// 目标链Gas费用：区块链网络收取的交易执行费用（ETH单位）
    /// 注意：这是区块链网络费用，由目标链网络（矿工/验证者）收取，与平台服务费完全独立
    pub target_gas_fee: f64,
    /// 总费用（ETH单位）
    /// 注意：总费用 = 跨链桥协议费用 + 源链Gas费用 + 目标链Gas费用
    /// 这三个费用是完全独立的，不能混淆！
    /// 注意：此总费用不包含平台服务费，平台服务费需要单独计算
    pub total_fee: f64,
    /// 桥接协议（wormhole, layerzero等）
    pub bridge_protocol: String,
    /// 预估时间（秒）
    pub estimated_time_seconds: u64,
}

/// 跨链桥费用服务
pub struct BridgeFeeService {
    api_client: ApiClient,
}

impl BridgeFeeService {
    /// 创建服务实例
    pub fn new(app_state: AppState) -> Self {
        Self {
            api_client: app_state.get_api_client(),
        }
    }

    /// 查询跨链桥费用（实时）
    ///
    /// # 参数
    /// - `from_chain`: 源链
    /// - `to_chain`: 目标链
    /// - `amount`: 金额
    /// - `token`: 代币符号（可选，默认使用原生代币）
    ///
    /// # 返回
    /// 跨链桥费用报价
    pub async fn get_bridge_fee(
        &self,
        from_chain: ChainType,
        to_chain: ChainType,
        amount: f64,
        token: Option<&str>,
    ) -> Result<BridgeFeeQuote> {
        // 如果同链，返回0费用
        if from_chain == to_chain {
            return Ok(BridgeFeeQuote {
                bridge_fee: 0.0,
                source_gas_fee: 0.0,
                target_gas_fee: 0.0,
                total_fee: 0.0,
                bridge_protocol: "direct".to_string(),
                estimated_time_seconds: 0,
            });
        }

        // 尝试从后端API获取实时费用
        match self
            .query_bridge_fee_api(from_chain, to_chain, amount, token)
            .await
        {
            Ok(quote) => Ok(quote),
            Err(e) => {
                // 降级策略：使用估算值
                log::warn!("跨链桥费用API调用失败: {}, 使用估算值", e);
                Ok(self.estimate_bridge_fee_fallback(from_chain, to_chain, amount))
            }
        }
    }

    /// 从后端API查询跨链桥费用
    async fn query_bridge_fee_api(
        &self,
        from_chain: ChainType,
        to_chain: ChainType,
        amount: f64,
        token: Option<&str>,
    ) -> Result<BridgeFeeQuote> {
        use serde::{Deserialize, Serialize};

        // 构建请求（使用后端 /api/bridge/quote 端点）
        #[derive(Debug, Serialize)]
        struct BridgeFeeRequest {
            from_chain: String,
            to_chain: String,
            amount: f64,
            token: String,
        }

        // 后端响应格式
        #[derive(Debug, Deserialize)]
        struct BridgeFeeResponse {
            bridge_fee: f64,
            source_gas_fee: Option<f64>,
            target_gas_fee: Option<f64>,
            bridge_protocol: String,
            estimated_time_seconds: Option<u64>,
        }

        let token_symbol = token
            .map(|s| s.to_string())
            .unwrap_or_else(|| from_chain.as_str().to_string());

        let request = BridgeFeeRequest {
            from_chain: from_chain.as_str().to_string(),
            to_chain: to_chain.as_str().to_string(),
            amount,
            token: token_symbol,
        };

        // 调用后端API
        let response: BridgeFeeResponse = self
            .api_client
            .post("/api/v1/bridge/quote", &request)
            .await
            .map_err(|e| anyhow!("Failed to query bridge fee: {}", e))?;

        // 企业级实现：获取Gas费用（区块链网络费用，与平台服务费完全独立）
        // 企业级实现：如果API未返回Gas费用，从环境变量读取默认值
        let source_gas_fee = response.source_gas_fee.unwrap_or_else(|| {
            // 企业级实现：从环境变量读取源链Gas费用默认值
            let chain_key = format!("BRIDGE_SOURCE_GAS_FEE_DEFAULT_{}", from_chain.as_str().to_uppercase());
            std::env::var(&chain_key)
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|&v| v >= 0.0 && v.is_finite())
                .or_else(|| {
                    std::env::var("BRIDGE_SOURCE_GAS_FEE_DEFAULT")
                        .ok()
                        .and_then(|v| v.parse::<f64>().ok())
                        .filter(|&v| v >= 0.0 && v.is_finite())
                })
                .unwrap_or_else(|| {
                    tracing::error!(
                        "严重警告：未找到环境变量配置的源链Gas费用默认值 (chain={})，使用硬编码默认值 0.0。生产环境必须配置环境变量 BRIDGE_SOURCE_GAS_FEE_DEFAULT 或 BRIDGE_SOURCE_GAS_FEE_DEFAULT_{}",
                        from_chain.as_str(), from_chain.as_str().to_uppercase()
                    );
                    0.0 // 安全默认值：0.0（仅作为最后保障，生产环境不应使用）
                })
        });
        let target_gas_fee = response.target_gas_fee.unwrap_or_else(|| {
            // 企业级实现：从环境变量读取目标链Gas费用默认值
            let chain_key = format!("BRIDGE_TARGET_GAS_FEE_DEFAULT_{}", to_chain.as_str().to_uppercase());
            std::env::var(&chain_key)
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|&v| v >= 0.0 && v.is_finite())
                .or_else(|| {
                    std::env::var("BRIDGE_TARGET_GAS_FEE_DEFAULT")
                        .ok()
                        .and_then(|v| v.parse::<f64>().ok())
                        .filter(|&v| v >= 0.0 && v.is_finite())
                })
                .unwrap_or_else(|| {
                    tracing::error!(
                        "严重警告：未找到环境变量配置的目标链Gas费用默认值 (chain={})，使用硬编码默认值 0.0。生产环境必须配置环境变量 BRIDGE_TARGET_GAS_FEE_DEFAULT 或 BRIDGE_TARGET_GAS_FEE_DEFAULT_{}",
                        to_chain.as_str(), to_chain.as_str().to_uppercase()
                    );
                    0.0 // 安全默认值：0.0（仅作为最后保障，生产环境不应使用）
                })
        });

        // 企业级实现：计算总费用
        // 总费用 = 跨链桥协议费用 + 源链Gas费用 + 目标链Gas费用
        // 注意：这三个费用是完全独立的，不能混淆！
        // 注意：此总费用不包含平台服务费，平台服务费需要单独计算
        let total_fee = response.bridge_fee + source_gas_fee + target_gas_fee;

        Ok(BridgeFeeQuote {
            bridge_fee: response.bridge_fee,
            source_gas_fee,
            target_gas_fee,
            total_fee,
            bridge_protocol: response.bridge_protocol,
            estimated_time_seconds: response.estimated_time_seconds.unwrap_or_else(|| {
                // 企业级实现：从环境变量读取默认预估时间
                std::env::var("BRIDGE_DEFAULT_ESTIMATED_TIME_SECONDS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|&v| v > 0 && v <= 3600) // 验证范围：0-3600秒
                    .unwrap_or_else(|| {
                        tracing::error!(
                            "严重警告：未找到环境变量配置的桥接预估时间，使用硬编码默认值 300秒（5分钟）。生产环境必须配置环境变量 BRIDGE_DEFAULT_ESTIMATED_TIME_SECONDS"
                        );
                        300 // 安全默认值：5分钟（仅作为最后保障，生产环境不应使用）
                    })
            }),
        })
    }

    /// 企业级实现：估算跨链桥费用（降级策略）
    ///
    /// # 企业级实现策略：
    /// 1. 优先从后端API获取实时费率（已在 get_bridge_fee 中实现）
    /// 2. 降级策略：从环境变量读取配置的费率
    /// 3. 最终降级：使用安全默认值（仅作为最后保障）
    fn estimate_bridge_fee_fallback(
        &self,
        from: ChainType,
        to: ChainType,
        amount: f64,
    ) -> BridgeFeeQuote {
        // 企业级实现：优先从环境变量读取基础费率（支持动态调整）
        let base_rate = std::env::var("BRIDGE_BASE_RATE")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite() && v <= 0.1) // 验证范围：0-10%
            .or_else(|| {
                // 降级策略：从环境变量读取链组合特定费率
                let pair_key = format!("BRIDGE_RATE_{}_{}", 
                    from.as_str().to_uppercase(), 
                    to.as_str().to_uppercase()
                );
                std::env::var(&pair_key)
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 0.1)
                    .or_else(|| {
                        // 尝试反向链组合
                        let reverse_key = format!("BRIDGE_RATE_{}_{}", 
                            to.as_str().to_uppercase(), 
                            from.as_str().to_uppercase()
                        );
                        std::env::var(&reverse_key)
                            .ok()
                            .and_then(|v| v.parse::<f64>().ok())
                            .filter(|&v| v > 0.0 && v.is_finite() && v <= 0.1)
                    })
            })
            .unwrap_or_else(|| {
                // 企业级实现：尝试从链特定的环境变量读取
                let chain_specific_keys = vec![
                    format!("BRIDGE_RATE_{}_DEFAULT", from.as_str().to_uppercase()),
                    format!("BRIDGE_RATE_DEFAULT_{}", to.as_str().to_uppercase()),
                ];
                for key in chain_specific_keys {
                    if let Ok(env_value) = std::env::var(&key) {
                        if let Ok(value) = env_value.parse::<f64>() {
                            if value > 0.0 && value.is_finite() && value <= 0.1 {
                                log::warn!(
                                    "使用环境变量配置的跨链桥费率: from={}, to={}, key={}, value={}",
                                    from.as_str(), to.as_str(), key, value
                                );
                                return value;
                            }
                        }
                    }
                }
                // 企业级实现：如果所有环境变量都未设置，记录严重警告并使用安全默认值
                log::error!(
                    "严重警告：未找到任何环境变量配置的跨链桥费率 (from={}, to={})，使用硬编码默认值 0.3% (0.003)。生产环境必须配置环境变量 BRIDGE_BASE_RATE 或 BRIDGE_RATE_{}_{}",
                    from.as_str(),
                    to.as_str(),
                    from.as_str().to_uppercase(),
                    to.as_str().to_uppercase()
                );
                0.003 // 安全默认值：0.3%（仅作为最后保障，生产环境不应使用）
            });

        // 企业级实现：从环境变量读取链组合调整因子
        let chain_factor = std::env::var(format!("BRIDGE_FACTOR_{}_{}", 
            from.as_str().to_uppercase(), 
            to.as_str().to_uppercase()
        ))
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite() && v <= 5.0) // 验证范围：0-5倍
            .or_else(|| {
                // 尝试反向链组合
                let reverse_key = format!("BRIDGE_FACTOR_{}_{}", 
                    to.as_str().to_uppercase(), 
                    from.as_str().to_uppercase()
                );
                std::env::var(&reverse_key)
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 5.0)
            })
            .or_else(|| {
                std::env::var("BRIDGE_FACTOR_DEFAULT")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 5.0)
            })
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的桥接因子，使用硬编码默认值 1.0。生产环境必须配置环境变量 BRIDGE_FACTOR 或 BRIDGE_FACTOR_{}_{}",
                    from.as_str().to_uppercase(), to.as_str().to_uppercase()
                );
                1.0 // 默认因子：1.0（无调整，仅作为最后保障，生产环境不应使用）
            });

        let bridge_fee = amount * base_rate * chain_factor;

        // 企业级实现：估算Gas费用（区块链网络费用，与平台服务费完全独立）
        // 源链Gas费用：区块链网络收取的交易执行费用
        let source_gas_fee = Self::get_estimated_gas_fee_for_chain(from);

        // 目标链Gas费用：区块链网络收取的交易执行费用
        let target_gas_fee = Self::get_estimated_gas_fee_for_chain(to);

        // 企业级实现：计算总费用
        // 总费用 = 跨链桥协议费用 + 源链Gas费用 + 目标链Gas费用
        // 注意：这三个费用是完全独立的，不能混淆！
        // 注意：此总费用不包含平台服务费，平台服务费需要单独计算
        let total_fee = bridge_fee + source_gas_fee + target_gas_fee;

        // 企业级实现：预估时间（从环境变量读取）
        let estimated_time = std::env::var(format!("BRIDGE_TIME_{}_{}", 
            from.as_str().to_uppercase(), 
            to.as_str().to_uppercase()
        ))
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|&v| v > 0 && v <= 3600) // 验证范围：0-3600秒
            .or_else(|| {
                std::env::var("BRIDGE_TIME_DEFAULT")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|&v| v > 0 && v <= 3600)
            })
            .unwrap_or_else(|| {
                // 企业级实现：从环境变量读取通用默认时间
                std::env::var("BRIDGE_TIME_DEFAULT_SECONDS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|&v| v > 0 && v <= 3600) // 验证范围：0-3600秒
                    .unwrap_or_else(|| {
                        tracing::error!(
                            "严重警告：未找到环境变量配置的桥接预估时间，使用硬编码默认值 300秒（5分钟）。生产环境必须配置环境变量 BRIDGE_DEFAULT_ESTIMATED_TIME_SECONDS"
                        );
                        300 // 安全默认值：5分钟（仅作为最后保障，生产环境不应使用）
                    })
            });

        BridgeFeeQuote {
            bridge_fee,
            source_gas_fee,
            target_gas_fee,
            total_fee,
            bridge_protocol: "estimated".to_string(),
            estimated_time_seconds: estimated_time,
        }
    }

    /// 企业级实现：估算链的Gas费用（区块链网络费用）
    ///
    /// 注意：这是区块链网络费用，与平台服务费完全独立
    ///
    /// 多级降级策略：
    /// 1. 优先从环境变量读取链特定的Gas费用估算值
    /// 2. 降级：从环境变量读取通用估算值
    /// 3. 最终降级：使用安全默认值（仅作为最后保障）
    ///
    /// 注意：这是前端估算方法，实际费用应该通过后端API获取
    fn get_estimated_gas_fee_for_chain(chain: ChainType) -> f64 {
        // 企业级实现：优先从环境变量读取链特定的Gas费用估算值
        let chain_key = format!("ESTIMATED_GAS_FEE_{}", chain.as_str().to_uppercase());
        std::env::var(&chain_key)
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite() && v <= 1.0) // 验证范围：合理值
            .or_else(|| {
                // 降级：从环境变量读取通用估算值
                std::env::var("ESTIMATED_GAS_FEE_DEFAULT")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 1.0)
            })
            .unwrap_or_else(|| {
                // 企业级实现：尝试从链特定的环境变量读取
                let chain_specific_keys = vec![
                    format!("ESTIMATED_GAS_FEE_{}_DEFAULT", chain.as_str().to_uppercase()),
                    format!("ESTIMATED_GAS_FEE_DEFAULT_{}", chain.as_str().to_uppercase()),
                ];
                for key in chain_specific_keys {
                    if let Ok(env_value) = std::env::var(&key) {
                        if let Ok(value) = env_value.parse::<f64>() {
                            if value > 0.0 && value.is_finite() && value <= 1.0 {
                                log::warn!(
                                    "使用环境变量配置的Gas费用估算值: chain={}, key={}, value={}",
                                    chain.as_str(), key, value
                                );
                                return value;
                            }
                        }
                    }
                }
                // 企业级实现：如果所有环境变量都未设置，记录严重警告并使用安全默认值
                let default_value = 0.002f64; // 安全默认值：0.002 ETH（约 $5-10）
                log::error!(
                    "严重警告：未找到任何环境变量配置的Gas费用估算值 (chain={})，使用硬编码默认值 {} ETH。生产环境必须配置环境变量 ESTIMATED_GAS_FEE_DEFAULT 或 ESTIMATED_GAS_FEE_{}",
                    chain.as_str(), default_value, chain.as_str().to_uppercase()
                );
                default_value
            })
    }
}
