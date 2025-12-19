# IronForge V2 - 技术栈选型

> 📅 创建日期: 2025-11-25  
> 🛠️ 版本: 2.0  
> 🎯 状态: 设计阶段

---

## 📋 目录

- [选型原则](#选型原则)
- [核心技术栈](#核心技术栈)
- [依赖库](#依赖库)
- [开发工具](#开发工具)
- [对比分析](#对比分析)

---

## 🎯 选型原则

### 核心原则

1. **性能优先** - 选择高性能的技术和库
2. **安全第一** - 安全性经过验证的成熟方案
3. **生态成熟** - 活跃的社区和丰富的资源
4. **团队熟悉** - 降低学习成本
5. **长期维护** - 技术路线稳定，持续更新
6. **许可友好** - MIT/Apache 等宽松许可

### 评估维度

| 维度 | 权重 | 说明 |
|------|------|------|
| **性能** | 25% | 运行时性能、包体积、加载速度 |
| **安全** | 25% | 安全漏洞历史、审计情况 |
| **生态** | 20% | 社区活跃度、文档质量 |
| **成熟度** | 15% | 版本稳定性、生产案例 |
| **开发体验** | 15% | API 设计、调试工具 |

---

## 🏗️ 核心技术栈

### 1. 前端框架：Dioxus

#### 选择 Dioxus 的理由

**优势** ✅
- 100% Rust 编写，类型安全
- 优秀的 WASM 支持，编译产物小
- React-like API，学习曲线平缓
- 支持 SSR、桌面、移动端
- 活跃的社区，持续更新

**劣势** ⚠️
- 相对年轻，生态不如 React 丰富
- 部分高级特性还在完善中
- 官方文档仍在完善中（参考: https://dioxuslabs.com/learn/0.7/，项目内见 `docs-v2/` 架构文档）

**评分**: ⭐⭐⭐⭐⭐ (9/10)

#### 版本选择

```toml
[dependencies]
dioxus = { version = "0.7", features = ["web", "router"] }
dioxus-core = "0.7"
dioxus-logger = "0.7"
```

**理由**:
- Dioxus 0.7 是当前最新稳定版
- Signals API 进一步完善，状态管理更高效
- 内置路由系统，无需额外依赖

#### 替代方案对比

| 框架 | 优势 | 劣势 | 评分 |
|------|------|------|------|
| **Dioxus** | Rust 原生、性能好 | 生态年轻 | ⭐ 9/10 |
| Leptos | 性能极佳、Signal 原生 | API 复杂 | ⭐ 8/10 |
| Yew | 成熟稳定 | 性能一般 | ⭐ 7/10 |
| Sycamore | 轻量高效 | 社区小 | ⭐ 7/10 |

**结论**: 选择 **Dioxus** - 平衡性能、易用性、生态

---

### 2. 构建工具：Trunk

#### 选择 Trunk 的理由

```toml
# Installation
cargo install trunk

# 开发模式
trunk serve --open

# 生产构建
trunk build --release
```

**优势** ✅
- Rust WASM 开发标准工具
- 零配置，开箱即用
- 支持热重载
- 自动优化 WASM

**劣势** ⚠️
- 功能相对简单
- 自定义能力有限

**评分**: ⭐⭐⭐⭐⭐ (9/10)

#### 配置示例

```toml
# Trunk.toml
[build]
target = "index.html"
release = true
dist = "dist"

[watch]
ignore = ["dist", "target"]

[serve]
address = "127.0.0.1"
port = 8080
open = true
```

---

### 3. 状态管理：Dioxus Signals

#### 选择 Signals 的理由

```rust
use dioxus::prelude::*;

// 创建 Signal
let count = use_signal(cx, || 0);

// 读取
let value = count.read();

// 写入
count.write().add_assign(1);

// 派生状态
let doubled = use_memo(cx, |count| count.read() * 2);
```

**优势** ✅
- 细粒度响应式，性能极佳
- API 简洁，易于理解
- 自动依赖追踪
- 避免不必要的重渲染

**评分**: ⭐⭐⭐⭐⭐ (10/10)

---

### 4. 路由：Dioxus Router

```rust
use dioxus::prelude::*;
use dioxus_router::prelude::*;

#[derive(Routable, Clone)]
enum Route {
    #[route("/")]
    Home {},
    
    #[route("/wallet")]
    WalletList {},
    
    #[route("/wallet/:id")]
    WalletDetail { id: String },
}

fn App(cx: Scope) -> Element {
    render! {
        Router::<Route> {}
    }
}
```

**优势** ✅
- 类型安全的路由
- 支持嵌套路由
- 路由守卫
- 懒加载

**评分**: ⭐⭐⭐⭐⭐ (9/10)

---

## 📦 依赖库

### 加密库

#### 1. **ed25519-dalek** - Ed25519 签名

```toml
ed25519-dalek = { version = "2.1", features = ["wasm"] }
```

- ✅ WASM 支持
- ✅ 高性能
- ✅ 审计通过
- 用途: TON、Solana 等链的签名

#### 2. **k256** - secp256k1 签名

```toml
k256 = { version = "0.13", features = ["ecdsa", "wasm"] }
```

- ✅ EVM 链标准
- ✅ WASM 优化
- 用途: Ethereum、BSC、Polygon

#### 3. **aes-gcm** - AES-256-GCM 加密

```toml
aes-gcm = "0.10"
```

- ✅ AEAD 加密
- ✅ 安全性高
- 用途: 本地数据加密

#### 4. **argon2** - 密码哈希

```toml
argon2 = "0.5"
```

- ✅ 抗暴力破解
- ✅ 内存难度可调
- 用途: 密码加密

#### 5. **bip39** - 助记词

```toml
bip39 = "2.0"
```

- ✅ BIP39 标准
- ✅ 多语言支持
- 用途: 助记词生成/验证

---

### HTTP 客户端

#### **gloo-net** - WASM HTTP 客户端

```toml
gloo-net = { version = "0.5", features = ["http"] }
```

```rust
use gloo_net::http::Request;

let resp = Request::get("/api/v1/wallets")
    .send()
    .await?;

let wallets: Vec<Wallet> = resp.json().await?;
```

**优势** ✅
- WASM 原生支持
- 基于浏览器 Fetch API
- 轻量级

**评分**: ⭐⭐⭐⭐⭐ (9/10)

---

### 存储库

#### 1. **gloo-storage** - LocalStorage

```toml
gloo-storage = "0.3"
```

```rust
use gloo_storage::{LocalStorage, Storage};

// 保存
LocalStorage::set("key", "value")?;

// 读取
let value: String = LocalStorage::get("key")?;
```

#### 2. **indexed_db** - IndexedDB

```toml
indexed_db = "0.4"
```

- 用途: 大量数据存储（钱包、交易历史）

---

### 序列化

#### **serde** + **serde_json**

```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

```rust
#[derive(Serialize, Deserialize)]
struct Wallet {
    id: String,
    name: String,
    address: String,
}

// 序列化
let json = serde_json::to_string(&wallet)?;

// 反序列化
let wallet: Wallet = serde_json::from_str(&json)?;
```

---

### 日期时间

#### **chrono**

```toml
chrono = { version = "0.4", features = ["serde"] }
```

```rust
use chrono::{DateTime, Utc};

let now: DateTime<Utc> = Utc::now();
let formatted = now.format("%Y-%m-%d %H:%M:%S").to_string();
```

---

### 国际化 (i18n)

#### **fluent-rs** - Mozilla Fluent国际化

```toml
fluent = "0.16"
fluent-bundle = "0.15"
unic-langid = "0.9"
```

```rust
use fluent::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

// 初始化
let lang: LanguageIdentifier = "zh-CN".parse().unwrap();
let mut bundle = FluentBundle::new(vec![lang]);

// 加载翻译文件
let ftl_string = include_str!("locales/zh-CN.ftl");
let resource = FluentResource::try_new(ftl_string).unwrap();
bundle.add_resource(resource).unwrap();

// 获取翻译
let msg = bundle.get_message("wallet-create-button").unwrap();
let pattern = msg.value().unwrap();
let mut errors = vec![];
let value = bundle.format_pattern(&pattern, None, &mut errors);
println!("{}", value); // "创建钱包"
```

**支持的语言（7种）** 🌍

| 代码 | 语言 | 旗帜 | 状态 |
|------|------|------|------|
| `en` | English | 🇺🇸 | ✅ Complete |
| `es` | Español | 🇪🇸 | ✅ Complete |
| `fr` | Français | 🇫🇷 | ✅ Complete |
| `zh-CN` | 简体中文 | 🇨🇳 | ✅ Complete |
| `zh-TW` | 繁體中文 | 🇹🇼 | ✅ Complete |
| `ja` | 日本語 | 🇯🇵 | ✅ Complete |
| `ko` | 한국어 | 🇰🇷 | ✅ Complete |

**特性**：
- ✅ 动态语言切换（无需刷新页面）
- ✅ 支持复数形式（plurals）
- ✅ 支持参数插值
- ✅ RTL语言支持（未来）
- ✅ 翻译文件热重载（开发模式）

**翻译文件结构**：
```
src/i18n/
├── en.ftl          # English
├── es.ftl          # Español
├── fr.ftl          # Français
├── zh-CN.ftl       # 简体中文
├── zh-TW.ftl       # 繁體中文
├── ja.ftl          # 日本語
└── ko.ftl          # 한국어
```

**示例 .ftl 文件**：
```fluent
# en.ftl
wallet-create-button = Create Wallet
wallet-balance = Balance: { $amount } { $currency }
transaction-count = 
    { $count ->
        [one] { $count } transaction
       *[other] { $count } transactions
    }

# zh-CN.ftl
wallet-create-button = 创建钱包
wallet-balance = 余额：{ $amount } { $currency }
transaction-count = { $count } 笔交易
```

**评分**: ⭐⭐⭐⭐⭐ (10/10)

---

### 日志

#### **log** + **console_log**

```toml
log = "0.4"
console_log = "1.0"
```

```rust
use log::{info, warn, error};

console_log::init_with_level(log::Level::Debug).ok();

info!("Application started");
warn!("Warning message");
error!("Error occurred");
```

---

### 错误处理

#### **anyhow** + **thiserror**

```toml
anyhow = "1.0"
thiserror = "1.0"
```

```rust
// 应用层错误
use anyhow::{Result, Context};

fn load_wallet() -> Result<Wallet> {
    let data = read_storage()
        .context("Failed to read storage")?;
    Ok(data)
}

// 领域层错误
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Wallet not found")]
    NotFound,
    
    #[error("Invalid mnemonic")]
    InvalidMnemonic,
}
```

---

### 国际化

#### **fluent** (推荐) 或 自定义 JSON

```toml
fluent = "0.16"
fluent-bundle = "0.15"
```

**或自定义方案**:

```rust
// i18n/en.json
{
  "wallet.create": "Create Wallet",
  "wallet.import": "Import Wallet"
}

// 加载
use serde_json::Value;

let translations: Value = serde_json::from_str(include_str!("i18n/en.json"))?;
let text = translations["wallet.create"].as_str();
```

---

## 🛠️ 开发工具

### 代码质量

#### **clippy** - Linter

```bash
cargo clippy -- -D warnings
```

#### **rustfmt** - 代码格式化

```bash
cargo fmt --check
```

#### **cargo-audit** - 安全审计

```bash
cargo install cargo-audit
cargo audit
```

---

### 测试工具

#### **cargo-nextest** - 测试运行器

```bash
cargo install cargo-nextest
cargo nextest run
```

- ✅ 并行测试
- ✅ 更快的执行速度
- ✅ 漂亮的输出

---

### 性能分析

#### **wasm-pack** - WASM 构建

```bash
wasm-pack build --target web --release
```

#### **twiggy** - WASM 大小分析

```bash
cargo install twiggy
twiggy top target/wasm32-unknown-unknown/release/ironforge.wasm
```

---

### CI/CD

#### **GitHub Actions**

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all
      - run: cargo clippy -- -D warnings
```

---

## 📊 对比分析

### V1 vs V2 技术栈对比

| 类别 | V1 | V2 | 改进 |
|------|----|----|------|
| **前端框架** | Dioxus 0.6 | Dioxus 0.7 | Signals API |
| **状态管理** | 自定义 | Signals | ⬆️ 性能提升 |
| **路由** | 自定义 | Dioxus Router | ⬆️ 类型安全 |
| **HTTP** | gloo-net | gloo-net | - |
| **存储** | gloo-storage | gloo + IndexedDB | ⬆️ 更多选择 |
| **加密** | 混合 | 统一标准库 | ⬆️ 一致性 |
| **测试** | cargo test | nextest | ⬆️ 速度快 |

---

## ✅ 最终技术栈

### 完整 Cargo.toml 配置

```toml
[package]
name = "ironforge"
version = "2.0.0"
edition = "2021"
rust-version = "1.75"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
# ========================================
# 前端框架 - Dioxus 0.7
# ========================================
dioxus = { version = "0.7", features = ["web", "router"] }
dioxus-core = "0.7"
dioxus-logger = "0.7"

# ========================================
# WASM 绑定
# ========================================
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "Document",
    "Element",
    "HtmlElement",
    "Node",
    "Window",
    "Location",
    "History",
    "Storage",
    "console",
    # IndexedDB 相关
    "IdbFactory",
    "IdbDatabase",
    "IdbObjectStore",
    "IdbTransaction",
    "IdbRequest",
    # Crypto 相关
    "Crypto",
    "SubtleCrypto",
] }

# ========================================
# HTTP 客户端
# ========================================
gloo-net = { version = "0.5", features = ["http", "json"] }
gloo-utils = "0.2"

# ========================================
# 存储
# ========================================
gloo-storage = "0.3"
rexie = "0.5"              # IndexedDB (推荐，比 indexed_db 更好)

# ========================================
# 加密库 (WASM 兼容)
# ========================================
# BIP39/BIP32 助记词和密钥派生
bip39 = { version = "2.0", features = ["rand"] }
tiny-bip39 = "1.0"         # 备用方案

# Ed25519 签名 (TON, Solana)
ed25519-dalek = { version = "2.1", features = ["rand_core"] }
curve25519-dalek = { version = "4.1", features = ["rand_core"] }

# secp256k1 签名 (Ethereum, Bitcoin)
k256 = { version = "0.13", features = ["ecdsa", "sha256"] }
libsecp256k1 = { version = "0.7", features = ["hmac"] }

# 对称加密
aes-gcm = "0.10"
chacha20poly1305 = "0.10"  # 备用加密算法

# 密码哈希
argon2 = "0.5"
pbkdf2 = { version = "0.12", features = ["sha2"] }

# 哈希函数
sha2 = "0.10"
sha3 = "0.10"
blake3 = { version = "1.5", features = ["traits-preview"] }

# 随机数生成
getrandom = { version = "0.2", features = ["js"] }
rand = { version = "0.8", features = ["getrandom"] }

# 内存安全
zeroize = { version = "1.7", features = ["derive"] }

# ========================================
# 序列化
# ========================================
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde-wasm-bindgen = "0.6"
bincode = "1.3"

# ========================================
# 日志
# ========================================
log = "0.4"
console_log = "1.0"
console_error_panic_hook = "0.1"  # 更好的 panic 信息

# ========================================
# 错误处理
# ========================================
anyhow = "1.0"
thiserror = "1.0"

# ========================================
# 异步运行时
# ========================================
tokio = { version = "1.35", features = ["sync"] }
futures = "0.3"
futures-util = "0.3"

# ========================================
# 工具库
# ========================================
chrono = { version = "0.4", features = ["serde", "wasmbind"] }
uuid = { version = "1.0", features = ["v4", "serde", "js"] }
base64 = "0.21"
hex = "0.4"
urlencoding = "2.1"

# ========================================
# 响应式布局 CSS-in-Rust
# ========================================
# Dioxus 内置样式支持，无需额外依赖
# 使用 style! 宏或内联样式

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
# WASM 优化配置
opt-level = "z"        # 优化体积
lto = true             # Link Time Optimization
codegen-units = 1      # 单个代码生成单元
panic = "abort"        # panic 时直接 abort
strip = true           # 移除符号信息

[profile.dev]
opt-level = 1          # 开发时适度优化
```

### Trunk.toml 配置

```toml
[build]
# 目标文件
target = "index.html"

# 输出目录
dist = "dist"

# 发布模式
release = true

# 公共路径（CDN 部署时修改）
public_url = "/"

# ========================================
# WASM 优化
# ========================================
[build.wasm_opt]
# 启用 wasm-opt 优化
enabled = true
# 优化级别：0-4 或 "z" (最小体积) 或 "s" (体积优先)
level = "z"

# ========================================
# 监听配置
# ========================================
[watch]
# 忽略的目录
ignore = [
    "dist",
    "target",
    ".git",
    "node_modules",
]

# ========================================
# 开发服务器
# ========================================
[serve]
# 绑定地址
address = "127.0.0.1"

# 端口
port = 8080

# 自动打开浏览器
open = true

# 启用热重载
reload = true

# WebSocket 端口
ws_port = 8081

# 代理配置（转发 API 请求到后端）
[[serve.proxy]]
backend = "http://localhost:8088"
path = "/api"

# ========================================
# 资源处理
# ========================================
[build.hooks]
# 构建前钩子（可选）
# pre_build = "npm run build-css"

# 构建后钩子（可选）
# post_build = "echo 'Build completed'"
```

### index.html 配置

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    
    <!-- PWA 支持 -->
    <meta name="theme-color" content="#8B5CF6">
    <meta name="mobile-web-app-capable" content="yes">
    <meta name="apple-mobile-web-app-capable" content="yes">
    <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
    
    <title>IronForge - Web3 Wallet</title>
    
    <!-- 预加载关键资源 -->
    <link rel="preconnect" href="https://fonts.googleapis.com">
    
    <!-- 全局 CSS 变量（响应式布局基础） -->
    <style>
        :root {
            /* ============================================
               响应式断点
               ============================================ */
            --breakpoint-mobile: 480px;
            --breakpoint-tablet: 768px;
            --breakpoint-desktop: 1024px;
            --breakpoint-wide: 1440px;
            
            /* ============================================
               间距系统（响应式）
               ============================================ */
            --spacing-xs: 0.25rem;   /* 4px */
            --spacing-sm: 0.5rem;    /* 8px */
            --spacing-md: 1rem;      /* 16px */
            --spacing-lg: 1.5rem;    /* 24px */
            --spacing-xl: 2rem;      /* 32px */
            --spacing-2xl: 3rem;     /* 48px */
            
            /* ============================================
               字体系统（响应式）
               ============================================ */
            --font-size-xs: 0.75rem;    /* 12px */
            --font-size-sm: 0.875rem;   /* 14px */
            --font-size-base: 1rem;     /* 16px */
            --font-size-lg: 1.125rem;   /* 18px */
            --font-size-xl: 1.25rem;    /* 20px */
            --font-size-2xl: 1.5rem;    /* 24px */
            --font-size-3xl: 1.875rem;  /* 30px */
            --font-size-4xl: 2.25rem;   /* 36px */
            
            /* ============================================
               容器宽度
               ============================================ */
            --container-sm: 640px;
            --container-md: 768px;
            --container-lg: 1024px;
            --container-xl: 1280px;
            --container-2xl: 1536px;
        }
        
        /* ============================================
           全局重置
           ============================================ */
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        html {
            font-size: 16px;
            /* 移动端字体放大 */
            -webkit-text-size-adjust: 100%;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 
                         'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 
                         'Fira Sans', 'Droid Sans', 'Helvetica Neue', 
                         sans-serif;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
            background: #0A0A0F;
            color: #FFFFFF;
            overflow-x: hidden;
        }
        
        /* ============================================
           响应式容器类
           ============================================ */
        .container {
            width: 100%;
            margin: 0 auto;
            padding: 0 var(--spacing-md);
        }
        
        /* 移动端 */
        @media (min-width: 640px) {
            .container { max-width: var(--container-sm); }
        }
        
        /* 平板 */
        @media (min-width: 768px) {
            .container { 
                max-width: var(--container-md); 
                padding: 0 var(--spacing-lg);
            }
        }
        
        /* 桌面 */
        @media (min-width: 1024px) {
            .container { 
                max-width: var(--container-lg);
                padding: 0 var(--spacing-xl);
            }
        }
        
        /* 宽屏 */
        @media (min-width: 1280px) {
            .container { max-width: var(--container-xl); }
        }
        
        @media (min-width: 1536px) {
            .container { max-width: var(--container-2xl); }
        }
        
        /* ============================================
           加载动画
           ============================================ */
        .loading-screen {
            position: fixed;
            inset: 0;
            background: #0A0A0F;
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 9999;
        }
        
        .spinner {
            width: 50px;
            height: 50px;
            border: 4px solid rgba(139, 92, 246, 0.3);
            border-top-color: #8B5CF6;
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }
        
        @keyframes spin {
            to { transform: rotate(360deg); }
        }
    </style>
</head>
<body>
    <!-- 加载屏幕 -->
    <div class="loading-screen">
        <div class="spinner"></div>
    </div>
    
    <!-- Dioxus 挂载点 -->
    <div id="main"></div>
    
    <!-- Service Worker 注册 -->
    <script>
        if ('serviceWorker' in navigator) {
            window.addEventListener('load', () => {
                navigator.serviceWorker.register('/service-worker.js')
                    .then(reg => console.log('SW registered:', reg))
                    .catch(err => console.log('SW registration failed:', err));
            });
        }
    </script>
</body>
</html>
```

---

## 🚀 下一步

- [ ] 创建项目脚手架
- [ ] 配置开发环境
- [ ] 编写代码规范
- [ ] 搭建 CI/CD 流程

---

**下一步**: 阅读 [API 设计](../03-api-design/01-api-specification.md)

**最后更新**: 2025-11-25
