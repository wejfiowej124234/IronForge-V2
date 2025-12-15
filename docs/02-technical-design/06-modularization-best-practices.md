# IronForge 模块化架构最佳实践

> 📚 **版本**: 2.0  
> 🎯 **目标**: 企业级模块化设计，防止代码腐化，实现最小粒度隔离  
> 🔗 **关联**: [系统架构](../01-architecture/01-system-architecture.md) | [开发规范](./04-development-guide.md)

---

## 📋 目录

1. [模块化设计原则](#模块化设计原则)
2. [按链分组架构](#按链分组架构)
3. [功能模块分组](#功能模块分组)
4. [错误边界组件](#错误边界组件)
5. [懒加载支持](#懒加载支持)
6. [页面独立模块](#页面独立模块)
7. [模块隔离策略](#模块隔离策略)
8. [防腐化层设计](#防腐化层设计)

---

## 🎯 模块化设计原则

### 核心原则

#### 1. **单一职责原则 (SRP)**
```rust
// ✅ 好的设计 - 每个模块职责单一
// src/crypto/bip39/mod.rs
pub mod generator;    // 只负责生成助记词
pub mod validator;    // 只负责验证助记词
pub mod wordlist;     // 只负责词库管理

// ❌ 坏的设计 - 职责混乱
// src/crypto/bip39.rs - 包含生成、验证、词库、派生所有逻辑（3000+行）
```

#### 2. **开闭原则 (OCP)**
```rust
// ✅ 对扩展开放，对修改封闭
pub trait ChainAdapter: Send + Sync {
    async fn get_balance(&self, address: &str) -> Result<Balance>;
    async fn send_transaction(&self, tx: Transaction) -> Result<TxHash>;
}

// 新增链无需修改现有代码
pub struct SolanaAdapter;
impl ChainAdapter for SolanaAdapter { /* ... */ }
```

#### 3. **依赖倒置原则 (DIP)**
```rust
// ✅ 依赖抽象，不依赖具体实现
pub struct WalletService {
    storage: Arc<dyn StorageAdapter>,    // 抽象
    crypto: Arc<dyn CryptoAdapter>,      // 抽象
}

// ❌ 直接依赖具体实现
pub struct WalletService {
    storage: IndexedDbStorage,  // 具体实现（难以测试/替换）
}
```

#### 4. **接口隔离原则 (ISP)**
```rust
// ✅ 接口最小化
pub trait BalanceProvider {
    async fn get_balance(&self, address: &str) -> Result<Balance>;
}

pub trait TransactionSender {
    async fn send_transaction(&self, tx: Transaction) -> Result<TxHash>;
}

// ❌ 臃肿接口
pub trait BlockchainClient {
    // 包含20+个方法，大部分客户端只需要其中2-3个
}
```

---

## 🔗 按链分组架构

### 1. 目录结构（按链隔离）

```
src/blockchain/
├── mod.rs                    # 统一导出
├── traits.rs                 # 通用 trait 定义
├── registry.rs               # 链注册中心
│
├── ethereum/                 # Ethereum 生态（独立模块）
│   ├── mod.rs
│   ├── client.rs            # ETH 客户端
│   ├── types.rs             # ETH 专用类型
│   ├── gas_estimator.rs     # Gas 估算
│   ├── erc20.rs             # ERC20 代币
│   ├── erc721.rs            # NFT 支持
│   └── tests.rs             # 单元测试
│
├── bsc/                      # BSC 生态（独立模块）
│   ├── mod.rs
│   ├── client.rs
│   ├── bep20.rs             # BEP20 代币
│   └── pancakeswap.rs       # PancakeSwap 集成
│
├── polygon/                  # Polygon 生态
│   ├── mod.rs
│   ├── client.rs
│   ├── matic_bridge.rs      # Matic 桥
│   └── quickswap.rs
│
├── bitcoin/                  # Bitcoin 生态
│   ├── mod.rs
│   ├── client.rs
│   ├── utxo_manager.rs      # UTXO 管理
│   ├── fee_estimator.rs     # 费用估算
│   ├── bip84.rs             # Bech32 地址
│   ├── psbt.rs              # PSBT 支持
│   └── lightning.rs         # Lightning Network
│
├── solana/                   # Solana 生态（规划中）
│   ├── mod.rs
│   ├── client.rs
│   ├── spl_token.rs         # SPL Token
│   ├── account_manager.rs
│   └── borsh_utils.rs
│
└── cosmos/                   # Cosmos 生态（规划中）
    ├── mod.rs
    ├── client.rs
    └── ibc_bridge.rs        # IBC 跨链
```

### 2. 链注册中心（动态加载）

```rust
// src/blockchain/registry.rs
use std::collections::HashMap;
use std::sync::Arc;

pub struct ChainRegistry {
    adapters: HashMap<String, Arc<dyn ChainAdapter>>,
}

impl ChainRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            adapters: HashMap::new(),
        };
        
        // 注册所有支持的链
        registry.register("ethereum", Arc::new(ethereum::EthereumAdapter::new()));
        registry.register("bsc", Arc::new(bsc::BscAdapter::new()));
        registry.register("polygon", Arc::new(polygon::PolygonAdapter::new()));
        registry.register("bitcoin", Arc::new(bitcoin::BitcoinAdapter::new()));
        
        // 条件编译 - 仅在启用 feature 时加载
        #[cfg(feature = "solana")]
        registry.register("solana", Arc::new(solana::SolanaAdapter::new()));
        
        registry
    }
    
    pub fn register(&mut self, name: &str, adapter: Arc<dyn ChainAdapter>) {
        self.adapters.insert(name.to_lowercase(), adapter);
    }
    
    pub fn get(&self, chain: &str) -> Option<Arc<dyn ChainAdapter>> {
        self.adapters.get(&chain.to_lowercase()).cloned()
    }
    
    pub fn supported_chains(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }
}

// 全局单例
lazy_static! {
    pub static ref CHAIN_REGISTRY: ChainRegistry = ChainRegistry::new();
}
```

### 3. 链选择器组件（UI）

```rust
// src/components/blockchain/chain_selector.rs
use dioxus::prelude::*;

#[component]
pub fn ChainSelector(
    selected_chain: Signal<String>,
    on_change: EventHandler<String>,
) -> Element {
    let chains = use_signal(|| vec![
        ChainInfo { id: "ethereum", name: "Ethereum", icon: "eth.svg", color: "#627EEA" },
        ChainInfo { id: "bsc", name: "BSC", icon: "bnb.svg", color: "#F3BA2F" },
        ChainInfo { id: "polygon", name: "Polygon", icon: "matic.svg", color: "#8247E5" },
        ChainInfo { id: "bitcoin", name: "Bitcoin", icon: "btc.svg", color: "#F7931A" },
    ]);
    
    rsx! {
        div { class: "chain-selector",
            for chain in chains() {
                ChainOption {
                    chain: chain.clone(),
                    selected: selected_chain() == chain.id,
                    on_click: move |_| on_change.call(chain.id.to_string()),
                }
            }
        }
    }
}

#[derive(Clone)]
struct ChainInfo {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    color: &'static str,
}
```

---

## 🧩 功能模块分组

### 1. 原子设计模式（Atomic Design）

```
src/components/
├── atoms/                    # 原子组件（最小单元）
│   ├── button.rs            # 按钮
│   ├── input.rs             # 输入框
│   ├── icon.rs              # 图标
│   ├── badge.rs             # 徽章
│   ├── spinner.rs           # 加载动画
│   └── tooltip.rs           # 提示框
│
├── molecules/               # 分子组件（原子组合）
│   ├── form_field.rs        # 表单字段 (label + input + error)
│   ├── search_box.rs        # 搜索框 (input + button)
│   ├── token_row.rs         # 代币行 (icon + name + balance)
│   └── transaction_item.rs  # 交易项 (icon + details + status)
│
├── organisms/               # 有机组件（复杂功能）
│   ├── wallet_card.rs       # 钱包卡片
│   ├── token_list.rs        # 代币列表
│   ├── transaction_history.rs # 交易历史
│   ├── send_form.rs         # 发送表单
│   └── navbar.rs            # 导航栏
│
├── templates/               # 模板（页面布局）
│   ├── dashboard_layout.rs  # 仪表盘布局
│   ├── auth_layout.rs       # 认证布局
│   └── modal_layout.rs      # 弹窗布局
│
└── pages/                   # 完整页面
    ├── home.rs
    ├── wallet.rs
    ├── send.rs
    └── settings.rs
```

### 2. 功能域模块（Feature Modules）

```
src/features/
├── wallet/                  # 钱包功能域
│   ├── mod.rs
│   ├── components/          # 钱包专用组件
│   │   ├── wallet_list.rs
│   │   ├── wallet_detail.rs
│   │   └── create_wallet_modal.rs
│   ├── services/            # 钱包服务
│   │   ├── wallet_service.rs
│   │   └── wallet_cache.rs
│   ├── state.rs             # 钱包状态
│   └── types.rs             # 钱包类型
│
├── transaction/             # 交易功能域
│   ├── mod.rs
│   ├── components/
│   │   ├── send_form.rs
│   │   ├── transaction_list.rs
│   │   └── transaction_detail.rs
│   ├── services/
│   │   ├── transaction_service.rs
│   │   └── gas_estimator.rs
│   ├── state.rs
│   └── types.rs
│
├── token/                   # 代币功能域
│   ├── mod.rs
│   ├── components/
│   │   ├── token_list.rs
│   │   ├── token_detail.rs
│   │   └── add_token_modal.rs
│   ├── services/
│   │   ├── token_detector.rs
│   │   └── token_price_service.rs
│   ├── state.rs
│   └── types.rs
│
└── auth/                    # 认证功能域
    ├── mod.rs
    ├── components/
    │   ├── login_form.rs
    │   └── unlock_modal.rs
    ├── services/
    │   └── auth_service.rs
    ├── state.rs
    └── types.rs
```

---

## 🛡️ 错误边界组件

### 1. 错误边界 Trait

```rust
// src/components/error_boundary/mod.rs
use dioxus::prelude::*;
use std::fmt;

pub trait ErrorRecovery: fmt::Display {
    fn error_code(&self) -> &'static str;
    fn is_recoverable(&self) -> bool;
    fn recovery_action(&self) -> Option<RecoveryAction>;
}

#[derive(Clone)]
pub enum RecoveryAction {
    Retry,
    Refresh,
    GoHome,
    Logout,
    ContactSupport,
}

// 定义错误级别
#[derive(Clone, Copy, PartialEq)]
pub enum ErrorLevel {
    Info,      // 信息提示
    Warning,   // 警告（可继续）
    Error,     // 错误（需要用户操作）
    Critical,  // 致命错误（需要重启/登出）
}
```

### 2. 错误边界组件实现

```rust
// src/components/error_boundary/boundary.rs
#[component]
pub fn ErrorBoundary(
    children: Element,
    fallback: Option<Element>,
    on_error: Option<EventHandler<AppError>>,
) -> Element {
    let error_state = use_signal(|| None::<AppError>);
    let retry_count = use_signal(|| 0);
    
    // 提供错误上下文给子组件
    use_context_provider(|| ErrorContext {
        set_error: move |err: AppError| {
            error_state.set(Some(err.clone()));
            if let Some(handler) = on_error {
                handler.call(err);
            }
        },
        clear_error: move || error_state.set(None),
    });
    
    match error_state() {
        Some(error) => {
            rsx! {
                ErrorDisplay {
                    error: error.clone(),
                    on_retry: move |_| {
                        retry_count.set(retry_count() + 1);
                        error_state.set(None);
                    },
                    on_dismiss: move |_| error_state.set(None),
                }
            }
        }
        None => children,
    }
}

// 错误上下文
#[derive(Clone, Copy)]
pub struct ErrorContext {
    pub set_error: fn(AppError),
    pub clear_error: fn(),
}

// 使用示例
#[component]
pub fn WalletPage() -> Element {
    rsx! {
        ErrorBoundary {
            on_error: move |err| {
                // 全局错误上报
                log_error(&err);
            },
            
            // 子组件可以安全抛出错误
            WalletList {}
            TransactionHistory {}
        }
    }
}
```

### 3. 分级错误处理

```rust
// src/components/error_boundary/display.rs
#[component]
pub fn ErrorDisplay(
    error: AppError,
    on_retry: EventHandler<()>,
    on_dismiss: EventHandler<()>,
) -> Element {
    let level = error.level();
    let icon = match level {
        ErrorLevel::Info => "ℹ️",
        ErrorLevel::Warning => "⚠️",
        ErrorLevel::Error => "❌",
        ErrorLevel::Critical => "🔥",
    };
    
    rsx! {
        div { class: "error-boundary {level}",
            div { class: "error-icon", "{icon}" }
            
            div { class: "error-content",
                h3 { "{error.title()}" }
                p { "{error.message()}" }
                
                // 根据错误级别显示不同操作
                match level {
                    ErrorLevel::Info | ErrorLevel::Warning => rsx! {
                        button { onclick: move |_| on_dismiss.call(()), "知道了" }
                    },
                    ErrorLevel::Error => rsx! {
                        button { onclick: move |_| on_retry.call(()), "重试" }
                        button { onclick: move |_| on_dismiss.call(()), "取消" }
                    },
                    ErrorLevel::Critical => rsx! {
                        button { onclick: move |_| {
                            // 清空缓存并重新登录
                            clear_storage();
                            navigate_to("/login");
                        }, "重新登录" }
                    },
                }
            }
            
            // 详细错误信息（开发模式）
            if cfg!(debug_assertions) {
                pre { class: "error-details",
                    "Error Code: {error.code()}\n"
                    "Stack: {error.backtrace()}"
                }
            }
        }
    }
}
```

### 4. 细粒度错误边界

```rust
// 为不同功能域设置独立错误边界
#[component]
pub fn Dashboard() -> Element {
    rsx! {
        div { class: "dashboard",
            // 钱包列表错误不影响交易历史
            ErrorBoundary {
                fallback: rsx! { WalletListSkeleton {} },
                WalletSection {}
            }
            
            // 交易历史错误不影响钱包列表
            ErrorBoundary {
                fallback: rsx! { TransactionHistorySkeleton {} },
                TransactionSection {}
            }
            
            // Token 列表错误不影响其他模块
            ErrorBoundary {
                fallback: rsx! { TokenListSkeleton {} },
                TokenSection {}
            }
        }
    }
}
```

---

## ⚡ 懒加载支持

### 1. 路由级别懒加载

```rust
// src/router/routes.rs
use dioxus::prelude::*;

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(MainLayout)]
        #[route("/")]
        Home {},
        
        // 懒加载 - 仅在访问时加载
        #[route("/wallet")]
        #[lazy]
        Wallet {},
        
        #[route("/send")]
        #[lazy]
        Send {},
        
        #[route("/settings")]
        #[lazy]
        Settings {},
}

// 懒加载实现
#[component(lazy)]
pub fn Wallet() -> Element {
    // 显示加载骨架
    let loaded = use_signal(|| false);
    
    use_effect(move || {
        spawn(async move {
            // 异步加载依赖
            load_wallet_module().await;
            loaded.set(true);
        });
    });
    
    if !loaded() {
        return rsx! { WalletSkeleton {} };
    }
    
    rsx! {
        div { class: "wallet-page",
            WalletList {}
            WalletDetails {}
        }
    }
}
```

### 2. 组件级别懒加载

```rust
// src/components/lazy_component.rs
#[component]
pub fn LazyComponent<T: Component>(
    loader: fn() -> T,
    fallback: Element,
) -> Element {
    let component = use_signal(|| None::<T>);
    let is_visible = use_signal(|| false);
    
    // 使用 Intersection Observer 检测可见性
    use_effect(move || {
        let observer = IntersectionObserver::new(move |entries| {
            if entries[0].is_intersecting() {
                is_visible.set(true);
            }
        });
        observer.observe(&element);
    });
    
    // 可见时才加载
    use_effect(move || {
        if is_visible() && component().is_none() {
            spawn(async move {
                let loaded = loader();
                component.set(Some(loaded));
            });
        }
    });
    
    match component() {
        Some(comp) => rsx! { {comp} },
        None => fallback,
    }
}

// 使用示例
#[component]
pub fn TokenList() -> Element {
    rsx! {
        div { class: "token-list",
            // 前5个立即渲染
            for token in tokens().iter().take(5) {
                TokenRow { token: token.clone() }
            }
            
            // 后续懒加载
            LazyComponent {
                loader: move || {
                    rsx! {
                        for token in tokens().iter().skip(5) {
                            TokenRow { token: token.clone() }
                        }
                    }
                },
                fallback: rsx! { 
                    div { "加载更多..." }
                },
            }
        }
    }
}
```

### 3. 数据懒加载（虚拟滚动）

```rust
// src/components/virtual_list.rs
#[component]
pub fn VirtualList<T: Clone + 'static>(
    items: Vec<T>,
    item_height: f64,
    render_item: fn(T) -> Element,
) -> Element {
    let scroll_top = use_signal(|| 0.0);
    let container_height = use_signal(|| 600.0);
    
    // 计算可见范围
    let visible_start = (scroll_top() / item_height).floor() as usize;
    let visible_count = (container_height() / item_height).ceil() as usize + 1;
    let visible_end = (visible_start + visible_count).min(items.len());
    
    // 仅渲染可见项
    let visible_items = items[visible_start..visible_end].to_vec();
    
    rsx! {
        div { 
            class: "virtual-list",
            style: "height: {container_height()}px; overflow-y: auto;",
            onscroll: move |evt| {
                scroll_top.set(evt.data.scroll_top());
            },
            
            // 占位符（保持滚动高度）
            div { 
                style: "height: {visible_start as f64 * item_height}px;" 
            }
            
            // 可见项
            for item in visible_items {
                {render_item(item)}
            }
            
            // 占位符
            div { 
                style: "height: {(items.len() - visible_end) as f64 * item_height}px;" 
            }
        }
    }
}

// 使用示例 - 1000+ 交易历史流畅滚动
#[component]
pub fn TransactionHistory() -> Element {
    let transactions = use_signal(|| fetch_transactions()); // 1000+ 条
    
    rsx! {
        VirtualList {
            items: transactions(),
            item_height: 72.0,
            render_item: |tx| rsx! {
                TransactionRow { transaction: tx }
            },
        }
    }
}
```

### 4. 图片懒加载

```rust
// src/components/lazy_image.rs
#[component]
pub fn LazyImage(
    src: String,
    alt: String,
    placeholder: Option<String>,
) -> Element {
    let loaded = use_signal(|| false);
    let is_visible = use_signal(|| false);
    let img_ref = use_node_ref();
    
    // Intersection Observer
    use_effect(move || {
        if let Some(element) = img_ref.get() {
            let observer = IntersectionObserver::new(move |entries| {
                if entries[0].is_intersecting() {
                    is_visible.set(true);
                }
            });
            observer.observe(&element);
        }
    });
    
    // 可见时加载图片
    use_effect(move || {
        if is_visible() && !loaded() {
            spawn(async move {
                preload_image(&src).await;
                loaded.set(true);
            });
        }
    });
    
    rsx! {
        div { 
            ref: img_ref,
            class: "lazy-image-container",
            
            if loaded() {
                img { src: "{src}", alt: "{alt}", class: "loaded" }
            } else {
                img { 
                    src: "{placeholder.unwrap_or("data:image/svg+xml;base64,...")}", 
                    alt: "{alt}",
                    class: "placeholder",
                }
            }
        }
    }
}
```

---

## 📄 页面独立模块

### 1. 页面模块结构

```
src/pages/
├── home/                    # 主页模块（完全独立）
│   ├── mod.rs              # 导出
│   ├── index.rs            # 主页组件
│   ├── components/         # 私有组件
│   │   ├── hero_section.rs
│   │   ├── feature_list.rs
│   │   └── cta_button.rs
│   ├── services/           # 私有服务
│   │   └── analytics.rs
│   ├── state.rs            # 页面状态
│   ├── styles.css          # 页面样式
│   └── tests.rs            # 页面测试
│
├── wallet/                  # 钱包页面模块
│   ├── mod.rs
│   ├── index.rs
│   ├── components/
│   │   ├── wallet_header.rs
│   │   ├── balance_card.rs
│   │   └── quick_actions.rs
│   ├── hooks/
│   │   ├── use_wallet_data.rs
│   │   └── use_refresh.rs
│   ├── state.rs
│   ├── styles.css
│   └── tests.rs
│
└── send/                    # 发送页面模块
    ├── mod.rs
    ├── index.rs
    ├── components/
    │   ├── recipient_input.rs
    │   ├── amount_input.rs
    │   ├── gas_selector.rs
    │   └── confirmation_modal.rs
    ├── hooks/
    │   ├── use_gas_estimation.rs
    │   └── use_transaction.rs
    ├── validation.rs
    ├── state.rs
    ├── styles.css
    └── tests.rs
```

### 2. 页面模块模板

```rust
// src/pages/wallet/mod.rs
mod index;
mod components;
mod hooks;
mod state;

pub use index::WalletPage;
pub use state::WalletPageState;

// 页面级别的错误类型
#[derive(Debug, thiserror::Error)]
pub enum WalletPageError {
    #[error("Failed to load wallet data")]
    LoadError,
    #[error("Failed to refresh balance")]
    RefreshError,
}
```

```rust
// src/pages/wallet/index.rs
use super::components::*;
use super::hooks::*;
use super::state::*;

#[component]
pub fn WalletPage() -> Element {
    // 页面级别状态（不污染全局）
    let page_state = use_signal(WalletPageState::default);
    
    // 页面级别 hooks
    let wallet_data = use_wallet_data();
    let refresh = use_refresh();
    
    // 错误边界
    rsx! {
        ErrorBoundary {
            on_error: move |err| {
                log::error!("WalletPage error: {}", err);
            },
            
            div { class: "wallet-page",
                // 页面私有组件
                WalletHeader { 
                    wallet: wallet_data(),
                    on_refresh: move |_| refresh.call(()),
                }
                
                BalanceCard {
                    balance: wallet_data().balance,
                }
                
                QuickActions {
                    wallet_id: wallet_data().id,
                }
            }
        }
    }
}
```

### 3. 页面间通信（解耦）

```rust
// src/router/navigation.rs
use dioxus::prelude::*;

// 使用事件总线，而非直接依赖
pub struct NavigationEvent {
    pub from: String,
    pub to: String,
    pub params: Option<serde_json::Value>,
}

#[component]
pub fn NavigationProvider(children: Element) -> Element {
    let nav_events = use_signal(|| Vec::<NavigationEvent>::new());
    
    use_context_provider(|| NavigationContext {
        navigate: move |event: NavigationEvent| {
            nav_events.write().push(event.clone());
            // 使用 Router navigate
            navigate_to(&event.to, event.params);
        },
    });
    
    children
}

// 页面使用
#[component]
pub fn SendButton() -> Element {
    let nav = use_context::<NavigationContext>();
    
    rsx! {
        button {
            onclick: move |_| {
                nav.navigate(NavigationEvent {
                    from: "wallet".to_string(),
                    to: "/send".to_string(),
                    params: Some(json!({ "wallet_id": "..." })),
                });
            },
            "发送"
        }
    }
}
```

---

## 🔒 模块隔离策略

### 1. 可见性控制

```rust
// ✅ 严格的可见性控制
pub mod wallet {
    // 只暴露必要的公开 API
    pub use self::service::WalletService;
    pub use self::types::{Wallet, WalletId};
    
    // 内部实现隐藏
    mod service;     // 私有
    mod repository;  // 私有
    mod cache;       // 私有
    pub(crate) mod types;  // 模块内可见
    
    #[cfg(test)]
    mod tests;       // 测试专用
}

// ❌ 错误的可见性 - 暴露所有实现细节
pub mod wallet {
    pub mod service;      // ❌ 实现细节暴露
    pub mod repository;   // ❌ 实现细节暴露
    pub mod cache;        // ❌ 实现细节暴露
}
```

### 2. 依赖注入（防止循环依赖）

```rust
// src/di/container.rs
use std::sync::Arc;

pub struct AppContainer {
    pub wallet_service: Arc<dyn WalletService>,
    pub transaction_service: Arc<dyn TransactionService>,
    pub storage: Arc<dyn StorageAdapter>,
}

impl AppContainer {
    pub fn new() -> Self {
        let storage = Arc::new(IndexedDbStorage::new());
        let wallet_service = Arc::new(WalletServiceImpl::new(storage.clone()));
        let transaction_service = Arc::new(TransactionServiceImpl::new(
            wallet_service.clone(),
            storage.clone(),
        ));
        
        Self {
            wallet_service,
            transaction_service,
            storage,
        }
    }
}

// 使用依赖注入容器
#[component]
pub fn App() -> Element {
    let container = use_signal(|| Arc::new(AppContainer::new()));
    
    use_context_provider(|| container());
    
    rsx! {
        Router::<Route> {}
    }
}

// 组件中使用
#[component]
pub fn WalletList() -> Element {
    let container = use_context::<Arc<AppContainer>>();
    let wallet_service = &container.wallet_service;
    
    // 使用 service...
}
```

### 3. Feature Flags（条件编译）

```toml
# Cargo.toml
[features]
default = ["ethereum", "bsc", "polygon"]

# 区块链支持
ethereum = ["ethers"]
bsc = ["ethers"]
polygon = ["ethers"]
bitcoin = ["bitcoin", "bitcoincore-rpc"]
solana = ["solana-sdk", "solana-client"]

# 硬件钱包
ledger = ["ledger-transport", "ledger-apdu"]
trezor = ["trezor-client"]

# 可选功能
analytics = ["mixpanel"]
sentry = ["sentry-rust"]
```

```rust
// 条件编译示例
#[cfg(feature = "ethereum")]
pub mod ethereum {
    pub struct EthereumAdapter;
    // ...
}

#[cfg(feature = "solana")]
pub mod solana {
    pub struct SolanaAdapter;
    // ...
}

// 运行时检查
pub fn is_chain_supported(chain: &str) -> bool {
    match chain {
        "ethereum" => cfg!(feature = "ethereum"),
        "solana" => cfg!(feature = "solana"),
        "bitcoin" => cfg!(feature = "bitcoin"),
        _ => false,
    }
}
```

---

## 🛡️ 防腐化层设计

### 1. 外部依赖适配器

```rust
// src/adapters/storage/mod.rs
// 防止外部库变更影响核心业务

// 定义自己的 trait（防腐化层）
#[async_trait]
pub trait StorageAdapter: Send + Sync {
    async fn set(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn remove(&self, key: &str) -> Result<()>;
}

// IndexedDB 适配器
pub struct IndexedDbAdapter {
    db: IdbDatabase,
}

#[async_trait]
impl StorageAdapter for IndexedDbAdapter {
    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        // 调用外部库 (indexed_db_futures)
        let tx = self.db.transaction(&["data"], IdbTransactionMode::Readwrite)?;
        let store = tx.object_store("data")?;
        store.put(&JsValue::from(value), &JsValue::from(key)).await?;
        Ok(())
    }
    // ...
}

// LocalStorage 适配器（同样实现 StorageAdapter）
pub struct LocalStorageAdapter;

#[async_trait]
impl StorageAdapter for LocalStorageAdapter {
    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        // 调用 Web API
        window().local_storage()?.set_item(key, &base64::encode(value))?;
        Ok(())
    }
    // ...
}

// 业务代码只依赖 StorageAdapter trait，不依赖具体实现
pub struct WalletService {
    storage: Arc<dyn StorageAdapter>,  // 抽象依赖
}
```

### 2. API 适配器

```rust
// src/adapters/api/mod.rs
// 防止后端 API 变更影响前端

// 内部数据模型（稳定）
#[derive(Debug, Clone)]
pub struct Wallet {
    pub id: String,
    pub name: String,
    pub address: String,
    pub chain: ChainType,
    pub balance: Balance,
}

// 外部 API 响应（可能变化）
#[derive(Deserialize)]
struct ApiWalletResponse {
    wallet_id: String,
    wallet_name: String,
    wallet_address: String,
    chain_type: String,
    balance_wei: String,
}

// 适配器（转换层）
pub struct ApiAdapter;

impl ApiAdapter {
    pub async fn fetch_wallet(wallet_id: &str) -> Result<Wallet> {
        // 调用外部 API
        let response: ApiWalletResponse = reqwest::get(format!("/api/wallets/{}", wallet_id))
            .await?
            .json()
            .await?;
        
        // 转换为内部模型（防腐化）
        Ok(Wallet {
            id: response.wallet_id,
            name: response.wallet_name,
            address: response.wallet_address,
            chain: ChainType::from_str(&response.chain_type)?,
            balance: Balance::from_wei(&response.balance_wei)?,
        })
    }
}
```

### 3. 版本隔离

```rust
// src/api/versions/mod.rs
pub mod v1;  // 旧版本 API
pub mod v2;  // 新版本 API

// 版本路由
#[derive(Routable)]
pub enum ApiRoute {
    #[route("/api/v1/*")]
    V1(v1::V1Route),
    
    #[route("/api/v2/*")]
    V2(v2::V2Route),
}

// 逐步迁移，新老版本共存
pub struct ApiClient {
    version: ApiVersion,
}

impl ApiClient {
    pub async fn fetch_wallet(&self, wallet_id: &str) -> Result<Wallet> {
        match self.version {
            ApiVersion::V1 => v1::fetch_wallet(wallet_id).await,
            ApiVersion::V2 => v2::fetch_wallet(wallet_id).await,
        }
    }
}
```

---

## 📊 模块化健康度检查清单

### 1. 模块独立性检查

```bash
# 使用 cargo-modules 可视化模块依赖
cargo install cargo-modules
cargo modules generate graph --lib | dot -Tpng > modules.png

# 检查循环依赖
cargo modules graph --lib --dependencies | grep "cycle"

# 检查模块耦合度
cargo clippy -- -W clippy::module_inception
```

### 2. 代码质量指标

| 指标 | 目标值 | 检查方法 |
|------|--------|----------|
| 单文件行数 | <500 行 | `find src -name "*.rs" -exec wc -l {} \; \| sort -n` |
| 单函数行数 | <50 行 | Clippy: `cognitive_complexity` |
| 模块依赖深度 | <5 层 | `cargo-modules` |
| 循环依赖 | 0 个 | `cargo-modules graph` |
| 公开 API 占比 | <20% | Clippy: `missing_docs` |

### 3. 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir ./coverage

# 目标覆盖率
# - 核心模块: >90%
# - 业务模块: >80%
# - UI 组件: >70%
```

---

## 🎯 最佳实践总结

### ✅ DO（应该做）

1. **一个模块只做一件事** - 单一职责
2. **使用 trait 抽象依赖** - 依赖倒置
3. **严格控制可见性** - 最小化公开 API
4. **按功能域组织代码** - 而非按技术层
5. **使用错误边界隔离故障** - 防止级联失败
6. **懒加载非关键资源** - 提升首屏性能
7. **每个页面独立模块** - 可单独测试/部署
8. **使用适配器模式** - 隔离外部依赖
9. **Feature flags 控制** - 按需编译
10. **定期检查模块健康度** - 防止腐化

### ❌ DON'T（不应该做）

1. **❌ 创建 God Object** - 一个模块包含所有功能
2. **❌ 循环依赖** - A 依赖 B，B 又依赖 A
3. **❌ 暴露实现细节** - 所有字段/方法都是 pub
4. **❌ 硬编码依赖** - 直接 new 具体类型
5. **❌ 全局状态滥用** - 所有状态都放 static
6. **❌ 跨层直接调用** - UI 直接调用 Storage
7. **❌ 混合技术关注点** - 业务逻辑混入 UI
8. **❌ 缺少错误边界** - 一处崩溃全局崩溃
9. **❌ 同步加载所有资源** - 首屏加载慢
10. **❌ 紧耦合外部库** - 库升级导致大量改动

---

## 📚 相关文档

- [系统架构设计](../01-architecture/01-system-architecture.md)
- [开发规范与最佳实践](./04-development-guide.md)
- [状态管理架构](./03-state-management.md)
- [错误处理设计](../03-api-design/03-error-handling.md)
- [测试策略](../07-testing/01-testing-strategy.md)

---

**批准**: ✅ 架构审核通过  
**版本**: 2.0  
**最后更新**: 2025-11-25
