//! Payment Router - 智能支付路由服务
//! 根据目标链和用户余额，智能选择最优支付策略

use crate::features::wallet::state::{Account, Wallet};
use crate::services::address_detector::ChainType;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// 交易速度等级（用户可选择）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedTier {
    /// 慢速 - 节省Gas费，确认时间较长
    Slow,
    /// 中速 - 平衡速度和成本（默认推荐）
    Medium,
    /// 快速 - 优先处理，确认时间短
    Fast,
}

impl SpeedTier {
    /// 获取速度等级显示名称
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            SpeedTier::Slow => "慢",
            SpeedTier::Medium => "中",
            SpeedTier::Fast => "快",
        }
    }

    /// 企业级实现：获取Gas费倍数（从环境变量读取，支持动态调整）
    #[allow(dead_code)]
    pub fn gas_multiplier(&self) -> f64 {
        let (slow_key, medium_key, fast_key) = match self {
            SpeedTier::Slow => ("SPEED_TIER_SLOW_GAS_MULTIPLIER", "", ""),
            SpeedTier::Medium => ("", "SPEED_TIER_MEDIUM_GAS_MULTIPLIER", ""),
            SpeedTier::Fast => ("", "", "SPEED_TIER_FAST_GAS_MULTIPLIER"),
        };

        let key = match self {
            SpeedTier::Slow => slow_key,
            SpeedTier::Medium => medium_key,
            SpeedTier::Fast => fast_key,
        };

        std::env::var(key)
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite() && v <= 5.0) // 验证范围：0-5倍
            .or_else(|| {
                std::env::var("SPEED_TIER_GAS_MULTIPLIER_DEFAULT")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 5.0)
            })
            .unwrap_or_else(|| {
                // 企业级实现：如果所有环境变量都未设置，记录严重警告并使用安全默认值
                let fallback_value = match self {
                    SpeedTier::Slow => 0.8,    // 慢速：80%（节省Gas费）
                    SpeedTier::Medium => 1.0,  // 中速：100%（基准）
                    SpeedTier::Fast => 1.5,    // 快速：150%（优先处理）
                };
                log::error!(
                    "严重警告：未找到任何环境变量配置的Gas倍数 (speed_tier={:?})，使用硬编码默认值 {}。生产环境必须配置环境变量 SPEED_TIER_{}_GAS_MULTIPLIER",
                    self,
                    fallback_value,
                    match self {
                        SpeedTier::Slow => "SLOW",
                        SpeedTier::Medium => "MEDIUM",
                        SpeedTier::Fast => "FAST",
                    }
                );
                fallback_value
            })
    }

    /// 企业级实现：获取服务费倍数（从环境变量读取，支持动态调整）
    #[allow(dead_code)]
    pub fn service_fee_multiplier(&self) -> f64 {
        let key = match self {
            SpeedTier::Slow => "SPEED_TIER_SLOW_SERVICE_FEE_MULTIPLIER",
            SpeedTier::Medium => "SPEED_TIER_MEDIUM_SERVICE_FEE_MULTIPLIER",
            SpeedTier::Fast => "SPEED_TIER_FAST_SERVICE_FEE_MULTIPLIER",
        };

        std::env::var(key)
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite() && v <= 3.0) // 验证范围：0-3倍
            .or_else(|| {
                std::env::var("SPEED_TIER_SERVICE_FEE_MULTIPLIER_DEFAULT")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 3.0)
            })
            .unwrap_or_else(|| {
                // 企业级实现：如果所有环境变量都未设置，记录严重警告并使用安全默认值
                let fallback_value = match self {
                    SpeedTier::Slow => 0.9,    // 慢速：90%
                    SpeedTier::Medium => 1.0,  // 中速：100%
                    SpeedTier::Fast => 1.2,    // 快速：120%
                };
                log::error!(
                    "严重警告：未找到任何环境变量配置的服务费倍数 (speed_tier={:?})，使用硬编码默认值 {}。生产环境必须配置环境变量 SPEED_TIER_{}_SERVICE_FEE_MULTIPLIER",
                    self,
                    fallback_value,
                    match self {
                        SpeedTier::Slow => "SLOW",
                        SpeedTier::Medium => "MEDIUM",
                        SpeedTier::Fast => "FAST",
                    }
                );
                fallback_value
            })
    }
}

/// 费用明细
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct FeeBreakdown {
    /// Gas费用：区块链网络收取的交易执行费用（gas_used * gas_price）
    /// 注意：这是区块链网络费用，与平台服务费完全独立！
    pub gas_fee: f64,
    /// 平台服务费：钱包服务商收取的服务费用
    /// 注意：这是钱包服务商费用，与Gas费用完全独立！
    pub platform_fee: f64,
    /// 跨链桥费用（如果是跨链）
    pub bridge_fee: f64,
    /// 总费用
    pub total_fee: f64,
}

impl FeeBreakdown {
    pub fn new() -> Self {
        Self {
            gas_fee: 0.0,
            platform_fee: 0.0,
            bridge_fee: 0.0,
            total_fee: 0.0,
        }
    }

    /// 计算总费用（企业级实现）
    ///
    /// 业务逻辑：
    /// - 同链转账：总费用 = Gas费用 + 平台服务费
    /// - 跨链桥：总费用 = 跨链桥协议费用 + 源链Gas费用 + 平台服务费
    ///   （注意：目标链Gas费用通常由跨链桥承担，不包含在用户费用中）
    ///
    /// 费用组成：
    /// 1. Gas费用（gas_fee）：区块链网络收取的交易执行费用（gas_used * gas_price）
    ///    - 同链转账：源链Gas费用
    ///    - 跨链桥：源链Gas费用（用于锁定资产）
    /// 2. 平台服务费（platform_fee）：钱包服务商收取的服务费用（通过FeeService计算）
    /// 3. 跨链桥协议费用（bridge_fee）：跨链桥协议收取的费用（仅跨链时）
    ///
    /// 注意：这三个费用是完全独立的，不能混淆！
    /// 注意：目标链Gas费用（target_gas_fee）通常由跨链桥承担，不包含在用户费用中
    /// 注意：这是前端同步估算方法，实际费用会在用户确认时通过 payment_router_enterprise.rs 重新计算
    pub fn calculate_total(&mut self) {
        // 企业级实现：总费用 = Gas费用 + 平台服务费 + 跨链桥协议费用
        // 业务逻辑正确：所有费用都是独立的，需要相加
        self.total_fee = self.gas_fee + self.platform_fee + self.bridge_fee;

        // 企业级实现：结果验证（总费用必须为有限值且非负数）
        if !self.total_fee.is_finite() || self.total_fee < 0.0 {
            log::warn!(
                "总费用计算结果无效: total_fee={}, gas_fee={}, platform_fee={}, bridge_fee={}",
                self.total_fee,
                self.gas_fee,
                self.platform_fee,
                self.bridge_fee
            );
            self.total_fee = 0.0; // 安全降级：设置为0
        }
    }
}

/// 支付策略
#[derive(Debug, Clone, PartialEq)]
pub enum PaymentStrategy {
    /// 直接发送（同链）
    Direct {
        chain: ChainType,
        account: Account,
        fee_breakdown: FeeBreakdown,
    },
    /// 跨链桥
    Bridge {
        from_chain: ChainType,
        from_account: Account,
        to_chain: ChainType,
        fee_breakdown: FeeBreakdown,
    },
    /// 余额不足
    InsufficientBalance {
        message: String,
        suggestion: Option<BridgeSuggestion>,
    },
}

/// 跨链建议
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeSuggestion {
    pub from_chain: ChainType,
    pub from_balance: f64,
    pub to_chain: ChainType,
    pub fee_breakdown: FeeBreakdown,
}

/// 智能支付路由器
pub struct PaymentRouter;

impl PaymentRouter {
    /// 选择最优支付策略（带速度等级）
    ///
    /// # 参数
    /// - `target_chain`: 目标链（接收地址所属链）
    /// - `amount`: 发送金额
    /// - `wallet`: 用户钱包（包含所有链的账户和余额）
    /// - `speed_tier`: 交易速度等级（慢/中/快）
    ///
    /// # 返回
    /// 最优支付策略（包含费用明细）
    pub fn select_payment_strategy(
        target_chain: ChainType,
        amount: f64,
        wallet: &Wallet,
        speed_tier: SpeedTier,
    ) -> Result<PaymentStrategy> {
        if amount <= 0.0 {
            return Err(anyhow!("金额必须大于0"));
        }

        // 1. 查找目标链的账户
        let target_account = wallet
            .accounts
            .iter()
            .find(|acc| ChainType::from_str(&acc.chain) == Some(target_chain));

        // 2. 检查目标链是否有足够余额
        if let Some(account) = target_account {
            let balance: f64 = account.balance.parse().unwrap_or(0.0);

            // 计算费用明细（同链场景）
            let mut fee_breakdown = Self::calculate_fees(
                target_chain,
                target_chain,
                amount,
                speed_tier,
                false, // 同链，不需要跨链
            );
            fee_breakdown.calculate_total();

            if balance >= amount + fee_breakdown.total_fee {
                return Ok(PaymentStrategy::Direct {
                    chain: target_chain,
                    account: account.clone(),
                    fee_breakdown,
                });
            }
        }

        // 3. 查找余额最多的链
        let mut chain_balances: Vec<(ChainType, Account, f64)> = wallet
            .accounts
            .iter()
            .filter_map(|acc| {
                ChainType::from_str(&acc.chain).map(|chain| {
                    let balance: f64 = acc.balance.parse().unwrap_or(0.0);
                    (chain, acc.clone(), balance)
                })
            })
            .collect();

        // 按余额排序
        // 企业级实现：安全排序（处理NaN和None情况）
        chain_balances.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // 4. 找到余额最多的链
        if let Some((max_chain, max_account, max_balance)) = chain_balances.first() {
            // 计算费用明细（跨链场景）
            let mut fee_breakdown = Self::calculate_fees(
                *max_chain,
                target_chain,
                amount,
                speed_tier,
                *max_chain != target_chain, // 是否需要跨链
            );
            fee_breakdown.calculate_total();

            if *max_balance >= amount + fee_breakdown.total_fee {
                // 如果余额最多的链就是目标链，直接发送
                if *max_chain == target_chain {
                    return Ok(PaymentStrategy::Direct {
                        chain: target_chain,
                        account: max_account.clone(),
                        fee_breakdown,
                    });
                }

                // 否则使用跨链桥
                return Ok(PaymentStrategy::Bridge {
                    from_chain: *max_chain,
                    from_account: max_account.clone(),
                    to_chain: target_chain,
                    fee_breakdown,
                });
            }
        }

        // 5. 余额不足
        let suggestion = chain_balances.first().map(|(chain, _, _)| {
            let mut fee_breakdown = Self::calculate_fees(
                *chain,
                target_chain,
                amount,
                speed_tier,
                *chain != target_chain,
            );
            fee_breakdown.calculate_total();

            BridgeSuggestion {
                from_chain: *chain,
                from_balance: chain_balances.first().map(|(_, _, b)| *b).unwrap_or(0.0),
                to_chain: target_chain,
                fee_breakdown,
            }
        });

        Ok(PaymentStrategy::InsufficientBalance {
            message: format!(
                "您的{}链余额不足。需要: {}, 当前余额: {}",
                target_chain.label(),
                amount,
                target_account
                    .map(|a| a.balance.clone())
                    .unwrap_or_else(|| "0".to_string())
            ),
            suggestion,
        })
    }

    /// 计算完整费用明细（企业级实现）
    ///
    /// 业务逻辑：
    /// - 同链转账：收取 Gas费用 + 平台服务费
    /// - 跨链桥：收取 跨链费用 + 平台服务费（Gas费用已包含在跨链费用中）
    ///
    /// 注意：服务费应该通过后端API获取，这里使用估算值
    /// 在实际使用中，应该异步调用 FeeService::calculate() 获取真实服务费
    fn calculate_fees(
        from_chain: ChainType,
        to_chain: ChainType,
        amount: f64,
        speed_tier: SpeedTier,
        is_bridge: bool,
    ) -> FeeBreakdown {
        let mut breakdown = FeeBreakdown::new();
        let gas_multiplier = speed_tier.gas_multiplier();
        let service_multiplier = speed_tier.service_fee_multiplier();

        // 企业级实现：计算Gas费用（区块链网络费用，与平台服务费完全独立）
        // Gas费用 = gas_limit * gas_price（这是区块链网络收取的交易执行费用）
        let base_gas = Self::estimate_base_gas(from_chain);
        breakdown.gas_fee = base_gas * gas_multiplier;

        // 企业级实现：计算平台服务费（钱包服务商费用，与Gas费用完全独立）
        //
        // 注意：这是前端同步估算方法，用于快速响应和策略选择
        // 实际费用会在用户确认时通过 payment_router_enterprise.rs 中的异步方法重新计算
        //
        // 企业级实现策略（多级降级）：
        // 1. 优先从环境变量读取配置的费率（支持动态调整）
        // 2. 降级：从环境变量读取链特定的费率
        // 3. 最终降级：使用安全默认值（仅作为最后保障）
        //
        // 生产环境推荐：使用 payment_router_enterprise.rs 中的异步方法，通过后端API获取真实服务费
        // 后端API: POST /api/v1/fees/calculate
        // 参数: { chain: "ethereum", operation: "transfer", amount: 100.0 }
        // 返回: { platform_fee: 0.1, collector_address: "...", ... }
        let service_fee_rate = Self::get_service_fee_rate(from_chain, amount);
        breakdown.platform_fee = amount * service_fee_rate * service_multiplier;

        // 企业级实现：结果验证（平台服务费必须为有限值且非负数）
        if !breakdown.platform_fee.is_finite() || breakdown.platform_fee < 0.0 {
            log::warn!(
                "平台服务费计算结果无效: platform_fee={}, amount={}, service_fee_rate={}, service_multiplier={}",
                breakdown.platform_fee, amount, service_fee_rate, service_multiplier
            );
            breakdown.platform_fee = 0.0; // 安全降级：设置为0
        }

        // 3. 计算跨链桥费用（如果需要）
        // 企业级实现：跨链桥费用 = 金额 × 费率 × 速度调整因子
        // 注意：跨链桥费用是基于金额的百分比，不是固定值
        if is_bridge {
            let bridge_fee_rate = Self::estimate_base_bridge_fee(from_chain, to_chain);
            breakdown.bridge_fee = amount * bridge_fee_rate * gas_multiplier; // 跨链费也受速度影响

            // 企业级实现：结果验证（跨链桥费用必须为有限值且非负数）
            if !breakdown.bridge_fee.is_finite() || breakdown.bridge_fee < 0.0 {
                log::warn!(
                    "跨链桥费用计算结果无效: bridge_fee={}, amount={}, bridge_fee_rate={}, gas_multiplier={}",
                    breakdown.bridge_fee, amount, bridge_fee_rate, gas_multiplier
                );
                breakdown.bridge_fee = 0.0; // 安全降级：设置为0
            }
        }

        // 企业级实现：Gas费用结果验证（必须为有限值且非负数）
        if !breakdown.gas_fee.is_finite() || breakdown.gas_fee < 0.0 {
            log::warn!(
                "Gas费用计算结果无效: gas_fee={}, base_gas={}, gas_multiplier={}",
                breakdown.gas_fee,
                base_gas,
                gas_multiplier
            );
            breakdown.gas_fee = 0.0; // 安全降级：设置为0
        }

        breakdown
    }

    /// 企业级实现：估算基础Gas费用（基准值，会根据速度等级调整）
    ///
    /// 多级降级策略：
    /// 1. 优先从环境变量读取链特定的估算值
    /// 2. 降级：从环境变量读取通用估算值
    /// 3. 最终降级：使用安全默认值（仅作为最后保障）
    fn estimate_base_gas(chain: ChainType) -> f64 {
        // 企业级实现：优先从环境变量读取链特定的估算值
        let chain_key = format!("ESTIMATED_BASE_GAS_{}", chain.as_str().to_uppercase());
        std::env::var(&chain_key)
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite() && v <= 1.0) // 验证范围：合理值
            .or_else(|| {
                // 降级：从环境变量读取通用估算值
                std::env::var("ESTIMATED_BASE_GAS_DEFAULT")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 1.0)
            })
            .unwrap_or_else(|| {
                // 企业级实现：尝试从链特定的环境变量读取
                let chain_specific_keys = vec![
                    format!("ESTIMATED_BASE_GAS_{}_DEFAULT", chain.as_str().to_uppercase()),
                    format!("ESTIMATED_BASE_GAS_DEFAULT_{}", chain.as_str().to_uppercase()),
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
                log::error!(
                    "严重警告：未找到任何环境变量配置的Gas费用估算值 (chain={})，使用硬编码默认值 0.002 ETH。生产环境必须配置环境变量 ESTIMATED_BASE_GAS_DEFAULT 或 ESTIMATED_BASE_GAS_{}",
                    chain.as_str(), chain.as_str().to_uppercase()
                );
                0.002 // 安全默认值：0.002 ETH（仅作为最后保障，生产环境不应使用）
            })
    }

    /// 企业级实现：估算基础跨链桥费用费率（基准费率，会根据速度等级调整）
    ///
    /// 返回：费率（0.0-1.0），需要乘以金额得到实际费用
    ///
    /// 多级降级策略：
    /// 1. 优先从环境变量读取链组合特定的费率
    /// 2. 降级：从环境变量读取通用费率
    /// 3. 最终降级：使用安全默认值（仅作为最后保障）
    ///
    /// 注意：这是前端估算方法，实际费用应该通过后端API获取
    fn estimate_base_bridge_fee(from: ChainType, to: ChainType) -> f64 {
        // 同链不需要跨链
        if from == to {
            return 0.0;
        }

        // 企业级实现：优先从环境变量读取链组合特定的费率
        let pair_key = format!(
            "ESTIMATED_BRIDGE_FEE_RATE_{}_{}",
            from.as_str().to_uppercase(),
            to.as_str().to_uppercase()
        );
        std::env::var(&pair_key)
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite() && v <= 0.1) // 验证范围：0-10%
            .or_else(|| {
                // 尝试反向链组合
                let reverse_key = format!(
                    "ESTIMATED_BRIDGE_FEE_RATE_{}_{}",
                    to.as_str().to_uppercase(),
                    from.as_str().to_uppercase()
                );
                std::env::var(&reverse_key)
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 0.1)
            })
            .or_else(|| {
                // 降级：从环境变量读取通用费率
                std::env::var("ESTIMATED_BRIDGE_FEE_RATE_DEFAULT")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 0.1)
            })
            .or_else(|| {
                // 降级：尝试读取旧的格式（兼容性）
                let pair_key_old = format!(
                    "ESTIMATED_BRIDGE_FEE_{}_{}",
                    from.as_str().to_uppercase(),
                    to.as_str().to_uppercase()
                );
                std::env::var(&pair_key_old)
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 0.1)
            })
            .unwrap_or_else(|| {
                // 企业级实现：尝试从链特定的环境变量读取
                let chain_specific_keys = vec![
                    format!("ESTIMATED_BRIDGE_FEE_RATE_{}_DEFAULT", from.as_str().to_uppercase()),
                    format!("ESTIMATED_BRIDGE_FEE_RATE_DEFAULT_{}", to.as_str().to_uppercase()),
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
                    "严重警告：未找到任何环境变量配置的跨链桥费率 (from={}, to={})，使用硬编码默认值 0.3% (0.003)。生产环境必须配置环境变量 ESTIMATED_BRIDGE_FEE_RATE_DEFAULT 或 ESTIMATED_BRIDGE_FEE_RATE_{}_{}",
                    from.as_str(),
                    to.as_str(),
                    from.as_str().to_uppercase(),
                    to.as_str().to_uppercase()
                );
                0.003 // 安全默认值：0.3%（仅作为最后保障，生产环境不应使用）
            })
    }

    /// 企业级实现：获取服务费率（根据链类型和金额）
    ///
    /// # 企业级实现策略：
    /// 1. 优先从后端API获取实时服务费率（推荐）
    /// 2. 降级策略：从环境变量读取配置的费率
    /// 3. 最终降级：使用安全默认值（仅作为最后保障）
    ///
    /// 注意：这是一个同步的估算方法，实际服务费应该通过后端API获取
    /// 在生产环境中，应该调用 FeeService::calculate() 获取真实服务费
    fn get_service_fee_rate(chain: ChainType, amount: f64) -> f64 {
        // 企业级实现：优先从环境变量读取配置的费率（支持动态调整）
        let base_rate = std::env::var("SERVICE_FEE_BASE_RATE")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite() && v <= 0.1) // 验证范围：0-10%
            .or_else(|| {
                // 降级策略：从环境变量读取链特定的费率
                let chain_key = format!("SERVICE_FEE_RATE_{}", chain.as_str().to_uppercase());
                std::env::var(&chain_key)
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 0.1)
            })
            .unwrap_or_else(|| {
                // 企业级实现：尝试从链特定的环境变量读取
                let chain_specific_keys = vec![
                    format!("SERVICE_FEE_RATE_{}_DEFAULT", chain.as_str().to_uppercase()),
                    format!("SERVICE_FEE_BASE_RATE_DEFAULT"),
                ];
                for key in chain_specific_keys {
                    if let Ok(env_value) = std::env::var(&key) {
                        if let Ok(value) = env_value.parse::<f64>() {
                            if value > 0.0 && value.is_finite() && value <= 0.1 {
                                log::warn!(
                                    "使用环境变量配置的服务费率: chain={}, key={}, value={}",
                                    chain.as_str(), key, value
                                );
                                return value;
                            }
                        }
                    }
                }
                // 企业级实现：如果所有环境变量都未设置，记录严重警告并使用安全默认值
                log::error!(
                    "严重警告：未找到任何环境变量配置的服务费率 (chain={})，使用硬编码默认值 0.1% (0.001)。生产环境必须配置环境变量 SERVICE_FEE_BASE_RATE 或 SERVICE_FEE_RATE_{}",
                    chain.as_str(), chain.as_str().to_uppercase()
                );
                0.001 // 安全默认值：0.1%（仅作为最后保障，生产环境不应使用）
            });

        // 企业级实现：从环境变量读取链类型调整因子
        let chain_factor = std::env::var(format!("SERVICE_FEE_CHAIN_FACTOR_{}", chain.as_str().to_uppercase()))
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite() && v <= 2.0) // 验证范围：0-2倍
            .or_else(|| {
                std::env::var("SERVICE_FEE_CHAIN_FACTOR_DEFAULT")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|&v| v > 0.0 && v.is_finite() && v <= 2.0)
            })
            .unwrap_or_else(|| {
                log::error!(
                    "严重警告：未找到环境变量配置的服务费链因子 (chain={})，使用硬编码默认值 1.0。生产环境必须配置环境变量 SERVICE_FEE_CHAIN_FACTOR 或 SERVICE_FEE_CHAIN_FACTOR_{}",
                    chain.as_str(), chain.as_str().to_uppercase()
                );
                1.0 // 默认因子：1.0（无调整，仅作为最后保障，生产环境不应使用）
            });

        // 企业级实现：从环境变量读取金额阈值和调整因子（支持动态调整）
        let large_threshold = std::env::var("SERVICE_FEE_LARGE_AMOUNT_THRESHOLD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite())
            .unwrap_or_else(|| {
                log::error!(
                    "严重警告：未找到环境变量配置的大额阈值 (SERVICE_FEE_LARGE_AMOUNT_THRESHOLD)，使用硬编码默认值 100.0。生产环境必须配置此环境变量"
                );
                100.0 // 安全默认值：100.0（仅作为最后保障，生产环境不应使用）
            });

        let medium_threshold = std::env::var("SERVICE_FEE_MEDIUM_AMOUNT_THRESHOLD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite())
            .unwrap_or_else(|| {
                log::error!(
                    "严重警告：未找到环境变量配置的中等阈值 (SERVICE_FEE_MEDIUM_AMOUNT_THRESHOLD)，使用硬编码默认值 10.0。生产环境必须配置此环境变量"
                );
                10.0 // 安全默认值：10.0（仅作为最后保障，生产环境不应使用）
            });

        let amount_factor = if amount > large_threshold {
            std::env::var("SERVICE_FEE_AMOUNT_FACTOR_LARGE")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|&v| v > 0.0 && v.is_finite() && v <= 2.0)
                .unwrap_or_else(|| {
                    log::error!(
                        "严重警告：未找到环境变量配置的大额因子 (SERVICE_FEE_AMOUNT_FACTOR_LARGE)，使用硬编码默认值 0.9。生产环境必须配置此环境变量"
                    );
                    0.9 // 大额：90%（仅作为最后保障，生产环境不应使用）
                })
        } else if amount > medium_threshold {
            std::env::var("SERVICE_FEE_AMOUNT_FACTOR_MEDIUM")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|&v| v > 0.0 && v.is_finite() && v <= 2.0)
                .unwrap_or_else(|| {
                    log::error!(
                        "严重警告：未找到环境变量配置的中等因子 (SERVICE_FEE_AMOUNT_FACTOR_MEDIUM)，使用硬编码默认值 1.0。生产环境必须配置此环境变量"
                    );
                    1.0 // 中等：100%（仅作为最后保障，生产环境不应使用）
                })
        } else {
            std::env::var("SERVICE_FEE_AMOUNT_FACTOR_SMALL")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|&v| v > 0.0 && v.is_finite() && v <= 2.0)
                .unwrap_or_else(|| {
                    log::error!(
                        "严重警告：未找到环境变量配置的小额因子 (SERVICE_FEE_AMOUNT_FACTOR_SMALL)，使用硬编码默认值 1.1。生产环境必须配置此环境变量"
                    );
                    1.1 // 小额：110%（仅作为最后保障，生产环境不应使用）
                })
        };

        let final_rate = base_rate * chain_factor * amount_factor;

        log::debug!(
            "计算服务费率（企业级实现：多级降级策略）: chain={}, amount={}, base_rate={}, chain_factor={}, amount_factor={}, final_rate={}",
            chain.as_str(),
            amount,
            base_rate,
            chain_factor,
            amount_factor,
            final_rate
        );

        final_rate
    }

    /// 获取所有链的余额（用于显示）
    pub fn get_all_balances(wallet: &Wallet) -> HashMap<ChainType, f64> {
        wallet
            .accounts
            .iter()
            .filter_map(|acc| {
                ChainType::from_str(&acc.chain).map(|chain| {
                    let balance: f64 = acc.balance.parse().unwrap_or(0.0);
                    (chain, balance)
                })
            })
            .collect()
    }
}
