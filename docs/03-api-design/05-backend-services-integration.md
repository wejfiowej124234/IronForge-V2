# 后端服务集成指南 - Gas估算与费用系统

> **版本**: V2.0  
> **更新日期**: 2025-11-25  
> **状态**: ✅ 后端已实现  
> **相关模块**: Gas估算、费用收取、RPC选择器、管理员系统

---

## 📋 目录

1. [架构概览](#架构概览)
2. [RPC智能选择器](#rpc智能选择器)
3. [Gas费用估算服务](#gas费用估算服务)
4. [平台费用收取系统](#平台费用收取系统)
5. [管理员系统](#管理员系统)
6. [前端集成方案](#前端集成方案)

---

## 架构概览

### 后端已实现模块

```
┌─────────────────────────────────────────────────────┐
│               Frontend (Dioxus WASM)                │
└────────────┬────────────────────────────────────────┘
             │ HTTP/REST API
             ▼
┌─────────────────────────────────────────────────────┐
│            Backend API Layer (Axum)                 │
│  - /api/v1/gas/estimate                             │
│  - /api/v1/fees/calculate                           │
│  - /api/admin/rpc-endpoints                         │
│  - /api/admin/fee-rules                             │
└────────────┬────────────────────────────────────────┘
             │
      ┌──────┴──────────┬──────────────┬─────────────┐
      ▼                 ▼              ▼             ▼
┌──────────────┐  ┌──────────────┐  ┌──────────┐  ┌──────────┐
│ RpcSelector  │  │ GasEstimator │  │ FeeService│  │ AdminAPI │
│              │  │              │  │          │  │          │
│ • 健康检测   │  │ • EIP-1559   │  │ • 规则引擎│  │ • 规则管理│
│ • 故障转移   │  │ • 多链支持   │  │ • 审计日志│  │ • RPC管理│
│ • 熔断器     │  │ • 三档速度   │  │ • 二级缓存│  │ • 操作日志│
└──────────────┘  └──────────────┘  └──────────┘  └──────────┘
      │                 │              │             │
      └─────────────────┴──────────────┴─────────────┘
                          │
                          ▼
         ┌──────────────────────────────┐
         │   PostgreSQL/CockroachDB     │
         │   • admin.rpc_endpoints      │
         │   • gas.platform_fee_rules   │
         │   • gas.fee_audit            │
         │   • gas.fee_collector_addrs  │
         └──────────────────────────────┘
```

---

## RPC智能选择器

### 实现位置
`backend/src/infrastructure/rpc_selector.rs`

### 核心功能

#### 1. 健康检测与熔断器

```rust
// 自动健康检测（每15秒）
pub struct RpcEndpoint {
    pub id: uuid::Uuid,
    pub chain: String,           // "ethereum", "bsc", "polygon"
    pub url: String,             // RPC 端点 URL
    pub priority: i64,           // 优先级（数字越小越优先）
    pub healthy: bool,           // 当前健康状态
    pub fail_count: i64,         // 连续失败次数
    pub avg_latency_ms: i64,     // 平均延迟（毫秒）
    pub circuit_state: String,   // "closed" | "open" | "half_open"
}

// 熔断器策略
// - fail_count >= 3 → circuit_state = "open" (完全断开)
// - open 状态持续 60 秒后 → "half_open" (尝试恢复)
// - half_open 状态下成功 → "closed" (恢复正常)
```

#### 2. 智能选择算法

```sql
-- 选择 RPC 节点的 SQL 逻辑
SELECT id, chain, url, priority, healthy, circuit_state, avg_latency_ms
FROM admin.rpc_endpoints
WHERE chain = $1                    -- 指定链
  AND healthy = true                -- 健康节点
  AND circuit_state = 'closed'      -- 熔断器关闭
ORDER BY 
  priority ASC,                     -- 按优先级排序
  avg_latency_ms ASC                -- 延迟低的优先
LIMIT 1;
```

#### 3. 数据库表结构

```sql
-- backend/migrations/0007_gas_admin_init.sql
CREATE TABLE IF NOT EXISTS admin.rpc_endpoints (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  chain STRING NOT NULL,
  url STRING NOT NULL,
  priority BIGINT DEFAULT 100,
  healthy BOOLEAN DEFAULT true,
  fail_count BIGINT DEFAULT 0,
  avg_latency_ms BIGINT DEFAULT 0,
  last_latency_ms BIGINT DEFAULT 0,
  circuit_state STRING DEFAULT 'closed',  -- closed | open | half_open
  last_checked_at TIMESTAMP,
  last_fail_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW(),
  CONSTRAINT uq_rpc_endpoint UNIQUE (chain, url)
);

-- 索引
CREATE INDEX idx_rpc_endpoints_chain_health 
ON admin.rpc_endpoints(chain, healthy, priority);

CREATE INDEX idx_rpc_endpoints_chain_circuit 
ON admin.rpc_endpoints(chain, circuit_state);
```

---

## Gas费用估算服务

### 实现位置
`backend/src/service/gas_estimator.rs`

### 核心功能

#### 1. EIP-1559 Gas 估算

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    pub base_fee: String,                  // 基础费用（Wei，十六进制 "0x..."）
    pub max_priority_fee: String,          // 最大优先费用（Wei，十六进制）
    pub max_fee_per_gas: String,           // 最大 Gas 费用（Wei，十六进制）
    pub estimated_time_seconds: u64,       // 预计确认时间（秒）
    pub base_fee_gwei: f64,                // 基础费用（Gwei，便于展示）
    pub max_priority_fee_gwei: f64,        // 优先费用（Gwei）
    pub max_fee_per_gas_gwei: f64,         // 最大费用（Gwei）
}

// API 端点
// POST /api/v1/gas/estimate
{
  "chain": "ethereum",    // "ethereum" | "bsc" | "polygon"
  "speed": "normal"       // "slow" | "normal" | "fast"
}

// 响应
{
  "base_fee": "0x3b9aca00",              // 1 Gwei in Wei
  "max_priority_fee": "0x77359400",      // 2 Gwei in Wei
  "max_fee_per_gas": "0xb2d05e00",       // 3 Gwei in Wei
  "estimated_time_seconds": 180,         // ~3 minutes
  "base_fee_gwei": 1.0,
  "max_priority_fee_gwei": 2.0,
  "max_fee_per_gas_gwei": 3.0
}
```

#### 2. 多链策略配置

```rust
// 不同链的费用策略
struct ChainGasConfig {
    pub priority_multipliers: [f64; 3],    // [slow, normal, fast]
    pub base_fee_multipliers: [f64; 3],
    pub estimated_times: [u64; 3],         // [slow, normal, fast] 秒
}

// Ethereum 配置
ChainGasConfig {
    priority_multipliers: [1.0, 1.5, 2.0],    // 慢速/标准/快速
    base_fee_multipliers: [1.0, 1.2, 1.5],
    estimated_times: [600, 180, 60],           // 10分钟/3分钟/1分钟
}

// BSC 配置（更快）
ChainGasConfig {
    priority_multipliers: [0.8, 1.2, 1.8],
    base_fee_multipliers: [1.0, 1.1, 1.3],
    estimated_times: [300, 90, 30],            // 5分钟/1.5分钟/30秒
}

// Polygon 配置（需要更高优先费）
ChainGasConfig {
    priority_multipliers: [1.0, 1.5, 2.5],
    base_fee_multipliers: [1.0, 1.2, 1.5],
    estimated_times: [180, 60, 20],            // 3分钟/1分钟/20秒
}
```

#### 3. 估算流程

```rust
// 后端内部流程
pub async fn estimate_gas(chain: &str, speed: GasSpeed) -> Result<GasEstimate> {
    // 1. 通过 RpcSelector 选择健康节点
    let endpoint = rpc_selector.select(chain).await?;
    
    // 2. JSON-RPC 请求获取 baseFeePerGas
    let base_fee = fetch_base_fee(&endpoint.url).await?;
    
    // 3. JSON-RPC 请求获取 maxPriorityFeePerGas
    let priority_fee = fetch_priority_fee(&endpoint.url).await?;
    
    // 4. 根据链和速度应用倍数
    let config = ChainGasConfig::for_chain(chain);
    let speed_index = match speed {
        GasSpeed::Slow => 0,
        GasSpeed::Normal => 1,
        GasSpeed::Fast => 2,
    };
    
    let adjusted_base = base_fee * config.base_fee_multipliers[speed_index];
    let adjusted_priority = priority_fee * config.priority_multipliers[speed_index];
    
    // 5. 计算 maxFeePerGas
    let max_fee = adjusted_base + adjusted_priority;
    
    Ok(GasEstimate { ... })
}
```

---

## 平台费用收取系统

### 实现位置
`backend/src/service/fee_service.rs`

### 核心功能

#### 1. 费用规则引擎

```rust
#[derive(Clone, Debug)]
pub struct FeeRule {
    pub id: uuid::Uuid,
    pub chain: String,              // "ethereum", "bsc", "polygon"
    pub operation: String,          // "transfer", "bridge", "swap"
    pub fee_type: String,           // "flat" | "percent" | "mixed"
    pub flat_amount: f64,           // 固定费用（如 0.001 ETH）
    pub percent_bp: i32,            // 百分比费率（基点，10000 = 100%）
    pub min_fee: f64,               // 最低费用
    pub max_fee: Option<f64>,       // 最高费用（可选）
    pub priority: i32,              // 规则优先级
    pub rule_version: i32,          // 版本号
}

// 费用计算结果
pub struct FeeCalcResult {
    pub platform_fee: f64,
    pub collector_address: String,
    pub applied_rule_id: uuid::Uuid,
    pub rule_version: i32,
}
```

#### 2. 费用计算逻辑

```rust
// 三种费用类型

// 1. flat（固定费用）
fee = rule.flat_amount

// 2. percent（百分比费用）
raw_fee = amount * (percent_bp / 10000)
fee = max(raw_fee, min_fee)
if max_fee { fee = min(fee, max_fee) }

// 3. mixed（固定 + 百分比）
percent_part = amount * (percent_bp / 10000)
percent_part = max(percent_part, min_fee)
fee = flat_amount + percent_part
if max_fee { fee = min(fee, max_fee) }

// 示例
// 交易金额: 100 ETH
// 费率: 0.4% (40 基点)
// 最低: 0.001 ETH
// 最高: 1 ETH
// 计算: 100 * 0.004 = 0.4 ETH
// 结果: 0.4 ETH (在最低和最高之间)
```

#### 3. 二级缓存系统

```rust
// L1: 本地内存缓存（60秒TTL）
Arc<RwLock<HashMap<String, CachedRule>>>

// L2: Redis 缓存（60秒TTL）
redis.get("fee:rule:{chain}:{operation}")

// L3: 数据库查询
SELECT * FROM gas.platform_fee_rules
WHERE chain = $1 AND operation = $2 
  AND active = true 
  AND effective_at <= NOW()
ORDER BY priority ASC
LIMIT 1;
```

#### 4. 数据库表结构

```sql
-- 费用规则表
CREATE TABLE IF NOT EXISTS gas.platform_fee_rules (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  chain STRING NOT NULL,
  operation STRING NOT NULL,
  fee_type STRING NOT NULL,        -- flat | percent | mixed
  flat_amount DECIMAL(30,8) DEFAULT 0,
  percent_bp INT DEFAULT 0,        -- 基点（40 = 0.4%）
  min_fee DECIMAL(30,8) DEFAULT 0,
  max_fee DECIMAL(30,8),           -- NULL = 无上限
  priority INT DEFAULT 100,
  active BOOLEAN DEFAULT true,
  effective_at TIMESTAMP DEFAULT NOW(),
  rule_version INT DEFAULT 1,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- 费用收款地址表
CREATE TABLE IF NOT EXISTS gas.fee_collector_addresses (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  chain STRING NOT NULL,
  address STRING NOT NULL,
  active BOOLEAN DEFAULT true,
  rotated_at TIMESTAMP,            -- 地址轮换时间
  created_at TIMESTAMP DEFAULT NOW()
);

-- 费用审计日志
CREATE TABLE IF NOT EXISTS gas.fee_audit (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL,
  chain STRING NOT NULL,
  operation STRING NOT NULL,
  original_amount DECIMAL(30,8) NOT NULL,
  platform_fee DECIMAL(30,8) NOT NULL,
  fee_type STRING NOT NULL,
  applied_rule UUID NOT NULL,
  collector_address STRING NOT NULL,
  wallet_address STRING NOT NULL,
  rule_version INT NOT NULL,
  tx_hash STRING,                  -- 交易哈希（后续回填）
  created_at TIMESTAMP DEFAULT NOW()
);
```

#### 5. API 端点

```rust
// 计算费用
// POST /api/v1/fees/calculate
{
  "chain": "ethereum",
  "operation": "transfer",
  "amount": 100.0               // ETH
}

// 响应
{
  "platform_fee": 0.4,
  "collector_address": "0x123...456",
  "applied_rule_id": "uuid-...",
  "rule_version": 1
}
```

---

## 管理员系统

### 实现位置
`backend/src/api/admin_api.rs`

### 核心功能

#### 1. RPC 端点管理

```rust
// 获取所有 RPC 端点
// GET /api/admin/rpc-endpoints?chain=ethereum

// 添加 RPC 端点
// POST /api/admin/rpc-endpoints
{
  "chain": "ethereum",
  "url": "https://eth-mainnet.g.alchemy.com/v2/...",
  "priority": 100
}

// 更新 RPC 端点状态
// PUT /api/admin/rpc-endpoints/:id
{
  "healthy": false,
  "priority": 200
}

// 删除 RPC 端点
// DELETE /api/admin/rpc-endpoints/:id
```

#### 2. 费用规则管理

```rust
// 获取所有费用规则
// GET /api/admin/fee-rules?chain=ethereum&operation=transfer

// 创建费用规则
// POST /api/admin/fee-rules
{
  "chain": "ethereum",
  "operation": "transfer",
  "fee_type": "percent",
  "percent_bp": 40,           // 0.4%
  "min_fee": 0.001,
  "max_fee": 1.0,
  "priority": 100
}

// 更新费用规则
// PUT /api/admin/fee-rules/:id
{
  "active": false,            // 停用规则
  "percent_bp": 50            // 调整为 0.5%
}

// 删除费用规则
// DELETE /api/admin/fee-rules/:id
```

#### 3. 费用收款地址管理

```rust
// 获取收款地址
// GET /api/admin/fee-collectors?chain=ethereum

// 添加收款地址
// POST /api/admin/fee-collectors
{
  "chain": "ethereum",
  "address": "0x123...456"
}

// 轮换收款地址（旧地址标记为 inactive）
// POST /api/admin/fee-collectors/:id/rotate
{
  "new_address": "0x789...abc"
}
```

#### 4. 审计日志查询

```rust
// 查询费用审计日志
// GET /api/admin/fee-audit?user_id=uuid&chain=ethereum&start_date=2025-01-01

// 响应
[
  {
    "id": "uuid-...",
    "user_id": "uuid-...",
    "chain": "ethereum",
    "operation": "transfer",
    "original_amount": 100.0,
    "platform_fee": 0.4,
    "collector_address": "0x123...456",
    "wallet_address": "0x789...abc",
    "tx_hash": "0xdef...",
    "created_at": "2025-11-25T10:30:00Z"
  }
]
```

#### 5. 操作日志

```sql
-- 管理员操作日志
CREATE TABLE IF NOT EXISTS admin.operation_log (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  admin_id UUID NOT NULL,
  action STRING NOT NULL,          -- FEE_RULE_UPDATE | RPC_ENDPOINT_UPDATE | etc.
  target_id UUID,                  -- 被操作的对象 ID
  old_value JSONB,
  new_value JSONB,
  ip_address STRING,
  user_agent STRING,
  created_at TIMESTAMP DEFAULT NOW()
);
```

---

## 前端集成方案

### 1. Gas 估算集成

```rust
// src/services/gas_estimation.rs
use serde::{Serialize, Deserialize};

pub struct GasEstimationService {
    api_client: Arc<ApiClient>,
}

impl GasEstimationService {
    pub async fn estimate_gas(
        &self,
        chain: &str,
        speed: GasSpeed,
    ) -> Result<GasEstimate, Error> {
        #[derive(Serialize)]
        struct Request {
            chain: String,
            speed: String,
        }
        
        let response = self.api_client
            .post("/api/v1/gas/estimate")
            .json(&Request {
                chain: chain.to_string(),
                speed: match speed {
                    GasSpeed::Slow => "slow",
                    GasSpeed::Normal => "normal",
                    GasSpeed::Fast => "fast",
                }.to_string(),
            })
            .send()
            .await?;
        
        let estimate: GasEstimate = response.json().await?;
        Ok(estimate)
    }
}

// 使用示例
let gas_service = use_context::<GasEstimationService>();
let estimate = gas_service.estimate_gas("ethereum", GasSpeed::Normal).await?;

println!("Base Fee: {} Gwei", estimate.base_fee_gwei);
println!("Priority Fee: {} Gwei", estimate.max_priority_fee_gwei);
println!("Max Fee: {} Gwei", estimate.max_fee_per_gas_gwei);
println!("Estimated Time: {} seconds", estimate.estimated_time_seconds);
```

### 2. 费用计算集成

```rust
// src/services/fee_service.rs
pub struct FeeService {
    api_client: Arc<ApiClient>,
}

impl FeeService {
    pub async fn calculate_platform_fee(
        &self,
        chain: &str,
        operation: &str,
        amount: f64,
    ) -> Result<Option<FeeCalcResult>, Error> {
        #[derive(Serialize)]
        struct Request {
            chain: String,
            operation: String,
            amount: f64,
        }
        
        let response = self.api_client
            .post("/api/v1/fees/calculate")
            .json(&Request {
                chain: chain.to_string(),
                operation: operation.to_string(),
                amount,
            })
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: FeeCalcResult = response.json().await?;
            Ok(Some(result))
        } else {
            Ok(None) // 无适用规则
        }
    }
}

// 使用示例
let fee_service = use_context::<FeeService>();
if let Some(fee) = fee_service
    .calculate_platform_fee("ethereum", "transfer", 100.0)
    .await? 
{
    println!("Platform Fee: {} ETH", fee.platform_fee);
    println!("Collector Address: {}", fee.collector_address);
}
```

### 3. 发送交易完整流程

```rust
// src/pages/send_transaction.rs
pub async fn submit_transaction(
    wallet_id: &str,
    to_address: &str,
    amount: f64,
    chain: &str,
) -> Result<String, Error> {
    let gas_service = use_context::<GasEstimationService>();
    let fee_service = use_context::<FeeService>();
    let tx_service = use_context::<TransactionService>();
    
    // 1. 估算 Gas 费用
    let gas_estimate = gas_service
        .estimate_gas(chain, GasSpeed::Normal)
        .await?;
    
    // 2. 计算平台服务费
    let platform_fee = fee_service
        .calculate_platform_fee(chain, "transfer", amount)
        .await?;
    
    // 3. 显示总计费用给用户确认
    let total_gas_eth = gas_estimate.max_fee_per_gas_gwei * 21000.0 / 1e9;
    let total_fee = platform_fee.as_ref().map(|f| f.platform_fee).unwrap_or(0.0);
    let total_cost = amount + total_gas_eth + total_fee;
    
    println!("Send Amount: {} ETH", amount);
    println!("Gas Fee: {} ETH", total_gas_eth);
    println!("Platform Fee: {} ETH", total_fee);
    println!("Total: {} ETH", total_cost);
    
    // 4. 用户确认后，构造交易
    let unsigned_tx = UnsignedTransaction {
        from: wallet_address,
        to: to_address.to_string(),
        value: ethers::utils::parse_ether(amount)?,
        gas_limit: 21000,
        max_fee_per_gas: ethers::utils::parse_units(
            gas_estimate.max_fee_per_gas_gwei, 
            "gwei"
        )?,
        max_priority_fee_per_gas: ethers::utils::parse_units(
            gas_estimate.max_priority_fee_gwei, 
            "gwei"
        )?,
        nonce: get_nonce(wallet_address).await?,
        chain_id: get_chain_id(chain),
        data: vec![],
    };
    
    // 5. 签名交易（客户端）
    let signed_tx = sign_transaction(wallet_id, unsigned_tx).await?;
    
    // 6. 广播交易
    let tx_hash = tx_service.broadcast(chain, &signed_tx).await?;
    
    // 7. 记录费用审计（后端自动处理）
    // 后端在接收到交易广播请求时会自动调用 FeeService::record_fee_audit
    
    Ok(tx_hash)
}
```

### 4. UI 组件更新

```rust
// src/components/gas_estimate_card.rs
#[component]
pub fn GasEstimateCard(chain: String, speed: Signal<GasSpeed>) -> Element {
    let gas_service = use_context::<GasEstimationService>();
    
    // 实时估算（当速度改变时）
    let estimate = use_resource(move || {
        let chain_clone = chain.clone();
        let speed_val = *speed.read();
        async move {
            gas_service.estimate_gas(&chain_clone, speed_val).await.ok()
        }
    });
    
    rsx! {
        div { class: "gas-estimate-card",
            match estimate.read().as_ref() {
                Some(Some(est)) => rsx! {
                    div { class: "gas-info",
                        div { "🔥 Gas 费用" }
                        div { class: "gas-amount",
                            "~{est.max_fee_per_gas_gwei:.2} Gwei"
                        }
                        div { class: "gas-details",
                            "Base: {est.base_fee_gwei:.2} + Priority: {est.max_priority_fee_gwei:.2}"
                        }
                        div { class: "estimated-time",
                            "预计 {est.estimated_time_seconds} 秒确认"
                        }
                    }
                },
                _ => rsx! {
                    LoadingSpinner { message: "正在估算 Gas..." }
                }
            }
        }
    }
}
```

---

## 总结

### ✅ 已实现功能

| 模块 | 后端实现 | 数据库表 | API端点 |
|------|---------|---------|--------|
| RPC智能选择器 | ✅ | admin.rpc_endpoints | /api/admin/rpc-endpoints |
| Gas费用估算 | ✅ | - | /api/v1/gas/estimate |
| 平台费用收取 | ✅ | gas.platform_fee_rules | /api/v1/fees/calculate |
| 费用审计日志 | ✅ | gas.fee_audit | /api/admin/fee-audit |
| 管理员系统 | ✅ | admin.operation_log | /api/admin/* |

### 🔧 配置说明

```toml
# backend/config.toml

[server]
enable_fee_system = true  # 启用费用收取系统

[fees]
bridge_fee_percentage = 0.004        # 桥接费用 0.4%
transaction_fee_percentage = 0.002   # 交易费用 0.2%

[database]
url = "postgresql://root@localhost:26257/ironcore"

[redis]
url = "redis://localhost:6379"
```

### 📊 监控指标

```rust
// backend/src/metrics.rs
pub struct Metrics {
    pub fee_calculation_total: u64,      // 费用计算次数
    pub fee_audit_write_fail: u64,       // 审计写入失败次数
    pub fee_total_amount: f64,           // 累计费用金额
    pub rpc_selector_cache_hit: u64,     // RPC选择器缓存命中
    pub rpc_selector_cache_miss: u64,    // RPC选择器缓存未命中
}
```

### 🔐 安全考虑

1. **费用规则版本化**: 所有规则带版本号，审计日志记录使用的版本
2. **地址轮换**: 支持收款地址定期轮换，降低风险
3. **操作日志**: 所有管理员操作都有完整日志
4. **缓存失效**: 规则更新后自动清除缓存
5. **熔断保护**: RPC节点故障自动熔断，避免雪崩

**状态**: ✅ 所有后端服务已完成生产级实现，前端只需调用API
