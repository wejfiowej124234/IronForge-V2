//! Provider Selection Service - 智能服务商选择服务
//! 企业级服务商选择机制，支持健康检查、智能选择算法、国家支持检查等

use crate::shared::api::ApiClient;
use crate::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

/// 获取当前 Unix 时间戳（秒）- WebAssembly 兼容
fn now_timestamp() -> u64 {
    js_sys::Date::new_0().get_time() as u64 / 1000
}

/// 服务商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderType {
    Ramp,
    MoonPay,
    Transak,
    Wyre,
    CoinbasePay,
}

impl ProviderType {
    pub fn name(&self) -> &'static str {
        match self {
            ProviderType::Ramp => "Ramp",
            ProviderType::MoonPay => "MoonPay",
            ProviderType::Transak => "Transak",
            ProviderType::Wyre => "Wyre",
            ProviderType::CoinbasePay => "Coinbase Pay",
        }
    }
}

/// 服务商状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderStatus {
    Healthy,   // 健康
    Degraded,  // 降级
    Unhealthy, // 不健康
    Unknown,   // 未知
}

/// 服务商健康信息
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub provider: ProviderType,
    pub status: ProviderStatus,
    pub last_check: u64, // Unix timestamp (seconds) - WebAssembly compatible
    pub response_time_ms: Option<u64>,
    pub success_rate: f64, // 0.0 - 1.0
    pub error_count: u32,
}

/// 服务商报价信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderQuote {
    pub provider: ProviderType,
    pub from_amount: String,
    pub to_amount: String,
    pub fee: String,
    pub fee_percentage: f64,
    pub estimated_time_seconds: u64,
    pub exchange_rate: String,
}

/// 服务商选择结果
#[derive(Debug, Clone)]
pub struct ProviderSelectionResult {
    pub selected_provider: ProviderType,
    pub quote: ProviderQuote,
    pub score: f64,                             // 综合得分 (0.0 - 100.0)
    pub alternatives: Vec<(ProviderType, f64)>, // 备选服务商及其得分
}

/// 国家支持信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountrySupport {
    pub country_code: String, // ISO 3166-1 alpha-2
    pub country_name: String,
    pub supported: bool,
    pub payment_methods: Vec<String>, // ["credit_card", "bank_transfer", "paypal"]
}

/// 服务商统计信息
#[derive(Debug, Clone)]
pub struct ProviderStatistics {
    pub provider: ProviderType,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub success_rate: f64,
    pub last_success_time: Option<u64>, // Unix timestamp (seconds) - WebAssembly compatible
    pub last_failure_time: Option<u64>, // Unix timestamp (seconds) - WebAssembly compatible
}

/// 服务商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: ProviderType,
    pub enabled: bool,
    pub priority: u8, // 0-255，数字越小优先级越高
    pub max_retries: u8,
    pub timeout_seconds: u64,
}

/// 故障切换结果
#[derive(Debug, Clone)]
pub struct FailoverResult {
    pub primary_provider: ProviderType,
    pub used_provider: ProviderType,
    pub attempts: Vec<(ProviderType, String)>, // (provider, error_message)
    pub success: bool,
}

/// 智能服务商选择服务
pub struct ProviderSelectionService {
    api_client: Arc<ApiClient>,
    health_cache: Arc<RwLock<HashMap<ProviderType, ProviderHealth>>>,
    statistics: Arc<RwLock<HashMap<ProviderType, ProviderStatistics>>>,
    config: Arc<RwLock<HashMap<ProviderType, ProviderConfig>>>,
}

impl ProviderSelectionService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        let mut default_config = HashMap::new();
        let providers = vec![
            ProviderType::Ramp,
            ProviderType::MoonPay,
            ProviderType::Transak,
            ProviderType::Wyre,
            ProviderType::CoinbasePay,
        ];

        // 初始化默认配置
        for (idx, provider) in providers.iter().enumerate() {
            default_config.insert(
                *provider,
                ProviderConfig {
                    provider: *provider,
                    enabled: true,
                    priority: idx as u8,
                    max_retries: 3,
                    timeout_seconds: 30,
                },
            );
        }

        Self {
            api_client: Arc::new(app_state.get_api_client()),
            health_cache: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(default_config)),
        }
    }

    /// 获取服务商配置
    pub fn get_config(&self, provider: ProviderType) -> Option<ProviderConfig> {
        self.config.read().ok()?.get(&provider).cloned()
    }

    /// 更新服务商配置
    pub fn update_config(&self, config: ProviderConfig) -> Result<(), String> {
        match self.config.write() {
            Ok(mut cfg) => {
                cfg.insert(config.provider, config);
                Ok(())
            }
            Err(e) => Err(format!("Failed to update config: {}", e)),
        }
    }

    /// 启用/禁用服务商
    pub fn set_provider_enabled(
        &self,
        provider: ProviderType,
        enabled: bool,
    ) -> Result<(), String> {
        match self.config.write() {
            Ok(mut cfg) => {
                if let Some(config) = cfg.get_mut(&provider) {
                    config.enabled = enabled;
                    Ok(())
                } else {
                    Err(format!("Provider {} not found in config", provider.name()))
                }
            }
            Err(e) => Err(format!("Failed to update config: {}", e)),
        }
    }

    /// 获取所有启用的服务商
    pub fn get_enabled_providers(&self) -> Vec<ProviderType> {
        match self.config.read() {
            Ok(cfg) => cfg
                .values()
                .filter(|c| c.enabled)
                .map(|c| c.provider)
                .collect(),
            Err(_) => vec![],
        }
    }

    /// 记录请求统计
    fn record_request(&self, provider: ProviderType, success: bool, response_time_ms: Option<u64>) {
        if let Ok(mut stats) = self.statistics.write() {
            let stat = stats.entry(provider).or_insert_with(|| ProviderStatistics {
                provider,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_response_time_ms: 0.0,
                success_rate: 0.0,
                last_success_time: None,
                last_failure_time: None,
            });

            stat.total_requests += 1;
            if success {
                stat.successful_requests += 1;
                stat.last_success_time = Some(now_timestamp());
            } else {
                stat.failed_requests += 1;
                stat.last_failure_time = Some(now_timestamp());
            }

            // 更新平均响应时间
            if let Some(rt) = response_time_ms {
                let total_rt = stat.average_response_time_ms * (stat.total_requests - 1) as f64;
                stat.average_response_time_ms = (total_rt + rt as f64) / stat.total_requests as f64;
            }

            // 更新成功率
            stat.success_rate = stat.successful_requests as f64 / stat.total_requests as f64;
        }
    }

    /// 获取服务商统计信息
    pub fn get_statistics(&self, provider: ProviderType) -> Option<ProviderStatistics> {
        self.statistics.read().ok()?.get(&provider).cloned()
    }

    /// 获取所有服务商统计信息
    pub fn get_all_statistics(&self) -> Vec<ProviderStatistics> {
        match self.statistics.read() {
            Ok(stats) => stats.values().cloned().collect(),
            Err(_) => vec![],
        }
    }

    /// 执行健康检查
    pub async fn check_health(&self, provider: ProviderType) -> Result<ProviderHealth, String> {
        let start_time = js_sys::Date::new_0().get_time();

        // 调用后端健康检查API
        let url = format!("/api/v1/providers/{}/health", provider.name().to_lowercase());
        let response: ProviderHealthResponse = self
            .api_client
            .get(&url)
            .await
            .map_err(|e| format!("Health check failed for {}: {}", provider.name(), e))?;

        let end_time = js_sys::Date::new_0().get_time();
        let response_time = Some((end_time - start_time) as u64);

        let health = ProviderHealth {
            provider,
            status: match response.status.as_str() {
                "healthy" => ProviderStatus::Healthy,
                "degraded" => ProviderStatus::Degraded,
                "unhealthy" => ProviderStatus::Unhealthy,
                _ => ProviderStatus::Unknown,
            },
            last_check: now_timestamp(),
            response_time_ms: response_time,
            success_rate: response.success_rate.unwrap_or(1.0),
            error_count: response.error_count.unwrap_or(0),
        };

        // 更新缓存
        if let Ok(mut cache) = self.health_cache.write() {
            cache.insert(provider, health.clone());
        }

        // 记录统计信息
        self.record_request(
            provider,
            matches!(health.status, ProviderStatus::Healthy),
            health.response_time_ms,
        );

        Ok(health)
    }

    /// 检查所有服务商的健康状态
    pub async fn check_all_providers(&self) -> Result<Vec<ProviderHealth>, String> {
        let providers = vec![
            ProviderType::Ramp,
            ProviderType::MoonPay,
            ProviderType::Transak,
            ProviderType::Wyre,
            ProviderType::CoinbasePay,
        ];

        let mut results = Vec::new();
        for provider in providers {
            match self.check_health(provider).await {
                Ok(health) => results.push(health),
                Err(e) => {
                    log::warn!("Failed to check health for {}: {}", provider.name(), e);
                    // 添加未知状态的健康信息
                    results.push(ProviderHealth {
                        provider,
                        status: ProviderStatus::Unknown,
                        last_check: now_timestamp(),
                        response_time_ms: None,
                        success_rate: 0.0,
                        error_count: 0,
                    });
                }
            }
        }

        Ok(results)
    }

    /// 获取服务商报价
    pub async fn get_quote(
        &self,
        provider: ProviderType,
        from_token: &str,
        to_token: &str,
        amount: &str,
        country_code: Option<&str>,
    ) -> Result<ProviderQuote, String> {
        let mut url = format!(
            "/api/v1/providers/{}/quote?from={}&to={}&amount={}",
            provider.name().to_lowercase(),
            from_token,
            to_token,
            amount
        );

        if let Some(country) = country_code {
            url.push_str(&format!("&country={}", country));
        }

        self.api_client
            .get::<ProviderQuote>(&url)
            .await
            .map_err(|e| format!("Failed to get quote from {}: {}", provider.name(), e))
    }

    /// 企业级实现：计算服务商综合得分
    ///
    /// # 企业级实现策略：
    /// 1. 优先从配置读取评分权重（支持动态调整）
    /// 2. 降级策略：使用安全默认权重（费用70%，响应时间20%，成功率10%）
    ///
    /// 默认权重：
    /// - 费用权重：70%（最重要，直接影响用户成本）
    /// - 响应时间权重：20%（影响用户体验）
    /// - 成功率权重：10%（影响可靠性）
    pub fn calculate_score(&self, quote: &ProviderQuote, health: &ProviderHealth) -> f64 {
        // 企业级实现：从配置读取评分权重
        let (fee_weight, response_time_weight, success_rate_weight) = Self::get_scoring_weights();

        // 企业级实现：从环境变量读取评分参数（支持动态调整）
        let max_fee_percentage = std::env::var("PROVIDER_SCORING_MAX_FEE_PERCENTAGE")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite())
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的服务商评分最大费率，使用硬编码默认值 10.0%。生产环境必须配置环境变量 PROVIDER_SCORING_MAX_FEE_PERCENTAGE"
                );
                10.0 // 安全默认值：10%（仅作为最后保障，生产环境不应使用）
            });

        let ideal_response_time_seconds = std::env::var("PROVIDER_SCORING_IDEAL_RESPONSE_TIME_SECONDS")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite())
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的服务商评分理想响应时间，使用硬编码默认值 1.0秒。生产环境必须配置环境变量 PROVIDER_SCORING_IDEAL_RESPONSE_TIME_SECONDS"
                );
                1.0 // 安全默认值：1秒（仅作为最后保障，生产环境不应使用）
            });

        let max_response_time_seconds = std::env::var("PROVIDER_SCORING_MAX_RESPONSE_TIME_SECONDS")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v > 0.0 && v.is_finite())
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的服务商评分最大响应时间，使用硬编码默认值 10.0秒。生产环境必须配置环境变量 PROVIDER_SCORING_MAX_RESPONSE_TIME_SECONDS"
                );
                10.0 // 安全默认值：10秒（仅作为最后保障，生产环境不应使用）
            });

        let unknown_response_time_score = std::env::var("PROVIDER_SCORING_UNKNOWN_RESPONSE_TIME_SCORE")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v >= 0.0 && v <= 100.0 && v.is_finite())
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的服务商评分未知响应时间分数，使用硬编码默认值 50.0分。生产环境必须配置环境变量 PROVIDER_SCORING_UNKNOWN_RESPONSE_TIME_SCORE"
                );
                50.0 // 安全默认值：50分（仅作为最后保障，生产环境不应使用）
            });

        // 费用得分（越低越好，转换为0-100分）
        let fee_score = (1.0 - quote.fee_percentage / max_fee_percentage)
            .max(0.0)
            .min(1.0)
            * 100.0;

        // 响应时间得分（越快越好，转换为0-100分）
        let response_time_score = if let Some(rt) = health.response_time_ms {
            let seconds = rt as f64 / 1000.0;
            let time_range = max_response_time_seconds - ideal_response_time_seconds;
            if time_range > 0.0 {
                (1.0 - (seconds - ideal_response_time_seconds).max(0.0) / time_range)
                    .max(0.0)
                    .min(1.0)
                    * 100.0
            } else {
                100.0 // 如果时间范围无效，给满分
            }
        } else {
            unknown_response_time_score // 未知响应时间，使用配置的分数
        };

        // 成功率得分（0-100分）
        let success_rate_score = health.success_rate * 100.0;

        // 企业级实现：使用配置的权重计算综合得分
        let total_score = fee_score * fee_weight
            + response_time_score * response_time_weight
            + success_rate_score * success_rate_weight;

        log::debug!(
            "服务商评分: fee_score={:.2} (权重={:.2}), response_time_score={:.2} (权重={:.2}), success_rate_score={:.2} (权重={:.2}), total_score={:.2}",
            fee_score, fee_weight, response_time_score, response_time_weight, success_rate_score, success_rate_weight, total_score
        );

        total_score
    }

    /// 企业级实现：从配置读取评分权重
    ///
    /// 企业级实现：获取评分权重（从环境变量读取，支持动态调整）
    /// 返回：(费用权重, 响应时间权重, 成功率权重)
    /// 权重总和应为 1.0
    fn get_scoring_weights() -> (f64, f64, f64) {
        // 企业级实现：优先从环境变量读取权重（前端环境变量通常在构建时注入）
        // 多级降级策略：
        // 1. 优先从环境变量读取权重
        // 2. 最终降级：使用安全默认权重
        let fee_weight = std::env::var("PROVIDER_SCORING_FEE_WEIGHT")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v >= 0.0 && v <= 1.0 && v.is_finite())
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的服务商评分费用权重，使用硬编码默认值 0.7 (70%)。生产环境必须配置环境变量 PROVIDER_SCORING_FEE_WEIGHT"
                );
                0.7 // 安全默认值：70%（仅作为最后保障，生产环境不应使用）
            });

        let response_time_weight = std::env::var("PROVIDER_SCORING_RESPONSE_TIME_WEIGHT")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v >= 0.0 && v <= 1.0 && v.is_finite())
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的服务商评分响应时间权重，使用硬编码默认值 0.2 (20%)。生产环境必须配置环境变量 PROVIDER_SCORING_RESPONSE_TIME_WEIGHT"
                );
                0.2 // 安全默认值：20%（仅作为最后保障，生产环境不应使用）
            });

        let success_rate_weight = std::env::var("PROVIDER_SCORING_SUCCESS_RATE_WEIGHT")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|&v| v >= 0.0 && v <= 1.0 && v.is_finite())
            .unwrap_or_else(|| {
                tracing::error!(
                    "严重警告：未找到环境变量配置的服务商评分成功率权重，使用硬编码默认值 0.1 (10%)。生产环境必须配置环境变量 PROVIDER_SCORING_SUCCESS_RATE_WEIGHT"
                );
                0.1 // 安全默认值：10%（仅作为最后保障，生产环境不应使用）
            });

        // 验证权重总和为 1.0
        let total = fee_weight + response_time_weight + success_rate_weight;
        let diff = if total > 1.0 {
            total - 1.0
        } else {
            1.0 - total
        };
        if diff > 0.01 {
            log::warn!("评分权重总和不为1.0: {}, 使用归一化权重", total);
            (
                fee_weight / total,
                response_time_weight / total,
                success_rate_weight / total,
            )
        } else {
            (fee_weight, response_time_weight, success_rate_weight)
        }
    }

    /// 智能选择最佳服务商（带自动故障切换）
    pub async fn select_best_provider(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
        country_code: Option<&str>,
    ) -> Result<ProviderSelectionResult, String> {
        // 使用自动故障切换机制
        self.select_with_failover(from_token, to_token, amount, country_code, None)
            .await
    }

    /// 带自动故障切换的服务商选择（最多尝试3个服务商）
    pub async fn select_with_failover(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
        country_code: Option<&str>,
        preferred_provider: Option<ProviderType>,
    ) -> Result<ProviderSelectionResult, String> {
        // 1. 获取启用的服务商
        let enabled_providers = self.get_enabled_providers();
        if enabled_providers.is_empty() {
            return Err("No enabled providers available".to_string());
        }

        // 2. 检查所有服务商的健康状态
        let health_statuses = self.check_all_providers().await?;

        // 3. 过滤掉不健康的服务商，并按配置优先级排序
        let mut candidate_providers: Vec<_> = health_statuses
            .iter()
            .filter(|h| {
                enabled_providers.contains(&h.provider)
                    && matches!(h.status, ProviderStatus::Healthy | ProviderStatus::Degraded)
            })
            .collect();

        if candidate_providers.is_empty() {
            return Err("No healthy providers available".to_string());
        }

        // 4. 按优先级和得分排序
        candidate_providers.sort_by(|a, b| {
            let priority_a = self
                .get_config(a.provider)
                .map(|c| c.priority)
                .unwrap_or(255);
            let priority_b = self
                .get_config(b.provider)
                .map(|c| c.priority)
                .unwrap_or(255);
            priority_a.cmp(&priority_b)
        });

        // 5. 如果指定了首选服务商，将其放在最前面
        if let Some(preferred) = preferred_provider {
            candidate_providers.sort_by(|a, b| {
                if a.provider == preferred {
                    std::cmp::Ordering::Less
                } else if b.provider == preferred {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            });
        }

        // 6. 尝试获取报价（最多3个服务商）
        let max_attempts = 3.min(candidate_providers.len());
        let mut attempts = Vec::new();
        let mut provider_scores = Vec::new();

        for health in candidate_providers.iter().take(max_attempts) {
            let start_time = js_sys::Date::new_0().get_time();
            match self
                .get_quote(health.provider, from_token, to_token, amount, country_code)
                .await
            {
                Ok(quote) => {
                    let end_time = js_sys::Date::new_0().get_time();
                    let response_time = Some((end_time - start_time) as u64); // milliseconds
                    let score = self.calculate_score(&quote, health);
                    provider_scores.push((health.provider, quote, score));
                    self.record_request(health.provider, true, response_time);
                }
                Err(e) => {
                    let end_time = js_sys::Date::new_0().get_time();
                    let response_time = Some((end_time - start_time) as u64); // milliseconds
                    attempts.push((health.provider, e.clone()));
                    self.record_request(health.provider, false, response_time);
                    log::warn!("Failed to get quote from {}: {}", health.provider.name(), e);
                }
            }
        }

        if provider_scores.is_empty() {
            return Err(format!(
                "All {} providers failed. Attempts: {:?}",
                attempts.len(),
                attempts
                    .iter()
                    .map(|(p, e)| format!("{}: {}", p.name(), e))
                    .collect::<Vec<_>>()
            ));
        }

        // 7. 按得分排序，选择最佳服务商
        provider_scores.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        let (selected_provider, selected_quote, selected_score) = provider_scores[0].clone();

        // 8. 准备备选服务商列表
        let alternatives: Vec<_> = provider_scores[1..]
            .iter()
            .map(|(p, _, s)| (*p, *s))
            .collect();

        Ok(ProviderSelectionResult {
            selected_provider,
            quote: selected_quote,
            score: selected_score,
            alternatives,
        })
    }

    /// 检查国家支持
    pub async fn check_country_support(
        &self,
        provider: ProviderType,
        country_code: &str,
    ) -> Result<CountrySupport, String> {
        let url = format!(
            "/api/v1/providers/{}/countries/{}",
            provider.name().to_lowercase(),
            country_code
        );

        self.api_client
            .get::<CountrySupport>(&url)
            .await
            .map_err(|e| format!("Failed to check country support: {}", e))
    }

    /// 获取服务商支持的国家列表
    pub async fn get_supported_countries(
        &self,
        provider: ProviderType,
    ) -> Result<Vec<CountrySupport>, String> {
        let url = format!(
            "/api/v1/providers/{}/countries",
            provider.name().to_lowercase()
        );

        self.api_client
            .get::<Vec<CountrySupport>>(&url)
            .await
            .map_err(|e| format!("Failed to get supported countries: {}", e))
    }

    /// 自动故障切换：尝试多个服务商直到成功
    /// 最多尝试3个服务商，自动切换到备选服务商
    pub async fn select_with_automatic_failover(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
        country_code: Option<&str>,
    ) -> Result<FailoverResult, String> {
        let primary_provider = match self
            .select_best_provider(from_token, to_token, amount, country_code)
            .await
        {
            Ok(result) => result.selected_provider,
            Err(e) => return Err(format!("Failed to select primary provider: {}", e)),
        };

        let mut attempts = Vec::new();
        let mut used_provider = primary_provider;
        let mut success = false;

        // 获取所有启用的服务商
        let enabled_providers = self.get_enabled_providers();
        let mut providers_to_try = vec![primary_provider];

        // 添加其他启用的服务商作为备选
        for provider in enabled_providers {
            if provider != primary_provider {
                providers_to_try.push(provider);
            }
        }

        // 最多尝试3个服务商
        for provider in providers_to_try.iter().take(3) {
            match self
                .get_quote(*provider, from_token, to_token, amount, country_code)
                .await
            {
                Ok(_) => {
                    used_provider = *provider;
                    success = true;
                    break;
                }
                Err(e) => {
                    attempts.push((*provider, e));
                    // 继续尝试下一个服务商
                }
            }
        }

        Ok(FailoverResult {
            primary_provider,
            used_provider,
            attempts,
            success,
        })
    }

    /// 重置服务商统计信息
    pub fn reset_statistics(&self, provider: Option<ProviderType>) {
        if let Ok(mut stats) = self.statistics.write() {
            if let Some(provider) = provider {
                stats.remove(&provider);
            } else {
                stats.clear();
            }
        }
    }

    /// 获取服务商配置列表
    pub fn get_all_configs(&self) -> Vec<ProviderConfig> {
        match self.config.read() {
            Ok(cfg) => cfg.values().cloned().collect(),
            Err(_) => vec![],
        }
    }
}

/// 健康检查响应
#[derive(Debug, Deserialize)]
struct ProviderHealthResponse {
    status: String,
    success_rate: Option<f64>,
    error_count: Option<u32>,
}
