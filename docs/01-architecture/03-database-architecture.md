# 数据库架构设计 (Database Architecture)

> **版本**: V2.0  
> **更新日期**: 2024-01-21  
> **参考**: Ecosystem-Study_生态学习指南/02-database/day02-database-architecture.md

---

## 1. 数据库组合选型

### 1.1 三数据库协同架构

```
┌─────────────────────────────────────────────────────────┐
│                   IronForge V2 Backend                  │
└───┬─────────────────┬──────────────────┬────────────────┘
    │                 │                  │
    ▼                 ▼                  ▼
┌─────────────┐  ┌──────────┐  ┌────────────────┐
│ CockroachDB │  │  Redis   │  │    Immudb      │
│  (主库)      │  │ (缓存)    │  │  (审计账本)     │
├─────────────┤  ├──────────┤  ├────────────────┤
│ 用户数据     │  │ 会话Token │  │ 关键操作日志   │
│ 钱包元数据   │  │ 余额缓存  │  │ 交易签名记录   │
│ 交易历史     │  │ Gas价格   │  │ 钱包创建记录   │
│ 多签策略     │  │ 限流计数  │  │ 合规审计证据   │
└─────────────┘  └──────────┘  └────────────────┘
  ACID事务          < 100ms       不可篡改
  强一致性          内存存储      加密验证
```

### 1.2 为什么需要三个数据库？

| 数据库 | 用途 | 优势 | 典型数据 |
|--------|------|------|----------|
| **CockroachDB** | 事务主库 | ✅ 分布式SQL<br>✅ ACID事务<br>✅ 水平扩展<br>✅ 跨区域容灾 | 用户、钱包、交易历史 |
| **Redis** | 高速缓存 | ✅ 内存级性能<br>✅ TTL自动过期<br>✅ 分布式锁<br>✅ 发布订阅 | JWT Token、余额快照、Gas价格 |
| **Immudb** | 审计账本 | ✅ 不可篡改<br>✅ 可验证证明<br>✅ 合规友好<br>✅ 时间旅行 | 关键操作日志、合规证据 |

---

## 2. CockroachDB 数据模型

### 2.1 完整 Schema（生产级）

```sql
-- ============================================
-- 多租户表（企业版功能）
-- ============================================
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    plan VARCHAR(50) DEFAULT 'free',      -- free, pro, enterprise
    max_wallets INT DEFAULT 10,
    max_transactions_per_day INT DEFAULT 100,
    api_rate_limit INT DEFAULT 100,       -- 请求/分钟
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    INDEX idx_tenants_plan (plan)
);

-- ============================================
-- 用户表
-- ============================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),  -- 多租户隔离
    email VARCHAR(255) UNIQUE NOT NULL,
    email_cipher TEXT,                    -- 加密邮箱（合规要求）
    phone_cipher TEXT,                    -- 加密手机号
    password_hash VARCHAR(255) NOT NULL,  -- Argon2id
    role VARCHAR(20) DEFAULT 'user',      -- user, admin, super_admin
    mfa_enabled BOOLEAN DEFAULT false,
    mfa_secret TEXT,                      -- 2FA密钥（加密）
    api_key_hash VARCHAR(255) UNIQUE,     -- API Key 哈希
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'active',  -- active, suspended, deleted
    
    INDEX idx_users_email (email),
    INDEX idx_users_tenant_id (tenant_id),
    INDEX idx_users_status (status),
    INDEX idx_users_api_key (api_key_hash)
);

-- ============================================
-- 钱包元数据表（❌ 不存储私钥）
-- ============================================
CREATE TABLE wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),        -- 多租户隔离
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,                   -- "My Main Wallet"
    chain VARCHAR(20) NOT NULL,                   -- "ethereum", "bitcoin", "ton"
    chain_id INT NOT NULL,                        -- 1 (ETH), 56 (BSC), 607 (TON)
    address VARCHAR(255) NOT NULL,                -- 公开地址
    pubkey TEXT,                                  -- 公钥（用于验证签名）
    derivation_path VARCHAR(50),                  -- "m/44'/60'/0'/0/0"
    policy_id UUID REFERENCES policies(id),       -- 多签策略（可选）
    balance DECIMAL(36, 18) DEFAULT 0,            -- 余额快照（缓存）
    balance_updated_at TIMESTAMPTZ,               -- 余额更新时间
    is_default BOOLEAN DEFAULT false,             -- 是否默认钱包
    is_watch_only BOOLEAN DEFAULT false,          -- 观察钱包（只能查询）
    tags TEXT[],                                  -- 自定义标签
    metadata JSONB,                               -- 扩展字段
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- ❌ 绝对不存储: encrypted_mnemonic, private_key, raw_seed
    
    UNIQUE (user_id, chain, address),
    INDEX idx_wallets_tenant_id (tenant_id),
    INDEX idx_wallets_user_id (user_id),
    INDEX idx_wallets_chain (chain),
    INDEX idx_wallets_chain_id (chain_id),
    INDEX idx_wallets_address (address),
    INDEX idx_wallets_is_default (user_id, is_default) WHERE is_default = true
);

-- ============================================
-- 多签策略表（企业功能）
-- ============================================
CREATE TABLE policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    required_approvals INT NOT NULL DEFAULT 1,   -- M-of-N 中的 M
    total_approvers INT NOT NULL DEFAULT 1,      -- N
    approvers UUID[] NOT NULL,                   -- 审批人 user_id 数组
    approval_timeout_hours INT DEFAULT 24,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    INDEX idx_policies_tenant_id (tenant_id),
    CHECK (required_approvals <= total_approvers),
    CHECK (cardinality(approvers) = total_approvers)
);

-- ============================================
-- 交易请求表（客户端发起）
-- ============================================
CREATE TABLE tx_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),
    wallet_id UUID NOT NULL REFERENCES wallets(id),
    user_id UUID NOT NULL REFERENCES users(id),
    chain VARCHAR(20) NOT NULL,
    chain_id INT NOT NULL,
    to_address VARCHAR(255) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    token_symbol VARCHAR(20),                     -- ETH, USDT, BTC
    token_contract VARCHAR(255),                  -- ERC20 合约地址
    data TEXT,                                    -- 合约调用数据（hex）
    nonce BIGINT,
    gas_limit BIGINT,
    gas_price DECIMAL(36, 18),
    max_fee_per_gas DECIMAL(36, 18),              -- EIP-1559
    max_priority_fee_per_gas DECIMAL(36, 18),     -- EIP-1559
    status VARCHAR(20) NOT NULL DEFAULT 'draft',  
    -- 状态机: draft → pending_approval → approved → signed → broadcasted → confirmed / failed
    signed_tx TEXT,                               -- 已签名交易（hex）
    tx_hash VARCHAR(255),                         -- 广播后的交易哈希
    error_message TEXT,
    metadata JSONB,                               -- 扩展字段
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,                       -- 请求过期时间
    
    INDEX idx_tx_requests_tenant_id (tenant_id),
    INDEX idx_tx_requests_wallet_id (wallet_id),
    INDEX idx_tx_requests_user_id (user_id),
    INDEX idx_tx_requests_status (status),
    INDEX idx_tx_requests_created_at (created_at DESC),
    INDEX idx_tx_requests_tx_hash (tx_hash) WHERE tx_hash IS NOT NULL
);

-- ============================================
-- 审批记录表（多签工作流）
-- ============================================
CREATE TABLE approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_request_id UUID NOT NULL REFERENCES tx_requests(id) ON DELETE CASCADE,
    approver_id UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL,                  -- pending, approved, rejected
    comment TEXT,
    signature TEXT,                               -- 审批签名（可选）
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE (tx_request_id, approver_id),
    INDEX idx_approvals_tx_request (tx_request_id),
    INDEX idx_approvals_approver (approver_id),
    INDEX idx_approvals_status (status)
);

-- ============================================
-- 交易历史表（链上已确认）
-- ============================================
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    tx_request_id UUID REFERENCES tx_requests(id),  -- 关联交易请求
    tx_hash VARCHAR(255) NOT NULL,                  -- 交易哈希
    chain VARCHAR(20) NOT NULL,
    chain_id INT NOT NULL,
    from_address VARCHAR(255) NOT NULL,
    to_address VARCHAR(255) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    token_symbol VARCHAR(20),
    token_contract VARCHAR(255),
    fee DECIMAL(36, 18),                            -- 实际Gas费用
    fee_currency VARCHAR(10),                       -- ETH, BNB, MATIC
    status VARCHAR(20) NOT NULL,                    -- pending, confirmed, failed
    block_number BIGINT,
    block_hash VARCHAR(255),
    confirmations INT DEFAULT 0,
    nonce BIGINT,
    input_data TEXT,                                -- 合约调用数据
    logs JSONB,                                     -- 事件日志
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE (chain, tx_hash),
    INDEX idx_tx_tenant_id (tenant_id),
    INDEX idx_tx_wallet_id (wallet_id),
    INDEX idx_tx_chain_hash (chain, tx_hash),
    INDEX idx_tx_from_address (from_address),
    INDEX idx_tx_to_address (to_address),
    INDEX idx_tx_timestamp (timestamp DESC),
    INDEX idx_tx_status (status),
    INDEX idx_tx_block_number (chain_id, block_number DESC)
);

-- ============================================
-- 代币余额表（多资产支持）
-- ============================================
CREATE TABLE token_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    chain VARCHAR(20) NOT NULL,
    token_symbol VARCHAR(20) NOT NULL,
    token_contract VARCHAR(255),                    -- NULL 表示原生代币
    balance DECIMAL(36, 18) DEFAULT 0,
    decimals INT DEFAULT 18,
    token_name VARCHAR(100),
    token_logo_url TEXT,
    price_usd DECIMAL(18, 8),                       -- USD 价格快照
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE (wallet_id, chain, token_contract),
    INDEX idx_token_wallet_id (wallet_id),
    INDEX idx_token_chain (chain),
    INDEX idx_token_symbol (token_symbol)
);

-- ============================================
-- 地址簿表（常用地址）
-- ============================================
CREATE TABLE address_book (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    chain VARCHAR(20) NOT NULL,
    address VARCHAR(255) NOT NULL,
    tags TEXT[],
    is_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE (user_id, chain, address),
    INDEX idx_address_book_user_id (user_id),
    INDEX idx_address_book_chain (chain)
);

-- ============================================
-- 审计日志表（操作记录）
-- ============================================
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),
    user_id UUID REFERENCES users(id),
    resource_type VARCHAR(50) NOT NULL,             -- wallet, transaction, user
    resource_id UUID,
    action VARCHAR(50) NOT NULL,                    -- create, update, delete, sign
    status VARCHAR(20) NOT NULL,                    -- success, failure
    ip_address INET,
    user_agent TEXT,
    request_id UUID,                                -- 链路追踪ID
    details JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    INDEX idx_audit_tenant_id (tenant_id),
    INDEX idx_audit_user_id (user_id),
    INDEX idx_audit_resource (resource_type, resource_id),
    INDEX idx_audit_created_at (created_at DESC)
);

-- ============================================
-- 会话表（可选，优先用 Redis）
-- ============================================
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    refresh_token_hash VARCHAR(255) UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    ip_address INET,
    user_agent TEXT,
    last_activity_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    INDEX idx_sessions_user_id (user_id),
    INDEX idx_sessions_expires_at (expires_at)
);
```

### 2.2 ER 关系图

```
┌──────────┐
│ Tenants  │
└────┬─────┘
     │1
     │
     │N
┌────▼─────┐         ┌──────────┐
│  Users   │────────▶│ Policies │ (M-of-N 多签)
└────┬─────┘         └────┬─────┘
     │1                   │
     │                    │
     │N                   │1
┌────▼─────┐◀────────────┘
│ Wallets  │
└────┬─────┘
     │1
     │
     │N
┌────▼────────────┐     ┌────────────┐
│  TxRequests     │────▶│ Approvals  │ (多签审批)
└────┬────────────┘     └────────────┘
     │1
     │
     │1
┌────▼────────────┐
│ Transactions    │ (链上确认)
└─────────────────┘

┌─────────────────┐
│ TokenBalances   │ (多资产)
└─────────────────┘

┌─────────────────┐
│  AuditLogs      │ (合规审计)
└─────────────────┘
```

### 2.3 数据库连接配置

```rust
// backend/src/infrastructure/database.rs
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(16)              // 最大连接数
        .min_connections(2)               // 最小连接数
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
```

---

## 3. Redis 缓存架构

### 3.1 缓存策略

```
# 分层缓存架构
┌─────────────────────────────────────────┐
│         应用层 (Application)              │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│    一级缓存: 进程内缓存 (moka)             │
│    容量: 1000 条                          │
│    TTL: 60 秒                             │
└──────────────┬──────────────────────────┘
               │ Miss
               ▼
┌─────────────────────────────────────────┐
│    二级缓存: Redis (分布式)                │
│    容量: 100,000+ 条                      │
│    TTL: 根据数据类型区分                   │
└──────────────┬──────────────────────────┘
               │ Miss
               ▼
┌─────────────────────────────────────────┐
│    数据源: CockroachDB                    │
└─────────────────────────────────────────┘
```

### 3.2 Redis 键命名规范

```redis
# 认证会话
user:session:{user_id}                    → JWT 数据 (TTL: 1 hour)
user:refresh:{user_id}                    → Refresh Token (TTL: 7 days)
user:api_key:{key_hash}                   → User Info (TTL: 24 hours)

# 业务数据缓存
wallet:info:{wallet_id}                   → 钱包元数据 (TTL: 10 min)
wallet:balance:{wallet_id}                → 余额数据 (TTL: 5 min)
wallet:list:{user_id}                     → 用户钱包列表 (TTL: 10 min)

tx:detail:{chain}:{tx_hash}               → 交易详情 (TTL: 1 hour)
tx:history:{wallet_id}:page:{n}           → 交易历史分页 (TTL: 5 min)
tx:pending:{wallet_id}                    → 待确认交易 (TTL: 1 hour)

# 区块链实时数据
chain:gas_price:{chain}                   → Gas 价格 (TTL: 30 sec)
chain:height:{chain}                      → 最新区块高度 (TTL: 15 sec)
chain:nonce:{chain}:{address}             → 地址 nonce (TTL: 30 sec)

token:price:{symbol}                      → 代币价格 (TTL: 1 min)
token:info:{chain}:{contract}             → 代币信息 (TTL: 1 hour)

# 限流计数器
rate_limit:user:{user_id}:{endpoint}      → 用户请求计数 (TTL: 60 sec)
rate_limit:ip:{ip}:{endpoint}             → IP 请求计数 (TTL: 60 sec)
rate_limit:global:{endpoint}              → 全局请求计数 (TTL: 1 sec)

# 分布式锁
lock:wallet:{wallet_id}                   → 钱包操作锁 (TTL: 30 sec)
lock:tx:{tx_hash}                         → 交易处理锁 (TTL: 10 sec)
lock:nonce:{chain}:{address}              → Nonce 分配锁 (TTL: 5 sec)

# 任务队列（可选，推荐用 RabbitMQ）
queue:tx_broadcast                        → 交易广播队列
queue:balance_update                      → 余额更新队列
```

### 3.3 Redis 集成代码

```rust
// backend/src/infrastructure/redis.rs
use redis::{Client, AsyncCommands, aio::ConnectionManager};
use serde::{Serialize, Deserialize};
use std::time::Duration;

pub struct RedisCtx {
    pub manager: ConnectionManager,
}

impl RedisCtx {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let client = Client::open(url)?;
        let manager = client.get_connection_manager().await?;
        Ok(Self { manager })
    }

    /// 通用 SET 操作（带 TTL）
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> anyhow::Result<()> {
        let json = serde_json::to_string(value)?;
        let mut conn = self.manager.clone();
        conn.set_ex(key, json, ttl.as_secs() as u64).await?;
        Ok(())
    }

    /// 通用 GET 操作
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> anyhow::Result<Option<T>> {
        let mut conn = self.manager.clone();
        let json: Option<String> = conn.get(key).await?;
        
        match json {
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
            None => Ok(None),
        }
    }

    /// 分布式锁（SETNX + EXPIRE）
    pub async fn acquire_lock(
        &self,
        key: &str,
        ttl: Duration,
    ) -> antml::Result<bool> {
        let mut conn = self.manager.clone();
        let lock_value = uuid::Uuid::new_v4().to_string();
        
        let result: bool = conn.set_nx(key, &lock_value).await?;
        if result {
            conn.expire(key, ttl.as_secs() as i64).await?;
        }
        Ok(result)
    }

    /// 限流检查（INCR + EXPIRE）
    pub async fn check_rate_limit(
        &self,
        key: &str,
        limit: u64,
        window: Duration,
    ) -> anyhow::Result<bool> {
        let mut conn = self.manager.clone();
        
        let count: u64 = conn.incr(key, 1).await?;
        if count == 1 {
            conn.expire(key, window.as_secs() as i64).await?;
        }
        
        Ok(count <= limit)
    }
}
```

---

## 4. Immudb 审计账本

### 4.1 审计数据模型

```sql
-- Immudb SQL Schema（不可篡改数据库）

CREATE TABLE audit_events (
    tx_id BIGINT PRIMARY KEY AUTO_INCREMENT,
    timestamp TIMESTAMP DEFAULT NOW(),
    tenant_id VARCHAR(36),
    user_id VARCHAR(36),
    event_type VARCHAR(50) NOT NULL,      -- wallet_created, tx_signed, tx_broadcasted
    resource_type VARCHAR(50) NOT NULL,   -- wallet, transaction, user, policy
    resource_id VARCHAR(36),
    action VARCHAR(50) NOT NULL,          -- create, update, delete, sign, approve
    status VARCHAR(20) NOT NULL,          -- success, failure
    ip_address VARCHAR(45),
    user_agent TEXT,
    request_id VARCHAR(36),               -- 分布式链路追踪 ID
    chain VARCHAR(20),
    tx_hash VARCHAR(255),
    details BLOB,                         -- JSON 序列化
    signature VARCHAR(255),               -- 事件签名（ed25519）
    INDEX idx_user_id (user_id),
    INDEX idx_resource (resource_type, resource_id),
    INDEX idx_event_type (event_type),
    INDEX idx_timestamp (timestamp)
);

-- Immudb 特性查询
-- 1. 验证数据库完整性
VERIFY DATABASE;

-- 2. 验证表完整性
VERIFY TABLE audit_events;

-- 3. 时间旅行查询（查询历史状态）
SELECT * FROM audit_events 
BEFORE TX 12345 
WHERE user_id = 'xxx';

-- 4. 获取加密证明
VERIFY ENTRY audit_events AT tx = 12345;
```

### 4.2 关键事件定义

| event_type | 触发时机 | 记录内容 |
|------------|----------|----------|
| `user_registered` | 用户注册 | email, ip_address, user_agent |
| `wallet_created` | 钱包创建 | chain, address, derivation_path |
| `tx_request_created` | 交易请求创建 | to_address, amount, chain |
| `tx_signed` | 交易签名 | tx_hash, signed_tx (无私钥) |
| `tx_broadcasted` | 交易广播 | tx_hash, chain, status |
| `tx_confirmed` | 交易确认 | tx_hash, block_number, confirmations |
| `approval_granted` | 多签审批 | tx_request_id, approver_id |
| `balance_changed` | 余额变化 | wallet_id, old_balance, new_balance |
| `api_key_created` | API密钥创建 | user_id, key_hash (无明文) |
| `login_success` | 登录成功 | user_id, ip_address, mfa_used |
| `login_failed` | 登录失败 | email, ip_address, reason |

### 4.3 Immudb 集成代码

```rust
// backend/src/infrastructure/immudb.rs
use immudb::client::ImmuClient;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct AuditEvent {
    pub user_id: Uuid,
    pub event_type: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub action: String,
    pub status: String,
    pub ip_address: Option<String>,
    pub details: serde_json::Value,
}

pub struct ImmuCtx {
    pub client: ImmuClient,
}

impl ImmuCtx {
    pub async fn new(addr: &str, user: &str, pass: &str, database: &str) -> anyhow::Result<Self> {
        let mut client = ImmuClient::new(addr).await?;
        client.login(user, pass).await?;
        client.use_database(database).await?;
        Ok(Self { client })
    }

    /// 记录审计事件
    pub async fn log_event(&self, event: &AuditEvent) -> anyhow::Result<u64> {
        let details_json = serde_json::to_string(&event.details)?;
        
        let sql = r#"
            INSERT INTO audit_events 
            (user_id, event_type, resource_type, resource_id, action, status, ip_address, details)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#;
        
        let tx_id = self.client.sql_exec(sql, vec![
            event.user_id.to_string(),
            event.event_type.clone(),
            event.resource_type.clone(),
            event.resource_id.to_string(),
            event.action.clone(),
            event.status.clone(),
            event.ip_address.clone().unwrap_or_default(),
            details_json,
        ]).await?;
        
        Ok(tx_id)
    }

    /// 验证审计链完整性
    pub async fn verify_integrity(&self) -> anyhow::Result<bool> {
        let result = self.client.verify_database().await?;
        Ok(result.is_valid)
    }

    /// 查询用户操作历史
    pub async fn get_user_history(
        &self,
        user_id: Uuid,
        limit: i32,
    ) -> anyhow::Result<Vec<AuditEvent>> {
        let sql = r#"
            SELECT user_id, event_type, resource_type, resource_id, action, status, details
            FROM audit_events
            WHERE user_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
        "#;
        
        let rows = self.client.sql_query(sql, vec![
            user_id.to_string(),
            limit.to_string(),
        ]).await?;
        
        // 解析结果...
        Ok(vec![])
    }
}
```

---

## 5. 多租户隔离策略

### 5.1 行级安全（Row-Level Security）

**核心原则**: 所有查询必须包含 `tenant_id` 过滤条件

```rust
// ❌ 错误示例（查询所有租户数据）
sqlx::query!("SELECT * FROM wallets WHERE user_id = $1", user_id)
    .fetch_all(&pool)
    .await?;

// ✅ 正确示例（包含 tenant_id）
sqlx::query!(
    "SELECT * FROM wallets WHERE tenant_id = $1 AND user_id = $2",
    tenant_id,
    user_id
)
.fetch_all(&pool)
.await?;
```

### 5.2 Repository 模式强制隔离

```rust
// backend/src/repository/wallet_repo.rs
use sqlx::PgPool;
use uuid::Uuid;

pub struct WalletRepository {
    pool: PgPool,
}

impl WalletRepository {
    /// 所有方法强制要求 tenant_id
    pub async fn find_by_id(
        &self,
        tenant_id: Uuid,
        wallet_id: Uuid,
    ) -> anyhow::Result<Option<Wallet>> {
        let wallet = sqlx::query_as!(
            Wallet,
            r#"
            SELECT * FROM wallets
            WHERE tenant_id = $1 AND id = $2
            "#,
            tenant_id,
            wallet_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(wallet)
    }

    pub async fn list_by_user(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<Wallet>> {
        let wallets = sqlx::query_as!(
            Wallet,
            r#"
            SELECT * FROM wallets
            WHERE tenant_id = $1 AND user_id = $2
            ORDER BY created_at DESC
            "#,
            tenant_id,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(wallets)
    }
}
```

### 5.3 中间件自动注入 tenant_id

```rust
// backend/src/api/middleware/tenant.rs
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

pub async fn tenant_extractor(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // 1. 从 JWT 中提取 tenant_id
    let claims = req.extensions().get::<JwtClaims>()
        .ok_or(ApiError::Unauthorized)?;
    
    let tenant_id = claims.tenant_id;
    
    // 2. 注入到请求上下文
    req.extensions_mut().insert(tenant_id);
    
    // 3. 继续处理
    Ok(next.run(req).await)
}
```

---

## 6. 数据迁移策略

### 6.1 迁移文件结构

```
backend/migrations/
├── 20240101000000_init_tenants.up.sql
├── 20240101000000_init_tenants.down.sql
├── 20240102000000_init_users.up.sql
├── 20240102000000_init_users.down.sql
├── 20240103000000_init_wallets.up.sql
├── 20240103000000_init_wallets.down.sql
├── 20240104000000_init_transactions.up.sql
└── 20240104000000_init_transactions.down.sql
```

### 6.2 自动迁移（启动时执行）

```rust
// backend/src/infrastructure/migration.rs
use sqlx::{PgPool, migrate::MigrateDatabase};

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    
    tracing::info!("Database migrations completed successfully");
    Ok(())
}
```

---

## 7. 性能优化

### 7.1 查询优化

```sql
-- ✅ 使用索引查询
EXPLAIN ANALYZE
SELECT * FROM transactions
WHERE wallet_id = '...' AND status = 'confirmed'
ORDER BY timestamp DESC
LIMIT 20;

-- ❌ 避免全表扫描
SELECT * FROM transactions WHERE amount > 1000;  -- 缺少索引

-- ✅ 使用复合索引
CREATE INDEX idx_tx_wallet_status_time 
ON transactions (wallet_id, status, timestamp DESC);
```

### 7.2 连接池配置

```toml
# config.toml
[database]
url = "postgres://root@localhost:26257/ironforge?sslmode=disable"
max_connections = 16
min_connections = 2
acquire_timeout_secs = 30
idle_timeout_secs = 600
max_lifetime_secs = 1800
```

### 7.3 缓存穿透防护

```rust
// 使用布隆过滤器 + 空值缓存
pub async fn get_wallet_balance(
    &self,
    wallet_id: Uuid,
) -> anyhow::Result<Option<Balance>> {
    // 1. 查 Redis
    let cache_key = format!("wallet:balance:{}", wallet_id);
    if let Some(balance) = self.redis.get(&cache_key).await? {
        return Ok(Some(balance));
    }
    
    // 2. 查数据库
    let balance = self.repo.get_balance(wallet_id).await?;
    
    // 3. 写回缓存（包括 None）
    match &balance {
        Some(b) => self.redis.set_with_ttl(&cache_key, b, Duration::from_secs(300)).await?,
        None => self.redis.set_with_ttl(&cache_key, "NULL", Duration::from_secs(60)).await?,
    }
    
    Ok(balance)
}
```

---

## 8. 备份与灾备

### 8.1 CockroachDB 备份策略

```bash
# 全量备份（每天凌晨）
cockroach sql --execute="BACKUP DATABASE ironforge TO 's3://backups/daily/2024-01-21';"

# 增量备份（每小时）
cockroach sql --execute="BACKUP DATABASE ironforge TO 's3://backups/hourly/2024-01-21-03' INCREMENTAL FROM 's3://backups/daily/2024-01-21';"

# 恢复数据
cockroach sql --execute="RESTORE DATABASE ironforge FROM 's3://backups/daily/2024-01-21';"
```

### 8.2 Redis 持久化

```conf
# redis.conf
save 900 1          # 15分钟内至少1个key改变
save 300 10         # 5分钟内至少10个key改变
save 60 10000       # 1分钟内至少10000个key改变

appendonly yes      # 开启 AOF
appendfsync everysec  # 每秒同步
```

---

## 9. 监控指标

```rust
// backend/src/infrastructure/monitoring.rs
use prometheus::{IntCounter, Histogram, register_int_counter, register_histogram};

lazy_static! {
    // 数据库查询计数
    pub static ref DB_QUERY_COUNT: IntCounter = 
        register_int_counter!("db_query_total", "Total database queries").unwrap();
    
    // 查询延迟
    pub static ref DB_QUERY_DURATION: Histogram = 
        register_histogram!("db_query_duration_seconds", "Query duration").unwrap();
    
    // 缓存命中率
    pub static ref CACHE_HIT_COUNT: IntCounter = 
        register_int_counter!("cache_hit_total", "Cache hits").unwrap();
    
    pub static ref CACHE_MISS_COUNT: IntCounter = 
        register_int_counter!("cache_miss_total", "Cache misses").unwrap();
}
```

---

## 10. 安全检查清单

- [ ] 所有 SQL 查询使用参数化（防 SQL 注入）
- [ ] 多租户查询强制包含 `tenant_id`
- [ ] 私钥/助记词**绝不存储**到后端数据库
- [ ] PII 数据加密存储（email_cipher, phone_cipher）
- [ ] 密码使用 Argon2id 哈希
- [ ] API Key 哈希存储（不存明文）
- [ ] 审计日志记录所有敏感操作
- [ ] 数据库连接使用 SSL/TLS
- [ ] Redis 启用密码认证
- [ ] Immudb 启用加密验证

---

**参考文档**:
- [Ecosystem-Study_生态学习指南/day02-database-architecture.md](../../Ecosystem-Study_生态学习指南/02-database/day02-database-architecture.md)
- [CockroachDB 官方文档](https://www.cockroachlabs.com/docs/)
- [Redis 最佳实践](https://redis.io/docs/manual/patterns/)
- [Immudb 审计指南](https://docs.immudb.io/)
