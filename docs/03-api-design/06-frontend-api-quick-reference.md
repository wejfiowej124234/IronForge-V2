# 前端API集成快速参考

> **版本**: V2.0  
> **更新日期**: 2025-11-25  
> **后端地址**: `http://localhost:8088` (开发环境)  
> **文档**: 完整集成指南见 `05-backend-services-integration.md`

---

## 📋 核心API端点

### 1. Gas费用估算

**无硬编码，后端实时查询链上数据**

```bash
# 请求
POST /api/v1/gas/estimate
Content-Type: application/json

{
  "chain": "ethereum",     # "ethereum" | "bsc" | "polygon"
  "speed": "normal"        # "slow" | "normal" | "fast"
}

# 响应
{
  "base_fee": "0x3b9aca00",              # Wei (十六进制)
  "max_priority_fee": "0x77359400",      # Wei (十六进制)
  "max_fee_per_gas": "0xb2d05e00",       # Wei (十六进制)
  "estimated_time_seconds": 180,         # 预计确认时间（秒）
  "base_fee_gwei": 1.0,                  # Gwei（展示用）
  "max_priority_fee_gwei": 2.0,
  "max_fee_per_gas_gwei": 3.0
}
```

**后端实现**:
- 通过 `RpcSelector` 智能选择健康的RPC节点
- 查询链上最新 `baseFeePerGas`
- 查询推荐的 `maxPriorityFeePerGas`
- 根据速度和链类型应用倍数策略

**前端代码**:

```rust
// src/services/gas_service.rs
pub struct GasService {
    api_client: Arc<ApiClient>,
}

impl GasService {
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
        
        response.json().await
    }
}

// 使用
let gas_service = use_context::<GasService>();
let estimate = gas_service.estimate_gas("ethereum", GasSpeed::Normal).await?;
println!("Gas: {} Gwei", estimate.max_fee_per_gas_gwei);
```

---

### 2. 平台服务费计算

**后端配置规则引擎，支持固定/百分比/混合费用**

```bash
# 请求
POST /api/v1/fees/calculate
Content-Type: application/json

{
  "chain": "ethereum",
  "operation": "transfer",    # "transfer" | "bridge" | "swap"
  "amount": 100.0             # 交易金额（ETH）
}

# 响应
{
  "platform_fee": 0.4,                     # 平台费用（ETH）
  "collector_address": "0x123...456",      # 收款地址
  "applied_rule_id": "uuid-...",           # 应用的规则ID
  "rule_version": 1                        # 规则版本号
}

# 无适用规则时返回 404
```

**后端实现**:
- 二级缓存（内存 + Redis，60秒TTL）
- 支持三种费用类型：
  - `flat`: 固定费用（如 0.001 ETH）
  - `percent`: 百分比费用（如 0.4% = 40基点）
  - `mixed`: 固定 + 百分比
- 自动记录审计日志

**前端代码**:

```rust
// src/services/fee_service.rs
pub struct FeeService {
    api_client: Arc<ApiClient>,
}

impl FeeService {
    pub async fn calculate_fee(
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
            Ok(Some(response.json().await?))
        } else {
            Ok(None) // 无适用规则
        }
    }
}

// 使用
let fee_service = use_context::<FeeService>();
if let Some(fee) = fee_service.calculate_fee("ethereum", "transfer", 100.0).await? {
    println!("Platform Fee: {} ETH", fee.platform_fee);
    println!("Collector: {}", fee.collector_address);
}
```

---

### 3. 代币价格查询

```bash
# 批量查询价格
POST /api/v1/prices/batch
Content-Type: application/json

{
  "symbols": ["ETH", "BTC", "SOL", "TON", "USDT"]
}

# 响应
{
  "ETH": {
    "price_usd": 3500.25,
    "change_24h": 2.5,
    "updated_at": 1732521000
  },
  "BTC": {
    "price_usd": 65000.50,
    "change_24h": -1.2,
    "updated_at": 1732521000
  },
  ...
}
```

**前端代码**:

```rust
pub async fn fetch_prices(symbols: &[String]) -> Result<HashMap<String, PriceData>> {
    let response = api_client
        .post("/api/v1/prices/batch")
        .json(&json!({ "symbols": symbols }))
        .send()
        .await?;
    
    response.json().await
}
```

---

### 4. 钱包余额查询

```bash
# 查询多链余额
POST /api/v1/wallets/balances
Content-Type: application/json
Authorization: Bearer {jwt_token}

{
  "addresses": {
    "ethereum": "0x123...456",
    "bsc": "0x123...456",
    "polygon": "0x123...456",
    "solana": "ABC...XYZ",
    "bitcoin": "bc1q...",
    "ton": "EQ..."
  }
}

# 响应
{
  "ethereum": {
    "native": {
      "symbol": "ETH",
      "balance": "1.5",
      "balance_usd": 5250.38
    },
    "tokens": [
      {
        "address": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        "symbol": "USDT",
        "name": "Tether USD",
        "balance": "1000.0",
        "balance_usd": 1000.0
      }
    ]
  },
  ...
}
```

---

## 🔧 完整发送交易流程

```rust
// src/pages/send_transaction.rs
pub async fn send_transaction_flow(
    wallet_id: &str,
    to_address: &str,
    amount: f64,
    chain: &str,
) -> Result<String, Error> {
    let gas_service = use_context::<GasService>();
    let fee_service = use_context::<FeeService>();
    
    // === 步骤 1: 估算 Gas 费用（后端API） ===
    let gas_estimate = gas_service
        .estimate_gas(chain, GasSpeed::Normal)
        .await?;
    
    println!("Gas Estimate:");
    println!("  Base Fee: {} Gwei", gas_estimate.base_fee_gwei);
    println!("  Priority Fee: {} Gwei", gas_estimate.max_priority_fee_gwei);
    println!("  Max Fee: {} Gwei", gas_estimate.max_fee_per_gas_gwei);
    println!("  Estimated Time: {}s", gas_estimate.estimated_time_seconds);
    
    // === 步骤 2: 计算平台服务费（后端API） ===
    let platform_fee = fee_service
        .calculate_fee(chain, "transfer", amount)
        .await?;
    
    if let Some(fee) = &platform_fee {
        println!("Platform Fee: {} ETH", fee.platform_fee);
        println!("Collector: {}", fee.collector_address);
    }
    
    // === 步骤 3: 计算总费用 ===
    let gas_limit = 21000u64;  // 标准转账
    let total_gas_wei = gas_limit as f64 * gas_estimate.max_fee_per_gas_gwei * 1e9;
    let total_gas_eth = total_gas_wei / 1e18;
    let service_fee_eth = platform_fee.as_ref().map(|f| f.platform_fee).unwrap_or(0.0);
    let total_cost = amount + total_gas_eth + service_fee_eth;
    
    println!("\n=== 交易总计 ===");
    println!("发送金额: {} ETH", amount);
    println!("Gas 费用: {} ETH", total_gas_eth);
    println!("服务费用: {} ETH", service_fee_eth);
    println!("总计: {} ETH", total_cost);
    
    // === 步骤 4: 用户确认（UI显示） ===
    if !confirm_transaction(&format!(
        "确认发送 {} ETH 到 {}？\n总费用: {} ETH (含 Gas {} ETH + 服务费 {} ETH)",
        amount, to_address, total_cost, total_gas_eth, service_fee_eth
    )) {
        return Err(Error::UserCancelled);
    }
    
    // === 步骤 5: 构造交易 ===
    let unsigned_tx = UnsignedTransaction {
        from: get_wallet_address(wallet_id)?,
        to: to_address.to_string(),
        value: ethers::utils::parse_ether(amount)?,
        gas_limit,
        max_fee_per_gas: ethers::utils::parse_units(
            gas_estimate.max_fee_per_gas_gwei,
            "gwei"
        )?,
        max_priority_fee_per_gas: ethers::utils::parse_units(
            gas_estimate.max_priority_fee_gwei,
            "gwei"
        )?,
        nonce: fetch_nonce(chain, get_wallet_address(wallet_id)?).await?,
        chain_id: get_chain_id(chain),
        data: vec![],
    };
    
    // === 步骤 6: 客户端签名 ===
    let password = prompt_password("请输入钱包密码：")?;
    let signed_tx = sign_transaction_local(wallet_id, &password, unsigned_tx).await?;
    
    // === 步骤 7: 广播交易 ===
    let tx_hash = broadcast_transaction(chain, &signed_tx).await?;
    
    println!("✅ 交易已提交: {}", tx_hash);
    
    // 后端会自动记录费用审计日志
    Ok(tx_hash)
}
```

---

## 🎨 UI组件示例

### Gas费用卡片

```rust
// src/components/gas_estimate_card.rs
#[component]
pub fn GasEstimateCard(
    chain: String,
    speed: Signal<GasSpeed>,
) -> Element {
    let gas_service = use_context::<GasService>();
    
    // 实时估算
    let estimate = use_resource(move || {
        let chain_clone = chain.clone();
        let speed_val = *speed.read();
        async move {
            gas_service.estimate_gas(&chain_clone, speed_val).await.ok()
        }
    });
    
    rsx! {
        div { class: "gas-card",
            match estimate.read().as_ref() {
                Some(Some(est)) => rsx! {
                    div { class: "gas-info",
                        div { class: "header",
                            span { "🔥 Gas 费用" }
                            button {
                                onclick: move |_| { estimate.restart(); },
                                "🔄"
                            }
                        }
                        
                        div { class: "amount",
                            "{est.max_fee_per_gas_gwei:.2} Gwei"
                        }
                        
                        div { class: "breakdown",
                            "Base: {est.base_fee_gwei:.2} + Priority: {est.max_priority_fee_gwei:.2}"
                        }
                        
                        div { class: "time",
                            "⏱ 预计 {est.estimated_time_seconds}秒 确认"
                        }
                        
                        // 速度选择器
                        SpeedSelector {
                            selected: speed,
                            on_change: move |s| { speed.set(s); }
                        }
                    }
                },
                Some(None) => rsx! {
                    div { class: "error", "Gas估算失败" }
                },
                None => rsx! {
                    LoadingSpinner { message: "正在估算..." }
                }
            }
        }
    }
}
```

---

## 🔐 认证

所有需要认证的API端点都需要JWT Token：

```rust
// 登录
POST /api/auth/login
{
  "email": "user@example.com",
  "password": "password123"
}

// 响应
{
  "jwt_token": "eyJ...",
  "refresh_token": "...",
  "expires_in": 3600
}

// 使用Token
let response = api_client
    .post("/api/v1/wallets/balances")
    .header("Authorization", format!("Bearer {}", jwt_token))
    .json(&request)
    .send()
    .await?;
```

---

## 🚀 环境配置

### 开发环境

```bash
# .env.development
API_BASE_URL=http://localhost:8088
WS_BASE_URL=ws://localhost:8088
ENABLE_DEBUG=true
```

### 生产环境

```bash
# .env.production
API_BASE_URL=https://api.ironforge.com
WS_BASE_URL=wss://api.ironforge.com
ENABLE_DEBUG=false
```

### 前端配置

```rust
// src/config.rs
pub struct ApiConfig {
    pub base_url: String,
    pub timeout_secs: u64,
}

impl ApiConfig {
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("API_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8088".to_string()),
            timeout_secs: std::env::var("API_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        }
    }
}
```

---

## 📊 错误处理

### 标准错误响应

```json
{
  "error": {
    "code": "GAS_ESTIMATION_FAILED",
    "message": "Failed to fetch base fee from RPC",
    "details": {
      "chain": "ethereum",
      "rpc_endpoint": "https://eth-mainnet.g.alchemy.com/...",
      "underlying_error": "connection timeout"
    }
  }
}
```

### 常见错误码

| 错误码 | 说明 | 解决方案 |
|-------|------|---------|
| `INVALID_CHAIN` | 不支持的链 | 检查chain参数 |
| `RPC_UNAVAILABLE` | RPC节点不可用 | 后端会自动重试其他节点 |
| `GAS_ESTIMATION_FAILED` | Gas估算失败 | 检查交易参数 |
| `NO_FEE_RULE` | 无适用费用规则 | 联系管理员配置规则 |
| `INSUFFICIENT_BALANCE` | 余额不足 | 提示用户充值 |

---

## 🔍 调试技巧

```rust
// 启用详细日志
RUST_LOG=debug cargo run

// 在前端代码中
tracing::debug!("Gas estimate: {:?}", estimate);
tracing::info!("Transaction hash: {}", tx_hash);
tracing::error!("Failed to fetch gas: {}", error);
```

---

## 📚 相关文档

- 完整后端服务集成指南：`03-api-design/05-backend-services-integration.md`
- 发送交易UI设计：`05-ui-ux/04-send-transaction-ui.md`
- 仪表盘设计：`05-ui-ux/03-dashboard-and-portfolio.md`
- 后端API参考：`backend/README.md`

**状态**: ✅ 所有API端点已实现并测试
