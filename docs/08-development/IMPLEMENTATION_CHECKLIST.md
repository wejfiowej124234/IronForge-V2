# IronForge V2 详细实现清单 (Implementation Checklist)

> **生成日期**: 2025-11-25
> **关联计划**: [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md)
> **说明**: 本清单将 `docs-v2` 中的技术文档拆解为可执行的原子任务，用于确保 100% 覆盖所有细节。

---

## 🔐 阶段 1: 安全核心 (Security Core)

### 1.1 密钥管理 (`04-security/01-key-management.md`)
- [ ] **BIP39 实现**
    - [ ] 使用 `bip39` crate 生成 12/24 助记词。
    - [ ] 实现助记词校验和验证 (Checksum Validation)。
    - [ ] 实现 `Mnemonic::to_seed` (带 passphrase 支持)。
- [ ] **BIP32/44 派生**
    - [ ] 实现 `m/44'/60'/0'/0/0` (Ethereum) 路径派生。
    - [ ] 实现 `m/44'/501'/0'/0'` (Solana) 路径派生。
    - [ ] 实现 `m/84'/0'/0'/0/0` (Bitcoin Native Segwit) 路径派生。
    - [ ] 实现 `m/44'/607'/0'/0'` (TON) 路径派生。
- [ ] **内存安全**
    - [ ] 引入 `zeroize` crate。
    - [ ] 确保 `Mnemonic`, `Seed`, `PrivateKey` 结构体实现 `Zeroize` trait。
    - [ ] 验证 Drop 时内存自动清零。

### 1.2 加密策略 (`04-security/02-encryption-strategy.md`)
- [ ] **Argon2id KDF**
    - [ ] 配置参数: m_cost=64MB, t_cost=3, p_cost=4 (参考文档标准)。
    - [ ] 实现 `derive_key(password, salt) -> [u8; 32]`。
- [ ] **AES-256-GCM**
    - [ ] 使用 `aes-gcm` crate。
    - [ ] 实现 `encrypt(key, nonce, plaintext)`。
    - [ ] 实现 `decrypt(key, nonce, ciphertext)`。
    - [ ] 确保 Nonce 随机生成且不重复。
- [ ] **Web Crypto 适配**
    - [ ] (可选) 如果 WASM 性能不足，通过 `web-sys` 调用浏览器原生 `SubtleCrypto`。

### 1.3 安全存储 (`04-security/05-production-encryption-guide.md`)
- [ ] **IndexedDB 封装**
    - [ ] 创建 `StorageAdapter` trait。
    - [ ] 实现 `EncryptedStorage` 结构体。
    - [ ] 确保存储前自动调用 `encrypt`，读取后自动调用 `decrypt`。
    - [ ] 严禁明文存储任何 Key Material。

---

## 🏗️ 阶段 2: 架构与状态 (`01-architecture`, `02-technical-design`)

### 2.1 模块化架构 (`02-technical-design/06-modularization-best-practices.md`)
- [x] **目录重构**
    - [x] 创建 `src/blockchain/{ethereum, solana, bitcoin, ton}`。
    - [x] 创建 `src/features/{wallet, settings, transactions}`。
    - [x] 创建 `src/shared/{components, hooks, utils}`。
- [x] **Chain Adapter**
    - [x] 定义 `trait ChainAdapter` (balance, history, broadcast)。
    - [x] 实现 `EthereumAdapter` (使用 `ethers-rs` 或 `alloy`)。
    - [ ] 实现 `SolanaAdapter` (使用 `solana-client-wasm`)。

### 2.2 状态管理 (`02-technical-design/03-state-management.md`)
- [x] **Signal Store**
    - [x] 实现 `WalletStore` (accounts, balances, selected_chain)。
    - [x] 实现 `SettingsStore` (theme, language, currency)。
- [x] **持久化**
    - [x] 实现 `PersistentSignal` 机制 (自动同步到 EncryptedStorage)。
    - [x] 实现状态恢复逻辑 (Hydration)。

---

## 🔌 阶段 3: API 与后端 (`03-api-design`)

### 3.1 API 客户端 (`03-api-design/02-frontend-api-layer.md`)
- [x] **HTTP Client**
    - [x] 封装 `reqwest` 或 `gloo-net`。
    - [x] 添加 Auth Interceptor (自动附加 JWT)。
    - [x] 添加 Error Interceptor (统一错误转换)。
- [ ] **Token Detection** (`03-api-design/04-token-detection-service.md`)
    - [ ] 实现 `fetch_token_list(chain_id)`。
    - [ ] 实现 `detect_assets(address)` (调用后端或链上 RPC)。

### 3.2 错误处理 (`03-api-design/03-error-handling.md`)
- [x] **AppError 枚举**
    - [x] 定义 `NetworkError`, `CryptoError`, `ValidationError`。
- [ ] **UI 反馈**
    - [ ] 实现全局 `Toast` 组件显示错误。
    - [ ] 实现 `ErrorBoundary` 捕获渲染错误。

---

## 🎨 阶段 4: UI/UX (`05-ui-ux`)

### 4.1 设计系统 (`05-ui-ux/01-design-system-v2.md`)
- [ ] **Tailwind 配置**
    - [ ] 配置 `colors`, `spacing`, `typography` 符合设计规范。
    - [ ] 配置 Dark Mode。
- [ ] **基础组件**
    - [ ] `Button` (Primary, Secondary, Ghost)。
    - [ ] `Input` (Text, Password, Number)。
    - [ ] `Card`, `Modal`, `Loader`。

### 4.2 核心页面 (`05-ui-ux/02-user-flows.md`)
- [ ] **Onboarding**
    - [ ] Welcome Page。
    - [ ] Create Wallet (Mnemonic Display + Verify)。
    - [ ] Import Wallet (Input + Validation)。
- [ ] **Dashboard**
    - [ ] Asset List (Token Icon, Name, Balance, Value)。
    - [ ] Chain Selector。
- [ ] **Transfer**
    - [ ] Recipient Input (Address Validation)。
    - [ ] Amount Input (Max button, Fiat conversion)。
    - [ ] Gas Estimation Display。

---

## 🚀 阶段 5: 生产优化 (`06-production`)

### 5.1 性能 (`02-technical-design/06-modularization-best-practices.md`)
- [ ] **Lazy Loading**
    - [ ] 对非首屏路由使用 `lazy` 加载。
- [ ] **Virtual List**
    - [ ] 对交易历史和代币列表使用虚拟滚动。

### 5.2 监控 (`06-production/04-monitoring-setup.md`)
- [ ] **日志**
    - [ ] 集成 `tracing` 或 `log` crate。
    - [ ] 实现日志上报接口。

---

## ✅ 阶段 6: 测试 (`07-testing`)

### 6.1 单元测试 (`07-testing/01-testing-strategy.md`)
- [ ] **Crypto Tests**: 覆盖所有加密/解密/派生逻辑。
- [ ] **Utils Tests**: 覆盖格式化、校验工具函数。

### 6.2 集成测试
- [ ] **Flow Tests**: 模拟完整的创建钱包流程。
