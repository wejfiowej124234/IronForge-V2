# 路由系统状态

> **日期**: 2025-11-27  
> **状态**: 已实现基础路由

---

## ✅ 已实现的路由

### 路由定义 (`src/router.rs`)

```rust
pub enum Route {
    #[route("/")]
    Landing {},
    
    #[route("/dashboard")]
    Dashboard {},
    
    #[route("/wallet/create")]
    CreateWallet {},
    
    #[route("/wallet/import")]
    ImportWallet {},
    
    #[route("/wallet/:id")]
    WalletDetail { id: String },
    
    #[route("/send")]
    Send {},
    
    #[route("/receive")]
    Receive {},
    
    #[route("/settings")]
    Settings {},
    
    #[route("/..")]
    NotFound {},
}
```

---

## 📦 已连接的页面

- ✅ `/` - Landing Page (营销首页)
- ✅ `/dashboard` - Dashboard Page (仪表盘)
- ✅ `/wallet/create` - Create Wallet Page (创建钱包)

---

## 🚧 待实现的页面

- ⏳ `/wallet/import` - Import Wallet Page (导入钱包)
- ⏳ `/wallet/:id` - Wallet Detail Page (钱包详情)
- ⏳ `/send` - Send Page (发送)
- ⏳ `/receive` - Receive Page (接收)
- ⏳ `/settings` - Settings Page (设置)

---

## 🔧 技术实现

- 使用 Dioxus Router (内置在 dioxus 0.7 中)
- 类型安全的路由定义
- 支持动态路由参数 (`:id`)

---

**最后更新**: 2025-11-27

