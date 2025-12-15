# 🔐 企业级认证状态管理 - 三层架构对齐报告

## 📊 对齐状态总览

| 层级 | 组件 | 状态 | Token过期时间 | 配置位置 |
|------|------|------|---------------|----------|
| **前端** | AuthManager | ✅ 已实现 | 3600s (硬编码) | `auth_manager.rs:47` |
| **前端** | UserState | ✅ 已实现 | 3600s (硬编码) | `state.rs:47` |
| **前端** | ApiClient | ✅ 已实现 | - | `api.rs:34` |
| **后端** | JWT生成 | ✅ 已实现 | 3600s (配置) | `jwt.rs:63` + `config.toml:22` |
| **后端** | Config | ✅ 已实现 | 3600s (默认) | `config.rs:181` |
| **数据库** | users表 | ✅ 已实现 | - | `0002_core_tables.sql:20` |
| **数据库** | refresh_tokens表 | ❌ 缺失 | - | 需要新建migration |

---

## ❌ 发现的问题

### 1. **前端Token过期时间硬编码**
**问题**: 前端两处硬编码3600秒，后端可配置但前端无法同步
```rust
// ❌ 问题代码 1: IronForge/src/features/auth/auth_manager.rs:47
if token_age >= 3600 {
    tracing::warn!("⚠️ Token已过期（{}秒）", token_age);
    return Ok(false);
}

// ❌ 问题代码 2: IronForge/src/features/auth/state.rs:47
if token_age >= 3600 {
    warn!("⚠️ Token已过期（{}s），自动清理", token_age);
}
```

**影响**:
- 后端修改token_expiry_secs时，前端不会同步
- 导致前端提前或延后判断token过期
- 可能出现"前端认为有效，后端返回401"或"前端提前清理，后端还有效"

---

### 2. **缺少Refresh Token机制**
**问题**: 数据库和后端都配置了refresh_token，但没有表结构和API

**后端配置**:
```toml
# IronCore/config.toml:23
refresh_token_expiry_secs = 2592000  # 30天
```

**缺失**:
- ❌ 数据库表: `refresh_tokens` (存储refresh_token及其过期时间)
- ❌ API端点: `POST /api/v1/auth/refresh` (用refresh_token换新access_token)
- ❌ 前端实现: `AuthManager::refresh_token_if_needed()` (当前标记TODO)

---

### 3. **数据库缺少Session管理表**
**问题**: 无法追踪用户登录会话、多设备登录、强制登出

**建议增加**:
```sql
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    access_token_jti TEXT NOT NULL UNIQUE,  -- JWT的jti字段
    refresh_token_hash TEXT,                -- refresh_token的SHA256
    device_info JSONB,                      -- 设备信息（浏览器/移动设备）
    ip_address TEXT,
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

---

### 4. **API URL配置不统一**
**问题**: 前端硬编码localhost:8088，生产环境需要手动修改

```rust
// ❌ IronForge/src/shared/api.rs:34
base_url: "http://localhost:8088".to_string(),
```

**后端配置**:
```toml
# IronCore/config.toml:26
[server]
bind_addr = "0.0.0.0:8088"
```

**建议**: 前端应从环境变量或.env文件读取API_BASE_URL

---

### 5. **CORS配置问题**
**浏览器错误**:
```
Access to fetch at 'http://localhost:8088/api/v1/auth/logout' from origin 'http://127.0.0.1:8080' 
has been blocked by CORS policy: Response to preflight request doesn't pass access control check
```

**原因**:
- 前端: `http://127.0.0.1:8080` (Trunk默认)
- 后端: `http://localhost:8088` (配置文件)
- 浏览器认为这是跨域请求（127.0.0.1 ≠ localhost）

---

## ✅ 修复方案

### 方案1: **前端从后端获取Token配置** (推荐)

#### 步骤1: 后端增加配置查询API
```rust
// IronCore/src/api/handlers/config.rs
#[derive(Serialize)]
pub struct PublicConfig {
    pub token_expiry_secs: u64,
    pub server_time: i64,  // 用于时钟同步
}

pub async fn get_public_config(
    State(config): State<Arc<Config>>,
) -> impl IntoResponse {
    Json(PublicConfig {
        token_expiry_secs: config.jwt.token_expiry_secs,
        server_time: Utc::now().timestamp(),
    })
}
```

#### 步骤2: 前端启动时获取配置
```rust
// IronForge/src/features/auth/auth_manager.rs
pub struct AuthManager {
    app_state: AppState,
    token_expiry_secs: Signal<u64>,  // 动态配置
}

impl AuthManager {
    pub async fn init(app_state: AppState) -> Self {
        let config = fetch_server_config().await.unwrap_or(PublicConfig {
            token_expiry_secs: 3600,  // 降级默认值
            server_time: (js_sys::Date::new_0().get_time() / 1000.0) as i64,
        });
        
        Self {
            app_state,
            token_expiry_secs: Signal::new(config.token_expiry_secs),
        }
    }
}
```

---

### 方案2: **环境变量注入** (次优)

#### Trunk构建时注入
```bash
# IronForge/.env
API_BASE_URL=http://localhost:8088
TOKEN_EXPIRY_SECS=3600
```

```rust
// IronForge/src/shared/api.rs
impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: option_env!("API_BASE_URL")
                .unwrap_or("http://localhost:8088")
                .to_string(),
            timeout: 30,
        }
    }
}
```

**缺点**: 需要重新编译WASM才能修改配置

---

### 方案3: **数据库Session表迁移**

```sql
-- IronCore/migrations/0050_user_sessions.sql
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token_jti TEXT NOT NULL UNIQUE,
    refresh_token_hash TEXT,
    device_info JSONB DEFAULT '{}',
    ip_address TEXT,
    user_agent TEXT,
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_user_sessions_user FOREIGN KEY (user_id) 
        REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX idx_user_sessions_jti ON user_sessions(access_token_jti);

-- 定期清理过期session
CREATE INDEX idx_user_sessions_cleanup 
ON user_sessions(expires_at) WHERE expires_at < CURRENT_TIMESTAMP;
```

**后端修改**:
```rust
// IronCore/src/api/handlers/auth.rs
pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    // ... 验证用户 ...
    
    let token = generate_token(user.id, user.tenant_id, user.role)?;
    let claims = decode_token_without_validation(&token)?;  // 获取jti
    
    // ✅ 记录session
    sqlx::query!(
        r#"INSERT INTO user_sessions 
           (user_id, access_token_jti, device_info, ip_address, expires_at)
           VALUES ($1, $2, $3, $4, NOW() + INTERVAL '1 hour')"#,
        user.id,
        claims.jti,
        json!({"user_agent": req_headers["user-agent"]}),
        extract_client_ip(req)
    )
    .execute(&pool)
    .await?;
    
    Ok(Json(LoginResponse { access_token: token }))
}

pub async fn logout(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<StatusCode> {
    // ✅ 删除session
    sqlx::query!("DELETE FROM user_sessions WHERE access_token_jti = $1", claims.jti)
        .execute(&pool)
        .await?;
    
    Ok(StatusCode::NO_CONTENT)
}
```

---

### 方案4: **修复CORS问题**

#### 后端统一CORS配置
```rust
// IronCore/src/main.rs
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin([
        "http://localhost:8080".parse()?,
        "http://127.0.0.1:8080".parse()?,
        "http://localhost:8081".parse()?,
        "http://127.0.0.1:8081".parse()?,
    ])
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
    .allow_headers(Any)
    .allow_credentials(true);

let app = Router::new()
    // ... routes ...
    .layer(cors);
```

#### 前端统一使用localhost
```toml
# IronForge/Trunk.toml
[[proxy]]
backend = "http://localhost:8088"
rewrite = "/api/"
```

**测试方法**:
```bash
# 清除浏览器缓存后重新测试
cd IronForge
rm -rf dist/
trunk serve --address 127.0.0.1 --port 8080
```

---

## 🎯 优先级排序

| 优先级 | 任务 | 估计工时 | 影响范围 |
|--------|------|----------|----------|
| **P0** | 修复CORS问题 | 30分钟 | 阻塞所有API调用 |
| **P0** | 清除浏览器缓存（trailing `&` bug） | 5分钟 | 当前主要问题 |
| **P1** | 前端从后端获取token配置 | 2小时 | 架构对齐核心 |
| **P2** | 数据库Session表迁移 | 3小时 | 会话管理增强 |
| **P2** | 实现Refresh Token机制 | 4小时 | 用户体验提升 |
| **P3** | 环境变量配置优化 | 1小时 | 可维护性 |

---

## 🔄 完整对齐流程

### 阶段1: 立即修复（今天）
1. ✅ 清除浏览器缓存 (Ctrl+Shift+R)
2. ✅ 修复CORS配置
3. ✅ 验证基础登录/登出流程

### 阶段2: 架构对齐（明天）
1. 🔄 实现 `GET /api/v1/config/public` API
2. 🔄 前端动态加载token配置
3. 🔄 AuthManager使用动态配置验证token

### 阶段3: 增强功能（下周）
1. 📋 创建user_sessions表迁移
2. 📋 后端实现session管理
3. 📋 实现Refresh Token机制
4. 📋 前端集成自动刷新token

---

## 📝 测试清单

### 功能测试
- [ ] 用户登录成功，token保存到LocalStorage
- [ ] Token过期自动清理（等待1小时或手动修改时间戳）
- [ ] 401错误触发统一登出
- [ ] 多标签页状态同步

### 配置测试
- [ ] 修改后端token_expiry_secs，前端能同步
- [ ] 前端使用正确的过期时间验证token
- [ ] 服务端时间与客户端时间偏差<5秒

### 数据库测试
- [ ] 登录时创建session记录
- [ ] 登出时删除session记录
- [ ] 过期session能被定时任务清理

---

## 🎓 最佳实践建议

### 1. 配置管理
✅ **DO**:
- 后端作为配置的唯一真实来源（Single Source of Truth）
- 前端启动时获取配置，缓存到内存
- 配置变更时前端热更新（WebSocket推送或轮询）

❌ **DON'T**:
- 前端硬编码业务配置
- 多处重复定义相同配置值
- 配置不一致时没有降级策略

### 2. Token管理
✅ **DO**:
- Access Token短期（1小时）+ Refresh Token长期（30天）
- 数据库记录所有活跃session
- 支持强制登出（删除session）
- Token中包含jti用于追踪

❌ **DON'T**:
- Access Token过长（>1天）
- 无状态JWT导致无法强制登出
- Token过期时间前后端不一致

### 3. 前后端通信
✅ **DO**:
- 统一使用Bearer Token认证
- 401错误时尝试刷新token，失败后登出
- 关键操作记录审计日志

❌ **DON'T**:
- 忽略401错误继续请求
- 前端缓存敏感信息（密码、私钥）
- CORS配置过于宽松（allow_origin: "*"）

---

## 📚 相关文档

- [BACKEND_FRONTEND_API_ARCHITECTURE.md](../docs/BACKEND_FRONTEND_API_ARCHITECTURE.md) - 完整架构设计
- [auth_manager.rs](src/features/auth/auth_manager.rs) - 企业级认证管理器实现
- [IronCore/config.toml](../IronCore/config.toml) - 后端配置示例
- [JWT最佳实践](https://tools.ietf.org/html/rfc8725) - RFC 8725

---

**生成时间**: 2025-12-06 16:30  
**版本**: v1.0  
**负责人**: AI Agent + Plant (User)
