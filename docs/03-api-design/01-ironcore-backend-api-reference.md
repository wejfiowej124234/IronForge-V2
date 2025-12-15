# IronCore 后端 API 参考文档

> **版本**: V2.0  
> **后端地址**: http://localhost:8088 (IronCore 统一后端)  
> **更新日期**: 2025-12-01  
> **用途**: IronForge 前端开发 API 对接参考

---

## 📋 目录

1. [概览](#概览)
2. [认证机制](#认证机制)
3. [API 端点列表](#api-端点列表)
   - [认证 API](#1-认证-api)
   - [钱包管理 API](#2-钱包管理-api)
   - [交易 API](#3-交易-api)
   - [跨链桥接 API](#4-跨链桥接-api)
   - [备份与恢复 API](#5-备份与恢复-api)
   - [多签 API](#6-多签-api)
   - [余额查询 API](#7-余额查询-api)
   - [系统监控 API](#8-系统监控-api)
4. [支持的区块链](#支持的区块链)
5. [错误码说明](#错误码说明)
6. [数据模型](#数据模型)

---

## 概览

### Base URL

```
IronCore (统一后端): http://localhost:8088
```

### 请求格式

- **Content-Type**: `application/json`
- **字符编码**: UTF-8
- **时区**: UTC

### 响应格式

#### 成功响应 (2xx)

```json
{
  "data": { ... },
  "message": "Success",
  "trace_id": "7b9c83a7..."
}
```

#### 错误响应 (4xx/5xx)

```json
{
  "error": "Error message",
  "code": "ERROR_CODE"
}
```

---

## 认证机制

### 认证方式

IronCore 支持两种认证方式：

1. **JWT Bearer Token** (推荐，用户身份认证)
2. **API Key** (备用，服务间通信)

### 请求头

```http
# JWT 认证（推荐）
Authorization: Bearer <jwt_token>

# API Key 认证（备用）
X-API-Key: <api_key>
```

### 获取 JWT Token

通过登录接口获取：

```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

响应：

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": null,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "role": "operator"
  },
  "expires_in": 86400
}
```

---

## API 端点列表

### 1. 认证 API

#### 1.1 用户注册

```http
POST /api/auth/register
```

**请求体**:

```json
{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "confirm_password": "SecurePass123!",
  "username": "myusername"  // 可选
}
```

**响应** (201 Created):

```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
    "email": "user@example.com",
    "role": "operator",
    "status": "active",
    "mfa_enabled": false,
    "created_at": "2025-11-25T10:00:00Z",
    "updated_at": "2025-11-25T10:00:00Z"
  },
  "message": "User registered successfully"
}
```

**错误码**:
- `400`: 密码不匹配或格式错误
- `409`: 邮箱已存在

---

#### 1.2 用户登录

```http
POST /api/auth/login
```

**请求体**:

```json
{
  "email": "user@example.com",
  "password": "SecurePass123!"
}
```

**响应** (200 OK):

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": null,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "role": "operator",
    "status": "active"
  },
  "expires_in": 86400
}
```

**错误码**:
- `401`: 邮箱或密码错误
- `403`: 账户被禁用

---

#### 1.3 修改密码

```http
POST /api/auth/change-password
Authorization: Bearer <token>
```

**请求体**:

```json
{
  "old_password": "OldPass123!",
  "new_password": "NewPass456!",
  "confirm_new_password": "NewPass456!"
}
```

**响应** (200 OK):

```json
{
  "message": "Password changed successfully"
}
```

**错误码**:
- `400`: 密码格式错误或不匹配
- `401`: 旧密码错误
- `403`: 未授权

---

#### 1.4 刷新 Token

```http
POST /api/auth/refresh
Authorization: Bearer <refresh_token>
```

**响应** (200 OK):

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 86400
}
```

---

#### 1.5 登出

```http
POST /api/auth/logout
Authorization: Bearer <token>
```

**响应** (200 OK):

```json
{
  "message": "Logged out successfully"
}
```

---

### 2. 钱包管理 API

#### 2.1 创建钱包

```http
POST /api/wallets
Authorization: Bearer <token>
```

**请求体**:

```json
{
  "name": "My ETH Wallet",
  "chain": "ethereum",
  "address": "0x742d35Cc6634C0532925a3b844Bc9e8Ef5bEd1e1",
  "pubkey": "0x04abc123...",  // 可选
  "derivation_path": "m/44'/60'/0'/0/0"  // 可选
}
```

**响应** (201 Created):

```json
{
  "id": "770e8400-e29b-41d4-a716-446655440002",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
  "name": "My ETH Wallet",
  "chain": "ethereum",
  "chain_id": 1,
  "address": "0x742d35Cc6634C0532925a3b844Bc9e8Ef5bEd1e1",
  "pubkey": "0x04abc123...",
  "derivation_path": "m/44'/60'/0'/0/0",
  "balance": "0",
  "is_default": false,
  "created_at": "2025-11-25T10:00:00Z",
  "updated_at": "2025-11-25T10:00:00Z"
}
```

**支持的链**:
- `ethereum` (chain_id: 1)
- `bsc` (chain_id: 56)
- `polygon` (chain_id: 137)
- `bitcoin` (mainnet)
- `ton` (chain_id: 607)

**错误码**:
- `400`: 参数错误
- `401`: 未授权
- `409`: 钱包地址已存在

---

#### 2.2 获取钱包详情

```http
GET /api/wallets/:id
Authorization: Bearer <token>
```

**路径参数**:
- `id`: 钱包 UUID

**响应** (200 OK):

```json
{
  "id": "770e8400-e29b-41d4-a716-446655440002",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "My ETH Wallet",
  "chain": "ethereum",
  "chain_id": 1,
  "address": "0x742d35Cc6634C0532925a3b844Bc9e8Ef5bEd1e1",
  "balance": "1500000000000000000",
  "balance_updated_at": "2025-11-25T10:05:00Z",
  "created_at": "2025-11-25T10:00:00Z"
}
```

**错误码**:
- `404`: 钱包不存在
- `403`: 无权访问此钱包

---

#### 2.3 获取用户钱包列表

```http
GET /api/wallets?page=0&page_size=20
Authorization: Bearer <token>
```

**查询参数**:
- `page`: 页码（默认 0）
- `page_size`: 每页数量（默认 20，最大 100）

**响应** (200 OK):

```json
{
  "wallets": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440002",
      "name": "My ETH Wallet",
      "chain": "ethereum",
      "address": "0x742d35Cc...",
      "balance": "1500000000000000000",
      "created_at": "2025-11-25T10:00:00Z"
    },
    {
      "id": "880e8400-e29b-41d4-a716-446655440003",
      "name": "My BSC Wallet",
      "chain": "bsc",
      "address": "0x1234567...",
      "balance": "5000000000000000000",
      "created_at": "2025-11-25T09:00:00Z"
    }
  ],
  "page": 0,
  "page_size": 20,
  "total": 2
}
```

---

#### 2.4 更新钱包

```http
PUT /api/wallets/:id
Authorization: Bearer <token>
```

**请求体**:

```json
{
  "name": "Updated Wallet Name"
}
```

**响应** (200 OK):

```json
{
  "id": "770e8400-e29b-41d4-a716-446655440002",
  "name": "Updated Wallet Name",
  "updated_at": "2025-11-25T10:10:00Z"
}
```

---

#### 2.5 删除钱包

```http
DELETE /api/wallets/:id
Authorization: Bearer <token>
```

**响应** (200 OK):

```json
{
  "message": "Wallet deleted successfully"
}
```

**注意**: 删除钱包不会删除私钥（私钥在前端 IndexedDB 中）

---

### 3. 交易 API

#### 3.1 创建交易请求

```http
POST /api/wallets/:wallet_id/transactions
Authorization: Bearer <token>
```

**请求体**:

```json
{
  "to_address": "0x742d35Cc6634C0532925a3b844Bc9e8Ef5bEd1e1",
  "amount": "1000000000000000000",  // 1 ETH in Wei
  "chain": "ethereum",
  "chain_id": 1,
  "token_symbol": "ETH",  // 可选
  "token_contract": null,  // ERC20 代币合约地址（可选）
  "gas_limit": 21000,  // 可选
  "gas_price": "20000000000",  // 20 Gwei，可选
  "data": null  // 合约调用数据（可选）
}
```

**响应** (201 Created):

```json
{
  "id": "990e8400-e29b-41d4-a716-446655440004",
  "wallet_id": "770e8400-e29b-41d4-a716-446655440002",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "chain": "ethereum",
  "chain_id": 1,
  "to_address": "0x742d35Cc6634C0532925a3b844Bc9e8Ef5bEd1e1",
  "amount": "1000000000000000000",
  "token_symbol": "ETH",
  "status": "draft",
  "nonce": null,
  "gas_limit": 21000,
  "gas_price": "20000000000",
  "created_at": "2025-11-25T10:15:00Z"
}
```

**交易状态流转**:
```
draft → pending_approval → approved → signed → broadcasted → confirmed / failed
```

---

#### 3.2 获取交易详情

```http
GET /api/transactions/:id
Authorization: Bearer <token>
```

**响应** (200 OK):

```json
{
  "id": "990e8400-e29b-41d4-a716-446655440004",
  "wallet_id": "770e8400-e29b-41d4-a716-446655440002",
  "to_address": "0x742d35Cc6634C0532925a3b844Bc9e8Ef5bEd1e1",
  "amount": "1000000000000000000",
  "status": "confirmed",
  "tx_hash": "0xabcdef1234567890...",
  "block_number": 18567890,
  "confirmations": 12,
  "fee": "420000000000000",  // 0.00042 ETH
  "created_at": "2025-11-25T10:15:00Z",
  "updated_at": "2025-11-25T10:20:00Z"
}
```

---

#### 3.3 获取钱包交易历史

```http
GET /api/wallets/:wallet_id/transactions?page=0&page_size=20
Authorization: Bearer <token>
```

**查询参数**:
- `page`: 页码（默认 0）
- `page_size`: 每页数量（默认 20，最大 100）

**响应** (200 OK):

```json
{
  "transactions": [
    {
      "id": "990e8400-e29b-41d4-a716-446655440004",
      "to_address": "0x742d35Cc...",
      "amount": "1000000000000000000",
      "status": "confirmed",
      "tx_hash": "0xabcdef...",
      "created_at": "2025-11-25T10:15:00Z"
    }
  ],
  "page": 0,
  "page_size": 20,
  "total": 1
}
```

---

### 4. 跨链桥接 API

#### 4.1 发起跨链转账

```http
POST /api/bridge
Authorization: Bearer <token>
```

**请求体**:

```json
{
  "from_wallet": "My ETH Wallet",
  "from_chain": "ethereum",
  "to_chain": "polygon",
  "amount": "10.5",
  "token": "USDC",
  "to_address": "0x742d35Cc6634C0532925a3b844Bc9e8Ef5bEd1e1"  // 可选
}
```

**响应** (201 Created):

```json
{
  "bridge_id": "brg_20251125_001",
  "bridge_tx_id": "0xbridge_tx_hash...",
  "status": "initiated",
  "from_chain": "ethereum",
  "target_chain": "polygon",
  "amount": "10.5",
  "token": "USDC"
}
```

**支持的桥接路径**:
- Ethereum ↔ Polygon
- Ethereum ↔ BSC
- Polygon ↔ BSC

---

#### 4.2 查询桥接历史

```http
GET /api/bridge/history?page=0&page_size=20
Authorization: Bearer <token>
```

**响应** (200 OK):

```json
{
  "bridges": [
    {
      "bridge_id": "brg_20251125_001",
      "from_chain": "ethereum",
      "to_chain": "polygon",
      "status": "completed",
      "amount": "10.5",
      "token": "USDC",
      "created_at": "2025-11-25T10:30:00Z"
    }
  ],
  "page": 0,
  "page_size": 20,
  "total": 1
}
```

---

#### 4.3 查询桥接状态

```http
GET /api/bridge/:id/status
Authorization: Bearer <token>
```

**响应** (200 OK):

```json
{
  "bridge_id": "brg_20251125_001",
  "status": "completed",
  "source_tx_hash": "0xabc123...",
  "target_tx_hash": "0xdef456...",
  "confirmations": 24,
  "estimated_time": "5 minutes",
  "updated_at": "2025-11-25T10:35:00Z"
}
```

---

### 5. 备份与恢复 API

#### 5.1 备份钱包

```http
POST /api/backup/export/:wallet_id
Authorization: Bearer <token>
```

**请求体**:

```json
{
  "password": "backup_password_123"
}
```

**响应** (200 OK):

```json
{
  "encrypted_backup": "AES256_ENCRYPTED_DATA_BASE64...",
  "backup_version": "2.0",
  "created_at": "2025-11-25T10:40:00Z"
}
```

**注意**: 
- 备份数据包含加密的私钥/助记词
- 仅在用户明确请求时使用
- 前端应提供下载功能

---

#### 5.2 恢复钱包

```http
POST /api/backup/import/:wallet_id
Authorization: Bearer <token>
```

**请求体**:

```json
{
  "encrypted_backup": "AES256_ENCRYPTED_DATA_BASE64...",
  "password": "backup_password_123"
}
```

**响应** (200 OK):

```json
{
  "wallet_id": "770e8400-e29b-41d4-a716-446655440002",
  "message": "Wallet restored successfully"
}
```

---

### 6. 多签 API

#### 6.1 轮换签名密钥

```http
POST /api/wallets/:wallet_id/rotate-key
Authorization: Bearer <token>
```

**请求体**:

```json
{
  "old_key": "0x1234567890abcdef...",
  "new_key": "0xfedcba0987654321..."
}
```

**响应** (200 OK):

```json
{
  "message": "Key rotated successfully",
  "new_pubkey": "0xfedcba0987654321..."
}
```

---

#### 6.2 发送多签交易

```http
POST /api/wallets/:wallet_id/send-multisig
Authorization: Bearer <token>
```

**请求体**:

```json
{
  "to_address": "0x742d35Cc6634C0532925a3b844Bc9e8Ef5bEd1e1",
  "amount": "1000000000000000000",
  "required_signatures": 2,
  "signers": [
    "0xsigner1_address...",
    "0xsigner2_address..."
  ]
}
```

**响应** (201 Created):

```json
{
  "tx_id": "aa0e8400-e29b-41d4-a716-446655440005",
  "status": "pending_approval",
  "required_signatures": 2,
  "current_signatures": 0
}
```

---

### 7. 余额查询 API

#### 7.1 获取钱包余额

```http
GET /api/wallets/:wallet_id/balance
Authorization: Bearer <token>
```

**响应** (200 OK):

```json
{
  "wallet_id": "770e8400-e29b-41d4-a716-446655440002",  // 🔵 示例UUID
  "chain": "ethereum",
  "balance": "1500000000000000000",  // 🔴 真实：从链上RPC查询
  "balance_eth": "1.5",  // 🔴 真实：balance / 1e18 转换
  "balance_usd": "3150.00",  // 🔴 真实：balance_eth × 价格(后端API)
  "updated_at": "2025-11-25T10:45:00Z"  // 🔴 真实：查询时间戳
}
```

---

#### 7.2 获取代币余额

```http
GET /api/wallets/:wallet_id/balance/:token
Authorization: Bearer <token>
```

**路径参数**:
- `token`: 代币符号（如 USDC, USDT, DAI）

**响应** (200 OK):

```json
{
  "wallet_id": "770e8400-e29b-41d4-a716-446655440002",  // 🔵 示例UUID
  "token": "USDC",
  "token_contract": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",  // 🔴 真实合约地址
  "balance": "1000000000",  // 🔴 真实：链上查询(eth_call balanceOf)
  "balance_formatted": "1000.0",  // 🔴 真实：balance / 10^decimals
  "balance_usd": "1000.00",  // 🔴 真实：balance_formatted × 价格
  "decimals": 6  // 🔴 真实：合约decimals()方法查询
}
```

---

#### 7.3 获取多资产余额

```http
GET /api/wallets/:wallet_id/multi-assets
Authorization: Bearer <token>
```

**响应** (200 OK):

```json
{
  "wallet_id": "770e8400-e29b-41d4-a716-446655440002",  // 🔵 示例UUID
  "total_balance_usd": "4150.00",  // 🔴 真实：所有资产USDT价值总和
  "assets": [
    {
      "symbol": "ETH",
      "balance": "1.5",  // 🔴 真实：链上RPC查询
      "balance_usd": "3150.00"  // 🔴 真实：后端价格API计算
    },
    {
      "symbol": "USDC",
      "balance": "1000.0",  // 🔴 真实：ERC20 balanceOf查询
      "balance_usd": "1000.00"  // 🔴 真实：USDC价格≈1.0
    }
  ]
}

// 🔵 数值为示例 | 🔴 实际API返回真实链上数据
```

---

### 8. 系统监控 API

#### 8.1 健康检查

```http
GET /health
```

**响应** (200 OK):

```json
{
  "status": "healthy",
  "version": "2.0.0",
  "timestamp": "2025-11-25T10:50:00Z",
  "services": {
    "database": "connected",
    "redis": "connected",
    "immudb": "connected"
  }
}
```

---

#### 8.2 增强健康检查

```http
GET /api/health/enhanced
```

**响应** (200 OK):

```json
{
  "status": "healthy",
  "timestamp": "2025-11-25T10:50:00Z",
  "components": {
    "database": {
      "status": "healthy",
      "response_time_ms": 5
    },
    "ethereum_rpc": {
      "status": "healthy",
      "block_number": 18567890
    },
    "redis": {
      "status": "healthy",
      "memory_used_mb": 256
    }
  }
}
```

---

#### 8.3 系统信息

```http
GET /api/system/info
Authorization: Bearer <token>
```

**响应** (200 OK):

```json
{
  "version": "2.0.0",
  "rust_version": "1.75.0",
  "uptime_seconds": 86400,
  "active_connections": 42,
  "supported_chains": ["ethereum", "bsc", "polygon", "bitcoin", "ton"]
}
```

---

#### 8.4 网络状态

```http
GET /api/system/network-status/:chain
Authorization: Bearer <token>
```

**响应** (200 OK):

```json
{
  "chain": "ethereum",
  "chain_id": 1,
  "status": "online",
  "block_number": 18567890,
  "gas_price": "20000000000",  // 20 Gwei
  "suggested_gas": {
    "slow": "18000000000",
    "standard": "20000000000",
    "fast": "25000000000"
  }
}
```

---

#### 8.5 Gas 费用建议

```http
GET /api/system/gas-suggest/:chain
Authorization: Bearer <token>
```

**响应** (200 OK):

```json
{
  "chain": "ethereum",
  "timestamp": "2025-11-25T10:55:00Z",
  "gas_prices": {
    "slow": {
      "gas_price": "18000000000",
      "estimated_time": "5 minutes"
    },
    "standard": {
      "gas_price": "20000000000",
      "estimated_time": "1 minute"
    },
    "fast": {
      "gas_price": "25000000000",
      "estimated_time": "15 seconds"
    }
  }
}
```

---

## 支持的区块链

| 区块链 | chain 参数 | chain_id | 测试网 | RPC 提供商 |
|--------|-----------|----------|--------|-----------|
| Ethereum Mainnet | `ethereum` | 1 | Sepolia (11155111) | Infura, Alchemy |
| BSC Mainnet | `bsc` | 56 | BSC Testnet (97) | BSC RPC |
| Polygon Mainnet | `polygon` | 137 | Mumbai (80001) | Polygon RPC |
| Bitcoin Mainnet | `bitcoin` | - | Testnet | Bitcoin Core |
| TON Mainnet | `ton` | 607 | Testnet | TON API |

### 派生路径 (BIP44)

| 区块链 | 派生路径 | 曲线 |
|--------|---------|------|
| Ethereum | `m/44'/60'/0'/0/0` | secp256k1 |
| BSC | `m/44'/60'/0'/0/0` | secp256k1 (兼容 ETH) |
| Polygon | `m/44'/60'/0'/0/0` | secp256k1 (兼容 ETH) |
| Bitcoin | `m/84'/0'/0'/0/0` | secp256k1 (SegWit) |
| TON | `m/44'/607'/0'/0/0` | ed25519 |

---

## 错误码说明

### HTTP 状态码

| 状态码 | 含义 | 常见场景 |
|--------|------|----------|
| 200 | OK | 请求成功 |
| 201 | Created | 资源创建成功 |
| 400 | Bad Request | 参数错误、格式错误 |
| 401 | Unauthorized | 未登录、Token 过期 |
| 403 | Forbidden | 无权限访问 |
| 404 | Not Found | 资源不存在 |
| 409 | Conflict | 资源冲突（如钱包已存在） |
| 429 | Too Many Requests | 请求过于频繁 |
| 500 | Internal Server Error | 服务器内部错误 |
| 503 | Service Unavailable | 服务不可用 |

### 业务错误码

| 错误码 | 含义 | HTTP 状态 |
|--------|------|-----------|
| `AUTH_FAILED` | 认证失败 | 401 |
| `INVALID_CREDENTIALS` | 用户名或密码错误 | 401 |
| `TOKEN_EXPIRED` | Token 已过期 | 401 |
| `WALLET_NOT_FOUND` | 钱包不存在 | 404 |
| `WALLET_EXISTS` | 钱包已存在 | 409 |
| `TRANSACTION_FAILED` | 交易失败 | 400 |
| `INSUFFICIENT_BALANCE` | 余额不足 | 400 |
| `INVALID_ADDRESS` | 地址格式错误 | 400 |
| `INVALID_AMOUNT` | 金额格式错误 | 400 |
| `BRIDGE_FAILED` | 跨链失败 | 400 |
| `UNSUPPORTED_CHAIN` | 不支持的链 | 400 |
| `RATE_LIMIT_EXCEEDED` | 请求频率超限 | 429 |
| `INTERNAL_ERROR` | 内部错误 | 500 |

---

## 数据模型

### User (用户)

```typescript
interface User {
  id: string;  // UUID
  tenant_id: string;  // UUID
  email: string;
  role: 'operator' | 'admin' | 'super_admin';
  status: 'active' | 'suspended' | 'deleted';
  mfa_enabled: boolean;
  created_at: string;  // ISO 8601
  updated_at: string;  // ISO 8601
}
```

### Wallet (钱包)

```typescript
interface Wallet {
  id: string;  // UUID
  user_id: string;  // UUID
  tenant_id: string;  // UUID
  name: string;
  chain: string;  // 'ethereum', 'bsc', 'polygon', etc.
  chain_id: number;
  address: string;
  pubkey?: string;
  derivation_path?: string;
  balance: string;  // Wei/Satoshi
  balance_updated_at?: string;
  is_default: boolean;
  tags?: string[];
  metadata?: Record<string, any>;
  created_at: string;
  updated_at: string;
}
```

### Transaction (交易)

```typescript
interface Transaction {
  id: string;  // UUID
  wallet_id: string;  // UUID
  user_id: string;  // UUID
  tenant_id: string;  // UUID
  chain: string;
  chain_id: number;
  to_address: string;
  amount: string;  // Wei/Satoshi
  token_symbol?: string;
  token_contract?: string;
  data?: string;  // Hex
  nonce?: number;
  gas_limit?: number;
  gas_price?: string;
  max_fee_per_gas?: string;  // EIP-1559
  max_priority_fee_per_gas?: string;  // EIP-1559
  status: 'draft' | 'pending_approval' | 'approved' | 'signed' | 'broadcasted' | 'confirmed' | 'failed';
  tx_hash?: string;
  block_number?: number;
  confirmations?: number;
  fee?: string;
  error_message?: string;
  created_at: string;
  updated_at: string;
}
```

### TokenBalance (代币余额)

```typescript
interface TokenBalance {
  wallet_id: string;
  chain: string;
  token_symbol: string;
  token_contract?: string;  // null for native token
  balance: string;
  decimals: number;
  token_name?: string;
  token_logo_url?: string;
  price_usd?: number;
  updated_at: string;
}
```

---

## 前端集成建议

### 1. API 客户端封装

```typescript
// api/client.ts
import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8088';  // New backend

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 请求拦截器：自动添加 Token
apiClient.interceptors.request.use((config) => {
  const token = localStorage.getItem('access_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// 响应拦截器：统一错误处理
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Token 过期，跳转登录
      localStorage.removeItem('access_token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);
```

### 2. API 服务封装

```typescript
// api/wallet.ts
export const walletApi = {
  // 创建钱包
  createWallet: async (data: CreateWalletRequest) => {
    const response = await apiClient.post('/api/wallets', data);
    return response.data;
  },

  // 获取钱包列表
  listWallets: async (page = 0, pageSize = 20) => {
    const response = await apiClient.get('/api/wallets', {
      params: { page, page_size: pageSize },
    });
    return response.data;
  },

  // 获取钱包详情
  getWallet: async (walletId: string) => {
    const response = await apiClient.get(`/api/wallets/${walletId}`);
    return response.data;
  },

  // 删除钱包
  deleteWallet: async (walletId: string) => {
    const response = await apiClient.delete(`/api/wallets/${walletId}`);
    return response.data;
  },
};
```

### 3. 状态管理 (Dioxus Signals)

```rust
// src/state/wallet.rs
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct WalletState {
    pub wallets: Vec<Wallet>,
    pub selected_wallet: Option<Wallet>,
    pub loading: bool,
    pub error: Option<String>,
}

pub fn use_wallet_state() -> Signal<WalletState> {
    use_context()
}

pub async fn fetch_wallets(state: Signal<WalletState>) {
    state.write().loading = true;
    
    match wallet_api::list_wallets().await {
        Ok(response) => {
            state.write().wallets = response.wallets;
            state.write().loading = false;
        }
        Err(e) => {
            state.write().error = Some(e.to_string());
            state.write().loading = false;
        }
    }
}
```

### 4. 错误处理

```typescript
// utils/error-handler.ts
export const handleApiError = (error: any): string => {
  if (error.response?.data?.error) {
    return error.response.data.error;
  }
  
  if (error.response?.status === 401) {
    return '未授权，请重新登录';
  }
  
  if (error.response?.status === 404) {
    return '资源不存在';
  }
  
  if (error.response?.status === 429) {
    return '请求过于频繁，请稍后再试';
  }
  
  return '网络错误，请检查连接';
};
```

---

## 注意事项

### 🔐 安全建议

1. **私钥管理**: 
   - 私钥**永远不发送**到后端
   - 交易签名在前端完成
   - 使用 IndexedDB 加密存储

2. **Token 管理**:
   - Token 存储在 LocalStorage/SessionStorage
   - Token 过期自动刷新
   - 登出时清除 Token

3. **HTTPS**:
   - 生产环境必须使用 HTTPS
   - 开发环境可使用 HTTP (localhost)

### ⚡ 性能优化

1. **缓存策略**:
   - 余额数据缓存 5 分钟
   - 钱包列表缓存 10 分钟
   - Gas 价格缓存 30 秒

2. **分页加载**:
   - 钱包列表默认 20 条/页
   - 交易历史默认 20 条/页
   - 支持无限滚动

3. **请求优化**:
   - 合并并发请求
   - 使用 WebSocket 实时更新
   - 避免频繁轮询

### 🧪 测试建议

1. **单元测试**: 测试 API 调用逻辑
2. **集成测试**: 测试完整流程（注册→创建钱包→发送交易）
3. **E2E 测试**: 使用 Playwright/Cypress 测试用户流程

---

## 更新日志

- **2025-11-25**: 初始版本，基于 IronCore v2.0
- 包含 46+ API 端点
- 支持 5 条主链（Ethereum, BSC, Polygon, Bitcoin, TON）
- 完整的认证、钱包、交易、桥接功能

---

**参考文档**:
- [IronCore 架构文档](../../IronCore/docs/01-architecture/ARCHITECTURE.md)
- [IronCore API 文档](../../IronCore/docs/03-api/API_DOCUMENTATION.md)
- [数据库架构](./03-database-architecture.md)
