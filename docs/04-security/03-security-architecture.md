# 安全架构总览

> **版本**: V2.0  
> **技术栈**: Rust + WASM + Web Crypto API  
> **更新日期**: 2025-11-25  
> **安全等级**: 🔴 Production-Grade  
> **威胁模型**: STRIDE + OWASP Top 10

---

## 📋 目录

1. [安全架构概览](#安全架构概览)
2. [零信任模型](#零信任模型)
3. [攻击面分析](#攻击面分析)
4. [防御措施](#防御措施)
5. [安全开发生命周期](#安全开发生命周期)
6. [安全检查清单](#安全检查清单)
7. [事件响应](#事件响应)
8. [合规性](#合规性)

---

## 安全架构概览

### 核心原则

1. **零信任架构**: 永不信任，始终验证
2. **纵深防御**: 多层安全控制
3. **最小权限**: 仅授予必要权限
4. **数据加密**: 传输加密 + 存储加密
5. **审计日志**: 所有敏感操作可追溯

### 安全分层

```
┌─────────────────────────────────────────────────────────┐
│              Layer 7: User Interface                     │
│  - 输入验证                                              │
│  - XSS 防护                                              │
│  - CSRF 防护                                             │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│         Layer 6: Application Logic                       │
│  - 业务逻辑验证                                           │
│  - 授权检查                                              │
│  - 速率限制                                              │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│            Layer 5: Cryptography                         │
│  - AES-256-GCM 加密                                      │
│  - Argon2id 密钥派生                                     │
│  - secp256k1/ed25519 签名                                │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│         Layer 4: Storage Security                        │
│  - IndexedDB 加密存储                                     │
│  - 敏感数据自动清零                                       │
│  - 安全存储分离                                           │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│        Layer 3: Network Security                         │
│  - HTTPS/TLS 1.3                                         │
│  - Certificate Pinning                                   │
│  - API 认证（JWT）                                       │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│         Layer 2: Runtime Security                        │
│  - WASM 沙箱隔离                                         │
│  - CSP (Content Security Policy)                         │
│  - SRI (Subresource Integrity)                          │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│       Layer 1: Infrastructure Security                   │
│  - CDN DDoS 防护                                         │
│  - WAF (Web Application Firewall)                        │
│  - 安全监控与告警                                         │
└─────────────────────────────────────────────────────────┘
```

---

## 零信任模型

### 架构设计

```rust
// src/security/zero_trust.rs

/// 零信任访问控制
pub struct ZeroTrustContext {
    /// 当前用户标识
    user_id: Option<String>,
    /// 会话有效期
    session_expires_at: Option<u64>,
    /// 设备指纹
    device_fingerprint: String,
    /// 位置信息（可选）
    location: Option<GeoLocation>,
    /// 风险评分（0-100）
    risk_score: u8,
}

impl ZeroTrustContext {
    /// 验证访问权限
    pub fn verify_access(&self, resource: &Resource) -> Result<(), SecurityError> {
        // 1. 会话验证
        if let Some(expires_at) = self.session_expires_at {
            if current_timestamp() > expires_at {
                return Err(SecurityError::SessionExpired);
            }
        } else {
            return Err(SecurityError::NoSession);
        }
        
        // 2. 风险评分检查
        if self.risk_score > 70 {
            // 高风险：需要额外验证（如 2FA）
            return Err(SecurityError::RequireAdditionalAuth);
        }
        
        // 3. 资源级权限检查
        if !self.has_permission(resource) {
            return Err(SecurityError::PermissionDenied);
        }
        
        Ok(())
    }
    
    /// 计算风险评分
    pub fn calculate_risk_score(&mut self) {
        let mut score = 0u8;
        
        // 设备变更
        if self.is_new_device() {
            score += 20;
        }
        
        // 地理位置异常
        if self.is_location_anomaly() {
            score += 30;
        }
        
        // 频繁失败尝试
        if self.has_recent_failures() {
            score += 25;
        }
        
        // 会话时长
        if self.is_session_expired() {
            score += 15;
        }
        
        self.risk_score = score;
    }
}

/// 敏感操作验证
#[derive(Debug, Clone)]
pub enum SensitiveOperation {
    SignTransaction,
    ExportMnemonic,
    ChangePassword,
    DeleteWallet,
}

impl SensitiveOperation {
    /// 所需安全级别
    pub fn required_security_level(&self) -> SecurityLevel {
        match self {
            Self::SignTransaction => SecurityLevel::Medium,
            Self::ExportMnemonic => SecurityLevel::High,
            Self::ChangePassword => SecurityLevel::High,
            Self::DeleteWallet => SecurityLevel::Critical,
        }
    }
    
    /// 验证操作权限
    pub async fn verify(
        &self,
        context: &ZeroTrustContext,
        credentials: &Credentials,
    ) -> Result<(), SecurityError> {
        let required_level = self.required_security_level();
        
        // 1. 基础会话验证
        context.verify_access(&Resource::SensitiveOperation)?;
        
        // 2. 密码验证
        verify_password(&credentials.password).await?;
        
        // 3. 根据安全级别要求额外验证
        match required_level {
            SecurityLevel::Critical => {
                // 需要二次确认 + 延迟（防止自动化攻击）
                require_confirmation().await?;
                delay_operation(Duration::from_secs(3)).await;
            }
            SecurityLevel::High => {
                // 需要二次确认
                require_confirmation().await?;
            }
            _ => {}
        }
        
        Ok(())
    }
}
```

---

## 攻击面分析

### STRIDE 威胁模型

| 威胁类型 | 攻击场景 | 防御措施 | 优先级 |
|---------|---------|---------|--------|
| **S**poofing (欺骗) | 钓鱼网站冒充钱包 | HTTPS + 域名验证 + Certificate Pinning | 🔴 高 |
| **T**ampering (篡改) | 中间人攻击修改交易 | TLS 1.3 + 签名验证 | 🔴 高 |
| **R**epudiation (抵赖) | 用户否认交易操作 | 完整审计日志 + 时间戳 | 🟡 中 |
| **I**nformation Disclosure (信息泄露) | 私钥/助记词泄露 | 加密存储 + 自动清零 | 🔴 高 |
| **D**enial of Service (拒绝服务) | DDoS 攻击 | CDN + 速率限制 | 🟡 中 |
| **E**levation of Privilege (权限提升) | 绕过授权访问敏感操作 | 最小权限 + 多层验证 | 🔴 高 |

### 具体攻击场景

#### 1. 钓鱼攻击

```
攻击者手段：
1. 创建假冒网站（ironforge-wallet.com → ironf0rge-wallet.com）
2. 诱导用户输入助记词
3. 盗取用户资产

防御措施：
✅ 官方域名标识（显示完整 URL）
✅ 浏览器地址栏警告（HTTPS + EV 证书）
✅ 明确提示"永不向任何人透露助记词"
✅ 检测剪贴板钓鱼（检测助记词复制）
```

#### 2. 中间人攻击 (MITM)

```
攻击者手段：
1. 拦截 HTTP 请求
2. 修改交易参数（接收地址、金额）
3. 用户签名后广播到攻击者地址

防御措施：
✅ 强制 HTTPS (HSTS)
✅ Certificate Pinning
✅ 交易参数二次确认（显示完整接收地址）
✅ 签名前显示完整交易详情
```

#### 3. XSS 注入

```
攻击者手段：
1. 注入恶意脚本到钱包名称、备注等字段
2. 窃取 LocalStorage/IndexedDB 中的敏感数据
3. 监听用户输入

防御措施：
✅ 输入验证与转义（所有用户输入）
✅ CSP (Content Security Policy)
✅ 敏感数据加密存储（即使泄露也无法解密）
✅ HttpOnly Cookie (JWT Token)
```

#### 4. 供应链攻击

```
攻击者手段：
1. 污染 npm/crates 依赖
2. 注入恶意代码
3. 窃取私钥或篡改交易

防御措施：
✅ 依赖锁定（Cargo.lock）
✅ 定期依赖审计（cargo audit）
✅ SRI (Subresource Integrity) 验证 CDN 资源
✅ 最小化依赖（减少攻击面）
```

---

## 防御措施

### 1. 输入验证

```rust
// src/security/validation.rs
use regex::Regex;
use once_cell::sync::Lazy;

/// 钱包名称验证
pub fn validate_wallet_name(name: &str) -> Result<(), ValidationError> {
    // 1. 长度检查
    if name.is_empty() {
        return Err(ValidationError::EmptyName);
    }
    if name.len() > 50 {
        return Err(ValidationError::NameTooLong);
    }
    
    // 2. 字符白名单（仅允许字母、数字、空格、连字符）
    static NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z0-9\s\-_]+$").unwrap()
    });
    
    if !NAME_REGEX.is_match(name) {
        return Err(ValidationError::InvalidCharacters);
    }
    
    // 3. XSS 关键字黑名单
    let dangerous_patterns = ["<script", "javascript:", "onerror=", "onload="];
    let name_lower = name.to_lowercase();
    
    for pattern in dangerous_patterns {
        if name_lower.contains(pattern) {
            return Err(ValidationError::SuspiciousContent);
        }
    }
    
    Ok(())
}

/// 以太坊地址验证
pub fn validate_ethereum_address(address: &str) -> Result<(), ValidationError> {
    // 1. 格式检查
    if !address.starts_with("0x") {
        return Err(ValidationError::MissingPrefix);
    }
    
    if address.len() != 42 {
        return Err(ValidationError::InvalidLength);
    }
    
    // 2. 十六进制验证
    static HEX_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^0x[0-9a-fA-F]{40}$").unwrap()
    });
    
    if !HEX_REGEX.is_match(address) {
        return Err(ValidationError::InvalidFormat);
    }
    
    // 3. 校验和验证（EIP-55）
    if !verify_checksum(address) {
        return Err(ValidationError::InvalidChecksum);
    }
    
    Ok(())
}

/// EIP-55 校验和验证
fn verify_checksum(address: &str) -> bool {
    use sha3::{Digest, Keccak256};
    
    let address_lower = address[2..].to_lowercase();
    let hash = Keccak256::digest(address_lower.as_bytes());
    
    for (i, c) in address[2..].chars().enumerate() {
        if c.is_ascii_alphabetic() {
            let hash_byte = hash[i / 2];
            let hash_nibble = if i % 2 == 0 {
                hash_byte >> 4
            } else {
                hash_byte & 0x0f
            };
            
            let should_be_uppercase = hash_nibble >= 8;
            let is_uppercase = c.is_uppercase();
            
            if should_be_uppercase != is_uppercase {
                return false;
            }
        }
    }
    
    true
}
```

### 2. CSP (Content Security Policy)

```html
<!-- index.html -->
<meta http-equiv="Content-Security-Policy" content="
    default-src 'self';
    script-src 'self' 'wasm-unsafe-eval';
    style-src 'self' 'unsafe-inline';
    img-src 'self' data: https:;
    font-src 'self';
    connect-src 'self' https://<your-backend-host>;
    frame-ancestors 'none';
    base-uri 'self';
    form-action 'self';
">
```

### 3. 速率限制

```rust
// src/security/rate_limiter.rs
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 滑动窗口速率限制器
pub struct RateLimiter {
    /// 操作 -> (时间窗口, 请求列表)
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    /// 时间窗口
    window: Duration,
    /// 最大请求数
    max_requests: usize,
}

impl RateLimiter {
    pub fn new(window: Duration, max_requests: usize) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            window,
            max_requests,
        }
    }
    
    /// 检查是否允许操作
    pub fn check_limit(&self, operation: &str) -> Result<(), RateLimitError> {
        let mut requests = self.requests.write().unwrap();
        let now = Instant::now();
        
        // 获取或创建操作的请求列表
        let operation_requests = requests.entry(operation.to_string()).or_insert_with(Vec::new);
        
        // 移除过期请求
        operation_requests.retain(|&time| now.duration_since(time) < self.window);
        
        // 检查是否超过限制
        if operation_requests.len() >= self.max_requests {
            return Err(RateLimitError::TooManyRequests {
                retry_after: self.window.as_secs(),
            });
        }
        
        // 记录本次请求
        operation_requests.push(now);
        
        Ok(())
    }
}

/// 敏感操作速率限制
pub static SENSITIVE_OPS_LIMITER: Lazy<RateLimiter> = Lazy::new(|| {
    RateLimiter::new(
        Duration::from_secs(60),  // 1 分钟窗口
        5,                         // 最多 5 次
    )
});

/// 使用示例
pub async fn sign_transaction() -> Result<(), AppError> {
    // 检查速率限制
    SENSITIVE_OPS_LIMITER.check_limit("sign_transaction")?;
    
    // 执行操作...
    Ok(())
}
```

### 4. 审计日志

```rust
// src/security/audit.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// 事件 ID（UUID）
    pub event_id: String,
    /// 时间戳（毫秒）
    pub timestamp: u64,
    /// 操作类型
    pub operation: AuditOperation,
    /// 用户 ID（如果有）
    pub user_id: Option<String>,
    /// 钱包 ID（如果有）
    pub wallet_id: Option<String>,
    /// 结果
    pub result: AuditResult,
    /// 额外上下文
    pub metadata: serde_json::Value,
    /// IP 地址（可选）
    pub ip_address: Option<String>,
    /// User-Agent（可选）
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOperation {
    WalletCreated,
    WalletUnlocked,
    WalletLocked,
    TransactionSigned,
    TransactionBroadcast,
    MnemonicExported,
    PasswordChanged,
    WalletDeleted,
    AuthenticationFailed,
    RateLimitExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure { reason: String },
}

impl AuditEvent {
    /// 记录到 IndexedDB
    pub async fn log(self) -> Result<(), StorageError> {
        let storage = use_context::<SecureStorage>();
        storage.save_audit_event(&self).await
    }
    
    /// 检测异常行为
    pub fn is_suspicious(&self) -> bool {
        match self.operation {
            AuditOperation::AuthenticationFailed => {
                // 连续失败 5 次
                check_consecutive_failures(&self.user_id, 5)
            }
            AuditOperation::RateLimitExceeded => true,
            _ => false,
        }
    }
}
```

---

## 安全开发生命周期

### 开发阶段检查

```markdown
## 设计阶段
- [ ] 威胁建模（STRIDE）
- [ ] 数据流图（DFD）
- [ ] 信任边界识别
- [ ] 最小权限设计

## 开发阶段
- [ ] 输入验证（所有用户输入）
- [ ] 输出编码（防 XSS）
- [ ] 参数化查询（防 SQL 注入 - 如适用）
- [ ] 敏感数据加密
- [ ] 错误处理不泄露信息

## 测试阶段
- [ ] 静态代码分析（Clippy）
- [ ] 依赖漏洞扫描（cargo audit）
- [ ] 模糊测试（Fuzzing）
- [ ] 渗透测试（OWASP Top 10）

## 部署阶段
- [ ] HTTPS/TLS 1.3
- [ ] CSP 配置
- [ ] HSTS 启用
- [ ] SRI 验证
- [ ] 安全头（X-Frame-Options, X-Content-Type-Options）

## 运维阶段
- [ ] 日志监控
- [ ] 异常检测
- [ ] 定期安全审计
- [ ] 事件响应计划
```

---

## 安全检查清单

### 代码提交前检查

```bash
#!/bin/bash
# scripts/security-check.sh

echo "🔒 Running security checks..."

# 1. 依赖漏洞扫描
echo "📦 Checking dependencies..."
cargo audit

# 2. 静态代码分析
echo "🔍 Running Clippy..."
cargo clippy -- -D warnings -D clippy::unwrap_used -D clippy::expect_used

# 3. 检查敏感数据泄露
echo "🕵️ Checking for secrets..."
gitleaks detect --source . --verbose

# 4. 检查硬编码凭证
echo "🔑 Checking for hardcoded credentials..."
grep -r "password\s*=\s*\"" src/ && exit 1
grep -r "api_key\s*=\s*\"" src/ && exit 1

echo "✅ All security checks passed!"
```

### 生产发布检查

```markdown
## Pre-Release Security Checklist

### 代码审查
- [ ] 所有代码经过安全审查
- [ ] 无 TODO/FIXME 涉及安全
- [ ] 敏感操作有审计日志

### 依赖管理
- [ ] Cargo.lock 已提交
- [ ] 无已知高危漏洞（cargo audit）
- [ ] 依赖来源可信

### 配置检查
- [ ] 生产环境配置独立
- [ ] 无硬编码密钥
- [ ] Debug 模式已禁用

### 加密验证
- [ ] 密钥派生使用 Argon2id（600k+ 迭代）
- [ ] 数据加密使用 AES-256-GCM
- [ ] 随机数生成使用 OsRng

### 网络安全
- [ ] 强制 HTTPS (HSTS)
- [ ] CSP 配置正确
- [ ] API 端点有速率限制

### 存储安全
- [ ] 敏感数据加密存储
- [ ] 会话密钥自动过期
- [ ] 用户数据隔离

### 监控告警
- [ ] 错误日志配置
- [ ] 异常行为告警
- [ ] 性能监控
```

---

## 事件响应

### 安全事件分类

| 级别 | 描述 | 响应时间 | 示例 |
|------|------|---------|------|
| P0 - 严重 | 私钥泄露、资金被盗 | < 1 小时 | 助记词明文存储 |
| P1 - 高危 | 认证绕过、权限提升 | < 4 小时 | JWT 签名验证缺失 |
| P2 - 中危 | XSS、CSRF | < 24 小时 | 输入验证缺失 |
| P3 - 低危 | 信息泄露（非敏感） | < 7 天 | 版本号暴露 |

### 响应流程

```
1. 识别 (Identify)
   - 安全监控告警
   - 用户报告
   - 安全研究员披露

2. 遏制 (Contain)
   - 隔离受影响系统
   - 禁用受损账户
   - 阻止攻击流量

3. 根除 (Eradicate)
   - 修复漏洞
   - 更新依赖
   - 部署补丁

4. 恢复 (Recover)
   - 恢复服务
   - 验证修复有效
   - 监控异常

5. 总结 (Lessons Learned)
   - 事件报告
   - 改进措施
   - 更新防御策略
```

---

## 合规性

### GDPR 合规

```markdown
## 数据最小化
- [ ] 仅收集必要数据
- [ ] 本地存储优先（不上传私钥/助记词）

## 用户权利
- [ ] 数据访问权（导出钱包元数据）
- [ ] 数据删除权（删除钱包）
- [ ] 数据可移植性（导出助记词）

## 安全措施
- [ ] 加密存储
- [ ] 访问控制
- [ ] 数据最小化
```

### OWASP ASVS (Application Security Verification Standard)

```markdown
## Level 2 合规（推荐）

### V1: Architecture
- [x] 安全架构文档
- [x] 信任边界定义
- [x] 最小权限原则

### V2: Authentication
- [x] 密码强度要求（≥8 位）
- [x] 会话超时（15 分钟）
- [x] 速率限制（5 次/分钟）

### V3: Session Management
- [x] 会话令牌随机生成
- [x] 会话过期自动登出
- [x] 并发会话控制

### V6: Cryptography
- [x] 加密算法符合标准（AES-256-GCM）
- [x] 密钥派生符合规范（Argon2id）
- [x] 随机数生成安全（OsRng）

### V8: Data Protection
- [x] 敏感数据加密存储
- [x] 内存自动清零
- [x] 传输加密（TLS 1.3）
```

---

## 参考资料

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [OWASP ASVS](https://owasp.org/www-project-application-security-verification-standard/)
- [STRIDE Threat Model](https://learn.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [Web3 Security Best Practices](https://github.com/ConsenSys/smart-contract-best-practices)
