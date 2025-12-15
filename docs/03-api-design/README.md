# API 设计 (API Design)

> 🔌 前后端 API 集成、错误处理、服务封装

---

## 📂 本分类文档

| 文档 | 描述 | 行数 | 最后更新 | 状态 |
|------|------|------|----------|------|
| [01-ironcore-backend-api-reference.md](./01-ironcore-backend-api-reference.md) | IronCore 后端 API 完整参考 | 1,826 | 2025-11-25 | ✅ 完成 |
| [02-frontend-api-layer.md](./02-frontend-api-layer.md) | 前端 API 封装层设计 | 835 | 2025-11-25 | ✅ 完成 |
| [03-error-handling.md](./03-error-handling.md) | 错误处理策略、用户提示 | 665 | 2025-11-25 | ✅ 完成 |
| [04-token-detection-service.md](./04-token-detection-service.md) | 代币检测服务实现 | 502 | 2025-11-25 | ✅ 完成 |
| [05-backend-services-integration.md](./05-backend-services-integration.md) | 后端服务集成指南 | 723 | 2025-11-25 | ✅ 完成 |
| [06-frontend-api-quick-reference.md](./06-frontend-api-quick-reference.md) | 前端 API 快速参考 | 564 | 2025-11-25 | ✅ 完成 |
| [PAYMENT_ANALYSIS.md](./PAYMENT_ANALYSIS.md) | MoonPay 支付集成分析 ⭐ | 420 | 2025-12-04 | ✅ 完成 |

**总计**: 7 个文档，5,535+ 行

---

## 🎯 快速导航

### API 使用
- 📘 **[后端 API 参考](./01-ironcore-backend-api-reference.md)** - 46+ REST API 完整文档
- 🔧 **[前端 API 封装](./02-frontend-api-layer.md)** - 如何在前端调用 API
- 📋 **[快速参考卡](./06-frontend-api-quick-reference.md)** - 常用 API 速查

### 错误处理
- ⚠️ **[错误处理策略](./03-error-handling.md)** - 统一错误处理、用户友好提示
- 🔐 **[401 错误诊断](../04-security/AUTH_401_DIAGNOSTIC_GUIDE.md)** - 认证问题排查

### 特殊服务
- 🪙 **[代币检测](./04-token-detection-service.md)** - 自动检测钱包代币余额
- 💳 **[支付集成](./PAYMENT_ANALYSIS.md)** - MoonPay 购买流程分析

---

## 🏗️ API 架构概览

### 前后端通信架构

```
┌─────────────────────────────────────────────────────┐
│              IronForge Frontend (WASM)              │
├─────────────────────────────────────────────────────┤
│                                                       │
│  ┌───────────────────────────────────────────────┐  │
│  │      UI Components (Pages/Components)         │  │
│  └────────────────┬──────────────────────────────┘  │
│                   │ call services                    │
│  ┌────────────────▼──────────────────────────────┐  │
│  │      Services Layer (Business Logic)          │  │
│  │   - WalletService                              │  │
│  │   - TransactionService                         │  │
│  │   - TokenService                               │  │
│  └────────────────┬──────────────────────────────┘  │
│                   │ use API client                   │
│  ┌────────────────▼──────────────────────────────┐  │
│  │      API Client (HTTP Wrapper)                │  │
│  │   - api_client.rs (统一封装)                   │  │
│  │   - Error handling                             │  │
│  │   - JWT token management                       │  │
│  └────────────────┬──────────────────────────────┘  │
│                   │                                  │
└───────────────────┼──────────────────────────────────┘
                    │ HTTP/JSON (Bearer Token)
                    ▼
┌─────────────────────────────────────────────────────┐
│          IronCore Backend (Axum + Rust)             │
├─────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────┐  │
│  │      API Handlers (46+ REST endpoints)        │  │
│  │   - Auth: /api/auth/register, /login          │  │
│  │   - Wallet: /api/wallets/* (CRUD)             │  │
│  │   - Transaction: /api/transactions/*          │  │
│  │   - Token: /api/tokens/*                      │  │
│  │   - Swap: /api/swap/*                         │  │
│  │   - Payment: /api/payments/*                  │  │
│  └────────────────┬──────────────────────────────┘  │
│                   │                                  │
│  ┌────────────────▼──────────────────────────────┐  │
│  │      Middleware (Auth, Rate Limit, CSRF)      │  │
│  └────────────────┬──────────────────────────────┘  │
│                   │                                  │
│  ┌────────────────▼──────────────────────────────┐  │
│  │      Database (CockroachDB/PostgreSQL)        │  │
│  │   - users, wallets, transactions, tokens      │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### API 设计原则

1. **RESTful** - 遵循 REST 规范，资源导向
2. **统一格式** - 所有响应使用统一 JSON 结构
3. **错误友好** - 清晰的错误码和提示信息
4. **版本控制** - 支持 API 版本演进
5. **安全第一** - JWT 认证、HTTPS、Rate Limiting

---

## 📚 API 文档详解

### 1️⃣ [IronCore 后端 API 参考](./01-ironcore-backend-api-reference.md)
**阅读时长**: 30 分钟  
**适合**: 全栈工程师、API 集成人员

**核心内容**:
- 🔐 **认证 API** (3 个): 注册、登录、刷新 Token
- 👛 **钱包 API** (8 个): CRUD、批量操作、导入导出
- 💸 **交易 API** (6 个): 查询历史、详情、统计
- 🪙 **代币 API** (5 个): 余额、价格、搜索、自定义代币
- 🔄 **Swap API** (4 个): 报价、执行、历史、配置
- 💳 **支付 API** (3 个): MoonPay 集成、Webhook
- 👤 **用户 API** (4 个): 个人资料、设置、KYC
- 🔔 **通知 API** (3 个): 推送、历史、偏好设置
- 📊 **统计 API** (5 个): 仪表盘数据、图表
- 🔧 **系统 API** (5 个): 健康检查、配置、版本

**API 示例**:
```typescript
// 获取钱包列表
GET /api/wallets
Authorization: Bearer <jwt_token>

Response:
{
  "code": 0,
  "message": "Success",
  "data": {
    "wallets": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "My Main Wallet",
        "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
        "chain": "ethereum",
        "created_at": "2025-01-01T00:00:00Z"
      }
    ],
    "total": 1
  }
}
```

---

### 2️⃣ [前端 API 封装层](./02-frontend-api-layer.md)
**阅读时长**: 15 分钟  
**适合**: 前端工程师

**核心内容**:
- 🔧 **ApiClient 设计** - 统一 HTTP 客户端封装
- 🔑 **Token 管理** - JWT 自动添加、刷新机制
- ⚠️ **错误处理** - 统一错误拦截和转换
- 🔄 **重试机制** - 网络失败自动重试
- 📦 **类型安全** - Rust 类型定义所有请求/响应

**代码示例**:
```rust
// src/infrastructure/api/client.rs
pub struct ApiClient {
    base_url: String,
    token: Signal<Option<String>>,
}

impl ApiClient {
    pub async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let response = Request::get(&url)
            .header("Authorization", &format!("Bearer {}", self.token()))
            .send()
            .await?;
        
        self.handle_response(response).await
    }
    
    async fn handle_response<T>(&self, resp: Response) -> Result<T> {
        match resp.status() {
            200..=299 => resp.json().await,
            401 => Err(ApiError::Unauthorized),
            404 => Err(ApiError::NotFound),
            _ => Err(ApiError::ServerError),
        }
    }
}
```

---

### 3️⃣ [错误处理策略](./03-error-handling.md)
**阅读时长**: 12 分钟  
**适合**: 前端工程师、QA

**核心内容**:
- ⚠️ **错误分类** - Network, Auth, Validation, Server, Business
- 📝 **用户提示** - 将技术错误转换为用户友好消息
- 🎨 **UI 展示** - Toast, Modal, Inline 错误提示
- 📊 **错误监控** - Sentry 集成、错误上报
- 🔄 **重试策略** - 哪些错误可以重试

**错误码映射**:
```rust
pub fn get_user_message(error: &ApiError, lang: Language) -> String {
    match error {
        ApiError::NetworkError => t("error.network", lang),
        ApiError::Unauthorized => t("error.unauthorized", lang),
        ApiError::ServerError => t("error.server", lang),
        ApiError::ValidationError(field) => {
            format!("{}: {}", t("error.validation", lang), field)
        }
    }
}
```

---

### 4️⃣ [代币检测服务](./04-token-detection-service.md)
**阅读时长**: 10 分钟  
**适合**: 前端工程师

**核心内容**:
- 🔍 **自动检测** - 扫描钱包地址，自动发现代币
- 🪙 **代币余额** - 批量查询 ERC-20/BEP-20 余额
- 💰 **价格查询** - 实时获取代币价格
- ⚡ **性能优化** - 批量请求、缓存机制
- 🔄 **定时刷新** - 后台自动更新余额

**实现示例**:
```rust
pub struct TokenDetectionService {
    api_client: ApiClient,
    cache: HashMap<String, Vec<Token>>,
}

impl TokenDetectionService {
    pub async fn detect_tokens(&self, address: &str) -> Result<Vec<Token>> {
        // 1. Check cache
        if let Some(cached) = self.cache.get(address) {
            return Ok(cached.clone());
        }
        
        // 2. Call backend API
        let tokens = self.api_client
            .get(&format!("/api/tokens/detect/{}", address))
            .await?;
        
        // 3. Update cache
        self.cache.insert(address.to_string(), tokens.clone());
        
        Ok(tokens)
    }
}
```

---

### 5️⃣ [后端服务集成](./05-backend-services-integration.md)
**阅读时长**: 15 分钟  
**适合**: 全栈工程师

**核心内容**:
- 🔗 **Service 层设计** - 前端如何封装后端服务
- 📦 **依赖注入** - 使用 Context 共享服务实例
- 🔄 **状态同步** - 前端状态与后端数据同步
- ⚡ **缓存策略** - 何时缓存、何时刷新
- 🧪 **Mock 服务** - 本地开发 Mock 数据

**Service 示例**:
```rust
pub struct WalletService {
    api: ApiClient,
    cache: Signal<Option<Vec<Wallet>>>,
}

impl WalletService {
    pub async fn get_wallets(&self) -> Result<Vec<Wallet>> {
        // 1. Return cache if available
        if let Some(cached) = self.cache() {
            return Ok(cached);
        }
        
        // 2. Fetch from API
        let wallets = self.api.get("/api/wallets").await?;
        
        // 3. Update cache
        self.cache.set(Some(wallets.clone()));
        
        Ok(wallets)
    }
    
    pub async fn create_wallet(&self, name: &str) -> Result<Wallet> {
        let wallet = self.api.post("/api/wallets", json!({ "name": name })).await?;
        
        // Invalidate cache
        self.cache.set(None);
        
        Ok(wallet)
    }
}
```

---

### 6️⃣ [前端 API 快速参考](./06-frontend-api-quick-reference.md)
**阅读时长**: 5 分钟  
**适合**: 快速查找 API 用法

**内容**: 最常用 API 的代码片段速查表

**示例**:
```rust
// 🔐 登录
let user = api_client.login("user@example.com", "password").await?;

// 👛 获取钱包列表
let wallets = wallet_service.get_wallets().await?;

// 💸 查询交易历史
let txs = tx_service.get_transactions(wallet_id, 1, 20).await?;

// 🪙 获取代币余额
let balances = token_service.get_balances(address).await?;

// 🔄 执行 Swap
let result = swap_service.execute_swap(from, to, amount).await?;
```

---

### 7️⃣ [支付集成分析](./PAYMENT_ANALYSIS.md) ⭐
**日期**: 2025-12-04  
**适合**: 产品经理、前端工程师

**核心内容**:
- 💳 **MoonPay 集成** - 法币购买加密货币流程
- 🔐 **签名机制** - API 签名验证
- 🔔 **Webhook 处理** - 支付状态回调
- 🎨 **UI 流程** - 购买弹窗、支付确认
- 🐛 **常见问题** - 支付失败排查

**MoonPay 流程**:
```
1. 用户点击 "Buy Crypto"
   ↓
2. 前端调用 /api/payments/moonpay/url
   ↓
3. 后端生成签名 URL
   ↓
4. 用户跳转到 MoonPay (新窗口)
   ↓
5. 用户完成支付
   ↓
6. MoonPay 回调 Webhook
   ↓
7. 后端更新订单状态
   ↓
8. 前端轮询查询状态
```

---

## 🔍 API 设计模式

### 统一响应格式

所有 API 响应使用统一格式：

```typescript
{
  "code": 0,              // 0=成功, >0=错误码
  "message": "Success",   // 用户友好提示
  "data": { ... }         // 业务数据
}
```

### 错误码规范

| 错误码 | 含义 | HTTP 状态 |
|--------|------|-----------|
| 0 | 成功 | 200 |
| 1001 | 参数错误 | 400 |
| 1002 | 认证失败 | 401 |
| 1003 | 权限不足 | 403 |
| 1004 | 资源不存在 | 404 |
| 2001 | 钱包已存在 | 409 |
| 2002 | 余额不足 | 400 |
| 5000 | 服务器错误 | 500 |

### 分页规范

```typescript
// Request
GET /api/wallets?page=1&page_size=20

// Response
{
  "code": 0,
  "data": {
    "items": [...],
    "total": 100,
    "page": 1,
    "page_size": 20,
    "total_pages": 5
  }
}
```

---

## 📊 API 性能指标

| 指标 | 目标 | 当前状态 |
|------|------|----------|
| **响应时间 (p95)** | < 100ms | 80ms ✅ |
| **错误率** | < 0.1% | 0.05% ✅ |
| **可用性** | 99.9% | 99.95% ✅ |
| **并发支持** | 10,000 req/s | 8,500 req/s 🔄 |

---

## 🔗 相关文档

- **系统架构**: [01-architecture/01-system-architecture.md](../01-architecture/01-system-architecture.md)
- **数据分离**: [01-architecture/02-data-separation-model.md](../01-architecture/02-data-separation-model.md)
- **安全架构**: [04-security/03-security-architecture.md](../04-security/03-security-architecture.md)
- **401 错误诊断**: [04-security/AUTH_401_DIAGNOSTIC_GUIDE.md](../04-security/AUTH_401_DIAGNOSTIC_GUIDE.md)
- **测试策略**: [07-testing/01-testing-strategy.md](../07-testing/01-testing-strategy.md)

---

**最后更新**: 2025-12-06  
**维护者**: API Team  
**审查者**: Backend Lead, Frontend Lead
