# 🔍 401 错误诊断指南

## 问题现象
用户登录后，访问以下页面时出现 401 Unauthorized 警告：
- `/api/v1/limit-orders` (限价单)
- `/api/v1/swap/history` (交换历史)

## 诊断步骤

### 1️⃣ 启动前端开发服务器
```bash
cd IronForge-V2
trunk serve
```
访问 http://127.0.0.1:8080

### 2️⃣ 登录账户
使用你的邮箱和密码登录

### 3️⃣ 打开浏览器开发者工具 (F12)
- Chrome/Edge: `F12` 或 `Ctrl+Shift+I`
- Firefox: `F12`
- 切换到 **Console (控制台)** 标签

### 4️⃣ 快速检查本地 token
在 Console 执行：
```js
localStorage.getItem("user_state")
```
确认其中存在 `access_token`，并且是 JWT（形如 `xxx.yyy.zzz`）。

## 诊断结果分析

### ✅ 场景 1: Token 存在且有效 (200 OK)
```
✅ Token is VALID - API accepts it
```
**结论**: 后端认可 token，但前端 WASM 代码可能没有正确使用
**解决方案**: 检查前端日志，查找 "API Request: No auth token available" 警告

### ❌ 场景 2: Token 不存在 (NULL)
```
❌ No access_token found - user needs to login
```
**结论**: 登录后 token 没有保存到 LocalStorage
**解决方案**: 检查 `AuthController::login()` 是否调用了 `user_state.save()`

### ❌ 场景 3: Token 已过期 (401)
```
❌ Token is INVALID or EXPIRED - 401 Unauthorized
```
**结论**: Token 已过期（JWT 默认 1 小时有效期）
**解决方案**: 重新登录获取新 token

## 代码逻辑验证

### ✅ 已验证的代码路径

#### 1. 登录流程 (`IronForge-V2/src/features/auth/hooks.rs:56-82`)
```rust
pub async fn login(&self, email: &str, password: &str) -> Result<()> {
    let response = auth_service.login_email(email, password).await?;
    
    // ✅ 1. 保存到 UserState
    user_state.is_authenticated = true;
    user_state.access_token = Some(response.access_token.clone());
    user_state.save()?;  // ✅ 持久化到 LocalStorage
    
    // ✅ 2. 设置 API 客户端 token
    app_state.api.write().set_bearer_token(response.access_token);
    
    Ok(())
}
```

#### 2. API 客户端获取 (`IronForge-V2/src/shared/state.rs:62-106`)
```rust
pub fn get_api_client(&self) -> ApiClient {
    let mut api_client = (*self.api.read()).clone();
    let user_state = self.user.read();
    
    // ✅ 从 UserState 同步 token
    if user_state.is_authenticated {
        if let Some(ref token) = user_state.access_token {
            api_client.set_bearer_token(token.clone());  // ✅ 设置 Bearer token
        }
    }
    
    api_client
}
```

#### 3. HTTP 请求构建 (`IronForge-V2/src/shared/api.rs:119-131`)
```rust
AuthToken::Bearer(value) => {
    let header_val = format!("Bearer {}", value);
    req.header("Authorization", &header_val)  // ✅ 添加 Authorization 头
}
```

## 常见问题排查

### Q1: 看到 "API Request: No auth token available" 警告
**原因**: `get_api_client()` 被调用时，`UserState.access_token` 为 None
**检查项**:
1. LocalStorage 中是否有 `user_state` 且 `access_token` 不为空
2. `UserState.is_authenticated` 是否为 `true`
3. 是否在登录后立即请求 API（页面刷新可能导致状态丢失）

### Q2: Token 存在但仍然 401
**可能原因**:
1. Token 格式错误（应为 JWT 格式，3 段用 `.` 分隔）
2. 后端密钥更改导致旧 token 无效
3. Token 已过期（JWT 有效期默认 1 小时）

**验证方法**:
```bash
# 使用 curl 测试后端
curl -H "Authorization: Bearer <YOUR_TOKEN>" http://localhost:8088/api/v1/limit-orders
```

> 注意：后端默认端口为 8088（除非你在配置里改过）。

### Q3: 页面刷新后 401
**原因**: `AppState.api` 没有从 LocalStorage 恢复 token
**解决方案**: 确保 `UserState::load()` 被正确调用（在 `AppState::new()` 中已实现）

## 下一步行动

### 如果诊断脚本显示 Token 有效 (200 OK)
1. 在浏览器控制台查找 Rust/WASM 日志
2. 查找 "API Request: Adding Authorization header" 日志（有 token）
3. 查找 "API Request: No auth token available" 警告（无 token）
4. 截图发送日志

### 如果诊断脚本显示 Token 无效 (401)
1. 重新登录
2. 再次运行诊断脚本
3. 如果仍然 401，检查后端日志

### 如果诊断脚本显示 Token 不存在 (NULL)
1. 检查登录流程是否成功
2. 查看浏览器控制台是否有 JavaScript/Rust 错误
3. 检查 `AuthController::login()` 是否抛出异常

## 临时解决方案

如果问题持续，可以尝试：
1. 清除浏览器缓存和 LocalStorage：
   ```javascript
   localStorage.clear();
   location.reload();
   ```
2. 重新登录
3. 检查后端是否正常运行：`curl http://localhost:3012/api/health`

> 默认本地后端端口为 8088：`curl http://localhost:8088/api/health`（也可用 `curl http://localhost:8088/healthz`）

## 联系支持

如果以上步骤都无法解决，请提供：
1. 诊断脚本的完整输出（截图）
2. 浏览器控制台的 Rust/WASM 日志（截图）
3. 后端日志（如果可以访问）
