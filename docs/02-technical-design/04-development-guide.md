# 开发规范与最佳实践

> **版本**: V2.0  
> **技术栈**: Rust + Dioxus 0.7 + Trunk  
> **更新日期**: 2025-11-25  
> **适用团队**: IronForge 前端开发团队

---

## 📋 目录

1. [代码风格](#代码风格)
2. [项目结构](#项目结构)
3. [命名规范](#命名规范)
4. [组件开发规范](#组件开发规范)
5. [Git 工作流](#git-工作流)
6. [测试规范](#测试规范)
7. [性能优化规范](#性能优化规范)
8. [文档规范](#文档规范)
9. [Code Review 清单](#code-review-清单)

---

## 代码风格

### Rust 代码规范

**基础规则**：遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

```toml
# .cargo/config.toml
[target.wasm32-unknown-unknown]
runner = 'wasm-bindgen-test-runner'

[build]
rustflags = ["-D", "warnings"]  # 将警告视为错误

[alias]
check-all = "clippy --all-targets --all-features -- -D warnings"
fmt-check = "fmt -- --check"
```

### Clippy 配置

```toml
# Cargo.toml
[lints.rust]
unsafe_code = "forbid"  # 禁止 unsafe（除非有充分理由）
missing_docs = "warn"   # 警告缺少文档

[lints.clippy]
# 性能相关
perf = "warn"
# 正确性检查
correctness = "deny"
# 可疑代码
suspicious = "deny"
# 复杂度警告
complexity = "warn"
# 风格建议
style = "warn"
# 特定规则
unwrap_used = "warn"         # 避免 unwrap()
expect_used = "warn"         # 避免 expect()
panic = "warn"               # 避免 panic!
todo = "warn"                # 避免 TODO
unimplemented = "warn"       # 避免 unimplemented!
```

### 格式化规范

```toml
# rustfmt.toml
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
edition = "2021"
merge_derives = true
use_try_shorthand = true
use_field_init_shorthand = true
force_explicit_abi = true
normalize_comments = true
wrap_comments = true
format_code_in_doc_comments = true
comment_width = 80
```

### 代码示例

```rust
// ✅ 好的代码风格
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// 用户钱包元数据
///
/// # Examples
///
/// ```
/// let metadata = WalletMetadata {
///     wallet_id: "abc123".to_string(),
///     name: "My Wallet".to_string(),
///     created_at: 1234567890,
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalletMetadata {
    /// 唯一钱包 ID
    pub wallet_id: String,
    /// 用户自定义名称
    pub name: String,
    /// 创建时间（Unix 时间戳）
    pub created_at: u64,
}

impl WalletMetadata {
    /// 创建新的钱包元数据
    pub fn new(wallet_id: String, name: String) -> Self {
        Self {
            wallet_id,
            name,
            created_at: current_timestamp(),
        }
    }
    
    /// 验证钱包 ID 格式
    pub fn validate_id(&self) -> Result<(), ValidationError> {
        if self.wallet_id.is_empty() {
            return Err(ValidationError::EmptyWalletId);
        }
        Ok(())
    }
}

// ❌ 避免的代码风格
pub struct Bad {
    pub a:String,pub b:u64} // 格式混乱

impl Bad{
fn do_thing(&self)->Option<String>{  // 缺少空格
    Some(self.a.clone())  // 不必要的 clone
}
}
```

---

## 项目结构

### 标准目录结构

```
IronForge/
├── src/
│   ├── main.rs                    # 应用入口
│   ├── app.rs                     # 根组件 + 路由
│   │
│   ├── domain/                    # 领域层（业务逻辑）
│   │   ├── wallet/                # 钱包领域
│   │   │   ├── mod.rs
│   │   │   ├── key_manager.rs     # 密钥管理
│   │   │   ├── wallet_service.rs  # 钱包服务
│   │   │   └── types.rs           # 钱包类型
│   │   ├── transaction/           # 交易领域
│   │   └── security/              # 安全领域
│   │
│   ├── infrastructure/            # 基础设施层
│   │   ├── api/                   # API 客户端
│   │   ├── storage/               # 存储适配器
│   │   └── crypto/                # 加密实现
│   │
│   ├── ui/                        # UI 层
│   │   ├── pages/                 # 页面组件
│   │   │   ├── mod.rs
│   │   │   ├── home.rs
│   │   │   ├── wallet.rs
│   │   │   └── send.rs
│   │   ├── components/            # 可复用组件
│   │   │   ├── atoms/             # 原子组件
│   │   │   ├── molecules/         # 分子组件
│   │   │   └── organisms/         # 有机组件
│   │   ├── theme/                 # 主题系统
│   │   └── hooks/                 # 自定义 Hooks
│   │
│   ├── state/                     # 状态管理
│   │   ├── mod.rs
│   │   ├── app_state.rs           # 全局状态
│   │   ├── wallet_state.rs        # 钱包状态
│   │   └── preferences.rs         # 用户偏好
│   │
│   ├── flows/                     # 用户流程
│   │   ├── wallet_creation.rs     # 创建钱包流程
│   │   └── send_transaction.rs    # 发送交易流程
│   │
│   └── utils/                     # 工具函数
│       ├── mod.rs
│       ├── format.rs              # 格式化工具
│       └── validation.rs          # 验证工具
│
├── tests/                         # 集成测试
│   ├── wallet_test.rs
│   └── transaction_test.rs
│
├── benches/                       # 性能测试
│   └── crypto_bench.rs
│
├── docs-v2/                       # V2 文档
├── assets/                        # 静态资源
│   ├── fonts/
│   ├── images/
│   └── styles/
│       └── main.css
│
├── Cargo.toml
├── Trunk.toml
├── rustfmt.toml
└── .clippy.toml
```

### 模块组织原则

1. **单一职责**：每个模块只负责一个明确的功能
2. **依赖方向**：UI → Domain → Infrastructure（单向依赖）
3. **文件大小**：单文件不超过 500 行（复杂逻辑拆分）
4. **导出控制**：只导出公开 API，内部实现用 `pub(crate)`

---

## 命名规范

### 文件命名

```
✅ 好的命名
wallet_service.rs        # snake_case
transaction_builder.rs
key_manager.rs

❌ 避免的命名
WalletService.rs         # 不要用 PascalCase
wallet-service.rs        # 不要用 kebab-case
wallet_svc.rs            # 不要缩写
```

### 类型命名

```rust
// ✅ 类型使用 PascalCase
pub struct WalletMetadata { }
pub enum TransactionStatus { }
pub trait KeyManager { }

// ✅ 常量使用 SCREAMING_SNAKE_CASE
pub const MAX_WALLET_NAME_LENGTH: usize = 50;
pub const DEFAULT_GAS_LIMIT: u64 = 21000;

// ✅ 函数/变量使用 snake_case
pub fn create_wallet() -> Result<Wallet> { }
let wallet_name = "My Wallet";

// ✅ 生命周期使用单字母小写
pub fn process<'a>(input: &'a str) -> &'a str { }

// ✅ 泛型类型使用单字母大写或描述性名称
pub struct Container<T> { }
pub struct ApiResponse<Data> { }
```

### 组件命名

```rust
// ✅ Dioxus 组件使用 PascalCase
pub fn WalletCard() -> Element { }
pub fn TransactionList() -> Element { }
pub fn SendButton() -> Element { }

// ✅ Hooks 使用 use_ 前缀
pub fn use_wallet_state() -> Signal<WalletState> { }
pub fn use_transaction_builder() -> TransactionBuilder { }

// ✅ Props 使用 组件名 + Props 后缀
#[derive(Props, PartialEq)]
pub struct WalletCardProps {
    pub wallet: WalletMetadata,
}
```

---

## 组件开发规范

### 组件结构模板

```rust
// src/ui/components/organisms/wallet_card.rs
use dioxus::prelude::*;
use crate::domain::wallet::WalletMetadata;

/// 钱包卡片组件
///
/// 显示钱包的基本信息和操作按钮
///
/// # Props
/// - `wallet`: 钱包元数据
/// - `on_select`: 选择钱包时的回调
///
/// # Example
/// ```rust
/// rsx! {
///     WalletCard {
///         wallet: my_wallet,
///         on_select: move |_| { /* 处理选择 */ }
///     }
/// }
/// ```
#[component]
pub fn WalletCard(
    wallet: WalletMetadata,
    #[props(optional)] on_select: Option<EventHandler<MouseEvent>>,
) -> Element {
    let theme = use_theme();
    
    rsx! {
        div {
            class: "wallet-card",
            style: "padding: {theme.spacing.md}px;",
            
            // 钱包名称
            h3 { class: "wallet-card__name", "{wallet.name}" }
            
            // 操作按钮
            if let Some(handler) = on_select {
                button {
                    onclick: move |evt| handler.call(evt),
                    "Select"
                }
            }
        }
    }
}
```

### Props 设计原则

```rust
// ✅ 好的 Props 设计
#[derive(Props, PartialEq, Clone)]
pub struct TransactionListProps {
    /// 必需：交易列表
    pub transactions: Vec<Transaction>,
    /// 可选：每页显示数量
    #[props(default = 10)]
    pub page_size: usize,
    /// 可选：点击交易回调
    #[props(optional)]
    pub on_transaction_click: Option<EventHandler<String>>,
}

// ❌ 避免的 Props 设计
pub struct BadProps {
    pub data: Vec<String>,  // 命名不清晰
    pub cb: Box<dyn Fn()>,  // 不使用 EventHandler
    // 缺少文档注释
}
```

### 组件拆分原则

```rust
// ❌ 避免：过大的组件（>200 行）
pub fn MassivePage() -> Element {
    // 500+ 行代码...
}

// ✅ 推荐：拆分为多个小组件
pub fn WalletPage() -> Element {
    rsx! {
        div {
            WalletHeader { }
            WalletBalance { }
            TransactionList { }
            WalletActions { }
        }
    }
}

// 每个子组件独立文件
// src/ui/components/wallet/header.rs
pub fn WalletHeader() -> Element { /* ... */ }
```

---

## Git 工作流

### 分支命名

```bash
# 功能分支
feature/wallet-creation
feature/multi-chain-support

# 修复分支
fix/transaction-signing-bug
fix/gas-estimation-error

# 文档分支
docs/api-documentation
docs/user-guide

# 性能优化
perf/wasm-optimization
perf/render-performance

# 重构
refactor/state-management
refactor/component-structure
```

### Commit 消息规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/)

```bash
# 格式
<type>(<scope>): <subject>

<body>

<footer>

# 类型 (type)
feat:     新功能
fix:      修复 bug
docs:     文档更新
style:    代码格式（不影响功能）
refactor: 重构（不是新功能也不是修复）
perf:     性能优化
test:     测试相关
chore:    构建/工具/依赖更新

# 示例
feat(wallet): add multi-signature wallet support

Implement BIP45 multi-sig wallet creation with 2-of-3 threshold.

Closes #123

---

fix(transaction): correct gas estimation for EIP-1559

Gas limit was being calculated incorrectly for Type 2 transactions.

Fixes #456

---

docs(api): update frontend API layer documentation

Add examples for error handling and retry logic.
```

### PR 规范

```markdown
## 📝 Description
简要描述本次变更的目的和内容

## 🎯 Type of Change
- [ ] 新功能 (non-breaking change)
- [ ] Bug 修复 (non-breaking change)
- [ ] 破坏性变更 (Breaking change)
- [ ] 文档更新

## ✅ Checklist
- [ ] 代码已通过 `cargo fmt` 格式化
- [ ] 代码已通过 `cargo clippy` 检查
- [ ] 新功能已添加单元测试
- [ ] 所有测试通过 (`cargo test`)
- [ ] 文档已更新（如果需要）
- [ ] 已在本地测试 WASM 构建 (`trunk build`)

## 🧪 Test Plan
描述如何测试本次变更

## 📸 Screenshots (if applicable)
相关截图

## 🔗 Related Issues
Closes #issue_number
```

---

## 测试规范

### 单元测试

```rust
// src/domain/wallet/wallet_service.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_name_validation() {
        // Given
        let valid_name = "My Wallet";
        let invalid_name = "";
        
        // When & Then
        assert!(validate_wallet_name(valid_name).is_ok());
        assert!(validate_wallet_name(invalid_name).is_err());
    }
    
    #[tokio::test]
    async fn test_wallet_creation() {
        // Given
        let service = WalletService::new_mock();
        
        // When
        let result = service.create_wallet(
            "Test Wallet".to_string(),
            "password123".to_string(),
        ).await;
        
        // Then
        assert!(result.is_ok());
        let wallet = result.unwrap();
        assert_eq!(wallet.name, "Test Wallet");
    }
}
```

### 组件测试

```rust
// tests/components/wallet_card_test.rs
#[cfg(test)]
mod tests {
    use dioxus::prelude::*;
    use dioxus_ssr::render;
    
    #[test]
    fn test_wallet_card_renders() {
        // Given
        let wallet = WalletMetadata {
            wallet_id: "test123".to_string(),
            name: "Test Wallet".to_string(),
            created_at: 1234567890,
        };
        
        // When
        let html = render(rsx! {
            WalletCard { wallet: wallet }
        });
        
        // Then
        assert!(html.contains("Test Wallet"));
    }
}
```

### 测试覆盖率目标

| 层级 | 目标覆盖率 | 优先级 |
|------|-----------|--------|
| Domain 层 | ≥ 90% | 🔴 高 |
| Infrastructure 层 | ≥ 80% | 🟡 中 |
| UI 组件 | ≥ 70% | 🟢 低 |

---

## 性能优化规范

### 避免不必要的克隆

```rust
// ❌ 避免：过度克隆
fn bad_example(wallet: &Wallet) -> String {
    wallet.name.clone()  // 不必要的 clone
}

// ✅ 推荐：返回引用
fn good_example(wallet: &Wallet) -> &str {
    &wallet.name
}
```

### 使用 Memo 缓存计算

```rust
// ✅ 对于昂贵的计算使用 use_memo
pub fn ExpensiveComponent() -> Element {
    let data = use_signal(|| vec![1, 2, 3, 4, 5]);
    
    let sum = use_memo(move || {
        data.read().iter().sum::<i32>()  // 只在 data 变化时重新计算
    });
    
    rsx! {
        div { "Sum: {sum}" }
    }
}
```

### WASM 二进制优化

```toml
# Cargo.toml
[profile.release]
opt-level = "z"           # 优化大小
lto = true                # Link Time Optimization
codegen-units = 1         # 更好的优化
strip = true              # 移除符号
panic = "abort"           # 减少 panic 代码
```

---

## 文档规范

### 公共 API 文档

```rust
/// 创建新钱包
///
/// 生成 BIP39 助记词并派生第一个账户。
///
/// # Arguments
///
/// * `name` - 钱包名称（1-50 字符）
/// * `password` - 加密密码（≥8 字符）
/// * `word_count` - 助记词长度（12 或 24）
///
/// # Returns
///
/// 返回 `WalletCreationResult`，包含钱包 ID 和助记词。
///
/// # Errors
///
/// * `KeyError::InvalidName` - 名称格式无效
/// * `KeyError::WeakPassword` - 密码强度不足
/// * `KeyError::MnemonicGeneration` - 助记词生成失败
///
/// # Examples
///
/// ```
/// use ironforge::KeyManager;
///
/// let manager = KeyManager::new();
/// let result = manager.create_wallet(
///     "My Wallet".to_string(),
///     "MySecurePassword123!".to_string(),
///     WordCount::TwentyFour,
/// ).await?;
///
/// println!("Wallet ID: {}", result.wallet_id);
/// ```
///
/// # Safety
///
/// ⚠️ 助记词仅返回一次，前端必须提示用户备份。
pub async fn create_wallet(
    &self,
    name: String,
    password: String,
    word_count: WordCount,
) -> Result<WalletCreationResult, KeyError> {
    // 实现...
}
```

### README 模板

```markdown
# 组件名称

简要描述（一句话）

## 功能

- 功能点 1
- 功能点 2

## 使用方法

\`\`\`rust
// 代码示例
\`\`\`

## API

| 参数 | 类型 | 必需 | 默认值 | 描述 |
|------|------|------|--------|------|
| prop1 | String | ✅ | - | 描述 |

## 测试

\`\`\`bash
cargo test
\`\`\`

## 性能

- 指标 1
- 指标 2
```

---

## Code Review 清单

### 审查者检查项

```markdown
## 功能
- [ ] 代码实现符合需求
- [ ] 边界条件已处理
- [ ] 错误处理完整

## 代码质量
- [ ] 命名清晰易懂
- [ ] 无重复代码
- [ ] 函数职责单一
- [ ] 注释充分（复杂逻辑）

## 安全
- [ ] 无 unwrap() / expect()（除非有充分理由）
- [ ] 敏感数据已清零 (Zeroize)
- [ ] 输入验证完整

## 性能
- [ ] 无不必要的 clone()
- [ ] 使用合适的数据结构
- [ ] 异步操作正确使用

## 测试
- [ ] 单元测试覆盖核心逻辑
- [ ] 测试用例有意义
- [ ] 无 TODO/FIXME 未解决

## 文档
- [ ] 公共 API 有文档注释
- [ ] 复杂算法有说明
- [ ] README 已更新（如需要）
```

---

## 开发工具推荐

### VSCode 插件

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",      // Rust 语言支持
    "tamasfe.even-better-toml",     // TOML 语法高亮
    "serayuzgur.crates",            // Cargo.toml 依赖管理
    "vadimcn.vscode-lldb",          // Rust 调试
    "esbenp.prettier-vscode",       // 代码格式化
    "streetsidesoftware.code-spell-checker" // 拼写检查
  ]
}
```

### 本地开发脚本

```bash
# scripts/dev.sh
#!/bin/bash
set -e

echo "🔍 Running code checks..."
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test

echo "🏗️ Building WASM..."
trunk build

echo "✅ All checks passed!"
```

---

## 参考资料

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Dioxus Best Practices](https://dioxuslabs.com/learn/0.7/guides/best_practices)
- [Google Rust Style Guide](https://google.github.io/comprehensive-rust/)
- [Conventional Commits](https://www.conventionalcommits.org/)
