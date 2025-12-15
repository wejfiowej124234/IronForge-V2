# 生产级监控配置

> **状态**: ✅ 生产就绪  
> **版本**: V2.0  
> **更新日期**: 2025-11-25

---

## 📋 目录

1. [Prometheus Metrics](#prometheus-metrics)
2. [健康检查](#健康检查)
3. [性能监控](#性能监控)
4. [告警规则](#告警规则)
5. [Grafana 仪表板](#grafana-仪表板)

---

## 📊 Prometheus Metrics

### Metrics 导出器

```rust
// src/monitoring/metrics.rs
use prometheus::{
    IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
    Histogram, HistogramVec, Registry, Encoder, TextEncoder,
};
use lazy_static::lazy_static;

lazy_static! {
    /// Prometheus Registry
    pub static ref REGISTRY: Registry = Registry::new();
    
    /// HTTP 请求总数
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = IntCounterVec::new(
        prometheus::opts!("http_requests_total", "Total HTTP requests"),
        &["method", "path", "status"]
    ).unwrap();
    
    /// HTTP 请求延迟（秒）
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = HistogramVec::new(
        prometheus::histogram_opts!(
            "http_request_duration_seconds",
            "HTTP request latency in seconds",
            vec![0.001, 0.01, 0.1, 0.5, 1.0, 2.5, 5.0, 10.0]
        ),
        &["method", "path"]
    ).unwrap();
    
    /// 钱包操作计数
    pub static ref WALLET_OPERATIONS: IntCounterVec = IntCounterVec::new(
        prometheus::opts!("wallet_operations_total", "Total wallet operations"),
        &["operation", "chain", "status"]
    ).unwrap();
    
    /// 交易计数
    pub static ref TRANSACTIONS: IntCounterVec = IntCounterVec::new(
        prometheus::opts!("transactions_total", "Total transactions"),
        &["chain", "status"]
    ).unwrap();
    
    /// 交易金额（美元）
    pub static ref TRANSACTION_AMOUNT: Histogram = Histogram::with_opts(
        prometheus::histogram_opts!(
            "transaction_amount_usd",
            "Transaction amount in USD",
            vec![1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0]
        )
    ).unwrap();
    
    /// 活跃用户数
    pub static ref ACTIVE_USERS: IntGauge = IntGauge::new(
        "active_users_total",
        "Number of active users"
    ).unwrap();
    
    /// 活跃钱包数
    pub static ref ACTIVE_WALLETS: IntGaugeVec = IntGaugeVec::new(
        prometheus::opts!("active_wallets_total", "Number of active wallets"),
        &["chain"]
    ).unwrap();
    
    /// RPC 调用计数
    pub static ref RPC_CALLS: IntCounterVec = IntCounterVec::new(
        prometheus::opts!("rpc_calls_total", "Total RPC calls"),
        &["chain", "method", "status"]
    ).unwrap();
    
    /// RPC 调用延迟
    pub static ref RPC_CALL_DURATION: HistogramVec = HistogramVec::new(
        prometheus::histogram_opts!(
            "rpc_call_duration_seconds",
            "RPC call latency in seconds",
            vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0]
        ),
        &["chain", "method"]
    ).unwrap();
    
    /// 错误计数
    pub static ref ERRORS: IntCounterVec = IntCounterVec::new(
        prometheus::opts!("errors_total", "Total errors"),
        &["error_type", "severity"]
    ).unwrap();
    
    /// IndexedDB 操作计数
    pub static ref INDEXEDDB_OPERATIONS: IntCounterVec = IntCounterVec::new(
        prometheus::opts!("indexeddb_operations_total", "Total IndexedDB operations"),
        &["operation", "status"]
    ).unwrap();
    
    /// 缓存命中率
    pub static ref CACHE_HITS: IntCounter = IntCounter::new(
        "cache_hits_total",
        "Total cache hits"
    ).unwrap();
    
    pub static ref CACHE_MISSES: IntCounter = IntCounter::new(
        "cache_misses_total",
        "Total cache misses"
    ).unwrap();
}

/// 初始化 Metrics
pub fn init_metrics() {
    REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(HTTP_REQUEST_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(WALLET_OPERATIONS.clone())).unwrap();
    REGISTRY.register(Box::new(TRANSACTIONS.clone())).unwrap();
    REGISTRY.register(Box::new(TRANSACTION_AMOUNT.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_USERS.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_WALLETS.clone())).unwrap();
    REGISTRY.register(Box::new(RPC_CALLS.clone())).unwrap();
    REGISTRY.register(Box::new(RPC_CALL_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(ERRORS.clone())).unwrap();
    REGISTRY.register(Box::new(INDEXEDDB_OPERATIONS.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_HITS.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_MISSES.clone())).unwrap();
}

/// 导出 Metrics（Prometheus 格式）
pub fn export_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

### Metrics 使用示例

```rust
use std::time::Instant;

/// 记录 HTTP 请求
pub async fn handle_request(method: &str, path: &str) -> Result<Response> {
    let start = Instant::now();
    
    let result = process_request(method, path).await;
    
    let duration = start.elapsed().as_secs_f64();
    let status = match &result {
        Ok(resp) => resp.status().as_u16().to_string(),
        Err(_) => "500".to_string(),
    };
    
    // 记录 metrics
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[method, path, &status])
        .inc();
    
    HTTP_REQUEST_DURATION
        .with_label_values(&[method, path])
        .observe(duration);
    
    result
}

/// 记录钱包操作
pub async fn create_wallet(name: &str, chain: &str) -> Result<Wallet> {
    let result = perform_wallet_creation(name, chain).await;
    
    let status = if result.is_ok() { "success" } else { "failure" };
    
    WALLET_OPERATIONS
        .with_label_values(&["create", chain, status])
        .inc();
    
    if let Ok(ref wallet) = result {
        ACTIVE_WALLETS
            .with_label_values(&[chain])
            .inc();
    }
    
    result
}

/// 记录交易
pub async fn send_transaction(tx: &Transaction) -> Result<String> {
    let result = broadcast_transaction(tx).await;
    
    let status = if result.is_ok() { "success" } else { "failure" };
    
    TRANSACTIONS
        .with_label_values(&[&tx.chain, status])
        .inc();
    
    if let Some(amount_usd) = tx.amount_usd {
        TRANSACTION_AMOUNT.observe(amount_usd);
    }
    
    result
}

/// 记录 RPC 调用
pub async fn call_rpc(chain: &str, method: &str, params: Vec<Value>) -> Result<Value> {
    let start = Instant::now();
    
    let result = execute_rpc_call(chain, method, params).await;
    
    let duration = start.elapsed().as_secs_f64();
    let status = if result.is_ok() { "success" } else { "failure" };
    
    RPC_CALLS
        .with_label_values(&[chain, method, status])
        .inc();
    
    RPC_CALL_DURATION
        .with_label_values(&[chain, method])
        .observe(duration);
    
    result
}

/// 记录错误
pub fn record_error(error: &AppError) {
    let error_type = error.error_type();
    let severity = match error.severity() {
        ErrorSeverity::Critical => "critical",
        ErrorSeverity::High => "high",
        ErrorSeverity::Medium => "medium",
        ErrorSeverity::Low => "low",
    };
    
    ERRORS
        .with_label_values(&[&error_type, severity])
        .inc();
}
```

---

## 🏥 健康检查

### 健康检查端点

```rust
// src/monitoring/health.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: Vec<ComponentHealth>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_check: i64,
}

/// 健康检查实现
pub async fn perform_health_check() -> HealthCheck {
    let mut checks = Vec::new();
    
    // 检查后端 API
    checks.push(check_backend_api().await);
    
    // 检查 RPC 节点
    checks.push(check_rpc_nodes().await);
    
    // 检查 IndexedDB
    checks.push(check_indexeddb().await);
    
    // 检查缓存
    checks.push(check_cache().await);
    
    // 整体状态
    let status = if checks.iter().all(|c| matches!(c.status, HealthStatus::Healthy)) {
        HealthStatus::Healthy
    } else if checks.iter().any(|c| matches!(c.status, HealthStatus::Unhealthy)) {
        HealthStatus::Unhealthy
    } else {
        HealthStatus::Degraded
    };
    
    HealthCheck {
        status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_uptime_seconds(),
        checks,
    }
}

async fn check_backend_api() -> ComponentHealth {
    match api_client().get("/health").send().await {
        Ok(resp) if resp.status().is_success() => ComponentHealth {
            name: "backend_api".to_string(),
            status: HealthStatus::Healthy,
            message: None,
            last_check: chrono::Utc::now().timestamp(),
        },
        Ok(resp) => ComponentHealth {
            name: "backend_api".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some(format!("HTTP {}", resp.status())),
            last_check: chrono::Utc::now().timestamp(),
        },
        Err(e) => ComponentHealth {
            name: "backend_api".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some(e.to_string()),
            last_check: chrono::Utc::now().timestamp(),
        },
    }
}

async fn check_rpc_nodes() -> ComponentHealth {
    // 检查关键 RPC 节点
    let chains = vec!["ethereum", "bsc", "polygon"];
    let mut healthy_count = 0;
    
    for chain in &chains {
        if is_rpc_healthy(chain).await {
            healthy_count += 1;
        }
    }
    
    let status = if healthy_count == chains.len() {
        HealthStatus::Healthy
    } else if healthy_count > 0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };
    
    ComponentHealth {
        name: "rpc_nodes".to_string(),
        status,
        message: Some(format!("{}/{} chains healthy", healthy_count, chains.len())),
        last_check: chrono::Utc::now().timestamp(),
    }
}

async fn check_indexeddb() -> ComponentHealth {
    match test_indexeddb_access().await {
        Ok(_) => ComponentHealth {
            name: "indexeddb".to_string(),
            status: HealthStatus::Healthy,
            message: None,
            last_check: chrono::Utc::now().timestamp(),
        },
        Err(e) => ComponentHealth {
            name: "indexeddb".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some(e.to_string()),
            last_check: chrono::Utc::now().timestamp(),
        },
    }
}
```

---

## 📈 性能监控

### 性能 Tracing

```rust
use tracing::{info_span, Instrument};

/// 性能追踪装饰器
pub async fn track_performance<F, T>(
    operation: &str,
    future: F,
) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let span = info_span!("performance", operation = %operation);
    
    future.instrument(span).await
}

// 使用示例
let wallet = track_performance("create_wallet", async {
    create_wallet_impl(name, chain).await
}).await?;
```

### Web Vitals 监控

```rust
// src/monitoring/web_vitals.rs

/// Core Web Vitals
#[derive(Serialize)]
pub struct WebVitals {
    /// Largest Contentful Paint（最大内容绘制）
    pub lcp: f64,
    /// First Input Delay（首次输入延迟）
    pub fid: f64,
    /// Cumulative Layout Shift（累积布局偏移）
    pub cls: f64,
    /// First Contentful Paint（首次内容绘制）
    pub fcp: f64,
    /// Time to Interactive（可交互时间）
    pub tti: f64,
}

pub fn collect_web_vitals() -> WebVitals {
    // 使用 web-sys 收集性能指标
    let performance = web_sys::window()
        .unwrap()
        .performance()
        .unwrap();
    
    WebVitals {
        lcp: get_lcp(&performance),
        fid: get_fid(&performance),
        cls: get_cls(&performance),
        fcp: get_fcp(&performance),
        tti: get_tti(&performance),
    }
}

/// 报告 Web Vitals
pub async fn report_web_vitals(vitals: &WebVitals) {
    // 发送到后端分析
    api_client()
        .post("/analytics/web-vitals")
        .json(vitals)
        .send()
        .await
        .ok();
}
```

---

## 🚨 告警规则

### Prometheus 告警规则

```yaml
# alerts/ironforge.yml
groups:
  - name: ironforge_alerts
    interval: 30s
    rules:
      # 错误率告警
      - alert: HighErrorRate
        expr: |
          rate(errors_total[5m]) > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors/sec"
      
      # API 延迟告警
      - alert: HighAPILatency
        expr: |
          histogram_quantile(0.95, 
            rate(http_request_duration_seconds_bucket[5m])
          ) > 5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High API latency"
          description: "P95 latency is {{ $value }}s"
      
      # 交易失败率告警
      - alert: HighTransactionFailureRate
        expr: |
          rate(transactions_total{status="failure"}[10m]) / 
          rate(transactions_total[10m]) > 0.1
        for: 15m
        labels:
          severity: high
        annotations:
          summary: "High transaction failure rate"
          description: "{{ $value | humanizePercentage }} transactions failing"
      
      # RPC 节点不可用
      - alert: RPCNodeDown
        expr: |
          rate(rpc_calls_total{status="failure"}[5m]) > 0.5
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "RPC node {{ $labels.chain }} is down"
          description: "RPC failure rate: {{ $value }}"
      
      # 活跃用户数下降
      - alert: ActiveUsersDropped
        expr: |
          (active_users_total - active_users_total offset 1h) / 
          active_users_total offset 1h < -0.3
        for: 30m
        labels:
          severity: warning
        annotations:
          summary: "Active users dropped by 30%"
          description: "Current: {{ $value }}, Previous: {{ $value offset 1h }}"
```

### AlertManager 配置

```yaml
# alertmanager.yml
global:
  resolve_timeout: 5m
  slack_api_url: '${SLACK_WEBHOOK_URL}'

route:
  group_by: ['alertname', 'severity']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'team-alerts'
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty-critical'
    - match:
        severity: high
      receiver: 'slack-high'
    - match:
        severity: warning
      receiver: 'slack-warnings'

receivers:
  - name: 'team-alerts'
    slack_configs:
      - channel: '#ironforge-alerts'
        title: 'IronForge Alert'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}\n{{ end }}'
  
  - name: 'pagerduty-critical'
    pagerduty_configs:
      - service_key: '${PAGERDUTY_SERVICE_KEY}'
  
  - name: 'slack-high'
    slack_configs:
      - channel: '#ironforge-high-priority'
  
  - name: 'slack-warnings'
    slack_configs:
      - channel: '#ironforge-warnings'
```

---

## 📊 Grafana 仪表板

### Dashboard JSON

```json
{
  "dashboard": {
    "title": "IronForge Production Metrics",
    "panels": [
      {
        "title": "Requests/sec",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])"
          }
        ]
      },
      {
        "title": "P95 Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))"
          }
        ]
      },
      {
        "title": "Active Users",
        "targets": [
          {
            "expr": "active_users_total"
          }
        ]
      },
      {
        "title": "Transaction Volume",
        "targets": [
          {
            "expr": "rate(transactions_total[1h])"
          }
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "rate(errors_total[5m])"
          }
        ]
      }
    ]
  }
}
```

---

## 📚 依赖项

```toml
[dependencies]
prometheus = "0.13"
lazy_static = "1.4"
serde = { version = "1.0", features = ["derive"] }
chrono = "0.4"
```

---

## 🔗 相关文档

- [日志系统](./03-logging-system.md)
- [错误处理](./02-error-handling-system.md)
- [部署指南](./05-deployment-guide.md)
