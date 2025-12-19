# 生产级配置管理系统

> **状态**: ✅ 生产就绪  
> **版本**: V2.0  
> **更新日期**: 2025-11-25

---

## 📋 配置文件概览

### 配置层级

```
环境变量 (.env)
    ↓ 覆盖
配置文件 (config.toml)
    ↓ 覆盖
默认值 (代码中)
```

### 配置文件

1. **`.env.example`** - 环境变量模板
2. **`config.toml.example`** - 主配置文件模板
3. **`config.rs`** - 配置加载逻辑（需实现）

---

## 🚀 快速开始

### 1. 初始化配置

```bash
# 复制环境变量模板
cp .env.example .env

# 复制配置文件模板
cp config.toml.example config.toml

# 编辑 .env 填入真实值
nano .env
```

### 2. 环境变量配置

#### 必填项（生产环境）

```bash
# 后端 API
API_BASE_URL=https://<your-backend-host>

# JWT 密钥（生成方式：openssl rand -base64 64）
JWT_SECRET=your-strong-random-key-min-32-bytes

# Sentry 监控
SENTRY_DSN=https://your-sentry-dsn@sentry.io/project-id
```

#### 区块链 RPC（推荐付费节点）

```bash
# Ethereum（推荐 Alchemy 或 Infura）
ETH_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY

# BSC
BSC_RPC_URL=https://bsc-dataseed.binance.org/

# Polygon
POLYGON_RPC_URL=https://polygon-rpc.com/
```

### 3. 配置验证

```rust
// src/config.rs
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub app: AppInfo,
    pub server: ServerConfig,
    pub backend: BackendConfig,
    pub auth: AuthConfig,
    pub blockchain: BlockchainConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
}

impl AppConfig {
    /// 从文件和环境变量加载配置
    pub fn from_env() -> Result<Self, ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
        
        let config = Config::builder()
            // 1. 加载默认配置
            .add_source(File::with_name("config.toml").required(false))
            // 2. 加载环境特定配置
            .add_source(File::with_name(&format!("config.{}.toml", env)).required(false))
            // 3. 环境变量覆盖（前缀 IRONFORGE_）
            .add_source(Environment::with_prefix("IRONFORGE").separator("__"))
            .build()?;
        
        config.try_deserialize()
    }
    
    /// 验证配置是否符合生产标准
    pub fn validate_production(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // 检查 HTTPS
        if self.app.environment == "production" {
            if !self.backend.base_url.starts_with("https://") {
                errors.push("生产环境必须使用 HTTPS".to_string());
            }
            
            // 检查 JWT 密钥长度
            if self.auth.jwt_secret.len() < 32 {
                errors.push("JWT 密钥长度必须至少 32 字节".to_string());
            }
            
            // 检查是否使用默认密钥
            if self.auth.jwt_secret.contains("CHANGE_THIS") {
                errors.push("必须更改默认 JWT 密钥".to_string());
            }
            
            // 检查 Sentry
            if !self.monitoring.sentry.enable {
                errors.push("生产环境必须启用 Sentry 监控".to_string());
            }
            
            // 检查日志级别
            if self.logging.level == "debug" || self.logging.level == "trace" {
                errors.push("生产环境日志级别应为 info/warn/error".to_string());
            }
            
            // 检查功能开关
            if self.features.enable_testnet {
                errors.push("生产环境必须禁用测试网络".to_string());
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// 配置结构体定义
#[derive(Debug, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub environment: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub use_https: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
    pub base_url: String,
    pub ws_url: String,
    pub timeout: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_secs: u64,
    pub refresh_token_expiry_secs: u64,
    pub session_timeout_mins: u32,
    pub auto_lock_timeout_mins: u32,
}

#[derive(Debug, Deserialize)]
pub struct SecurityConfig {
    pub encryption: EncryptionConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: String,
    pub argon2_memory_kb: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,
    pub key_rotation_days: u32,
}

#[derive(Debug, Deserialize)]
pub struct MonitoringConfig {
    pub sentry: SentryConfig,
}

#[derive(Debug, Deserialize)]
pub struct SentryConfig {
    pub enable: bool,
    pub dsn: Option<String>,
    pub environment: String,
    pub trace_sample_rate: f32,
}
```

---

## 🔒 安全最佳实践

### 1. 密钥管理

```bash
# ❌ 错误：直接在代码中硬编码
const JWT_SECRET = "my-secret-key";

# ✅ 正确：从环境变量读取
let jwt_secret = std::env::var("JWT_SECRET")
    .expect("JWT_SECRET must be set");

# ✅ 更好：使用密钥管理服务
# AWS Secrets Manager / HashiCorp Vault / Azure Key Vault
```

### 2. 密钥轮转

```rust
/// 密钥轮转策略
pub struct KeyRotationPolicy {
    /// 当前密钥 ID
    pub current_key_id: String,
    /// 密钥创建时间
    pub created_at: DateTime<Utc>,
    /// 轮转周期（天）
    pub rotation_days: u32,
    /// 历史密钥（用于解密旧数据）
    pub previous_keys: Vec<KeyVersion>,
}

impl KeyRotationPolicy {
    /// 检查是否需要轮转
    pub fn should_rotate(&self) -> bool {
        let now = Utc::now();
        let age_days = (now - self.created_at).num_days();
        age_days >= self.rotation_days as i64
    }
    
    /// 执行密钥轮转
    pub async fn rotate(&mut self) -> Result<()> {
        // 1. 生成新密钥
        let new_key = generate_strong_key()?;
        
        // 2. 保存旧密钥到历史
        self.previous_keys.push(KeyVersion {
            key_id: self.current_key_id.clone(),
            created_at: self.created_at,
            expires_at: Utc::now() + Duration::days(30),
        });
        
        // 3. 更新当前密钥
        self.current_key_id = new_key.id;
        self.created_at = Utc::now();
        
        // 4. 通知服务重新加载密钥
        notify_key_rotation(&new_key).await?;
        
        Ok(())
    }
}
```

### 3. 配置加密

```rust
/// 加密敏感配置项
pub fn encrypt_config_value(value: &str, master_key: &[u8]) -> Result<String> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;
    
    let cipher = Aes256Gcm::new(master_key.into());
    let nonce = Nonce::from_slice(b"unique nonce");
    
    let ciphertext = cipher.encrypt(nonce, value.as_bytes())
        .map_err(|_| Error::EncryptionFailed)?;
    
    // Base64 编码
    Ok(base64::encode(ciphertext))
}

// 配置文件中存储加密值
// [auth]
// jwt_secret = "ENC:base64encodedvalue"
```

---

## 🌍 多环境配置

### 环境划分

```
development  - 开发环境（本地）
staging      - 预发布环境（测试）
production   - 生产环境（线上）
```

### 环境切换

```bash
# 方式 1：环境变量
export APP_ENV=production
cargo run

# 方式 2：命令行参数
cargo run -- --env production

# 方式 3：配置文件
# 自动加载 config.production.toml
```

### 环境特定配置

```toml
# config.development.toml
[logging]
level = "debug"

[monitoring.sentry]
enable = false

# config.production.toml
[logging]
level = "warn"
file = true

[monitoring.sentry]
enable = true
trace_sample_rate = 0.05
```

---

## 📊 配置监控

### 配置变更审计

```rust
/// 配置变更日志
pub struct ConfigAuditLog {
    pub timestamp: DateTime<Utc>,
    pub user: String,
    pub key: String,
    pub old_value: Option<String>,  // 脱敏
    pub new_value: Option<String>,  // 脱敏
    pub environment: String,
}

impl ConfigAuditLog {
    /// 记录配置变更
    pub async fn log_change(
        key: &str,
        old_value: Option<&str>,
        new_value: Option<&str>,
    ) -> Result<()> {
        let log = ConfigAuditLog {
            timestamp: Utc::now(),
            user: get_current_user()?,
            key: key.to_string(),
            old_value: old_value.map(|v| sanitize_value(v)),
            new_value: new_value.map(|v| sanitize_value(v)),
            environment: get_environment(),
        };
        
        // 发送到审计日志系统
        send_to_audit_log(&log).await?;
        Ok(())
    }
}

/// 脱敏敏感值
fn sanitize_value(value: &str) -> String {
    if value.len() > 8 {
        format!("{}****{}", &value[..4], &value[value.len()-4..])
    } else {
        "****".to_string()
    }
}
```

---

## 🧪 配置测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_development_config() {
        std::env::set_var("APP_ENV", "development");
        let config = AppConfig::from_env().unwrap();
        
        assert_eq!(config.app.environment, "development");
        assert_eq!(config.logging.level, "debug");
    }
    
    #[test]
    fn test_production_validation() {
        let mut config = AppConfig::default();
        config.app.environment = "production".to_string();
        config.backend.base_url = "http://api.example.com".to_string();
        
        let result = config.validate_production();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("HTTPS")));
    }
    
    #[test]
    fn test_jwt_secret_strength() {
        let config = AppConfig::from_env().unwrap();
        
        // 生产环境检查
        if config.app.environment == "production" {
            assert!(config.auth.jwt_secret.len() >= 32);
            assert!(!config.auth.jwt_secret.contains("CHANGE_THIS"));
        }
    }
}
```

---

## 📚 依赖项

```toml
# Cargo.toml
[dependencies]
config = "0.13"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
```

---

## 🚨 生产环境检查清单

部署前必须确认：

- [ ] 所有 `.example` 文件已复制并填入真实值
- [ ] JWT_SECRET 已更改为强随机密钥（≥32字节）
- [ ] 所有 API 密钥已更新为生产环境密钥
- [ ] backend.base_url 使用 HTTPS
- [ ] Sentry 监控已启用
- [ ] 日志级别设置为 warn 或 error
- [ ] 测试网络已禁用
- [ ] 敏感配置已加密存储
- [ ] 配置变更已通过审计
- [ ] 备份当前配置文件

---

## 🔗 相关文档

- [安全架构](../04-security/03-security-architecture.md)
- [密钥管理](../04-security/01-key-management.md)
- [监控配置](./03-monitoring-setup.md)
- [部署指南](./04-deployment-guide.md)
