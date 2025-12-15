# 生产级错误处理系统

> **状态**: ✅ 生产就绪  
> **版本**: V2.0  
> **更新日期**: 2025-11-25

---

## 📋 目录

1. [错误类型设计](#错误类型设计)
2. [错误上下文](#错误上下文)
3. [Sentry 集成](#sentry-集成)
4. [用户友好错误](#用户友好错误)
5. [错误恢复策略](#错误恢复策略)

---

## 🎯 错误类型设计

### 领域错误层次

```rust
// src/error/mod.rs
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// 顶层应用错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum AppError {
    /// 钱包相关错误
    #[error("Wallet error: {0}")]
    Wallet(#[from] WalletError),
    
    /// 交易相关错误
    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),
    
    /// 认证错误
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
    
    /// API 错误
    #[error("API error: {0}")]
    Api(#[from] ApiError),
    
    /// 加密错误
    #[error("Cryptography error: {0}")]
    Crypto(#[from] CryptoError),
    
    /// 存储错误
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    /// 网络错误
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    /// 验证错误
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// 内部错误（不应暴露给用户）
    #[error("Internal error: {0}")]
    Internal(String),
}

/// 钱包错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum WalletError {
    #[error("Wallet not found: {wallet_id}")]
    NotFound { wallet_id: String },
    
    #[error("Invalid mnemonic phrase")]
    InvalidMnemonic,
    
    #[error("Wallet already exists: {address}")]
    AlreadyExists { address: String },
    
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },
    
    #[error("Wallet is locked")]
    Locked,
    
    #[error("Failed to derive key: {reason}")]
    KeyDerivationFailed { reason: String },
}

/// 交易错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TransactionError {
    #[error("Invalid recipient address: {address}")]
    InvalidAddress { address: String },
    
    #[error("Invalid amount: {amount}")]
    InvalidAmount { amount: String },
    
    #[error("Gas estimation failed: {reason}")]
    GasEstimationFailed { reason: String },
    
    #[error("Transaction failed: {tx_hash}")]
    TransactionFailed { tx_hash: String },
    
    #[error("Transaction timeout")]
    Timeout,
    
    #[error("Nonce too low: expected {expected}, got {actual}")]
    NonceTooLow { expected: u64, actual: u64 },
    
    #[error("Insufficient gas: required {required}, provided {provided}")]
    InsufficientGas { required: u64, provided: u64 },
}

/// 认证错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Token invalid")]
    TokenInvalid,
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("2FA required")]
    TwoFactorRequired,
    
    #[error("2FA code invalid")]
    TwoFactorInvalid,
}

/// API 错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    #[error("Request failed: {status_code} - {message}")]
    RequestFailed { status_code: u16, message: String },
    
    #[error("Timeout after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },
    
    #[error("Rate limited: retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },
    
    #[error("Service unavailable")]
    ServiceUnavailable,
    
    #[error("Parse error: {message}")]
    ParseError { message: String },
}

/// 加密错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum CryptoError {
    #[error("Encryption failed")]
    EncryptionFailed,
    
    #[error("Decryption failed")]
    DecryptionFailed,
    
    #[error("Invalid key")]
    InvalidKey,
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Hash mismatch")]
    HashMismatch,
}

/// 存储错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum StorageError {
    #[error("Failed to read from storage: {reason}")]
    ReadFailed { reason: String },
    
    #[error("Failed to write to storage: {reason}")]
    WriteFailed { reason: String },
    
    #[error("Key not found: {key}")]
    KeyNotFound { key: String },
    
    #[error("Database error: {message}")]
    DatabaseError { message: String },
    
    #[error("Storage quota exceeded")]
    QuotaExceeded,
}

/// 网络错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum NetworkError {
    #[error("Connection failed: {reason}")]
    ConnectionFailed { reason: String },
    
    #[error("DNS resolution failed")]
    DnsResolutionFailed,
    
    #[error("TLS handshake failed")]
    TlsHandshakeFailed,
    
    #[error("Network timeout")]
    Timeout,
    
    #[error("No internet connection")]
    NoInternet,
}
```

---

## 📝 错误上下文

### 使用 anyhow 添加上下文

```rust
use anyhow::{Context, Result};

/// 创建钱包（带上下文）
pub async fn create_wallet(
    mnemonic: &str,
    password: &str,
) -> Result<Wallet> {
    // 验证助记词
    let mnemonic = Mnemonic::from_phrase(mnemonic)
        .context("Invalid mnemonic phrase")?;
    
    // 派生种子
    let seed = mnemonic.to_seed("")
        .context("Failed to derive seed from mnemonic")?;
    
    // 派生密钥
    let master_key = ExtendedPrivKey::new_master(Network::Bitcoin, &seed)
        .context("Failed to derive master key")?;
    
    // 加密助记词
    let encrypted = encrypt_mnemonic(mnemonic.phrase(), password)
        .context("Failed to encrypt mnemonic")?;
    
    // 保存到存储
    save_to_storage(&encrypted)
        .await
        .context("Failed to save wallet to storage")?;
    
    Ok(Wallet { /* ... */ })
}
```

### 错误链追踪

```rust
use std::error::Error;

/// 打印完整错误链
pub fn print_error_chain(err: &dyn Error) {
    eprintln!("Error: {}", err);
    
    let mut source = err.source();
    let mut level = 1;
    
    while let Some(err) = source {
        eprintln!("  Caused by ({}): {}", level, err);
        source = err.source();
        level += 1;
    }
}

// 使用示例
if let Err(e) = create_wallet(&mnemonic, &password).await {
    print_error_chain(&e);
}
```

---

## 🚨 Sentry 集成

### 初始化 Sentry

```rust
// src/monitoring/sentry.rs
use sentry::{ClientOptions, IntoDsn};

pub fn init_sentry(config: &SentryConfig) -> Option<sentry::ClientInitGuard> {
    if !config.enable {
        return None;
    }
    
    let guard = sentry::init((
        config.dsn.as_ref()?,
        ClientOptions {
            release: Some(env!("CARGO_PKG_VERSION").into()),
            environment: Some(config.environment.clone().into()),
            sample_rate: 1.0,
            traces_sample_rate: config.trace_sample_rate,
            attach_stacktrace: true,
            send_default_pii: false,  // 🔴 生产环境禁止发送 PII
            before_send: Some(Arc::new(|mut event| {
                // 过滤敏感信息
                filter_sensitive_data(&mut event);
                Some(event)
            })),
            ..Default::default()
        },
    ));
    
    Some(guard)
}

/// 过滤敏感信息
fn filter_sensitive_data(event: &mut sentry::protocol::Event<'static>) {
    // 移除私钥、助记词等敏感字段
    if let Some(extra) = &mut event.extra {
        extra.remove("private_key");
        extra.remove("mnemonic");
        extra.remove("password");
        extra.remove("jwt_token");
    }
    
    // 脱敏用户信息
    if let Some(user) = &mut event.user {
        if let Some(email) = &user.email {
            user.email = Some(mask_email(email));
        }
    }
}

/// 邮箱脱敏
fn mask_email(email: &str) -> String {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() == 2 {
        let name = parts[0];
        if name.len() > 2 {
            format!("{}***@{}", &name[..2], parts[1])
        } else {
            format!("***@{}", parts[1])
        }
    } else {
        "***@***".to_string()
    }
}
```

### 错误上报

```rust
/// 上报错误到 Sentry
pub fn report_error(error: &AppError, context: Option<ErrorContext>) {
    sentry::with_scope(|scope| {
        // 设置错误级别
        scope.set_level(match error {
            AppError::Internal(_) => sentry::Level::Error,
            AppError::Api(_) => sentry::Level::Warning,
            AppError::Validation(_) => sentry::Level::Info,
            _ => sentry::Level::Error,
        });
        
        // 添加标签
        scope.set_tag("error_type", error.error_type());
        
        // 添加上下文
        if let Some(ctx) = context {
            scope.set_context("error_context", sentry::protocol::Context::Other(
                serde_json::to_value(&ctx).unwrap()
            ));
        }
        
        // 添加用户信息（脱敏）
        if let Some(user_id) = get_current_user_id() {
            scope.set_user(Some(sentry::User {
                id: Some(user_id),
                ..Default::default()
            }));
        }
        
        // 上报错误
        sentry::capture_error(error);
    });
}

#[derive(Serialize)]
pub struct ErrorContext {
    pub wallet_id: Option<String>,
    pub chain: Option<String>,
    pub tx_hash: Option<String>,
    pub timestamp: i64,
}
```

### 性能监控

```rust
/// 性能监控
pub fn start_transaction(name: &str, op: &str) -> sentry::TransactionOrSpan {
    let ctx = sentry::TransactionContext::new(name, op);
    sentry::start_transaction(ctx)
}

// 使用示例
pub async fn send_transaction(tx: Transaction) -> Result<String> {
    let transaction = start_transaction("send_transaction", "transaction");
    
    // 阶段 1: 估算 Gas
    let span1 = transaction.start_child("estimate_gas", "gas");
    let gas_estimate = estimate_gas(&tx).await?;
    span1.finish();
    
    // 阶段 2: 签名交易
    let span2 = transaction.start_child("sign_transaction", "crypto");
    let signed_tx = sign_transaction(&tx).await?;
    span2.finish();
    
    // 阶段 3: 广播交易
    let span3 = transaction.start_child("broadcast_transaction", "rpc");
    let tx_hash = broadcast_transaction(&signed_tx).await?;
    span3.finish();
    
    transaction.finish();
    Ok(tx_hash)
}
```

---

## 👥 用户友好错误

### 错误消息国际化

```rust
// src/error/messages.rs
use std::collections::HashMap;

pub struct ErrorMessages {
    messages: HashMap<String, HashMap<String, String>>,
}

impl ErrorMessages {
    pub fn new() -> Self {
        let mut messages = HashMap::new();
        
        // 英文
        let mut en = HashMap::new();
        en.insert("wallet.not_found".to_string(), "Wallet not found. Please check the wallet ID.".to_string());
        en.insert("wallet.insufficient_balance".to_string(), "Insufficient balance. You need {required} but only have {available}.".to_string());
        en.insert("tx.invalid_address".to_string(), "Invalid recipient address. Please check and try again.".to_string());
        en.insert("auth.invalid_credentials".to_string(), "Invalid email or password.".to_string());
        messages.insert("en".to_string(), en);
        
        // 中文
        let mut zh = HashMap::new();
        zh.insert("wallet.not_found".to_string(), "找不到钱包，请检查钱包 ID。".to_string());
        zh.insert("wallet.insufficient_balance".to_string(), "余额不足。需要 {required}，但只有 {available}。".to_string());
        zh.insert("tx.invalid_address".to_string(), "收款地址无效，请检查后重试。".to_string());
        zh.insert("auth.invalid_credentials".to_string(), "邮箱或密码错误。".to_string());
        messages.insert("zh".to_string(), zh);
        
        Self { messages }
    }
    
    /// 获取用户友好的错误消息
    pub fn get(&self, error: &AppError, lang: &str) -> String {
        let key = error.message_key();
        
        self.messages
            .get(lang)
            .and_then(|lang_msgs| lang_msgs.get(&key))
            .cloned()
            .unwrap_or_else(|| error.to_string())
    }
}

impl AppError {
    /// 获取错误消息键
    pub fn message_key(&self) -> String {
        match self {
            AppError::Wallet(WalletError::NotFound { .. }) => "wallet.not_found".to_string(),
            AppError::Wallet(WalletError::InsufficientBalance { .. }) => "wallet.insufficient_balance".to_string(),
            AppError::Transaction(TransactionError::InvalidAddress { .. }) => "tx.invalid_address".to_string(),
            AppError::Auth(AuthError::InvalidCredentials) => "auth.invalid_credentials".to_string(),
            _ => "error.unknown".to_string(),
        }
    }
    
    /// 是否可以恢复
    pub fn is_recoverable(&self) -> bool {
        match self {
            AppError::Network(_) => true,
            AppError::Api(ApiError::Timeout { .. }) => true,
            AppError::Api(ApiError::RateLimited { .. }) => true,
            AppError::Storage(_) => true,
            _ => false,
        }
    }
    
    /// 获取错误级别
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::Internal(_) => ErrorSeverity::Critical,
            AppError::Crypto(_) => ErrorSeverity::Critical,
            AppError::Auth(_) => ErrorSeverity::High,
            AppError::Wallet(_) => ErrorSeverity::Medium,
            AppError::Transaction(_) => ErrorSeverity::Medium,
            AppError::Validation(_) => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorSeverity {
    Critical,  // 系统级错误
    High,      // 安全错误
    Medium,    // 业务错误
    Low,       // 验证错误
}
```

### UI 错误展示

```rust
// src/components/error_display.rs
use dioxus::prelude::*;

#[component]
pub fn ErrorDisplay(error: AppError, onclose: EventHandler<()>) -> Element {
    let messages = use_context::<ErrorMessages>();
    let lang = use_context::<Language>();
    
    let message = messages.get(&error, &lang.current());
    let icon = match error.severity() {
        ErrorSeverity::Critical => "🔴",
        ErrorSeverity::High => "🟠",
        ErrorSeverity::Medium => "🟡",
        ErrorSeverity::Low => "🔵",
    };
    
    rsx! {
        div {
            class: "error-notification",
            div { class: "error-icon", "{icon}" }
            div { class: "error-content",
                h3 { class: "error-title", "Error" }
                p { class: "error-message", "{message}" }
                
                // 可恢复错误显示重试按钮
                if error.is_recoverable() {
                    button {
                        class: "btn-retry",
                        onclick: move |_| {
                            // 重试逻辑
                        },
                        "Retry"
                    }
                }
            }
            button {
                class: "btn-close",
                onclick: move |_| onclose.call(()),
                "×"
            }
        }
    }
}
```

---

## 🔄 错误恢复策略

### 重试机制

```rust
use tokio::time::{sleep, Duration};

/// 指数退避重试
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_retries: u32,
    initial_delay_ms: u64,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = initial_delay_ms;
    
    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(err) if attempt >= max_retries => {
                tracing::error!("Operation failed after {} retries: {}", max_retries, err);
                return Err(err);
            }
            Err(err) => {
                attempt += 1;
                tracing::warn!("Attempt {} failed: {}. Retrying in {}ms...", attempt, err, delay);
                
                sleep(Duration::from_millis(delay)).await;
                
                // 指数退避: 1s, 2s, 4s, 8s, ...
                delay = (delay * 2).min(30000);  // 最大 30 秒
            }
        }
    }
}

// 使用示例
let result = retry_with_backoff(
    || fetch_balance(&wallet_address),
    3,  // 最多重试 3 次
    1000,  // 初始延迟 1 秒
).await?;
```

### 熔断器模式

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,    // 正常
    Open,      // 熔断
    HalfOpen,  // 半开（尝试恢复）
}

/// 熔断器
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout_duration: Duration,
    failure_count: Arc<RwLock<u32>>,
    success_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    pub fn new(
        failure_threshold: u32,
        success_threshold: u32,
        timeout_duration: Duration,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_threshold,
            success_threshold,
            timeout_duration,
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }
    
    /// 执行操作（带熔断保护）
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // 检查熔断器状态
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Open => {
                // 检查是否可以尝试恢复
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() > self.timeout_duration {
                        // 转换为半开状态
                        *self.state.write().await = CircuitState::HalfOpen;
                        *self.success_count.write().await = 0;
                    } else {
                        // 仍在熔断中
                        return Err(/* CircuitOpenError */);
                    }
                }
            }
            _ => {}
        }
        
        // 执行操作
        match operation() {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(err) => {
                self.on_failure().await;
                Err(err)
            }
        }
    }
    
    async fn on_success(&self) {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::HalfOpen => {
                *self.success_count.write().await += 1;
                if *self.success_count.read().await >= self.success_threshold {
                    // 恢复到正常状态
                    *self.state.write().await = CircuitState::Closed;
                    *self.failure_count.write().await = 0;
                    tracing::info!("Circuit breaker recovered");
                }
            }
            _ => {
                *self.failure_count.write().await = 0;
            }
        }
    }
    
    async fn on_failure(&self) {
        let state = *self.state.read().await;
        
        *self.failure_count.write().await += 1;
        *self.last_failure_time.write().await = Some(Instant::now());
        
        if state != CircuitState::Open && *self.failure_count.read().await >= self.failure_threshold {
            // 触发熔断
            *self.state.write().await = CircuitState::Open;
            tracing::warn!("Circuit breaker opened after {} failures", self.failure_threshold);
        }
    }
}
```

---

## 📚 依赖项

```toml
[dependencies]
thiserror = "1.0"
anyhow = "1.0"
sentry = { version = "0.32", features = ["tracing"] }
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["time"] }
```

---

## 🔗 相关文档

- [日志系统](./03-logging-system.md)
- [监控配置](./04-monitoring-setup.md)
- [告警规则](./05-alerting-rules.md)
