// Feature Flags System
// Configuration-based feature toggles for gradual rollout and remote control

use crate::shared::error::AppError;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 功能开关系统，用于未来功能
pub struct FeatureFlag {
    pub key: String,
    pub enabled: bool,
    pub description: String,
    pub rollout_percentage: Option<u8>,     // 0-100
    pub allowed_users: Option<Vec<String>>, // Whitelist
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 功能开关配置，用于未来功能
pub struct FeatureFlagsConfig {
    pub flags: HashMap<String, FeatureFlag>,
    pub last_updated: u64,
}

impl Default for FeatureFlagsConfig {
    fn default() -> Self {
        let mut flags = HashMap::new();

        // Define default feature flags
        flags.insert(
            "websocket_realtime".to_string(),
            FeatureFlag {
                key: "websocket_realtime".to_string(),
                enabled: false,
                description: "Enable WebSocket real-time updates".to_string(),
                rollout_percentage: Some(10), // 10% rollout
                allowed_users: None,
            },
        );

        flags.insert(
            "token_auto_detect".to_string(),
            FeatureFlag {
                key: "token_auto_detect".to_string(),
                enabled: true,
                description: "Auto-detect user tokens on wallet load".to_string(),
                rollout_percentage: Some(100),
                allowed_users: None,
            },
        );

        flags.insert(
            "siwe_auth".to_string(),
            FeatureFlag {
                key: "siwe_auth".to_string(),
                enabled: true,
                description: "Sign In With Ethereum authentication".to_string(),
                rollout_percentage: Some(50), // 50% rollout
                allowed_users: None,
            },
        );

        flags.insert(
            "cross_chain_bridge".to_string(),
            FeatureFlag {
                key: "cross_chain_bridge".to_string(),
                enabled: false,
                description: "Enable cross-chain bridge feature".to_string(),
                rollout_percentage: Some(0), // Beta phase
                allowed_users: Some(vec!["0xdev1".to_string(), "0xdev2".to_string()]),
            },
        );

        flags.insert(
            "defi_staking".to_string(),
            FeatureFlag {
                key: "defi_staking".to_string(),
                enabled: false,
                description: "DeFi staking and yield farming".to_string(),
                rollout_percentage: Some(0),
                allowed_users: None,
            },
        );

        flags.insert(
            "nft_gallery".to_string(),
            FeatureFlag {
                key: "nft_gallery".to_string(),
                enabled: true,
                description: "NFT gallery and management".to_string(),
                rollout_percentage: Some(100),
                allowed_users: None,
            },
        );

        Self {
            flags,
            last_updated: now_secs(),
        }
    }
}

fn now_secs() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::new_0().get_time() / 1000.0) as u64
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)] // 功能开关管理器，用于未来功能
pub struct FeatureFlagsManager {
    config: Signal<FeatureFlagsConfig>,
}

#[allow(dead_code)] // 功能开关管理器，用于未来功能
impl FeatureFlagsManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            config: Signal::new(FeatureFlagsConfig::default()),
        }
    }

    /// Check if a feature is enabled
    ///
    /// # Arguments
    /// * `feature_key` - Feature identifier
    ///
    /// # Returns
    /// True if feature is enabled
    pub fn is_enabled(&self, feature_key: &str) -> bool {
        let config = self.config.read();

        if let Some(flag) = config.flags.get(feature_key) {
            if !flag.enabled {
                return false;
            }

            // Check rollout percentage
            if let Some(percentage) = flag.rollout_percentage {
                if percentage == 0 {
                    return false;
                }
                if percentage < 100 {
                    // Deterministic rollout based on feature key hash
                    return self.is_in_rollout(feature_key, percentage);
                }
            }

            true
        } else {
            // Unknown feature: default to disabled
            false
        }
    }

    /// Check if a feature is enabled for a specific user
    ///
    /// # Arguments
    /// * `feature_key` - Feature identifier
    /// * `user_id` - User identifier (e.g., wallet address)
    ///
    /// # Returns
    /// True if feature is enabled for this user
    #[allow(dead_code)]
    pub fn is_enabled_for_user(&self, feature_key: &str, user_id: &str) -> bool {
        let config = self.config.read();

        if let Some(flag) = config.flags.get(feature_key) {
            // Check whitelist first
            if let Some(allowed) = &flag.allowed_users {
                if allowed.contains(&user_id.to_string()) {
                    return true;
                }
            }

            // Then check general enabled status
            return self.is_enabled(feature_key);
        }

        false
    }

    /// Get all feature flags
    #[allow(dead_code)]
    pub fn get_all_flags(&self) -> HashMap<String, FeatureFlag> {
        self.config.read().flags.clone()
    }

    /// Update feature flag configuration (admin only)
    ///
    /// # Arguments
    /// * `new_config` - New configuration
    #[allow(dead_code)]
    pub fn update_config(&mut self, new_config: FeatureFlagsConfig) {
        tracing::info!("Updating feature flags configuration");
        self.config.set(new_config);
    }

    /// Fetch remote feature flags from backend
    ///
    /// 注意：此方法需要 AppState 来获取 ApiClient
    /// 建议使用 AppState 的 ApiClient 来调用此方法
    #[allow(dead_code)]
    pub async fn fetch_remote_config_with_client(
        &mut self,
        api_client: &crate::shared::api::ApiClient,
    ) -> Result<(), AppError> {
        // deserialize 方法已自动提取 data 字段
        // 后端返回: {code: 0, message: "success", data: FeatureFlagsConfig}
        let config: Option<FeatureFlagsConfig> = api_client
            .get("/api/v1/features")
            .await
            .map_err(AppError::Api)?;

        if let Some(config) = config {
            self.update_config(config);
        }
        Ok(())
    }

    /// Fetch remote feature flags from backend (legacy method, kept for backward compatibility)
    ///
    /// 注意：此方法已废弃，请使用 `fetch_remote_config_with_client` 方法
    #[allow(dead_code)]
    #[deprecated(note = "Use fetch_remote_config_with_client instead")]
    pub async fn fetch_remote_config(&mut self, api_url: &str) -> Result<(), AppError> {
        // 创建临时 ApiClient（不推荐，但保持向后兼容）
        use crate::shared::api::{ApiClient, ApiConfig};
        let config = ApiConfig {
            base_url: api_url.to_string(),
            timeout: 30,
        };
        let api_client = ApiClient::new(config);
        self.fetch_remote_config_with_client(&api_client).await
    }

    /// Toggle a feature flag (admin only)
    #[allow(dead_code)]
    pub fn toggle_feature(&mut self, feature_key: &str) {
        let mut config = self.config.write();

        if let Some(flag) = config.flags.get_mut(feature_key) {
            flag.enabled = !flag.enabled;
            tracing::info!("Toggled feature '{}' to {}", feature_key, flag.enabled);
        }
    }

    /// Deterministic rollout calculation
    #[allow(dead_code)]
    fn is_in_rollout(&self, feature_key: &str, percentage: u8) -> bool {
        // Use simple hash for deterministic rollout
        let hash = self.simple_hash(feature_key);
        (hash % 100) < percentage as u32
    }

    /// Simple hash function for rollout
    #[allow(dead_code)]
    fn simple_hash(&self, s: &str) -> u32 {
        s.bytes().fold(0u32, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(u32::from(b))
        })
    }
}

/// Hook for using feature flags in components
#[allow(dead_code)] // Feature flags hook, used in future features
pub fn use_feature_flags() -> Signal<FeatureFlagsManager> {
    use_signal(FeatureFlagsManager::new)
}

/// Hook for checking if a feature is enabled
#[allow(dead_code)] // Feature check hook, used in future features
pub fn use_feature(feature_key: &str) -> bool {
    let manager = use_feature_flags();
    let guard = manager.read();
    guard.is_enabled(feature_key)
}

/// Conditional rendering based on feature flag
#[component]
pub fn FeatureGate(
    feature: String,
    #[props(default = false)] fallback_enabled: bool,
    children: Element,
) -> Element {
    let manager = use_feature_flags();
    let is_enabled = manager.read().is_enabled(&feature);

    if is_enabled {
        rsx! { {children} }
    } else if fallback_enabled {
        rsx! { {children} }
    } else {
        VNode::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_flags() {
        let config = FeatureFlagsConfig::default();

        assert!(config.flags.contains_key("token_auto_detect"));
        assert!(config.flags.contains_key("siwe_auth"));
        assert_eq!(config.flags.len(), 6);
    }

    #[test]
    #[cfg_attr(not(target_arch = "wasm32"), ignore)]
    fn test_feature_enabled_check() {
        let manager = FeatureFlagsManager::new();

        // token_auto_detect is enabled by default at 100%
        assert!(manager.is_enabled("token_auto_detect"));

        // cross_chain_bridge is disabled by default
        assert!(!manager.is_enabled("cross_chain_bridge"));

        // Unknown feature should be disabled
        assert!(!manager.is_enabled("unknown_feature"));
    }

    #[test]
    #[cfg_attr(not(target_arch = "wasm32"), ignore)]
    fn test_user_whitelist() {
        let manager = FeatureFlagsManager::new();

        // cross_chain_bridge has whitelist
        assert!(manager.is_enabled_for_user("cross_chain_bridge", "0xdev1"));
        assert!(!manager.is_enabled_for_user("cross_chain_bridge", "0xrandom"));
    }

    #[test]
    #[cfg_attr(not(target_arch = "wasm32"), ignore)]
    fn test_rollout_percentage() {
        let mut config = FeatureFlagsConfig::default();

        // Create a feature with 50% rollout
        config.flags.insert(
            "test_50".to_string(),
            FeatureFlag {
                key: "test_50".to_string(),
                enabled: true,
                description: "Test".to_string(),
                rollout_percentage: Some(50),
                allowed_users: None,
            },
        );

        let manager = FeatureFlagsManager {
            config: Signal::new(config),
        };

        // Rollout should be deterministic
        let result1 = manager.is_enabled("test_50");
        let result2 = manager.is_enabled("test_50");
        assert_eq!(result1, result2);
    }

    #[test]
    #[cfg_attr(not(target_arch = "wasm32"), ignore)]
    fn test_simple_hash() {
        let manager = FeatureFlagsManager::new();

        let hash1 = manager.simple_hash("test");
        let hash2 = manager.simple_hash("test");
        assert_eq!(hash1, hash2); // Deterministic

        let hash3 = manager.simple_hash("other");
        assert_ne!(hash1, hash3); // Different inputs
    }
}
