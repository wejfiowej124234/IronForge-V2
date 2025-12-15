# Dioxus 状态管理架构

> **版本**: V2.0  
> **技术栈**: Dioxus 0.7 Signals + Context API + LocalStorage  
> **更新日期**: 2025-11-25  
> **适用范围**: IronForge 全局状态管理

---

## 📋 目录

1. [架构概览](#架构概览)
2. [状态分层设计](#状态分层设计)
3. [全局状态实现](#全局状态实现)
4. [状态持久化](#状态持久化)
5. [性能优化](#性能优化)
6. [最佳实践](#最佳实践)
7. [完整示例](#完整示例)
8. [测试策略](#测试策略)

---

## 架构概览

### 设计原则

1. **单一数据源 (Single Source of Truth)**: 全局状态统一管理
2. **响应式更新**: Signal 自动追踪依赖，精准更新
3. **类型安全**: 所有状态都有明确的 Rust 类型
4. **持久化分离**: 敏感数据走 IndexedDB 加密，偏好走 LocalStorage
5. **测试友好**: 状态逻辑可独立测试

### 状态流向

```
┌─────────────────────────────────────────────────────────┐
│                    UI Components                         │
│  (WalletPage, SendPage, SettingsPage)                   │
└────────────┬────────────────────────────────────────────┘
             │ read/write via hooks
             ▼
┌─────────────────────────────────────────────────────────┐
│              Global State (Dioxus Context)              │
│  - AppState (use_context)                               │
│  - WalletState (Signal)                                 │
│  - UserPreferences (Signal)                             │
│  - TransactionState (Signal)                            │
└────────────┬────────────────────────────────────────────┘
             │ sync to storage
             ▼
┌─────────────────────────────────────────────────────────┐
│                   Persistence Layer                      │
│  - LocalStorage (偏好、缓存)                             │
│  - IndexedDB (加密钱包数据)                              │
│  - SessionStorage (临时会话密钥)                         │
└─────────────────────────────────────────────────────────┘
```

---

## 状态分层设计

### 1. 全局应用状态 (AppState)

```rust
// src/state/app_state.rs
use dioxus::prelude::*;
use std::sync::Arc;

/// 全局应用状态（通过 Context 注入）
#[derive(Clone)]
pub struct AppState {
    /// 用户认证状态
    pub auth: Signal<UserAuthState>,
    /// 钱包状态
    pub wallet: Signal<WalletState>,
    /// 用户偏好
    pub preferences: Signal<UserPreferences>,
    /// 交易状态
    pub transaction: Signal<TransactionState>,
    /// 网络状态
    pub network: Signal<NetworkState>,
    /// UI 状态
    pub ui: Signal<UiState>,
}

impl AppState {
    /// 初始化应用状态（从 LocalStorage 恢复）
    pub async fn new() -> Self {
        let auth = Signal::new(UserAuthState::load_from_storage().await);
        let wallet = Signal::new(WalletState::load_from_storage().await);
        let preferences = Signal::new(UserPreferences::load_from_storage().await);
        let transaction = Signal::new(TransactionState::default());
        let network = Signal::new(NetworkState::default());
        let ui = Signal::new(UiState::default());

        Self {
            auth,
            wallet,
            preferences,
            transaction,
            network,
            ui,
        }
    }
}

/// 在 App 根组件中注入
pub fn App() -> Element {
    // 初始化全局状态（仅执行一次）
    use_context_provider(|| {
        spawn(async {
            AppState::new().await
        })
    });

    rsx! {
        Router::<Route> {}
    }
}
```

### 2. 用户认证状态 (UserAuthState)

```rust
// src/state/auth_state.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserAuthState {
    /// 用户是否已登录
    pub is_authenticated: bool,
    /// 用户 ID
    pub user_id: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// JWT Token
    pub jwt_token: Option<String>,
    /// Token 过期时间（Unix 时间戳）
    pub token_expires_at: Option<u64>,
}

impl UserAuthState {
    /// 从 LocalStorage 加载
    pub async fn load_from_storage() -> Self {
        match web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item("ironforge_auth_state").ok().flatten())
        {
            Some(json) => serde_json::from_str(&json).unwrap_or_default(),
            None => Self::default(),
        }
    }

    /// 保存到 LocalStorage
    pub async fn save_to_storage(&self) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
        {
            let json = serde_json::to_string(self).unwrap();
            let _ = storage.set_item("ironforge_auth_state", &json);
        }
    }

    /// 设置登录状态
    pub fn set_authenticated(&mut self, user_id: String, email: String, jwt_token: String, expires_at: u64) {
        self.is_authenticated = true;
        self.user_id = Some(user_id);
        self.email = Some(email);
        self.jwt_token = Some(jwt_token);
        self.token_expires_at = Some(expires_at);
    }

    /// 登出
    pub fn logout(&mut self) {
        self.is_authenticated = false;
        self.user_id = None;
        self.email = None;
        self.jwt_token = None;
        self.token_expires_at = None;
    }

    /// 检查 Token 是否过期
    pub fn is_token_expired(&self) -> bool {
        match self.token_expires_at {
            Some(expires_at) => current_timestamp() > expires_at,
            None => true,
        }
    }
}
```

### 3. 钱包状态 (WalletState)

```rust
// src/state/wallet_state.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WalletState {
    /// 当前活跃钱包
    pub active_wallet: Option<WalletMetadata>,
    /// 所有钱包列表（仅元数据，不含私钥）
    pub wallets: Vec<WalletMetadata>,
    /// 当前选择的链
    pub selected_chain: ChainId,
    /// 余额缓存（链ID -> 余额）
    pub balances: HashMap<ChainId, Balance>,
    /// 钱包是否已解锁
    pub is_unlocked: bool,
    /// 会话过期时间（Unix 时间戳）
    pub session_expires_at: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletMetadata {
    pub wallet_id: String,
    pub name: String,
    pub created_at: u64,
    /// 各链地址映射（ChainType -> 地址）
    pub addresses: HashMap<ChainType, String>,
    /// 是否已锁定
    pub is_locked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainType {
    Bitcoin,
    EVM, // Ethereum, BSC, Polygon
    Solana,
    TON,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Balance {
    pub native_balance: String,
    pub tokens: Vec<TokenBalance>,
    pub last_updated: u64,
}

impl WalletState {
    /// 从 LocalStorage 加载
    pub async fn load_from_storage() -> Self {
        match web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item("ironforge_wallet_state").ok().flatten())
        {
            Some(json) => serde_json::from_str(&json).unwrap_or_default(),
            None => Self::default(),
        }
    }

    /// 保存到 LocalStorage
    pub async fn save_to_storage(&self) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
        {
            let json = serde_json::to_string(self).unwrap();
            let _ = storage.set_item("ironforge_wallet_state", &json);
        }
    }

    /// 设置活跃钱包
    pub fn set_active_wallet(&mut self, wallet: WalletMetadata) {
        self.active_wallet = Some(wallet);
        self.is_unlocked = true;
        self.session_expires_at = Some(current_timestamp() + 15 * 60); // 15 分钟
    }

    /// 检查会话是否过期
    pub fn is_session_expired(&self) -> bool {
        match self.session_expires_at {
            Some(expires_at) => current_timestamp() > expires_at,
            None => true,
        }
    }

    /// 锁定钱包（清空敏感状态）
    pub fn lock(&mut self) {
        self.is_unlocked = false;
        self.session_expires_at = None;
        self.balances.clear();
    }
}
```

### 3. 用户偏好 (UserPreferences)

```rust
// src/state/preferences.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPreferences {
    /// 语言设置
    pub language: Language,
    /// 主题
    pub theme: Theme,
    /// 默认链
    pub default_chain: ChainId,
    /// 货币单位
    pub fiat_currency: String,
    /// Gas 设置偏好
    pub gas_preference: GasPreference,
    /// 通知设置
    pub notifications: NotificationSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Language {
    English,
    Chinese,
    Japanese,
    Korean,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    Auto, // 跟随系统
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GasPreference {
    Low,    // 慢速
    Medium, // 标准
    High,   // 快速
    Custom(String), // 自定义 Gwei
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub transaction_updates: bool,
    pub price_alerts: bool,
    pub security_alerts: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: Language::English,
            theme: Theme::Auto,
            default_chain: ChainId::Ethereum,
            fiat_currency: "USD".to_string(),
            gas_preference: GasPreference::Medium,
            notifications: NotificationSettings {
                transaction_updates: true,
                price_alerts: false,
                security_alerts: true,
            },
        }
    }
}

impl UserPreferences {
    pub async fn load_from_storage() -> Self {
        match web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item("ironforge_preferences").ok().flatten())
        {
            Some(json) => serde_json::from_str(&json).unwrap_or_default(),
            None => Self::default(),
        }
    }

    pub async fn save_to_storage(&self) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
        {
            let json = serde_json::to_string(self).unwrap();
            let _ = storage.set_item("ironforge_preferences", &json);
        }
    }
}
```

### 4. 交易状态 (TransactionState)

```rust
// src/state/transaction_state.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TransactionState {
    /// 待签名交易
    pub pending_transaction: Option<UnsignedTransaction>,
    /// 交易历史（最近 20 条，全量在 API）
    pub recent_transactions: Vec<TransactionRecord>,
    /// 交易构建状态
    pub build_state: TransactionBuildState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnsignedTransaction {
    pub from: String,
    pub to: String,
    pub value: String,
    pub chain_id: u64,
    pub gas_limit: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee: String,
    pub nonce: u64,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub status: TransactionStatus,
    pub timestamp: u64,
    pub chain_id: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum TransactionBuildState {
    #[default]
    Idle,
    BuildingTransaction,
    EstimatingGas,
    WaitingForSignature,
    Broadcasting,
    Confirmed(String), // tx_hash
    Failed(String),    // error message
}
```

---

## 全局状态实现

### 使用 Context 访问状态

```rust
// src/hooks/use_app_state.rs
use dioxus::prelude::*;
use crate::state::AppState;

/// 获取全局应用状态
pub fn use_app_state() -> AppState {
    use_context::<AppState>()
}

/// 获取钱包状态（只读）
pub fn use_wallet_state() -> Signal<WalletState> {
    use_app_state().wallet
}

/// 获取用户偏好（只读）
pub fn use_preferences() -> Signal<UserPreferences> {
    use_app_state().preferences
}

/// 获取交易状态（只读）
pub fn use_transaction_state() -> Signal<TransactionState> {
    use_app_state().transaction
}
```

### 自定义 Hooks

```rust
// src/hooks/use_wallet.rs
use dioxus::prelude::*;
use crate::state::{WalletState, WalletMetadata};

/// 钱包操作 Hook
pub fn use_wallet() -> WalletActions {
    let mut wallet_state = use_wallet_state();
    
    WalletActions {
        set_active_wallet: move |wallet: WalletMetadata| {
            wallet_state.write().set_active_wallet(wallet);
            spawn(async move {
                wallet_state.read().save_to_storage().await;
            });
        },
        lock_wallet: move || {
            wallet_state.write().lock();
            spawn(async move {
                wallet_state.read().save_to_storage().await;
            });
        },
        is_unlocked: move || wallet_state.read().is_unlocked,
        check_session: move || {
            if wallet_state.read().is_session_expired() {
                wallet_state.write().lock();
            }
        },
    }
}

pub struct WalletActions {
    pub set_active_wallet: Box<dyn Fn(WalletMetadata)>,
    pub lock_wallet: Box<dyn Fn()>,
    pub is_unlocked: Box<dyn Fn() -> bool>,
    pub check_session: Box<dyn Fn()>,
}
```

```rust
// src/hooks/use_preferences.rs
use dioxus::prelude::*;
use crate::state::{UserPreferences, Language, Theme};

pub fn use_preferences_actions() -> PreferencesActions {
    let mut preferences = use_preferences();
    
    PreferencesActions {
        set_language: move |lang: Language| {
            preferences.write().language = lang;
            spawn(async move {
                preferences.read().save_to_storage().await;
            });
        },
        set_theme: move |theme: Theme| {
            preferences.write().theme = theme;
            spawn(async move {
                preferences.read().save_to_storage().await;
            });
        },
        set_default_chain: move |chain: ChainId| {
            preferences.write().default_chain = chain;
            spawn(async move {
                preferences.read().save_to_storage().await;
            });
        },
    }
}

pub struct PreferencesActions {
    pub set_language: Box<dyn Fn(Language)>,
    pub set_theme: Box<dyn Fn(Theme)>,
    pub set_default_chain: Box<dyn Fn(ChainId)>,
}
```

---

## 状态持久化

### 持久化策略

| 状态类型 | 存储位置 | 加密 | 过期时间 | 原因 |
|---------|---------|------|---------|------|
| 用户偏好 | LocalStorage | ❌ | 永久 | 非敏感，需快速加载 |
| 钱包元数据 | LocalStorage | ❌ | 永久 | 仅地址、名称，无私钥 |
| 余额缓存 | LocalStorage | ❌ | 5 分钟 | 减少 API 请求 |
| 加密助记词 | IndexedDB | ✅ | 永久 | 敏感数据，需加密 |
| 会话密钥 | SessionStorage | ❌ | 15 分钟 | 临时解锁状态 |
| 交易草稿 | SessionStorage | ❌ | 会话结束 | 临时数据 |

### 自动持久化实现

```rust
// src/state/persistence.rs
use dioxus::prelude::*;
use std::time::Duration;

/// 自动持久化 Hook（定期保存到 LocalStorage）
pub fn use_auto_persist<T>(
    signal: Signal<T>,
    key: &'static str,
    interval_secs: u64,
) where
    T: serde::Serialize + Clone,
{
    use_effect(move || {
        spawn(async move {
            loop {
                gloo_timers::future::sleep(Duration::from_secs(interval_secs)).await;
                
                let data = signal.read().clone();
                if let Some(storage) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                {
                    if let Ok(json) = serde_json::to_string(&data) {
                        let _ = storage.set_item(key, &json);
                    }
                }
            }
        });
    });
}

/// 使用示例
pub fn SomeComponent() -> Element {
    let wallet_state = use_wallet_state();
    
    // 每 30 秒自动保存一次
    use_auto_persist(wallet_state, "ironforge_wallet_state", 30);
    
    rsx! {
        // ...
    }
}
```

---

## 性能优化

### 1. 细粒度更新（避免全局刷新）

```rust
// ❌ 错误：修改整个对象会触发所有依赖更新
wallet_state.write().clone_and_modify();

// ✅ 正确：仅修改需要的字段
wallet_state.with_mut(|state| {
    state.balances.insert(ChainId::Ethereum, new_balance);
});
```

### 2. 使用 Memo 缓存计算结果

```rust
// src/hooks/use_total_balance.rs
use dioxus::prelude::*;

/// 计算总余额（缓存结果）
pub fn use_total_balance() -> Signal<f64> {
    let wallet_state = use_wallet_state();
    
    use_memo(move || {
        wallet_state
            .read()
            .balances
            .values()
            .filter_map(|b| b.native_balance.parse::<f64>().ok())
            .sum()
    })
}
```

### 3. 避免不必要的序列化

```rust
// src/state/cache.rs
use std::sync::Arc;
use parking_lot::RwLock;

/// 内存缓存（不持久化）
pub struct MemoryCache<K, V> {
    data: Arc<RwLock<HashMap<K, V>>>,
}

impl<K: Eq + Hash, V: Clone> MemoryCache<K, V> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        self.data.read().get(key).cloned()
    }
    
    pub fn insert(&self, key: K, value: V) {
        self.data.write().insert(key, value);
    }
}

/// 使用示例：缓存 Gas 价格（避免频繁 API 调用）
static GAS_PRICE_CACHE: Lazy<MemoryCache<ChainId, GasPrice>> = Lazy::new(MemoryCache::new);
```

---

## 最佳实践

### 1. 状态命名规范

```rust
// ✅ 好的命名
pub struct WalletState { ... }
pub fn use_wallet_state() -> Signal<WalletState> { ... }
pub fn use_wallet_actions() -> WalletActions { ... }

// ❌ 避免的命名
pub struct State { ... } // 太泛化
pub fn get_wallet() -> ... // 不符合 Hook 规范
```

### 2. 避免循环依赖

```rust
// ❌ 错误：A 依赖 B，B 又依赖 A
pub fn ComponentA() -> Element {
    let state_b = use_context::<StateB>();
    // ...
}

pub fn ComponentB() -> Element {
    let state_a = use_context::<StateA>(); // 循环依赖！
    // ...
}

// ✅ 正确：统一放在 AppState 中管理
pub struct AppState {
    pub state_a: Signal<StateA>,
    pub state_b: Signal<StateB>,
}
```

### 3. 状态初始化顺序

```rust
// src/main.rs
fn main() {
    dioxus_web::launch(App);
}

fn App() -> Element {
    // 1. 首先初始化全局状态
    use_context_provider(|| async {
        AppState::new().await
    });
    
    // 2. 然后初始化服务（依赖全局状态）
    use_context_provider(|| {
        ApiClient::new(use_app_state().preferences.read().api_base_url.clone())
    });
    
    // 3. 最后渲染路由
    rsx! {
        Router::<Route> {}
    }
}
```

---

## 完整示例

### 钱包页面使用状态

```rust
// src/pages/wallet.rs
use dioxus::prelude::*;
use crate::{
    hooks::{use_wallet_state, use_wallet_actions, use_total_balance},
    components::WalletCard,
};

pub fn WalletPage() -> Element {
    let wallet_state = use_wallet_state();
    let wallet_actions = use_wallet_actions();
    let total_balance = use_total_balance();
    
    // 检查会话是否过期
    use_effect(move || {
        spawn(async move {
            loop {
                gloo_timers::future::sleep(Duration::from_secs(60)).await;
                (wallet_actions.check_session)();
            }
        });
    });
    
    // 如果未解锁，跳转到解锁页面
    if !(wallet_actions.is_unlocked)() {
        return rsx! {
            Redirect { to: Route::UnlockWallet {} }
        };
    }
    
    rsx! {
        div { class: "wallet-page",
            // 总余额显示
            div { class: "total-balance",
                h2 { "Total Balance" }
                span { class: "balance-amount", "${total_balance:.2}" }
            }
            
            // 钱包列表
            div { class: "wallet-list",
                {wallet_state.read().wallets.iter().map(|wallet| {
                    rsx! {
                        WalletCard {
                            wallet: wallet.clone(),
                            on_select: move |_| {
                                (wallet_actions.set_active_wallet)(wallet.clone());
                            }
                        }
                    }
                })}
            }
            
            // 锁定按钮
            button {
                onclick: move |_| (wallet_actions.lock_wallet)(),
                "🔒 Lock Wallet"
            }
        }
    }
}
```

---

## 测试策略

### 1. 状态逻辑单元测试

```rust
// tests/state/wallet_state_test.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_expiration() {
        let mut state = WalletState::default();
        
        // 设置会话过期时间为 1 秒后
        state.session_expires_at = Some(current_timestamp() + 1);
        assert!(!state.is_session_expired());
        
        // 等待 2 秒
        std::thread::sleep(Duration::from_secs(2));
        assert!(state.is_session_expired());
    }
    
    #[test]
    fn test_lock_clears_sensitive_data() {
        let mut state = WalletState {
            is_unlocked: true,
            balances: vec![(ChainId::Ethereum, Balance::default())].into_iter().collect(),
            ..Default::default()
        };
        
        state.lock();
        
        assert!(!state.is_unlocked);
        assert!(state.balances.is_empty());
    }
}
```

### 2. 持久化测试（WASM 环境）

```rust
// tests/wasm/persistence_test.rs
#[wasm_bindgen_test]
async fn test_preferences_persistence() {
    let prefs = UserPreferences {
        language: Language::Chinese,
        theme: Theme::Dark,
        ..Default::default()
    };
    
    prefs.save_to_storage().await;
    
    let loaded = UserPreferences::load_from_storage().await;
    assert_eq!(loaded.language, Language::Chinese);
    assert_eq!(loaded.theme, Theme::Dark);
}
```

---

## 调试工具

### Dioxus DevTools 集成

```rust
// src/dev_tools.rs
#[cfg(debug_assertions)]
pub fn install_devtools(app_state: AppState) {
    use dioxus_devtools::Devtools;
    
    Devtools::install(move || {
        vec![
            ("Wallet State", format!("{:#?}", app_state.wallet.read())),
            ("Preferences", format!("{:#?}", app_state.preferences.read())),
            ("Transaction State", format!("{:#?}", app_state.transaction.read())),
        ]
    });
}
```

---

## 参考资料

- [Dioxus 0.7 Signals 文档](https://dioxuslabs.com/learn/0.7/reference/signals)
- [Dioxus Context API](https://dioxuslabs.com/learn/0.7/reference/context)
- [React State Management Best Practices](https://react.dev/learn/managing-state)
- [Zustand State Management Philosophy](https://github.com/pmndrs/zustand)
