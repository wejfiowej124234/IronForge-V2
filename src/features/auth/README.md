# 🔐 认证模块 (Authentication Module)

## 📁 文件结构

```
src/features/auth/
├── mod.rs              # 模块导出
├── state.rs            # UserState数据结构 + LocalStorage持久化
├── hooks.rs            # 登录/注册/登出/同步 hooks
├── auth_manager.rs     # ← 新增：统一认证状态管理器
└── README.md           # ← 本文件
```

## 🎯 职责划分

### 1. `state.rs` - 数据层
**职责**：定义UserState数据结构，提供LocalStorage持久化

```rust
pub struct UserState {
    pub is_authenticated: bool,
    pub access_token: Option<String>,
    pub token_created_at: Option<u64>,  // ← 关键：Token创建时间戳
    pub email: String,
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
}

impl UserState {
    pub fn load() -> Self { /* 从LocalStorage加载 */ }
    pub fn save(&self) { /* 保存到LocalStorage */ }
}
```

**关键更新**：
- ✅ 添加 `token_created_at` 字段追踪Token年龄
- ✅ `load()` 时自动检测过期Token并清理

### 2. `hooks.rs` - 业务逻辑层
**职责**：提供认证相关的hooks（登录/注册/登出/同步）

```rust
pub struct UseAuth {
    pub login: Box<dyn Fn(LoginRequest) -> Result<()>>,
    pub register: Box<dyn Fn(RegisterRequest) -> Result<()>>,
    pub logout: Box<dyn Fn()>,
    pub sync_wallets: Box<dyn Fn() -> Result<()>>,
}

pub fn use_auth() -> UseAuth { /* ... */ }
```

**最佳实践**：
- ✅ 登录/注册成功后立即调用 `AuthManager::set_token()`
- ✅ 登出时调用 `AuthManager::clear_auth()`
- ✅ 同步钱包前调用 `AuthManager::validate_token()`

### 3. `auth_manager.rs` - 状态管理层（新增）
**职责**：统一管理Token生命周期和认证状态同步

```rust
pub struct AuthManager {
    app_state: AppState,
}

impl AuthManager {
    // Token管理
    pub async fn set_token(&self, token: String);
    pub async fn refresh_token_if_needed(&self) -> Result<bool>;
    pub fn clear_auth(&self);
    
    // Token验证
    pub fn validate_token(&self) -> Result<bool>;
    pub fn get_token_remaining_seconds(&self) -> Option<u64>;
    pub fn is_authenticated(&self) -> bool;
    
    // 状态同步
    pub async fn sync_to_api_client(&self);
}

// 全局401错误处理
pub async fn handle_unauthorized(app_state: AppState);
pub fn is_unauthorized_error(error: &AppError) -> bool;
```

## 🔄 认证流程

### 流程1：用户登录
```
User Input (email/password)
    ↓
hooks.rs: login()
    ↓
POST /api/v1/auth/login → Backend
    ↓
Response { access_token, user_info }
    ↓
AuthManager::set_token(token) ← 新增：统一设置Token
    ├─ 1. Update UserState (with token_created_at)
    ├─ 2. Wait 100ms (Signal propagation)
    ├─ 3. Sync to ApiClient.set_bearer_token()
    └─ 4. Save to LocalStorage
    ↓
Navigate to /dashboard
```

### 流程2：Token过期检测
```
App Startup
    ↓
UserState::load() from LocalStorage
    ↓
AuthManager::validate_token() ← 新增：统一验证
    ├─ Check: token exists?
    ├─ Check: token_created_at exists?
    └─ Check: age < 3600 seconds?
    ↓
if expired:
    AuthManager::clear_auth() ← 新增：统一清理
    Navigate to /login
else:
    AuthManager::sync_to_api_client()
    Stay on current page
```

### 流程3：401错误处理
```
API Request (with Bearer Token)
    ↓
Backend validates JWT
    ├─ Valid → 200 OK
    └─ Invalid/Expired → 401 Unauthorized
          ↓
Frontend catches error
    ↓
is_unauthorized_error(error)? ← 新增：统一判断
    ↓
handle_unauthorized(app_state) ← 新增：统一处理
    ├─ AuthManager::clear_auth()
    ├─ Log: "🚨 Token失效，已清理状态"
    └─ Optional: Navigate to /login
```

## 🛠️ 使用指南

### 在组件中使用AuthManager

```rust
use crate::features::auth::auth_manager::AuthManager;
use crate::shared::state::use_app_state;

#[component]
pub fn MyComponent() -> Element {
    let app_state = use_app_state();
    let auth_manager = AuthManager::new(app_state);
    
    // 检查认证状态
    if !auth_manager.is_authenticated() {
        return rsx! { "请先登录" };
    }
    
    // 获取剩余时间
    let remaining = auth_manager.get_token_remaining_seconds();
    
    rsx! {
        div { "Token剩余: {remaining.unwrap_or(0)}秒" }
    }
}
```

### 在Service中处理401错误

```rust
use crate::features::auth::auth_manager::{handle_unauthorized, is_unauthorized_error};

impl WalletService {
    pub async fn list_wallets(&self) -> Result<Vec<WalletDto>, AppError> {
        let api = self.api();
        let path = "/api/v1/wallets?page=1&page_size=100";
        
        match api.get::<ListWalletsResp>(&path).await {
            Ok(resp) => Ok(resp.wallets),
            Err(e) => {
                // ✅ 统一401错误处理
                if is_unauthorized_error(&e) {
                    handle_unauthorized(self.app_state).await;
                }
                Err(e.into())
            }
        }
    }
}
```

### 在登录Hook中设置Token

```rust
pub async fn login(email: String, password: String) {
    let app_state = use_app_state();
    let auth_manager = AuthManager::new(app_state);
    
    // 调用后端登录API
    let api = app_state.api.read();
    let response = api.post::<LoginResponse>("/api/v1/auth/login", &LoginRequest {
        email, password
    }).await?;
    
    // ✅ 使用AuthManager统一设置Token
    auth_manager.set_token(response.access_token).await;
    
    // 导航到首页
    nav.push("/dashboard");
}
```

## 🎯 最佳实践

### ✅ DO（推荐）

1. **集中管理Token**
   ```rust
   // ✅ 好：通过AuthManager设置Token
   auth_manager.set_token(token).await;
   ```

2. **统一错误处理**
   ```rust
   // ✅ 好：使用全局401处理器
   if is_unauthorized_error(&error) {
       handle_unauthorized(app_state).await;
   }
   ```

3. **定期验证Token**
   ```rust
   // ✅ 好：在关键操作前验证
   if !auth_manager.validate_token()? {
       return Err("Token已过期");
   }
   ```

4. **显式状态同步**
   ```rust
   // ✅ 好：登录后等待同步完成
   auth_manager.set_token(token).await;
   TimeoutFuture::new(100).await;  // 等待Signal传播
   ```

### ❌ DON'T（避免）

1. **直接操作UserState和ApiClient**
   ```rust
   // ❌ 差：手动同步容易遗漏步骤
   user_state.access_token = Some(token);
   api.set_bearer_token(token);
   user_state.save();
   ```

2. **重复的401错误处理**
   ```rust
   // ❌ 差：每个service都写一遍
   if error.status_code == 401 {
       user_state.is_authenticated = false;
       api.clear_auth();
   }
   ```

3. **忽略Token过期**
   ```rust
   // ❌ 差：不检查直接使用
   let token = user_state.access_token.unwrap();
   ```

## 🔧 配置项

```rust
// src/features/auth/auth_manager.rs

// Token有效期（秒）
const TOKEN_EXPIRY_SECONDS: u64 = 3600;  // 1小时

// Token刷新阈值（秒）
const TOKEN_REFRESH_THRESHOLD: u64 = 3300;  // 55分钟

// Signal传播等待时间（毫秒）
const SIGNAL_PROPAGATION_DELAY_MS: u32 = 100;
```

## 📊 监控指标

建议在生产环境监控以下指标：

```rust
// 在AuthManager中添加metrics
use crate::metrics;

impl AuthManager {
    pub async fn set_token(&self, token: String) {
        metrics::auth_token_set_count();
        // ...
    }
    
    pub fn clear_auth(&self) {
        metrics::auth_clear_count();
        // ...
    }
    
    pub fn validate_token(&self) -> Result<bool> {
        let is_valid = /* ... */;
        if !is_valid {
            metrics::auth_token_expired_count();
        }
        // ...
    }
}
```

## 🧪 测试策略

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_expiry_detection() {
        // 测试Token过期检测逻辑
    }
    
    #[test]
    fn test_auth_state_sync() {
        // 测试UserState ↔ ApiClient同步
    }
    
    #[test]
    fn test_401_error_handling() {
        // 测试401错误统一处理
    }
}
```

## 🚀 迁移指南

### 从旧代码迁移到AuthManager

#### Step 1: 更新登录逻辑

```diff
// src/features/auth/hooks.rs

pub async fn login() {
-   user_state.access_token = Some(token);
-   user_state.token_created_at = Some(now());
-   user_state.save();
-   api.write().set_bearer_token(token);

+   let auth_manager = AuthManager::new(app_state);
+   auth_manager.set_token(token).await;
}
```

#### Step 2: 更新登出逻辑

```diff
pub async fn logout() {
-   user_state.is_authenticated = false;
-   user_state.access_token = None;
-   user_state.save();
-   api.write().clear_auth();

+   let auth_manager = AuthManager::new(app_state);
+   auth_manager.clear_auth();
}
```

#### Step 3: 更新Service错误处理

```diff
// src/services/wallet.rs

match api.get(&path).await {
    Err(e) => {
-       if crate::shared::auth_handler::is_unauthorized_error(&e) {
-           self.app_state.user.write().is_authenticated = false;
-           self.app_state.api.write().clear_auth();
-       }

+       if is_unauthorized_error(&e) {
+           handle_unauthorized(self.app_state).await;
+       }
        Err(e.into())
    }
}
```

## 📚 相关文档

- [BACKEND_FRONTEND_API_ARCHITECTURE.md](../../../docs/BACKEND_FRONTEND_API_ARCHITECTURE.md) - 完整架构设计
- [IronCore JWT Authentication](../../../IronCore/docs/04-security/JWT_AUTHENTICATION.md) - 后端JWT实现
- [Frontend Security](../../docs/04-security/03-security-architecture.md) - 前端安全架构
