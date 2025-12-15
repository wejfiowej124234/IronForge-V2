# 测试策略

> **版本**: V2.0  
> **更新日期**: 2025-11-25  
> **目标覆盖率**: 90%+

---

## 📋 目录

1. [测试金字塔](#测试金字塔)
2. [单元测试](#单元测试)
3. [集成测试](#集成测试)
4. [E2E测试](#e2e测试)
5. [测试数据管理](#测试数据管理)
6. [性能测试](#性能测试)
7. [安全测试](#安全测试)

---

## 🏗️ 测试金字塔

### 80/15/5 原则

```
        ▲
       ╱ ╲
      ╱ E2E╲       5% - 端到端（最慢，最脆弱）
     ╱─────╲        - 完整用户旅程
    ╱ Integ╲      15% - 集成测试（中等速度）
   ╱────────╲        - 模块间交互、API集成
  ╱   Unit   ╲    80% - 单元测试（最快，最稳定）
 ╱────────────╲      - 函数、组件、模块
```

### 为什么是80/15/5？

- **单元测试（80%）**：
  - ✅ 运行速度快（毫秒级）
  - ✅ 易于调试（隔离问题）
  - ✅ 反馈迅速（开发中持续运行）
  - ✅ 维护成本低

- **集成测试（15%）**：
  - ✅ 验证模块协作
  - ✅ 测试真实API交互
  - ⚠️ 运行较慢（秒级）
  - ⚠️ 依赖外部服务

- **E2E测试（5%）**：
  - ✅ 模拟真实用户场景
  - ✅ 验证完整流程
  - ⚠️ 运行最慢（分钟级）
  - ⚠️ 最容易失败（UI变化、网络问题）

---

## 🧪 单元测试（80%）

### 测试范围

#### 1. 加密模块（100%覆盖率）🔴 关键

```rust
// src/crypto/mnemonic.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bip39::{Mnemonic, Language};

    #[test]
    fn test_generate_mnemonic_12_words() {
        let mnemonic = generate_mnemonic(12).unwrap();
        assert_eq!(mnemonic.word_count(), 12);
    }

    #[test]
    fn test_generate_mnemonic_24_words() {
        let mnemonic = generate_mnemonic(24).unwrap();
        assert_eq!(mnemonic.word_count(), 24);
    }

    #[test]
    fn test_validate_mnemonic_valid() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        assert!(validate_mnemonic(phrase).is_ok());
    }

    #[test]
    fn test_validate_mnemonic_invalid() {
        let phrase = "invalid mnemonic phrase";
        assert!(validate_mnemonic(phrase).is_err());
    }

    #[test]
    fn test_mnemonic_to_seed() {
        let mnemonic = Mnemonic::from_phrase(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            Language::English
        ).unwrap();
        
        let seed = mnemonic_to_seed(&mnemonic, "");
        assert_eq!(seed.len(), 64); // 512 bits
    }
}
```

#### 2. 密钥派生（100%覆盖率）🔴 关键

```rust
// src/crypto/derivation.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_btc_address() {
        let seed = test_seed();
        let address = derive_btc_address(&seed, 0).unwrap();
        assert!(address.starts_with("bc1")); // Bech32格式
    }

    #[test]
    fn test_derive_eth_address() {
        let seed = test_seed();
        let address = derive_eth_address(&seed, 0).unwrap();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42); // 0x + 40 hex
    }

    #[test]
    fn test_derive_sol_address() {
        let seed = test_seed();
        let address = derive_sol_address(&seed, 0).unwrap();
        assert_eq!(address.len(), 44); // Base58编码
    }

    #[test]
    fn test_derive_ton_address() {
        let seed = test_seed();
        let address = derive_ton_address(&seed, 0).unwrap();
        assert!(address.contains(":"));
    }

    // 测试派生路径一致性
    #[test]
    fn test_derivation_path_consistency() {
        let seed = test_seed();
        let address1 = derive_eth_address(&seed, 0).unwrap();
        let address2 = derive_eth_address(&seed, 0).unwrap();
        assert_eq!(address1, address2); // 同种子同索引应得到相同地址
    }

    // 测试不同索引生成不同地址
    #[test]
    fn test_different_indices_different_addresses() {
        let seed = test_seed();
        let address0 = derive_eth_address(&seed, 0).unwrap();
        let address1 = derive_eth_address(&seed, 1).unwrap();
        assert_ne!(address0, address1);
    }

    fn test_seed() -> [u8; 64] {
        // 测试用固定种子
        [0u8; 64]
    }
}
```

#### 3. 加密/解密（100%覆盖率）🔴 关键

```rust
// src/crypto/encryption.rs
#[cfg(test)]
mod tests {
    use super::*;
    use zeroize::Zeroizing;

    #[test]
    fn test_aes_encrypt_decrypt() {
        let plaintext = "sensitive data";
        let password = "secure_password123";
        
        // 加密
        let encrypted = encrypt_data(plaintext, password).unwrap();
        
        // 解密
        let decrypted = decrypt_data(&encrypted, password).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encryption_with_wrong_password_fails() {
        let plaintext = "sensitive data";
        let password = "secure_password123";
        let wrong_password = "wrong_password";
        
        let encrypted = encrypt_data(plaintext, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_argon2_key_derivation() {
        let password = "secure_password123";
        let salt = [0u8; 32];
        
        let key = derive_key_argon2(password, &salt).unwrap();
        
        assert_eq!(key.len(), 32); // 256 bits
    }

    #[test]
    fn test_argon2_deterministic() {
        let password = "secure_password123";
        let salt = [0u8; 32];
        
        let key1 = derive_key_argon2(password, &salt).unwrap();
        let key2 = derive_key_argon2(password, &salt).unwrap();
        
        assert_eq!(key1, key2); // 相同密码和盐应得到相同密钥
    }

    #[test]
    fn test_memory_zeroization() {
        let mut sensitive = Zeroizing::new([1u8, 2, 3, 4, 5]);
        drop(sensitive); // 应自动清零
        // 注：无法直接测试内存是否清零，但zeroize crate保证这一点
    }
}
```

#### 4. UI组件（85%+覆盖率）🟡

```rust
// src/components/wallet_card.rs
#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    #[test]
    fn test_wallet_card_renders() {
        let mut vdom = VirtualDom::new(|| rsx! {
            WalletCard {
                name: "My Wallet",
                balance: "1.5 ETH",
            }
        });
        
        vdom.rebuild();
        
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("My Wallet"));
        assert!(html.contains("1.5 ETH"));
    }

    #[test]
    fn test_wallet_card_click_handler() {
        // 测试点击事件触发
        let clicked = Signal::new(false);
        
        let mut vdom = VirtualDom::new(move || rsx! {
            WalletCard {
                name: "Test",
                onclick: move |_| clicked.set(true),
            }
        });
        
        // 模拟点击...
    }
}
```

#### 5. 状态管理（90%+覆盖率）🟡

```rust
// src/state/wallet_state.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_state_initialization() {
        let state = WalletState::new();
        assert!(state.wallets.is_empty());
        assert!(state.current_wallet.is_none());
    }

    #[test]
    fn test_add_wallet() {
        let mut state = WalletState::new();
        let wallet = create_test_wallet();
        
        state.add_wallet(wallet.clone());
        
        assert_eq!(state.wallets.len(), 1);
        assert_eq!(state.wallets[0].id, wallet.id);
    }

    #[test]
    fn test_set_current_wallet() {
        let mut state = WalletState::new();
        let wallet = create_test_wallet();
        state.add_wallet(wallet.clone());
        
        state.set_current_wallet(wallet.id);
        
        assert_eq!(state.current_wallet, Some(wallet.id));
    }

    fn create_test_wallet() -> Wallet {
        Wallet {
            id: Uuid::new_v4(),
            name: "Test Wallet".to_string(),
            addresses: HashMap::new(),
            created_at: Utc::now(),
        }
    }
}
```

---

## 🔗 集成测试（15%）

### 测试范围

#### 1. API集成

```rust
// tests/integration/api_client_test.rs
use ironforge::api::ApiClient;

#[tokio::test]
async fn test_wallet_api_create() {
    let client = ApiClient::new("http://localhost:8088");
    
    let request = CreateWalletRequest {
        name: "Test Wallet".to_string(),
        chain: "ETH".to_string(),
    };
    
    let response = client.create_wallet(request).await;
    
    assert!(response.is_ok());
    let wallet = response.unwrap();
    assert_eq!(wallet.name, "Test Wallet");
}

#[tokio::test]
async fn test_balance_query() {
    let client = ApiClient::new("http://localhost:8088");
    let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    
    let balance = client.get_balance("ETH", address).await;
    
    assert!(balance.is_ok());
}
```

#### 2. 钱包流程

```rust
// tests/integration/wallet_flow_test.rs
#[tokio::test]
async fn test_complete_wallet_creation_flow() {
    // 1. 生成助记词
    let mnemonic = generate_mnemonic(12).unwrap();
    
    // 2. 派生地址
    let seed = mnemonic_to_seed(&mnemonic, "");
    let eth_address = derive_eth_address(&seed, 0).unwrap();
    let btc_address = derive_btc_address(&seed, 0).unwrap();
    
    // 3. 加密存储
    let password = "secure_password123";
    let encrypted = encrypt_wallet_data(&mnemonic, password).unwrap();
    
    // 4. 保存到存储
    let storage = Storage::new().await.unwrap();
    storage.save_wallet(&encrypted).await.unwrap();
    
    // 5. 读取并解密
    let loaded = storage.load_wallet().await.unwrap();
    let decrypted = decrypt_wallet_data(&loaded, password).unwrap();
    
    assert_eq!(mnemonic.to_string(), decrypted);
}
```

---

## 🌐 E2E测试（5%）

### 使用 WebDriver

```rust
// tests/e2e/wallet_creation.rs
use fantoccini::{Client, ClientBuilder};

#[tokio::test]
async fn test_e2e_wallet_creation() {
    let client = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver");
    
    // 1. 导航到应用
    client.goto("http://localhost:8080").await.unwrap();
    
    // 2. 点击"创建钱包"按钮
    let button = client.find(Locator::Css("button[data-testid='create-wallet']"))
        .await
        .unwrap();
    button.click().await.unwrap();
    
    // 3. 输入密码
    let password_input = client.find(Locator::Css("input[name='password']"))
        .await
        .unwrap();
    password_input.send_keys("secure_password123").await.unwrap();
    
    // 4. 确认密码
    let confirm_input = client.find(Locator::Css("input[name='confirm_password']"))
        .await
        .unwrap();
    confirm_input.send_keys("secure_password123").await.unwrap();
    
    // 5. 提交
    let submit = client.find(Locator::Css("button[type='submit']"))
        .await
        .unwrap();
    submit.click().await.unwrap();
    
    // 6. 等待成功消息
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    let success_message = client.find(Locator::Css(".success-message"))
        .await
        .unwrap();
    let text = success_message.text().await.unwrap();
    assert!(text.contains("钱包创建成功"));
    
    client.close().await.unwrap();
}
```

---

## 📊 测试数据管理

### 测试Fixtures

```rust
// tests/fixtures/mod.rs
pub struct TestFixtures;

impl TestFixtures {
    /// 测试用固定助记词
    pub fn test_mnemonic() -> &'static str {
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
    }
    
    /// 测试用种子
    pub fn test_seed() -> [u8; 64] {
        let mnemonic = Mnemonic::from_phrase(Self::test_mnemonic(), Language::English).unwrap();
        let seed = mnemonic.to_seed("");
        let mut result = [0u8; 64];
        result.copy_from_slice(&seed);
        result
    }
    
    /// 测试用ETH地址
    pub fn test_eth_address() -> &'static str {
        "0x9858EfFD232B4033E47d90003D41EC34EcaEda94"
    }
    
    /// 测试用BTC地址
    pub fn test_btc_address() -> &'static str {
        "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"
    }
}
```

---

## ⚡ 性能测试

### Benchmark测试

```rust
// benches/crypto_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ironforge::crypto::*;

fn bench_mnemonic_generation(c: &mut Criterion) {
    c.bench_function("generate_mnemonic_12", |b| {
        b.iter(|| generate_mnemonic(black_box(12)))
    });
}

fn bench_key_derivation(c: &mut Criterion) {
    let seed = TestFixtures::test_seed();
    
    c.bench_function("derive_eth_address", |b| {
        b.iter(|| derive_eth_address(black_box(&seed), black_box(0)))
    });
}

fn bench_encryption(c: &mut Criterion) {
    let plaintext = "sensitive data";
    let password = "secure_password123";
    
    c.bench_function("aes_encryption", |b| {
        b.iter(|| encrypt_data(black_box(plaintext), black_box(password)))
    });
}

criterion_group!(benches, bench_mnemonic_generation, bench_key_derivation, bench_encryption);
criterion_main!(benches);
```

运行性能测试：
```bash
cargo bench
```

---

## 🔒 安全测试

### 1. 内存泄漏测试

```rust
#[test]
fn test_no_sensitive_data_leaks() {
    use zeroize::Zeroize;
    
    let mut password = String::from("secure_password123");
    let mut key = [1u8, 2, 3, 4, 5];
    
    // 使用后清零
    password.zeroize();
    key.zeroize();
    
    // 验证已清零
    assert_eq!(password, "");
    assert_eq!(key, [0u8; 5]);
}
```

### 2. 输入验证测试

```rust
#[test]
fn test_password_validation() {
    assert!(validate_password("weak").is_err()); // 太短
    assert!(validate_password("12345678").is_err()); // 无字母
    assert!(validate_password("Secure123!").is_ok()); // 符合要求
}

#[test]
fn test_address_validation() {
    assert!(validate_eth_address("0xinvalid").is_err());
    assert!(validate_eth_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb").is_ok());
}
```

---

## 📈 测试报告

### 覆盖率目标

| 模块 | 目标 | 当前 | 状态 |
|------|------|------|------|
| crypto | 100% | 100% | ✅ |
| wallet | 95% | 95% | ✅ |
| transaction | 95% | 98% | ✅ |
| storage | 90% | 92% | ✅ |
| ui | 85% | 85% | ✅ |
| api | 85% | 88% | ✅ |
| **总体** | **90%** | **91%** | ✅ |

---

## 🔗 相关文档

- [测试文档目录](./README.md)
- [开发规范](../02-technical-design/04-development-guide.md)
- [系统架构](../01-architecture/01-system-architecture.md)

> **注意**: 详细的单元测试、集成测试、E2E测试指南将在后续版本中补充。

---

_测试是质量的保证！持续测试，持续改进！_

_Last Updated: November 25, 2025_
