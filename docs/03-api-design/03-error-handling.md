# 错误处理方案

> **版本**: V2.0  
> **技术栈**: Rust + Dioxus 0.7 + anyhow/thiserror  
> **更新日期**: 2025-11-25  
> **设计目标**: 类型安全、用户友好、可调试

---

## 📋 目录

1. [错误处理架构](#错误处理架构)
2. [错误类型设计](#错误类型设计)
3. [前端错误映射](#前端错误映射)
4. [用户友好提示](#用户友好提示)
5. [错误日志记录](#错误日志记录)
6. [重试策略](#重试策略)
7. [完整实现示例](#完整实现示例)
8. [测试错误处理](#测试错误处理)

---

## 错误处理架构

### 分层错误处理

```
┌─────────────────────────────────────────────────────────┐
│                    UI Layer                              │
│  显示用户友好的错误消息                                    │
│  - "Network connection failed"                           │
│  - "Insufficient balance"                                │
└────────────────┬────────────────────────────────────────┘
                 │ DisplayError (用户可见)
                 │
┌────────────────▼────────────────────────────────────────┐
│              Service Layer                               │
│  业务逻辑错误（领域特定）                                  │
│  - WalletError                                           │
│  - TransactionError                                      │
│  - SecurityError                                         │
└────────────────┬────────────────────────────────────────┘
                 │ DomainError (业务错误)
                 │
┌────────────────▼────────────────────────────────────────┐
│         Infrastructure Layer                             │
│  底层错误（技术性）                                        │
│  - ApiError (HTTP 错误)                                  │
│  - StorageError (IndexedDB 错误)                        │
│  - CryptoError (加密错误)                               │
└─────────────────────────────────────────────────────────┘
```

### 设计原则

1. **类型安全**: 使用 Rust 的 Result<T, E> 而非异常
2. **错误链**: 保留原始错误上下文（使用 `anyhow::Context`）
3. **用户友好**: 技术错误转换为可读消息
4. **可调试**: 开发环境显示详细堆栈
5. **国际化**: 支持多语言错误消息

---

## 错误类型设计

### 基础错误类型

```rust
// src/error/mod.rs
use thiserror::Error;

/// 应用级错误（顶层）
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Wallet error: {0}")]
    Wallet(#[from] WalletError),
    
    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),
    
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("API error: {0}")]
    Api(#[from] ApiError),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// 转换为用户友好的错误消息
    pub fn to_display_message(&self) -> String {
        match self {
            Self::Wallet(e) => e.to_display_message(),
            Self::Transaction(e) => e.to_display_message(),
            Self::Security(e) => e.to_display_message(),
            Self::Api(e) => e.to_display_message(),
            Self::Storage(e) => e.to_display_message(),
            Self::Network(_) => "Network connection failed. Please check your internet connection.".to_string(),
            Self::Validation(msg) => msg.clone(),
            Self::Internal(_) => "An unexpected error occurred. Please try again.".to_string(),
        }
    }
    
    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::Api(ApiError::Timeout) | Self::Api(ApiError::ServerError(_))
        )
    }
    
    /// 获取错误代码（用于日志/追踪）
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Wallet(e) => e.error_code(),
            Self::Transaction(e) => e.error_code(),
            Self::Security(e) => e.error_code(),
            Self::Api(e) => e.error_code(),
            Self::Storage(e) => e.error_code(),
            Self::Network(_) => "ERR_NETWORK",
            Self::Validation(_) => "ERR_VALIDATION",
            Self::Internal(_) => "ERR_INTERNAL",
        }
    }
}
```

### 钱包错误

```rust
// src/error/wallet_error.rs
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum WalletError {
    #[error("Wallet not found: {0}")]
    NotFound(String),
    
    #[error("Wallet name already exists: {0}")]
    NameAlreadyExists(String),
    
    #[error("Invalid wallet name: {0}")]
    InvalidName(String),
    
    #[error("Wallet is locked")]
    Locked,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Invalid password")]
    InvalidPassword,
    
    #[error("Mnemonic generation failed: {0}")]
    MnemonicGenerationFailed(String),
    
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    
    #[error("Address generation failed: {0}")]
    AddressGenerationFailed(String),
}

impl WalletError {
    pub fn to_display_message(&self) -> String {
        match self {
            Self::NotFound(name) => format!("Wallet '{}' not found.", name),
            Self::NameAlreadyExists(name) => format!("A wallet named '{}' already exists. Please choose a different name.", name),
            Self::InvalidName(reason) => format!("Invalid wallet name: {}", reason),
            Self::Locked => "Wallet is locked. Please unlock it first.".to_string(),
            Self::SessionExpired => "Your session has expired. Please unlock your wallet again.".to_string(),
            Self::InvalidPassword => "Incorrect password. Please try again.".to_string(),
            Self::MnemonicGenerationFailed(_) => "Failed to generate recovery phrase. Please try again.".to_string(),
            Self::InvalidMnemonic(_) => "Invalid recovery phrase. Please check and try again.".to_string(),
            Self::KeyDerivationFailed(_) => "Failed to derive wallet keys. Please contact support.".to_string(),
            Self::AddressGenerationFailed(_) => "Failed to generate wallet address. Please try again.".to_string(),
        }
    }
    
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "WALLET_NOT_FOUND",
            Self::NameAlreadyExists(_) => "WALLET_NAME_EXISTS",
            Self::InvalidName(_) => "WALLET_INVALID_NAME",
            Self::Locked => "WALLET_LOCKED",
            Self::SessionExpired => "WALLET_SESSION_EXPIRED",
            Self::InvalidPassword => "WALLET_INVALID_PASSWORD",
            Self::MnemonicGenerationFailed(_) => "WALLET_MNEMONIC_GEN_FAILED",
            Self::InvalidMnemonic(_) => "WALLET_INVALID_MNEMONIC",
            Self::KeyDerivationFailed(_) => "WALLET_KEY_DERIVATION_FAILED",
            Self::AddressGenerationFailed(_) => "WALLET_ADDRESS_GEN_FAILED",
        }
    }
}
```

### 交易错误

```rust
// src/error/transaction_error.rs
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TransactionError {
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },
    
    #[error("Gas estimation failed: {0}")]
    GasEstimationFailed(String),
    
    #[error("Invalid recipient address: {0}")]
    InvalidRecipient(String),
    
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    
    #[error("Transaction signing failed: {0}")]
    SigningFailed(String),
    
    #[error("Transaction broadcast failed: {0}")]
    BroadcastFailed(String),
    
    #[error("Transaction rejected by user")]
    RejectedByUser,
    
    #[error("Nonce too low")]
    NonceTooLow,
    
    #[error("Gas price too low")]
    GasPriceTooLow,
    
    #[error("Transaction timeout")]
    Timeout,
    
    #[error("Transaction failed: {0}")]
    Failed(String),
}

impl TransactionError {
    pub fn to_display_message(&self) -> String {
        match self {
            Self::InsufficientBalance { required, available } => {
                format!("Insufficient balance. Required: {}, Available: {}", required, available)
            }
            Self::GasEstimationFailed(_) => "Unable to estimate gas fees. The transaction may fail.".to_string(),
            Self::InvalidRecipient(addr) => format!("Invalid recipient address: {}", addr),
            Self::InvalidAmount(reason) => format!("Invalid amount: {}", reason),
            Self::SigningFailed(_) => "Transaction signing failed. Please try again.".to_string(),
            Self::BroadcastFailed(_) => "Failed to send transaction. Please check your network connection.".to_string(),
            Self::RejectedByUser => "Transaction cancelled by user.".to_string(),
            Self::NonceTooLow => "Transaction nonce conflict. Please refresh and try again.".to_string(),
            Self::GasPriceTooLow => "Gas price too low. Increase gas price and try again.".to_string(),
            Self::Timeout => "Transaction timeout. It may still be processing.".to_string(),
            Self::Failed(reason) => format!("Transaction failed: {}", reason),
        }
    }
    
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InsufficientBalance { .. } => "TX_INSUFFICIENT_BALANCE",
            Self::GasEstimationFailed(_) => "TX_GAS_ESTIMATION_FAILED",
            Self::InvalidRecipient(_) => "TX_INVALID_RECIPIENT",
            Self::InvalidAmount(_) => "TX_INVALID_AMOUNT",
            Self::SigningFailed(_) => "TX_SIGNING_FAILED",
            Self::BroadcastFailed(_) => "TX_BROADCAST_FAILED",
            Self::RejectedByUser => "TX_REJECTED_BY_USER",
            Self::NonceTooLow => "TX_NONCE_TOO_LOW",
            Self::GasPriceTooLow => "TX_GAS_PRICE_TOO_LOW",
            Self::Timeout => "TX_TIMEOUT",
            Self::Failed(_) => "TX_FAILED",
        }
    }
}
```

### API 错误

```rust
// src/error/api_error.rs
use thiserror::Error;
use gloo_net::http::Response;

#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("Network request failed: {0}")]
    NetworkError(String),
    
    #[error("Request timeout")]
    Timeout,
    
    #[error("Bad request (400): {0}")]
    BadRequest(String),
    
    #[error("Unauthorized (401)")]
    Unauthorized,
    
    #[error("Forbidden (403)")]
    Forbidden,
    
    #[error("Not found (404)")]
    NotFound,
    
    #[error("Rate limited (429)")]
    RateLimited,
    
    #[error("Server error (500): {0}")]
    ServerError(String),
    
    #[error("Service unavailable (503)")]
    ServiceUnavailable,
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

impl ApiError {
    /// 从 HTTP 响应创建错误
    pub async fn from_response(response: Response) -> Self {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        
        match status {
            400 => Self::BadRequest(body),
            401 => Self::Unauthorized,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            429 => Self::RateLimited,
            500..=599 => Self::ServerError(body),
            503 => Self::ServiceUnavailable,
            _ => Self::NetworkError(format!("HTTP {}: {}", status, body)),
        }
    }
    
    pub fn to_display_message(&self) -> String {
        match self {
            Self::NetworkError(_) => "Network connection failed. Please check your internet.".to_string(),
            Self::Timeout => "Request timeout. Please try again.".to_string(),
            Self::BadRequest(msg) => format!("Invalid request: {}", msg),
            Self::Unauthorized => "Authentication required. Please log in.".to_string(),
            Self::Forbidden => "Access denied.".to_string(),
            Self::NotFound => "Resource not found.".to_string(),
            Self::RateLimited => "Too many requests. Please wait a moment and try again.".to_string(),
            Self::ServerError(_) => "Server error. Please try again later.".to_string(),
            Self::ServiceUnavailable => "Service temporarily unavailable. Please try again later.".to_string(),
            Self::InvalidResponse(_) => "Invalid server response. Please contact support.".to_string(),
        }
    }
    
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NetworkError(_) => "API_NETWORK_ERROR",
            Self::Timeout => "API_TIMEOUT",
            Self::BadRequest(_) => "API_BAD_REQUEST",
            Self::Unauthorized => "API_UNAUTHORIZED",
            Self::Forbidden => "API_FORBIDDEN",
            Self::NotFound => "API_NOT_FOUND",
            Self::RateLimited => "API_RATE_LIMITED",
            Self::ServerError(_) => "API_SERVER_ERROR",
            Self::ServiceUnavailable => "API_SERVICE_UNAVAILABLE",
            Self::InvalidResponse(_) => "API_INVALID_RESPONSE",
        }
    }
}
```

---

## 前端错误映射

### 后端错误码映射

```rust
// src/infrastructure/api/error_mapper.rs
use serde::{Deserialize, Serialize};
use crate::error::{AppError, WalletError, TransactionError};

/// 后端 API 错误响应格式
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub context: Option<serde_json::Value>,
}

/// 将后端错误映射为前端错误
pub fn map_api_error(api_error: ApiErrorResponse) -> AppError {
    match api_error.error.as_str() {
        // 钱包错误
        "WalletNotFound" => AppError::Wallet(WalletError::NotFound(
            api_error.context
                .and_then(|c| c.get("wallet_name").and_then(|n| n.as_str().map(String::from)))
                .unwrap_or_default()
        )),
        "WalletNameExists" => AppError::Wallet(WalletError::NameAlreadyExists(
            api_error.message
        )),
        "InvalidPassword" => AppError::Wallet(WalletError::InvalidPassword),
        
        // 交易错误
        "InsufficientBalance" => {
            let required = api_error.context
                .as_ref()
                .and_then(|c| c.get("required").and_then(|v| v.as_str()))
                .unwrap_or("0")
                .to_string();
            let available = api_error.context
                .as_ref()
                .and_then(|c| c.get("available").and_then(|v| v.as_str()))
                .unwrap_or("0")
                .to_string();
            
            AppError::Transaction(TransactionError::InsufficientBalance { required, available })
        }
        "GasEstimationFailed" => AppError::Transaction(
            TransactionError::GasEstimationFailed(api_error.message)
        ),
        "NonceTooLow" => AppError::Transaction(TransactionError::NonceTooLow),
        
        // 默认映射
        _ => AppError::Api(ApiError::ServerError(api_error.message)),
    }
}
```

---

## 用户友好提示

### 错误提示组件

```rust
// src/ui/components/atoms/error_message.rs
use dioxus::prelude::*;
use crate::error::AppError;

#[derive(Props, PartialEq, Clone)]
pub struct ErrorMessageProps {
    pub error: AppError,
    #[props(optional)]
    pub on_retry: Option<EventHandler<()>>,
    #[props(optional)]
    pub on_dismiss: Option<EventHandler<()>>,
}

pub fn ErrorMessage(props: ErrorMessageProps) -> Element {
    let error_message = props.error.to_display_message();
    let error_code = props.error.error_code();
    let is_retryable = props.error.is_retryable();
    
    rsx! {
        div {
            class: "error-message",
            role: "alert",
            "aria-live": "polite",
            
            // 错误图标
            div { class: "error-message__icon", "⚠️" }
            
            // 错误消息
            div { class: "error-message__content",
                p { class: "error-message__text", "{error_message}" }
                
                // 开发模式显示错误代码
                {#[cfg(debug_assertions)]
                rsx! {
                    small { class: "error-message__code", "Error Code: {error_code}" }
                }}
            }
            
            // 操作按钮
            div { class: "error-message__actions",
                if is_retryable {
                    if let Some(on_retry) = props.on_retry {
                        button {
                            onclick: move |_| on_retry.call(()),
                            "Retry"
                        }
                    }
                }
                
                if let Some(on_dismiss) = props.on_dismiss {
                    button {
                        onclick: move |_| on_dismiss.call(()),
                        "Dismiss"
                    }
                }
            }
        }
    }
}
```

### Toast 通知

```rust
// src/ui/components/molecules/toast.rs
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ToastType {
    Error,
    Warning,
    Success,
    Info,
}

pub fn show_error_toast(error: AppError) {
    let toast_service = use_context::<ToastService>();
    toast_service.show(Toast {
        toast_type: ToastType::Error,
        message: error.to_display_message(),
        duration: 5000, // 5 秒
        action: if error.is_retryable() {
            Some(ToastAction {
                label: "Retry".to_string(),
                callback: Box::new(|| {
                    // 重试逻辑
                }),
            })
        } else {
            None
        },
    });
}
```

---

## 错误日志记录

### 日志系统

```rust
// src/infrastructure/logging/error_logger.rs
use tracing::{error, warn, info};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorLogEntry {
    pub timestamp: u64,
    pub error_code: String,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub user_id: Option<String>,
    pub context: serde_json::Value,
}

pub fn log_error(error: &AppError, context: serde_json::Value) {
    let entry = ErrorLogEntry {
        timestamp: current_timestamp(),
        error_code: error.error_code().to_string(),
        error_message: format!("{}", error),
        stack_trace: Some(format!("{:?}", error)),
        user_id: get_current_user_id(),
        context,
    };
    
    // 生产环境：发送到日志服务
    #[cfg(not(debug_assertions))]
    {
        // 发送到 Sentry/LogRocket 等服务
        send_to_logging_service(&entry);
    }
    
    // 开发环境：打印到控制台
    #[cfg(debug_assertions)]
    {
        error!(
            error_code = %entry.error_code,
            error_message = %entry.error_message,
            "Application error occurred"
        );
    }
}
```

---

## 重试策略

### 指数退避重试

```rust
// src/infrastructure/retry.rs
use std::time::Duration;
use gloo_timers::future::sleep;

pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

/// 带重试的异步操作
pub async fn retry_with_backoff<F, T, E>(
    config: RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut delay = config.initial_delay;
    
    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt == config.max_attempts => return Err(e),
            Err(e) => {
                tracing::warn!(
                    attempt = attempt,
                    max_attempts = config.max_attempts,
                    error = ?e,
                    "Operation failed, retrying..."
                );
                
                sleep(delay).await;
                
                // 指数退避
                delay = std::cmp::min(
                    Duration::from_secs_f64(delay.as_secs_f64() * config.backoff_multiplier),
                    config.max_delay,
                );
            }
        }
    }
    
    unreachable!()
}

/// 使用示例
pub async fn fetch_balance_with_retry(address: &str) -> Result<String, ApiError> {
    retry_with_backoff(
        RetryConfig::default(),
        || {
            let address = address.to_string();
            Box::pin(async move {
                api_client.get_balance(&address).await
            })
        },
    ).await
}
```

---

## 完整实现示例

### 带错误处理的发送交易

```rust
// src/pages/send_transaction.rs
pub fn SendTransactionPage() -> Element {
    let mut error = use_signal(|| Option::<AppError>::None);
    let mut is_submitting = use_signal(|| false);
    
    let submit_transaction = move |tx: UnsignedTransaction| {
        spawn(async move {
            is_submitting.set(true);
            error.set(None);
            
            // 使用 ? 操作符传播错误
            let result: Result<String, AppError> = async {
                // 1. 签名交易
                let signed_tx = key_manager
                    .sign_transaction(wallet_id, chain_id, tx)
                    .await
                    .map_err(AppError::from)?;
                
                // 2. 广播交易（带重试）
                let tx_hash = retry_with_backoff(
                    RetryConfig::default(),
                    || Box::pin(api_client.broadcast_transaction(&signed_tx.raw_transaction))
                ).await
                .map_err(AppError::from)?;
                
                Ok(tx_hash)
            }.await;
            
            is_submitting.set(false);
            
            match result {
                Ok(tx_hash) => {
                    // 成功：跳转到交易详情
                    navigator().push(Route::TransactionDetail { tx_hash });
                }
                Err(e) => {
                    // 失败：显示错误
                    log_error(&e, json!({ "operation": "send_transaction" }));
                    error.set(Some(e));
                }
            }
        });
    };
    
    rsx! {
        div { class: "send-transaction-page",
            // 错误提示
            if let Some(err) = error() {
                ErrorMessage {
                    error: err,
                    on_retry: move |_| {
                        error.set(None);
                        // 重新提交
                    },
                    on_dismiss: move |_| error.set(None),
                }
            }
            
            // 表单...
        }
    }
}
```

---

## 测试错误处理

```rust
// tests/error_handling_test.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_error_display_message() {
        let error = WalletError::InvalidPassword;
        assert_eq!(
            error.to_display_message(),
            "Incorrect password. Please try again."
        );
    }
    
    #[test]
    fn test_error_retryability() {
        let network_error = AppError::Network(NetworkError::Timeout);
        assert!(network_error.is_retryable());
        
        let validation_error = AppError::Validation("Invalid input".to_string());
        assert!(!validation_error.is_retryable());
    }
    
    #[tokio::test]
    async fn test_retry_with_backoff() {
        let mut attempt = 0;
        
        let result = retry_with_backoff(
            RetryConfig { max_attempts: 3, ..Default::default() },
            || {
                attempt += 1;
                Box::pin(async move {
                    if attempt < 3 {
                        Err("Simulated failure")
                    } else {
                        Ok("Success")
                    }
                })
            },
        ).await;
        
        assert_eq!(result, Ok("Success"));
        assert_eq!(attempt, 3);
    }
}
```

---

## 参考资料

- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror Documentation](https://docs.rs/thiserror/)
- [anyhow Documentation](https://docs.rs/anyhow/)
- [Error Handling in Production Rust](https://www.lpalmieri.com/posts/error-handling-rust/)
