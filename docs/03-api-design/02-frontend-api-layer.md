# 前端 API 封装层设计

> **版本**: V2.0  
> **技术栈**: Dioxus 0.7 + gloo-net + serde  
> **更新日期**: 2025-11-25  
> **参考**: [IronCore Backend API Reference](./01-ironcore-backend-api-reference.md)

---

## 📋 目录

1. [设计原则](#设计原则)
2. [API 客户端架构](#api-客户端架构)
3. [请求/响应类型](#请求响应类型)
4. [认证管理](#认证管理)
5. [错误处理](#错误处理)
6. [缓存策略](#缓存策略)
7. [完整实现示例](#完整实现示例)

---

## 设计原则

### 核心理念

1. **类型安全**: 所有请求/响应都有明确的 Rust 类型
2. **错误处理**: 统一的错误类型和处理流程
3. **自动重试**: 网络错误自动重试（最多 3 次）
4. **Token 管理**: 自动注入 JWT Token，自动刷新
5. **请求拦截**: 统一添加认证、日志、监控
6. **响应拦截**: 统一处理错误、缓存、状态码

### 分层设计

```
┌─────────────────────────────────────────────┐
│         UI Layer (Pages/Components)         │
└────────────────┬────────────────────────────┘
                 │
┌────────────────▼────────────────────────────┐
│      Service Layer (业务逻辑封装)            │
│  - WalletService                            │
│  - TransactionService                       │
│  - AuthService                              │
└────────────────┬────────────────────────────┘
                 │
┌────────────────▼────────────────────────────┐
│     API Client Layer (HTTP 请求封装)         │
│  - ApiClient (统一 HTTP 客户端)             │
│  - RequestBuilder (请求构造器)              │
│  - ResponseHandler (响应处理器)             │
└────────────────┬────────────────────────────┘
                 │
┌────────────────▼────────────────────────────┐
│         Infrastructure Layer                │
│  - TokenManager (Token 管理)                │
│  - CacheManager (缓存管理)                  │
│  - ErrorMapper (错误映射)                   │
└─────────────────────────────────────────────┘
```

---

## API 客户端架构

### 1. 核心 ApiClient

```rust
// src/domain/services/api_client.rs

use gloo_net::http::{Request, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// API 客户端配置
#[derive(Clone, Debug)]
pub struct ApiConfig {
    /// 后端 API 基础 URL
    pub base_url: String,
    /// 请求超时时间（秒）
    pub timeout: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 是否启用缓存
    pub enable_cache: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: option_env!("API_BASE_URL")
                .unwrap_or("http://localhost:8088")  // New backend (modern)
                .to_string(),
            timeout: 30,
            max_retries: 3,
            enable_cache: true,
        }
    }
}

/// API 客户端
pub struct ApiClient {
    config: ApiConfig,
    token_manager: Arc<TokenManager>,
    cache_manager: Arc<CacheManager>,
}

impl ApiClient {
    /// 创建新的 API 客户端
    pub fn new() -> Self {
        Self {
            config: ApiConfig::default(),
            token_manager: Arc::new(TokenManager::new()),
            cache_manager: Arc::new(CacheManager::new()),
        }
    }

    /// 使用自定义配置创建客户端
    pub fn with_config(config: ApiConfig) -> Self {
        Self {
            config,
            token_manager: Arc::new(TokenManager::new()),
            cache_manager: Arc::new(CacheManager::new()),
        }
    }

    /// 构建完整的 URL
    fn build_url(&self, path: &str) -> String {
        if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", self.config.base_url, path)
        }
    }

    /// 发送 GET 请求
    pub async fn get<T>(&self, path: &str) -> Result<T, ApiError>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.request(Method::Get, path, None::<()>).await
    }

    /// 发送 POST 请求
    pub async fn post<B, T>(&self, path: &str, body: B) -> Result<T, ApiError>
    where
        B: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        self.request(Method::Post, path, Some(body)).await
    }

    /// 发送 PUT 请求
    pub async fn put<B, T>(&self, path: &str, body: B) -> Result<T, ApiError>
    where
        B: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        self.request(Method::Put, path, Some(body)).await
    }

    /// 发送 DELETE 请求
    pub async fn delete<T>(&self, path: &str) -> Result<T, ApiError>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.request(Method::Delete, path, None::<()>).await
    }

    /// 核心请求方法（带重试和缓存）
    async fn request<B, T>(
        &self,
        method: Method,
        path: &str,
        body: Option<B>,
    ) -> Result<T, ApiError>
    where
        B: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        let url = self.build_url(path);

        // 检查缓存（仅 GET 请求）
        if matches!(method, Method::Get) && self.config.enable_cache {
            if let Some(cached) = self.cache_manager.get::<T>(&url).await {
                tracing::debug!("Cache hit: {}", url);
                return Ok(cached);
            }
        }

        // 重试逻辑
        let mut retries = 0;
        loop {
            match self.execute_request(&method, &url, &body).await {
                Ok(response) => {
                    // 缓存成功响应（仅 GET 请求）
                    if matches!(method, Method::Get) && self.config.enable_cache {
                        self.cache_manager.set(&url, &response, 300).await; // 5分钟
                    }
                    return Ok(response);
                }
                Err(e) if retries < self.config.max_retries && e.is_retryable() => {
                    retries += 1;
                    tracing::warn!("Request failed, retrying ({}/{}): {:?}", 
                        retries, self.config.max_retries, e);
                    gloo_timers::future::TimeoutFuture::new(1000 * retries).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// 执行单次请求
    async fn execute_request<B, T>(
        &self,
        method: &Method,
        url: &str,
        body: &Option<B>,
    ) -> Result<T, ApiError>
    where
        B: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        // 构建请求
        let mut request = match method {
            Method::Get => Request::get(url),
            Method::Post => Request::post(url),
            Method::Put => Request::put(url),
            Method::Delete => Request::delete(url),
        };

        // 添加通用请求头
        request = request
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        // 添加认证 Token
        if let Some(token) = self.token_manager.get_token().await {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        // 添加请求体
        if let Some(body) = body {
            let json = serde_json::to_string(body)
                .map_err(|e| ApiError::SerializationError(e.to_string()))?;
            request = request.body(json)?;
        }

        // 发送请求
        let response = request
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        // 处理响应
        self.handle_response(response).await
    }

    /// 处理 HTTP 响应
    async fn handle_response<T>(
        &self,
        response: gloo_net::http::Response,
    ) -> Result<T, ApiError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        let status_text = response.status_text();

        // 处理成功响应
        if response.ok() {
            let data: T = response
                .json()
                .await
                .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
            return Ok(data);
        }

        // 处理错误响应
        let error_body: Result<ErrorResponse, _> = response.json().await;

        match status {
            401 => {
                // Token 过期，清除并返回错误
                self.token_manager.clear_token().await;
                Err(ApiError::Unauthorized)
            }
            403 => Err(ApiError::Forbidden),
            404 => Err(ApiError::NotFound),
            429 => Err(ApiError::RateLimitExceeded),
            500..=599 => Err(ApiError::ServerError(
                error_body
                    .as_ref()
                    .map(|e| e.error.clone())
                    .unwrap_or_else(|_| status_text.to_string()),
            )),
            _ => Err(ApiError::BadRequest(
                error_body
                    .map(|e| e.error)
                    .unwrap_or_else(|_| "Unknown error".to_string()),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Method {
    Get,
    Post,
    Put,
    Delete,
}
```

### 2. Token 管理器

```rust
// src/domain/services/token_manager.rs

use gloo_storage::{LocalStorage, Storage};
use std::sync::RwLock;

const TOKEN_KEY: &str = "auth_token";
const REFRESH_TOKEN_KEY: &str = "refresh_token";

/// Token 管理器
pub struct TokenManager {
    /// 内存中的 Token 缓存
    token_cache: RwLock<Option<String>>,
}

impl TokenManager {
    pub fn new() -> Self {
        // 启动时从 LocalStorage 加载 Token
        let token = LocalStorage::get::<String>(TOKEN_KEY).ok();
        Self {
            token_cache: RwLock::new(token),
        }
    }

    /// 获取当前 Token
    pub async fn get_token(&self) -> Option<String> {
        self.token_cache.read().unwrap().clone()
    }

    /// 设置 Token
    pub async fn set_token(&self, token: String) {
        // 更新内存缓存
        *self.token_cache.write().unwrap() = Some(token.clone());

        // 持久化到 LocalStorage
        let _ = LocalStorage::set(TOKEN_KEY, token);
    }

    /// 清除 Token
    pub async fn clear_token(&self) {
        *self.token_cache.write().unwrap() = None;
        LocalStorage::delete(TOKEN_KEY);
    }

    /// 检查 Token 是否有效（解析 JWT 并验证过期时间）
    pub async fn is_valid(&self) -> bool {
        if let Some(token) = self.get_token().await {
            // Parse JWT and check expiration
            match jsonwebtoken::decode::<jwt::Claims>(
                &token,
                &jwt::DecodingKey::from_secret(b"secret"),  // Production: load from config
                &jwt::Validation::default(),
            ) {
                Ok(token_data) => {
                    // Check if token is expired
                    let now = chrono::Utc::now().timestamp() as usize;
                    token_data.claims.exp > now
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// 刷新 Token（调用后端 /api/auth/refresh）
    pub async fn refresh_token(&self, api_client: &ApiClient) -> Result<String, ApiError> {
        let refresh_token = LocalStorage::get::<String>(REFRESH_TOKEN_KEY)
            .map_err(|_| ApiError::Unauthorized)?;

        let response: RefreshTokenResponse = api_client
            .post("/api/auth/refresh", serde_json::json!({
                "refresh_token": refresh_token
            }))
            .await?;

        self.set_token(response.access_token.clone()).await;
        Ok(response.access_token)
    }
}
```

### 3. 缓存管理器

```rust
// src/domain/services/cache_manager.rs

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// 缓存项
struct CacheEntry {
    data: String,
    expires_at: Instant,
}

/// 缓存管理器
pub struct CacheManager {
    cache: RwLock<HashMap<String, CacheEntry>>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// 获取缓存
    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let cache = self.cache.read().unwrap();
        if let Some(entry) = cache.get(key) {
            if entry.expires_at > Instant::now() {
                return serde_json::from_str(&entry.data).ok();
            }
        }
        None
    }

    /// 设置缓存
    pub async fn set<T>(&self, key: &str, value: &T, ttl_secs: u64)
    where
        T: Serialize,
    {
        if let Ok(data) = serde_json::to_string(value) {
            let entry = CacheEntry {
                data,
                expires_at: Instant::now() + Duration::from_secs(ttl_secs),
            };
            self.cache.write().unwrap().insert(key.to_string(), entry);
        }
    }

    /// 清除缓存
    pub async fn clear(&self, key: &str) {
        self.cache.write().unwrap().remove(key);
    }

    /// 清除所有缓存
    pub async fn clear_all(&self) {
        self.cache.write().unwrap().clear();
    }

    /// 清理过期缓存
    pub async fn cleanup(&self) {
        let now = Instant::now();
        self.cache
            .write()
            .unwrap()
            .retain(|_, entry| entry.expires_at > now);
    }
}
```

---

## 请求/响应类型

### 通用类型定义

```rust
// src/domain/types/api_types.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 通用 API 响应
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

/// 错误响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// 分页响应
#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u64,
    pub page_size: u64,
    pub total: usize,
}
```

### 认证相关类型

```rust
// src/domain/types/auth_types.rs

/// 注册请求
#[derive(Debug, Clone, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// 登录请求
#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// 登录响应
#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub user: UserDto,
    pub expires_in: u64,
}

/// 用户 DTO
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub status: String,
    pub mfa_enabled: bool,
    pub created_at: String,
}
```

### 钱包相关类型

```rust
// src/domain/types/wallet_types.rs

/// 创建钱包请求（存储钱包元数据到后端）
#[derive(Debug, Clone, Serialize)]
pub struct CreateWalletRequest {
    /// 钱包名称
    pub name: String,
    /// 多链地址映射 {"BTC": "bc1q...", "EVM": "0x...", "Solana": "...", "TON": "..."}
    pub addresses: HashMap<String, String>,
    /// 选择的链类型列表
    pub selected_chains: Vec<String>, // ["BTC", "EVM", "Solana", "TON"]
    /// 创建时间
    pub created_at: u64,
}

/// 钱包 DTO（从后端返回）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletDto {
    /// 钱包 ID
    pub wallet_id: String,
    /// 用户 ID
    pub user_id: String,
    /// 钱包名称
    pub name: String,
    /// 多链地址映射
    pub addresses: HashMap<String, String>,
    /// 支持的链类型
    pub chains: Vec<String>,
    /// 是否为默认钱包
    pub is_default: bool,
    /// 是否锁定
    pub is_locked: bool,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
}

/// 恢复钱包请求
#[derive(Debug, Clone, Serialize)]
pub struct RecoverWalletRequest {
    /// 钱包名称
    pub name: String,
    /// 多链地址映射（从助记词派生）
    pub addresses: HashMap<String, String>,
    /// 恢复的链类型
    pub selected_chains: Vec<String>,
    /// 恢复时间
    pub recovered_at: u64,
}

/// 钱包列表响应
#[derive(Debug, Clone, Deserialize)]
pub struct WalletListResponse {
    pub wallets: Vec<WalletDto>,
    pub page: u64,
    pub page_size: u64,
    pub total: usize,
}

/// 更新钱包请求
#[derive(Debug, Clone, Serialize)]
pub struct UpdateWalletRequest {
    /// 新的钱包名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 是否设置为默认（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}
```

### 交易相关类型

```rust
// src/domain/types/transaction_types.rs

/// 创建交易请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateTransactionRequest {
    pub to_address: String,
    pub amount: String,
    pub chain: String,
    pub chain_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_contract: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// 交易 DTO
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionDto {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub to_address: String,
    pub amount: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmations: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

---

## 认证管理

### AuthService 封装

```rust
// src/domain/services/auth_service.rs

use super::api_client::ApiClient;
use super::token_manager::TokenManager;
use crate::domain::types::*;
use std::sync::Arc;

pub struct AuthService {
    api_client: Arc<ApiClient>,
    token_manager: Arc<TokenManager>,
}

impl AuthService {
    pub fn new(api_client: Arc<ApiClient>) -> Self {
        Self {
            token_manager: api_client.token_manager.clone(),
            api_client,
        }
    }

    /// 用户注册
    pub async fn register(&self, email: String, password: String) -> Result<RegisterResponse, ApiError> {
        let request = RegisterRequest {
            email: email.clone(),
            password,
        };
        
        let response: RegisterResponse = self.api_client.post("/api/auth/register", request).await?;

        // 自动登录，保存 Token
        self.token_manager.set_token(response.jwt_token.clone()).await;

        Ok(response)
    }

    /// 用户登录
    pub async fn login(&self, email: String, password: String, remember_me: bool) -> Result<LoginResponse, ApiError> {
        let request = LoginRequest {
            email,
            password,
            remember_me,
        };
        
        let response: LoginResponse = self.api_client.post("/api/auth/login", request).await?;

        // 保存 Token
        self.token_manager.set_token(response.jwt_token.clone()).await;

        Ok(response)
    }

    /// 登出
    pub async fn logout(&self) -> Result<(), ApiError> {
        // 调用后端登出接口（可选）
        let _ = self.api_client.post::<(), ()>("/api/auth/logout", ()).await;

        // 清除本地 Token
        self.token_manager.clear_token().await;

        Ok(())
    }

    /// 检查是否已登录
    pub async fn is_authenticated(&self) -> bool {
        self.token_manager.is_valid().await
    }

    /// 获取当前用户信息
    pub async fn get_current_user(&self) -> Result<UserInfo, ApiError> {
        self.api_client.get("/api/auth/me").await
    }

    /// 修改密码
    pub async fn change_password(
        &self,
        old_password: String,
        new_password: String,
    ) -> Result<(), ApiError> {
        let request = ChangePasswordRequest {
            old_password,
            new_password: new_password.clone(),
            confirm_new_password: new_password,
        };
        
        self.api_client.post("/api/auth/change-password", request).await
    }

    /// 刷新 Token
    pub async fn refresh_token(&self) -> Result<RefreshTokenResponse, ApiError> {
        let response: RefreshTokenResponse = self.api_client.post("/api/auth/refresh", ()).await?;
        
        // 更新 Token
        self.token_manager.set_token(response.jwt_token.clone()).await;
        
        Ok(response)
    }
}

// ===== 请求/响应类型 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub email: String,
    pub jwt_token: String,
    pub token_expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub remember_me: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub email: String,
    pub jwt_token: String,
    pub token_expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub email: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
    pub confirm_new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub jwt_token: String,
    pub token_expires_at: u64,
}
```

---

## 错误处理

### ApiError 定义

```rust
// src/domain/errors/api_error.rs

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ApiError {
    /// 网络错误
    #[error("Network error: {0}")]
    NetworkError(String),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// 反序列化错误
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// 未授权（401）
    #[error("Unauthorized")]
    Unauthorized,

    /// 禁止访问（403）
    #[error("Forbidden")]
    Forbidden,

    /// 资源不存在（404）
    #[error("Not found")]
    NotFound,

    /// 请求参数错误（400）
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// 请求频率超限（429）
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// 服务器错误（5xx）
    #[error("Server error: {0}")]
    ServerError(String),

    /// 超时错误
    #[error("Request timeout")]
    Timeout,

    /// 未知错误
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ApiError {
    /// 判断是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ApiError::NetworkError(_)
                | ApiError::Timeout
                | ApiError::ServerError(_)
                | ApiError::RateLimitExceeded
        )
    }

    /// 转换为用户友好的错误消息
    pub fn to_user_message(&self) -> String {
        match self {
            ApiError::NetworkError(_) => "网络连接失败，请检查网络设置".to_string(),
            ApiError::Unauthorized => "请先登录".to_string(),
            ApiError::Forbidden => "您没有权限执行此操作".to_string(),
            ApiError::NotFound => "请求的资源不存在".to_string(),
            ApiError::BadRequest(msg) => format!("请求参数错误: {}", msg),
            ApiError::RateLimitExceeded => "请求过于频繁，请稍后再试".to_string(),
            ApiError::ServerError(_) => "服务器错误，请稍后再试".to_string(),
            ApiError::Timeout => "请求超时，请稍后再试".to_string(),
            _ => "未知错误，请联系客服".to_string(),
        }
    }
}
```

---

## 缓存策略

### 缓存配置

| 数据类型 | TTL | 策略 |
|---------|-----|------|
| 用户信息 | 10分钟 | 内存缓存 |
| 钱包列表 | 5分钟 | 内存缓存 |
| 余额数据 | 30秒 | 内存缓存 |
| 交易历史 | 5分钟 | 内存缓存 |
| Gas 价格 | 15秒 | 内存缓存 |

### 缓存失效策略

1. **时间失效**: 超过 TTL 自动失效
2. **主动失效**: 数据更新后清除相关缓存
3. **容量控制**: 超过 1000 条自动淘汰最旧的

---

## 完整实现示例

### WalletService 完整示例

```rust
// src/domain/services/wallet_service.rs

use super::api_client::ApiClient;
use crate::domain::types::*;
use std::sync::Arc;
use uuid::Uuid;

pub struct WalletService {
    api_client: Arc<ApiClient>,
}

impl WalletService {
    pub fn new(api_client: Arc<ApiClient>) -> Self {
        Self { api_client }
    }

    /// 创建钱包元数据（存储到后端）
    /// 注意：私钥/助记词仅存储在前端 IndexedDB，不会上传到后端
    pub async fn create_wallet_metadata(
        &self,
        request: CreateWalletRequest,
    ) -> Result<WalletDto, ApiError> {
        self.api_client.post("/api/wallets", request).await
    }

    /// 恢复钱包元数据
    pub async fn recover_wallet_metadata(
        &self,
        request: RecoverWalletRequest,
    ) -> Result<WalletDto, ApiError> {
        self.api_client.post("/api/wallets/recover", request).await
    }

    /// 获取当前用户的所有钱包列表
    pub async fn get_wallets(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<WalletListResponse, ApiError> {
        self.api_client
            .get(&format!("/api/wallets?page={}&page_size={}", page, page_size))
            .await
    }

    /// 获取钱包详情
    pub async fn get_wallet(&self, wallet_id: &str) -> Result<WalletDto, ApiError> {
        self.api_client
            .get(&format!("/api/wallets/{}", wallet_id))
            .await
    }

    /// 更新钱包名称
    pub async fn update_wallet_name(
        &self,
        wallet_id: &str,
        new_name: String,
    ) -> Result<WalletDto, ApiError> {
        let request = UpdateWalletRequest {
            name: Some(new_name),
            is_default: None,
        };
        
        self.api_client
            .put(&format!("/api/wallets/{}", wallet_id), request)
            .await
    }

    /// 设置默认钱包
    pub async fn set_default_wallet(&self, wallet_id: &str) -> Result<WalletDto, ApiError> {
        let request = UpdateWalletRequest {
            name: None,
            is_default: Some(true),
        };
        
        self.api_client
            .put(&format!("/api/wallets/{}", wallet_id), request)
            .await
    }

    /// 删除钱包（仅删除后端元数据）
    pub async fn delete_wallet(&self, wallet_id: &str) -> Result<(), ApiError> {
        self.api_client
            .delete(&format!("/api/wallets/{}", wallet_id))
            .await
    }

    /// 获取钱包余额
    pub async fn get_balance(&self, wallet_id: Uuid) -> Result<BalanceDto, ApiError> {
        self.api_client
            .get(&format!("/api/wallets/{}/balance", wallet_id))
            .await
    }
}
```

### 使用示例

```rust
// src/presentation/pages/wallet_list.rs

use dioxus::prelude::*;

#[component]
pub fn WalletListPage() -> Element {
    let api_client = use_context::<Arc<ApiClient>>();
    let wallet_service = use_memo(move || WalletService::new(api_client()));
    
    let wallets = use_signal(|| Vec::<WalletDto>::new());
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);

    // 加载钱包列表
    let load_wallets = move |_| {
        spawn(async move {
            loading.set(true);
            error.set(None);

            match wallet_service().get_wallets(0, 20).await {
                Ok(response) => {
                    wallets.set(response.wallets);
                }
                Err(e) => {
                    error.set(Some(e.to_user_message()));
                }
            }

            loading.set(false);
        });
    };

    // 页面加载时获取钱包列表
    use_effect(move || {
        load_wallets(());
    });

    rsx! {
        div { class: "wallet-list-page",
            h1 { "我的钱包" }

            if loading() {
                p { "加载中..." }
            }

            if let Some(err) = error() {
                div { class: "error", "{err}" }
            }

            div { class: "wallet-list",
                for wallet in wallets() {
                    div { key: "{wallet.id}",
                        class: "wallet-card",
                        h3 { "{wallet.name}" }
                        p { "地址: {wallet.address}" }
                        p { "余额: {wallet.balance}" }
                    }
                }
            }
        }
    }
}
```

---

## 总结

### 关键特性

✅ **类型安全**: 100% Rust 类型，编译时检查  
✅ **自动重试**: 网络错误自动重试 3 次  
✅ **Token 管理**: 自动注入、自动刷新  
✅ **统一错误**: 所有错误通过 `ApiError` 处理  
✅ **缓存优化**: 减少不必要的网络请求  
✅ **可测试**: 易于 Mock 和单元测试  

### 下一步

- [ ] 实现 WebSocket 实时通知
- [ ] 添加请求取消功能
- [ ] 实现离线队列（失败重发）
- [ ] 添加 Metrics 监控
- [ ] 完善单元测试覆盖

---

**参考文档**:
- [IronCore Backend API Reference](./01-ironcore-backend-api-reference.md)
- [错误处理设计](./03-error-handling.md)
- [状态管理方案](../02-technical-design/03-state-management.md)
