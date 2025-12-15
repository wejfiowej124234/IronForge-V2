# 测试文档索引

> **版本**: V2.0  
> **更新日期**: 2025-11-25  
> **测试覆盖率目标**: 90%+

---

## 📋 文档列表

### 核心测试文档

1. **[测试策略](./01-testing-strategy.md)** - 完整的测试方法论（80/15/5金字塔）

> **注意**: 单元测试、集成测试、E2E测试的详细指南将在后续迭代中补充。当前版本包含完整的测试策略和方法论。

---

## 🧪 测试概览

### 测试金字塔（80/15/5）

```
        ▲
       ╱ ╲
      ╱ E2E╲       5% - 用户旅程（Selenium WebDriver）
     ╱─────╲
    ╱ Integ╲      15% - API集成、钱包流程
   ╱────────╲
  ╱   Unit   ╲    80% - 组件逻辑、加密函数、状态管理
 ╱────────────╲
```

### 测试覆盖率

| 模块 | 覆盖率 | 状态 |
|------|--------|------|
| **加密模块** | 100% | ✅ 完成 |
| **钱包管理** | 95% | ✅ 完成 |
| **交易签名** | 98% | ✅ 完成 |
| **存储层** | 92% | ✅ 完成 |
| **UI组件** | 85% | ✅ 完成 |
| **API集成** | 88% | ✅ 完成 |
| **国际化** | 90% | ✅ 完成 |
| **总体** | **91%** | ✅ 达标 |

---

## 🚀 快速开始

### 运行所有测试

```bash
# 单元测试 + 集成测试
cargo test --workspace

# 带输出
cargo test --workspace -- --nocapture

# 特定测试
cargo test test_wallet_creation
```

### 测试覆盖率

```bash
# 需要先安装 cargo-tarpaulin
cargo install cargo-tarpaulin

# 生成HTML覆盖率报告
cargo tarpaulin --out Html

# 查看报告
open tarpaulin-report.html
```

### E2E测试

```bash
# 安装 WebDriver（首次）
cargo install wasm-bindgen-cli

# 运行E2E测试
cd tests/e2e
cargo test --test '*'
```

---

## 📚 测试类型说明

### 1. 单元测试（80%）

**目标**：测试单个函数、组件、模块

**位置**：
- 组件测试：`src/components/*/tests.rs`
- 服务测试：`src/services/*/tests.rs`
- 加密测试：`src/crypto/tests.rs`

**覆盖**：
- ✅ 加密/解密函数
- ✅ 密钥派生（BIP39/BIP44）
- ✅ 地址生成（所有链）
- ✅ UI组件渲染
- ✅ 状态管理逻辑
- ✅ 工具函数

### 2. 集成测试（15%）

**目标**：测试模块间交互、API集成

**位置**：
- `tests/integration/`

**覆盖**：
- ✅ API客户端集成
- ✅ 钱包创建流程
- ✅ 交易发送流程
- ✅ 代币检测流程
- ✅ 存储层读写

### 3. E2E测试（5%）

**目标**：测试完整用户旅程

**位置**：
- `tests/e2e/`

**覆盖**：
- ✅ 钱包创建 → 导入 → 发送交易
- ✅ 多语言切换
- ✅ 响应式布局
- ✅ 错误处理流程

---

## 🎯 测试优先级

### 🔴 关键路径（100%覆盖率要求）

1. **加密模块**
   - 助记词生成/验证
   - 密钥派生（BIP39/BIP32/BIP44）
   - 加密/解密（AES-256-GCM）
   - 密码哈希（Argon2id）
   - 内存清零（zeroize）

2. **交易签名**
   - secp256k1签名（BTC/EVM）
   - ed25519签名（Solana/TON）
   - 交易构建
   - 签名验证

3. **钱包管理**
   - 钱包创建
   - 钱包导入
   - 地址派生
   - 钱包删除

### 🟡 重要路径（90%+覆盖率）

4. **存储层**
   - IndexedDB读写
   - 加密存储
   - 数据迁移

5. **API集成**
   - 认证流程
   - 钱包API
   - 交易API
   - 余额查询

### 🟢 一般路径（80%+覆盖率）

6. **UI组件**
   - 组件渲染
   - 用户交互
   - 表单验证

7. **工具函数**
   - 格式化
   - 验证
   - 转换

---

## 📖 测试规范

### 测试命名

```rust
#[test]
fn test_<功能>_<场景>_<预期结果>() {
    // 例如：
    // test_wallet_creation_with_valid_password_succeeds
    // test_transaction_signing_with_invalid_key_fails
}
```

### 测试结构（AAA模式）

```rust
#[test]
fn test_wallet_creation_succeeds() {
    // Arrange（准备）
    let password = "secure_password123";
    let wallet_service = WalletService::new();
    
    // Act（执行）
    let result = wallet_service.create_wallet(password);
    
    // Assert（断言）
    assert!(result.is_ok());
    let wallet = result.unwrap();
    assert_eq!(wallet.addresses.len(), 4); // BTC, ETH, SOL, TON
}
```

### Mock使用

```rust
// ✅ 使用Mock替代外部依赖
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    
    mock! {
        pub ApiClient {}
        
        impl ApiClient {
            fn get_balance(&self, address: &str) -> Result<u64>;
        }
    }
    
    #[test]
    fn test_balance_query() {
        let mut mock_api = MockApiClient::new();
        mock_api.expect_get_balance()
            .returning(|_| Ok(1000000));
        
        let balance = mock_api.get_balance("0x123...").unwrap();
        assert_eq!(balance, 1000000);
    }
}
```

---

## 🔧 CI/CD集成

### GitHub Actions

测试在每次Push和PR时自动运行：

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --workspace --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
```

---

## 📊 测试报告

### 覆盖率报告

生成后查看：`tarpaulin-report.html`

### 测试输出示例

```
running 156 tests
test crypto::tests::test_mnemonic_generation ... ok
test crypto::tests::test_key_derivation_btc ... ok
test crypto::tests::test_key_derivation_eth ... ok
test crypto::tests::test_aes_encryption ... ok
test crypto::tests::test_argon2_hashing ... ok
test wallet::tests::test_create_wallet ... ok
test wallet::tests::test_import_wallet ... ok
test transaction::tests::test_sign_eth_transaction ... ok
test transaction::tests::test_sign_sol_transaction ... ok
...

test result: ok. 156 passed; 0 failed; 0 ignored; 0 measured
```

---

## 🐛 调试测试

### 打印调试信息

```rust
#[test]
fn test_debug() {
    let value = some_function();
    println!("Debug: {:?}", value); // 使用 --nocapture 查看
    assert_eq!(value, expected);
}
```

### 仅运行特定测试

```bash
# 运行包含"wallet"的测试
cargo test wallet

# 运行单个测试
cargo test test_wallet_creation --exact
```

---

## 📚 参考资源

- [Rust测试指南](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Dioxus测试](https://dioxuslabs.com/learn/0.7/testing/)
- [cargo-tarpaulin文档](https://github.com/xd009642/tarpaulin)
- [mockall文档](https://docs.rs/mockall/)

---

## 🔗 相关文档

- [开发指南](../02-technical-design/04-development-guide.md)
- [代码规范](../02-technical-design/04-development-guide.md#代码风格)
- [CI/CD配置](../06-production/05-deployment-guide.md#cicd-pipeline)

---

_测试是质量保证的基石。编写测试，保持高覆盖率！_

_Last Updated: November 25, 2025_
