# 生产级日志系统

> **状态**: ✅ 生产就绪  
> **版本**: V2.0  
> **更新日期**: 2025-11-25

---

## 📋 目录

1. [日志框架](#日志框架)
2. [日志级别](#日志级别)
3. [结构化日志](#结构化日志)
4. [PII 过滤](#pii-过滤)
5. [日志聚合](#日志聚合)
6. [性能优化](#性能优化)

---

## 🎯 日志框架

### Tracing 设置

```rust
// src/logging/mod.rs
use tracing::{info, warn, error, debug, trace};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Registry,
};
use tracing_appender::{non_blocking, rolling};

/// 初始化日志系统
pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))
        .unwrap();
    
    let registry = Registry::default().with(filter);
    
    // 控制台输出
    let console_layer = if config.console {
        Some(fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_level(true)
            .with_ansi(config.format == "pretty"))
    } else {
        None
    };
    
    // 文件输出
    let file_layer = if config.file {
        let file_appender = rolling::daily(&config.file_path, "ironforge.log");
        let (non_blocking, _guard) = non_blocking(file_appender);
        
        Some(fmt::layer()
            .json()
            .with_writer(non_blocking)
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_file(true))
    } else {
        None
    };
    
    // 组合所有层
    registry
        .with(console_layer)
        .with(file_layer)
        .init();
    
    info!("Logging initialized with level: {}", config.level);
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub console: bool,
    pub file: bool,
    pub file_path: String,
    pub filter_pii: bool,
}
```

---

## 📊 日志级别

### 级别定义

```rust
/// 日志级别使用指南
/// 
/// TRACE: 非常详细的调试信息（通常在生产环境禁用）
///   - 每次函数调用
///   - 循环迭代
///   - 数据转换步骤
/// 
/// DEBUG: 调试信息（生产环境禁用或最小化）
///   - 方法进入/退出
///   - 中间计算结果
///   - 条件分支判断
/// 
/// INFO: 正常操作信息（生产环境默认）
///   - 应用启动/关闭
///   - 用户操作（登录、创建钱包等）
///   - 重要状态变更
/// 
/// WARN: 警告信息
///   - 使用了已弃用的功能
///   - 资源即将耗尽
///   - 重试操作
///   - 配置问题
/// 
/// ERROR: 错误信息
///   - 操作失败
///   - 异常捕获
///   - 资源不可用

// 使用示例
pub async fn create_wallet(name: &str, mnemonic: &str) -> Result<Wallet> {
    info!(wallet_name = %name, "Creating new wallet");
    
    // 验证助记词
    debug!("Validating mnemonic phrase");
    let mnemonic = match Mnemonic::from_phrase(mnemonic) {
        Ok(m) => {
            trace!("Mnemonic validation successful");
            m
        }
        Err(e) => {
            error!(error = %e, "Invalid mnemonic phrase");
            return Err(WalletError::InvalidMnemonic.into());
        }
    };
    
    // 派生密钥
    debug!("Deriving wallet keys");
    let wallet = derive_wallet(&mnemonic, name)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to derive wallet keys");
            e
        })?;
    
    info!(wallet_id = %wallet.id, address = %wallet.address, "Wallet created successfully");
    Ok(wallet)
}
```

---

## 📝 结构化日志

### 使用结构化字段

```rust
use tracing::{info, error, Span};
use serde::Serialize;

/// 钱包操作日志
#[derive(Serialize)]
struct WalletLog {
    wallet_id: String,
    wallet_name: String,
    address: String,
    chain: String,
    operation: String,
    timestamp: i64,
}

impl WalletLog {
    pub fn log(&self) {
        info!(
            wallet_id = %self.wallet_id,
            wallet_name = %self.wallet_name,
            address = %self.address,
            chain = %self.chain,
            operation = %self.operation,
            timestamp = self.timestamp,
            "Wallet operation"
        );
    }
}

/// 交易日志
#[derive(Serialize)]
struct TransactionLog {
    tx_hash: String,
    from: String,
    to: String,
    value: String,
    chain_id: u64,
    status: String,
    gas_used: Option<u64>,
    timestamp: i64,
}

impl TransactionLog {
    pub fn log(&self) {
        info!(
            tx_hash = %self.tx_hash,
            from = %self.from,
            to = %self.to,
            value = %self.value,
            chain_id = self.chain_id,
            status = %self.status,
            gas_used = ?self.gas_used,
            timestamp = self.timestamp,
            "Transaction processed"
        );
    }
}

/// API 请求日志
pub fn log_api_request(
    method: &str,
    path: &str,
    status: u16,
    duration_ms: u64,
) {
    info!(
        http.method = %method,
        http.path = %path,
        http.status_code = status,
        duration_ms = duration_ms,
        "API request"
    );
}
```

### Span 追踪

```rust
use tracing::{instrument, Span};

/// 自动追踪函数执行
#[instrument(
    name = "send_transaction",
    skip(wallet, tx),
    fields(
        wallet_id = %wallet.id,
        chain = %tx.chain,
        to = %tx.to,
        value = %tx.value
    )
)]
pub async fn send_transaction(
    wallet: &Wallet,
    tx: Transaction,
) -> Result<String> {
    // 估算 Gas
    let gas_estimate = {
        let span = Span::current();
        span.record("step", &"estimate_gas");
        
        estimate_gas(&tx).await?
    };
    
    // 签名交易
    let signed_tx = {
        let span = Span::current();
        span.record("step", &"sign_transaction");
        
        sign_transaction(wallet, &tx, &gas_estimate).await?
    };
    
    // 广播交易
    let tx_hash = {
        let span = Span::current();
        span.record("step", &"broadcast_transaction");
        
        broadcast_transaction(&signed_tx).await?
    };
    
    Span::current().record("tx_hash", &tx_hash.as_str());
    info!("Transaction sent successfully");
    
    Ok(tx_hash)
}
```

---

## 🔒 PII 过滤

### 敏感数据过滤

```rust
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // 邮箱正则
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"
    ).unwrap();
    
    // 手机号正则（国际格式）
    static ref PHONE_REGEX: Regex = Regex::new(
        r"\+?[1-9]\d{1,14}"
    ).unwrap();
    
    // 以太坊地址正则
    static ref ETH_ADDRESS_REGEX: Regex = Regex::new(
        r"0x[a-fA-F0-9]{40}"
    ).unwrap();
    
    // 私钥正则（64字符十六进制）
    static ref PRIVATE_KEY_REGEX: Regex = Regex::new(
        r"(?i)[a-f0-9]{64}"
    ).unwrap();
}

/// PII 过滤器
pub struct PiiFilter;

impl PiiFilter {
    /// 脱敏邮箱
    pub fn mask_email(email: &str) -> String {
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
    
    /// 脱敏手机号
    pub fn mask_phone(phone: &str) -> String {
        let len = phone.len();
        if len > 4 {
            format!("{}****{}", &phone[..2], &phone[len-2..])
        } else {
            "****".to_string()
        }
    }
    
    /// 脱敏地址
    pub fn mask_address(address: &str) -> String {
        if address.len() > 10 {
            format!("{}...{}", &address[..6], &address[address.len()-4..])
        } else {
            address.to_string()
        }
    }
    
    /// 完全隐藏私钥
    pub fn mask_private_key(_key: &str) -> String {
        "[REDACTED]".to_string()
    }
    
    /// 过滤日志消息中的敏感信息
    pub fn filter_message(message: &str) -> String {
        let mut filtered = message.to_string();
        
        // 替换邮箱
        filtered = EMAIL_REGEX.replace_all(&filtered, |caps: &regex::Captures| {
            Self::mask_email(&caps[0])
        }).to_string();
        
        // 替换手机号
        filtered = PHONE_REGEX.replace_all(&filtered, |caps: &regex::Captures| {
            Self::mask_phone(&caps[0])
        }).to_string();
        
        // 替换以太坊地址
        filtered = ETH_ADDRESS_REGEX.replace_all(&filtered, |caps: &regex::Captures| {
            Self::mask_address(&caps[0])
        }).to_string();
        
        // 替换私钥
        filtered = PRIVATE_KEY_REGEX.replace_all(&filtered, |_caps: &regex::Captures| {
            "[REDACTED]"
        }).to_string();
        
        filtered
    }
}

/// 自定义日志格式化器（带 PII 过滤）
pub struct PiiFilteringFormatter;

impl<S, N> tracing_subscriber::fmt::FormatEvent<S, N> for PiiFilteringFormatter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        // 获取原始消息
        let mut message = String::new();
        event.record(&mut MessageVisitor(&mut message));
        
        // 过滤敏感信息
        let filtered = PiiFilter::filter_message(&message);
        
        // 写入过滤后的消息
        write!(writer, "{}", filtered)
    }
}

struct MessageVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            *self.0 = format!("{:?}", value);
        }
    }
}
```

### 使用示例

```rust
// ❌ 错误：直接记录敏感信息
error!("Login failed for user@example.com");
error!("Private key: {}", private_key);

// ✅ 正确：记录前先脱敏
let masked_email = PiiFilter::mask_email("user@example.com");
error!(email = %masked_email, "Login failed");

// ✅ 更好：使用结构化字段
error!(
    user_id = %user.id,  // 使用 ID 而非邮箱
    "Login failed"
);

// 私钥永不记录
// error!("Private key: [REDACTED]");
```

---

## 📦 日志聚合

### Fluentd 集成

```toml
# fluent.conf
<source>
  @type tail
  path /var/log/ironforge/*.log
  pos_file /var/log/td-agent/ironforge.log.pos
  tag ironforge.logs
  <parse>
    @type json
    time_key timestamp
    time_format %Y-%m-%dT%H:%M:%S.%NZ
  </parse>
</source>

<filter ironforge.logs>
  @type record_transformer
  <record>
    hostname "#{Socket.gethostname}"
    environment "#{ENV['APP_ENV']}"
  </record>
</filter>

<match ironforge.logs>
  @type elasticsearch
  host elasticsearch.example.com
  port 9200
  logstash_format true
  logstash_prefix ironforge
  <buffer>
    @type file
    path /var/log/td-agent/buffer/ironforge
    flush_interval 10s
  </buffer>
</match>
```

### ELK Stack 查询

```json
// Elasticsearch 查询示例

// 1. 查询最近 1 小时的错误日志
{
  "query": {
    "bool": {
      "must": [
        { "match": { "level": "ERROR" }},
        { "range": { "timestamp": { "gte": "now-1h" }}}
      ]
    }
  }
}

// 2. 按操作类型聚合
{
  "aggs": {
    "by_operation": {
      "terms": { "field": "operation.keyword" }
    }
  }
}

// 3. 查询特定钱包的操作
{
  "query": {
    "term": { "wallet_id": "wallet-123" }
  }
}
```

---

## ⚡ 性能优化

### 异步日志

```rust
use tracing_appender::non_blocking;

pub fn init_async_logging() {
    let file_appender = rolling::daily("./logs", "ironforge.log");
    let (non_blocking, _guard) = non_blocking(file_appender);
    
    tracing_subscriber::fmt()
        .json()
        .with_writer(non_blocking)
        .init();
    
    // _guard 必须保存，否则日志会丢失
    std::mem::forget(_guard);
}
```

### 采样日志

```rust
use tracing::Level;

/// 高频操作采样记录
pub struct SamplingFilter {
    sample_rate: f64,
}

impl SamplingFilter {
    /// 决定是否记录此日志
    pub fn should_log(&self, level: &Level) -> bool {
        match *level {
            Level::ERROR | Level::WARN => true,  // 错误和警告总是记录
            Level::INFO => {
                // INFO 级别按采样率记录
                rand::random::<f64>() < self.sample_rate
            }
            Level::DEBUG | Level::TRACE => {
                // DEBUG/TRACE 仅在开发环境记录
                cfg!(debug_assertions)
            }
        }
    }
}

// 使用示例：高频 API 调用
pub fn log_api_call(path: &str, status: u16) {
    static SAMPLER: Lazy<SamplingFilter> = Lazy::new(|| {
        SamplingFilter { sample_rate: 0.1 }  // 10% 采样率
    });
    
    if SAMPLER.should_log(&Level::INFO) {
        info!(http.path = %path, http.status = status, "API call");
    }
}
```

### 日志轮转

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};

/// 配置日志轮转
pub fn configure_log_rotation(config: &LoggingConfig) -> RollingFileAppender {
    match config.rotation_policy.as_str() {
        "hourly" => rolling::hourly(&config.file_path, "ironforge"),
        "daily" => rolling::daily(&config.file_path, "ironforge"),
        "never" => rolling::never(&config.file_path, "ironforge.log"),
        _ => rolling::daily(&config.file_path, "ironforge"),
    }
}

/// 清理旧日志
pub async fn cleanup_old_logs(log_dir: &Path, max_age_days: u64) -> Result<()> {
    let cutoff = SystemTime::now() - Duration::from_secs(max_age_days * 24 * 3600);
    
    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        
        if let Ok(modified) = metadata.modified() {
            if modified < cutoff {
                fs::remove_file(entry.path())?;
                info!("Deleted old log file: {:?}", entry.path());
            }
        }
    }
    
    Ok(())
}
```

---

## 📚 依赖项

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-appender = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
lazy_static = "1.4"
```

---

## 🚨 生产环境检查清单

- [ ] 日志级别设置为 `warn` 或 `error`
- [ ] 启用 PII 过滤
- [ ] 配置日志轮转（每日或每小时）
- [ ] 设置日志保留期（例如 30 天）
- [ ] 启用结构化日志（JSON 格式）
- [ ] 配置日志聚合（Fluentd/Logstash）
- [ ] 设置日志告警（错误率、磁盘空间）
- [ ] 测试日志查询性能
- [ ] 验证敏感信息过滤
- [ ] 配置异步日志写入

---

## 🔗 相关文档

- [错误处理](./02-error-handling-system.md)
- [监控配置](./04-monitoring-setup.md)
- [安全架构](../04-security/03-security-architecture.md)
