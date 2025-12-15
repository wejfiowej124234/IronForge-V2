# IronForge V2 - 数据分离模型

> 📅 创建日期: 2025-11-25  
> 🔒 版本: 2.0  
> 🎯 架构: 非托管钱包 + 企业级 API

---

## 📋 目录

- [核心理念](#核心理念)
- [数据分类](#数据分类)
- [前端数据存储](#前端数据存储)
- [后端数据存储](#后端数据存储)
- [数据流设计](#数据流设计)
- [安全保证](#安全保证)

---

## 🎯 核心理念

### 非托管 + 企业 API 混合模式

```
┌─────────────────────────────────────────────────────────────┐
│                      设计哲学                                 │
├─────────────────────────────────────────────────────────────┤
│ 1. 私钥用户掌控 - 前端加密存储，后端永不接触               │
│ 2. 元数据云端管理 - 提供企业级查询、统计、分析服务         │
│ 3. 可恢复性平衡 - 后端数据丢失可恢复，私钥丢失永久无法找回 │
│ 4. 零信任架构 - 假设后端可能被攻破，私钥仍然安全           │
└─────────────────────────────────────────────────────────────┘
```

### 为什么需要后端？

虽然是非托管钱包，但后端提供以下**增值服务**：

1. **多设备同步** - 钱包列表、配置在多设备间同步
2. **交易历史** - 聚合查询所有链的历史记录
3. **统计分析** - 资产分布、交易趋势、收益统计
4. **通知服务** - 交易到账提醒、价格预警
5. **审计合规** - 满足企业审计和监管要求
6. **备份恢复** - 元数据备份（不含私钥）

---

## 📊 数据分类

### 按敏感程度分类

| 级别 | 数据类型 | 存储位置 | 加密方式 | 示例 |
|------|---------|---------|---------|------|
| **🔴 绝密** | 私钥、助记词 | 前端 IndexedDB | AES-256-GCM | `mnemonic`, `private_key` |
| **🟠 敏感** | 用户密码 | 前端 + 后端 | Argon2id 哈希 | `password_hash` |
| **🟡 机密** | 会话 Token | 前端 + 后端缓存 | JWT 签名 | `access_token` |
| **🟢 内部** | 钱包元数据 | 后端数据库 | TLS 传输 | `wallet_name`, `address` |
| **⚪ 公开** | 交易哈希 | 后端 + 链上 | 无需加密 | `tx_hash` |

### 按可恢复性分类

| 数据类型 | 丢失后果 | 恢复方式 |
|---------|---------|---------|
| **私钥/助记词** | ❌ 永久无法找回资产 | 无法恢复（用户责任） |
| **用户密码** | ⚠️ 无法解锁本地钱包 | 可通过助记词重新导入 |
| **钱包元数据** | ✅ 只需重新添加钱包 | 后端数据库备份 |
| **交易历史** | ✅ 不影响资产安全 | 区块链浏览器查询 |
| **会话 Token** | ✅ 重新登录即可 | 重新认证 |

---

## 💻 前端数据存储

### 存储方案对比

| 方案 | 容量 | 性能 | 安全性 | 用途 |
|------|------|------|--------|------|
| **IndexedDB** | ~50MB+ | 高 | 中 (需加密) | 🔑 私钥、助记词 |
| **LocalStorage** | ~5MB | 中 | 低 | ⚙️ 配置、缓存 |
| **SessionStorage** | ~5MB | 中 | 低 | 🔄 临时会话 |
| **Memory (Signal)** | 无限 | 极高 | 高 | 🧠 运行时状态 |

### IndexedDB 数据结构

```typescript
// 数据库名称
const DB_NAME = "ironforge_wallet";
const DB_VERSION = 1;

// Object Stores (表)
interface WalletStore {
  id: string;                    // UUID
  name: string;                  // 钱包名称
  encrypted_mnemonic: string;    // 🔒 AES-256-GCM 加密的助记词
  password_hash: string;         // Argon2id 哈希
  created_at: number;            // 时间戳
  updated_at: number;
  
  // 派生的地址（明文，公开数据）
  addresses: {
    chain: string;               // "ethereum", "bitcoin", "ton"
    address: string;             // 公开地址
    derivation_path: string;     // "m/44'/60'/0'/0/0"
  }[];
  
  // 本地配置
  settings: {
    auto_lock_minutes: number;
    default_chain: string;
    currency: string;            // "USD", "CNY"
  };
}

// 加密密钥派生
interface KeyDerivationParams {
  salt: Uint8Array;              // 32 字节随机盐
  iterations: number;            // Argon2 迭代次数
  memory: number;                // 内存消耗 (KB)
  parallelism: number;           // 并行度
}
```

### 加密流程

```rust
// src/infrastructure/crypto/encryption.rs

use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{Argon2, PasswordHasher};
use rand::RngCore;

pub struct WalletEncryption;

impl WalletEncryption {
    /// 用户密码 → 加密密钥 (Argon2id)
    pub fn derive_key(password: &str, salt: &[u8]) -> Result<Vec<u8>> {
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(
            password.as_bytes(),
            salt,
        )?;
        
        Ok(password_hash.hash.unwrap().as_bytes().to_vec())
    }
    
    /// AES-256-GCM 加密助记词
    pub fn encrypt_mnemonic(
        mnemonic: &str,
        password: &str,
    ) -> Result<String> {
        // 1. 生成随机盐
        let mut salt = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);
        
        // 2. 派生加密密钥
        let key = Self::derive_key(password, &salt)?;
        let cipher = Aes256Gcm::new(Key::from_slice(&key));
        
        // 3. 生成随机 Nonce
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        
        // 4. 加密
        let ciphertext = cipher.encrypt(
            Nonce::from_slice(&nonce),
            mnemonic.as_bytes(),
        )?;
        
        // 5. 组合: salt + nonce + ciphertext
        let mut result = Vec::new();
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        // 6. Base64 编码
        Ok(base64::encode(&result))
    }
    
    /// AES-256-GCM 解密助记词
    pub fn decrypt_mnemonic(
        encrypted: &str,
        password: &str,
    ) -> Result<String> {
        // 1. Base64 解码
        let data = base64::decode(encrypted)?;
        
        // 2. 分离: salt (32) + nonce (12) + ciphertext
        let salt = &data[0..32];
        let nonce = &data[32..44];
        let ciphertext = &data[44..];
        
        // 3. 派生解密密钥
        let key = Self::derive_key(password, salt)?;
        let cipher = Aes256Gcm::new(Key::from_slice(&key));
        
        // 4. 解密
        let plaintext = cipher.decrypt(
            Nonce::from_slice(nonce),
            ciphertext,
        )?;
        
        // 5. 转换为字符串
        Ok(String::from_utf8(plaintext)?)
    }
}
```

### 前端存储 Rust 实现

```rust
// src/infrastructure/storage/wallet_storage.rs

use rexie::{Rexie, Store, ObjectStore};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct StoredWallet {
    pub id: String,
    pub name: String,
    pub encrypted_mnemonic: String,
    pub password_hash: String,
    pub addresses: Vec<Address>,
    pub created_at: i64,
}

pub struct WalletStorage {
    db: Rexie,
}

impl WalletStorage {
    /// 初始化 IndexedDB
    pub async fn new() -> Result<Self> {
        let db = Rexie::builder("ironforge_wallet")
            .version(1)
            .add_object_store(
                ObjectStore::new("wallets")
                    .key_path("id")
                    .auto_increment(false)
            )
            .build()
            .await?;
        
        Ok(Self { db })
    }
    
    /// 保存钱包（加密后）
    pub async fn save_wallet(&self, wallet: &StoredWallet) -> Result<()> {
        let tx = self.db.transaction(&["wallets"], TransactionMode::ReadWrite)?;
        let store = tx.store("wallets")?;
        
        store.put(&serde_wasm_bindgen::to_value(wallet)?).await?;
        tx.commit().await?;
        
        Ok(())
    }
    
    /// 读取钱包
    pub async fn get_wallet(&self, id: &str) -> Result<Option<StoredWallet>> {
        let tx = self.db.transaction(&["wallets"], TransactionMode::ReadOnly)?;
        let store = tx.store("wallets")?;
        
        let value = store.get(&id.into()).await?;
        
        match value {
            Some(v) => Ok(Some(serde_wasm_bindgen::from_value(v)?)),
            None => Ok(None),
        }
    }
    
    /// 列出所有钱包
    pub async fn list_wallets(&self) -> Result<Vec<StoredWallet>> {
        let tx = self.db.transaction(&["wallets"], TransactionMode::ReadOnly)?;
        let store = tx.store("wallets")?;
        
        let values = store.get_all(None).await?;
        
        let wallets = values
            .into_iter()
            .filter_map(|v| serde_wasm_bindgen::from_value(v).ok())
            .collect();
        
        Ok(wallets)
    }
    
    /// 删除钱包
    pub async fn delete_wallet(&self, id: &str) -> Result<()> {
        let tx = self.db.transaction(&["wallets"], TransactionMode::ReadWrite)?;
        let store = tx.store("wallets")?;
        
        store.delete(&id.into()).await?;
        tx.commit().await?;
        
        Ok(())
    }
}
```

---

## 🏢 后端数据存储

### 后端数据库设计 (CockroachDB / PostgreSQL)

#### 数据库组合方案

```
┌─────────────────────────────────────────┐
│   CockroachDB (事务主库)                 │
│   用途：业务数据存储                      │
│   特点：ACID事务、分布式、强一致性       │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│   Redis (缓存层)                         │
│   用途：会话、限流、热点数据缓存          │
│   特点：内存存储、高性能、TTL过期        │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│   Immudb (审计账本)                      │
│   用途：关键事件存证、合规审计            │
│   特点：不可篡改、可验证证明              │
└─────────────────────────────────────────┘
```

#### 完整 SQL Schema

```sql
-- ============================================
-- 多租户表（可选，企业版功能）
-- ============================================
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    plan VARCHAR(50) DEFAULT 'free',      -- free, pro, enterprise
    max_wallets INT DEFAULT 10,
    max_transactions_per_day INT DEFAULT 100,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    INDEX idx_tenants_plan (plan)
);

-- ============================================
-- 用户表
-- ============================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),  -- 可选，多租户场景
    email VARCHAR(255) UNIQUE NOT NULL,
    email_cipher TEXT,                    -- 加密后的邮箱（企业合规）
    password_hash VARCHAR(255) NOT NULL,  -- Argon2id 哈希
    role VARCHAR(20) DEFAULT 'user',      -- user, admin, super_admin
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'active',  -- active, suspended, deleted
    
    INDEX idx_users_email (email),
    INDEX idx_users_tenant_id (tenant_id),
    INDEX idx_users_status (status)
);

-- ============================================
-- 钱包元数据表（不含私钥）
-- ============================================
CREATE TABLE wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),        -- 多租户隔离
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,                   -- "My Main Wallet"
    chain VARCHAR(20) NOT NULL,                   -- "ethereum", "bitcoin", "ton"
    chain_id INT NOT NULL,                        -- 1 (ETH), 56 (BSC), 607 (TON)
    address VARCHAR(255) NOT NULL,                -- 公开地址
    pubkey TEXT,                                  -- 公钥（可选）
    derivation_path VARCHAR(50),                  -- "m/44'/60'/0'/0/0"
    policy_id UUID REFERENCES policies(id),       -- 多签策略（可选）
    balance DECIMAL(36, 18) DEFAULT 0,            -- 余额快照（缓存）
    balance_updated_at TIMESTAMPTZ,               -- 余额更新时间
    is_default BOOLEAN DEFAULT false,             -- 是否默认钱包
    tags TEXT[],                                  -- 标签数组
    metadata JSONB,                               -- 扩展元数据
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- ❌ 不存储: encrypted_mnemonic, private_key
    
    UNIQUE (user_id, chain, address),
    INDEX idx_wallets_tenant_id (tenant_id),
    INDEX idx_wallets_user_id (user_id),
    INDEX idx_wallets_chain (chain),
    INDEX idx_wallets_chain_id (chain_id),
    INDEX idx_wallets_address (address)
);

-- ============================================
-- 多签策略表（可选，企业功能）
-- ============================================
CREATE TABLE policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),
    name VARCHAR(100) NOT NULL,
    required_approvals INT NOT NULL DEFAULT 1,
    approvers UUID[] NOT NULL,                    -- 审批人 user_id 数组
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    INDEX idx_policies_tenant_id (tenant_id)
);

-- ============================================
-- 审批记录表（可选，企业功能）
-- ============================================
CREATE TABLE approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_request_id UUID NOT NULL REFERENCES tx_requests(id),
    approver_id UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL,                  -- pending, approved, rejected
    comment TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE (tx_request_id, approver_id),
    INDEX idx_approvals_tx_request (tx_request_id),
    INDEX idx_approvals_approver (approver_id)
);

-- ============================================
-- 交易请求表（发起交易）
-- ============================================
CREATE TABLE tx_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),
    wallet_id UUID NOT NULL REFERENCES wallets(id),
    chain VARCHAR(20) NOT NULL,
    chain_id INT NOT NULL,
    to_address VARCHAR(255) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    token_symbol VARCHAR(20),
    token_contract VARCHAR(255),
    data TEXT,                            -- 合约调用数据
    nonce BIGINT,
    gas_limit BIGINT,
    gas_price DECIMAL(36, 18),
    status VARCHAR(20) NOT NULL DEFAULT 'draft',  
    -- draft/pending_approval/approved/signed/broadcasted/confirmed/failed
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    INDEX idx_tx_requests_tenant_id (tenant_id),
    INDEX idx_tx_requests_wallet_id (wallet_id),
    INDEX idx_tx_requests_status (status)
);

-- ============================================
-- 交易历史表（链上已确认）
-- ============================================
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    tx_request_id UUID REFERENCES tx_requests(id),  -- 关联交易请求
    tx_hash VARCHAR(255) NOT NULL,        -- 交易哈希
    chain VARCHAR(20) NOT NULL,
    chain_id INT NOT NULL,
    from_address VARCHAR(255) NOT NULL,
    to_address VARCHAR(255) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    token_symbol VARCHAR(20),             -- "ETH", "BTC", "USDT"
    token_contract VARCHAR(255),          -- ERC20 合约地址
    fee DECIMAL(36, 18),                  -- Gas 费用
    status VARCHAR(20) NOT NULL,          -- pending, confirmed, failed
    block_number BIGINT,
    confirmations INT DEFAULT 0,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE (chain, tx_hash),
    INDEX idx_tx_tenant_id (tenant_id),
    INDEX idx_tx_wallet_id (wallet_id),
    INDEX idx_tx_chain_hash (chain, tx_hash),
    INDEX idx_tx_timestamp (timestamp DESC),
    INDEX idx_tx_status (status)
);

-- ============================================
-- 代币余额表
-- ============================================
CREATE TABLE token_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    chain VARCHAR(20) NOT NULL,
    token_symbol VARCHAR(20) NOT NULL,
    token_name VARCHAR(100),
    token_contract VARCHAR(255),
    balance DECIMAL(36, 18) DEFAULT 0,
    decimals INT DEFAULT 18,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE (wallet_id, chain, token_contract),
    INDEX idx_token_wallet_id (wallet_id)
);

-- ============================================
-- 审计日志表（重要操作记录）
-- ============================================
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    wallet_id UUID REFERENCES wallets(id),
    action VARCHAR(50) NOT NULL,          -- "wallet_created", "transaction_sent"
    ip_address INET,
    user_agent TEXT,
    details JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    INDEX idx_audit_user_id (user_id),
    INDEX idx_audit_created_at (created_at DESC)
);

-- ============================================
-- 会话表（可选，Redis 替代方案）
-- ============================================
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    INDEX idx_sessions_user_id (user_id),
    INDEX idx_sessions_expires_at (expires_at)
);
```

### Redis 缓存结构

```
# 会话缓存
session:{token_hash}
  -> { user_id, email, expires_at }
  TTL: 24 hours

# 余额缓存
balance:{wallet_id}
  -> { balance, updated_at }
  TTL: 5 minutes

# API 响应缓存
api:wallets:{user_id}
  -> [ wallet1, wallet2, ... ]
  TTL: 1 minute

# 速率限制
ratelimit:{user_id}:{endpoint}
  -> request_count
  TTL: 1 minute
```

---

## 🔄 数据流设计

### 创建钱包流程

```
┌────────────────────────────────────────────────────────────┐
│ 1. 用户输入钱包名称 + 密码                                   │
└──────────────────────┬─────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│ 2. 前端生成 BIP39 助记词 (12/24 词)                         │
│    - 使用 getrandom (WASM 随机数)                           │
│    - 完全在浏览器执行                                        │
└──────────────────────┬─────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│ 3. 前端用密码加密助记词                                      │
│    - Argon2id 派生加密密钥                                   │
│    - AES-256-GCM 加密                                       │
└──────────────────────┬─────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│ 4. 前端保存到 IndexedDB                                      │
│    - wallet_id: UUID                                        │
│    - encrypted_mnemonic: base64                             │
│    - addresses: [{ chain, address }]                        │
└──────────────────────┬─────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│ 5. 前端调用后端 API: POST /api/wallets                      │
│    Body: {                                                  │
│      name: "My Wallet",                                     │
│      chain: "ethereum",                                     │
│      address: "0x1234..."  // 公开地址                      │
│    }                                                        │
│    ❌ 不发送: mnemonic, private_key, password              │
└──────────────────────┬─────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│ 6. 后端保存元数据到数据库                                    │
│    - INSERT INTO wallets (user_id, name, address)           │
│    - 返回 wallet_id                                         │
└────────────────────────────────────────────────────────────┘
```

### 发送交易流程

```
┌────────────────────────────────────────────────────────────┐
│ 1. 用户输入: 接收地址 + 金额                                │
└──────────────────────┬─────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│ 2. 前端弹出密码确认框                                        │
└──────────────────────┬─────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│ 3. 前端用密码解密助记词                                      │
│    - 从 IndexedDB 读取 encrypted_mnemonic                   │
│    - AES-256-GCM 解密                                       │
└──────────────────────┬─────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│ 4. 前端派生私钥并签名交易                                    │
│    - BIP32 派生: mnemonic → seed → private_key              │
│    - 签名: sign(tx, private_key)                            │
│    - 立即清零: zeroize(private_key)                         │
└──────────────────────┬─────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│ 5. 前端广播到区块链 RPC                                      │
│    - 直接调用链的 RPC 节点                                   │
│    - 或通过后端代理: POST /api/transactions/broadcast       │
└──────────────────────┬─────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│ 6. 前端通知后端记录交易                                      │
│    POST /api/transactions                                   │
│    Body: {                                                  │
│      wallet_id: "uuid",                                     │
│      tx_hash: "0xabc...",                                   │
│      from: "0x123...",                                      │
│      to: "0x456...",                                        │
│      amount: "1.5"                                          │
│    }                                                        │
└──────────────────────┬─────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│ 7. 后端保存交易记录到数据库                                  │
│    - INSERT INTO transactions                               │
│    - 提供历史查询                                            │
└────────────────────────────────────────────────────────────┘
```

---

## 🔒 安全保证

### 1. 私钥永不离开浏览器

```rust
// ✅ 正确：私钥仅在内存中临时存在
let private_key = derive_private_key(&mnemonic);
let signature = sign_transaction(&tx, &private_key);
zeroize::Zeroize::zeroize(&mut private_key);  // 立即清零

// ❌ 错误：私钥发送到后端
let response = api_client.post("/api/sign")
    .json(&private_key)  // 绝对禁止！
    .send()
    .await?;
```

### 2. 后端无法解密助记词

后端即使获取到 `encrypted_mnemonic`，也无法解密，因为：
- 加密密钥从用户密码派生（后端不知道）
- 每个钱包使用不同的随机盐
- 使用 Argon2id（慢速哈希，防暴力破解）

### 3. 零信任架构

```
假设场景：后端数据库被黑客攻破

黑客获得：
  ✅ 用户邮箱
  ✅ 钱包地址
  ✅ 交易历史
  ❌ 私钥（前端存储）
  ❌ 助记词（前端加密）
  ❌ 用户密码（Argon2id 哈希）

结果：
  - 黑客无法转移资产 ✅
  - 黑客无法签名交易 ✅
  - 用户资产仍然安全 ✅
```

### 4. 审计日志不可篡改

使用 Immudb（不可变数据库）记录所有敏感操作：
- 钱包创建
- 交易发送
- 登录记录
- 配置修改

### 5. 多层防护

| 层级 | 防护措施 |
|------|---------|
| **传输层** | HTTPS + Certificate Pinning |
| **应用层** | JWT Token + CSRF Token |
| **数据层** | AES-256-GCM + Argon2id |
| **物理层** | IndexedDB 限制同源访问 |
| **审计层** | Immudb 防篡改日志 |

---

## ✅ 总结

IronForge V2 的数据分离模型确保：

1. **用户资产安全** - 私钥本地存储，用户完全掌控
2. **便利性** - 后端提供企业级查询和统计服务
3. **可恢复性** - 元数据丢失可恢复，私钥丢失用户责任
4. **合规性** - 满足审计和监管要求
5. **可扩展性** - 支持多设备、多链、多用户

这是 **非托管钱包** 和 **企业服务** 的完美平衡。

---

**下一步**: 阅读 [安全架构设计](../04-security/01-security-architecture.md)

**最后更新**: 2025-11-25
