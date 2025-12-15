# IronForge V2 开发计划书（重排版）

> 生成日期: 2025-11-25（本版重排: 2025-11-27）
> 基于文档: IronForge/docs-v2
> 目标: 构建企业级、模块化、高安全的 Web3 钱包前端

本计划遵循 `docs-v2` 技术规范与架构设计，按优先级与依赖关系划分为 7 个阶段（0-6）。

—

## 0. 准备与规范 (Preparation & Standards)

**目标**: 确立开发规范，搭建符合模块化最佳实践的项目骨架。

1.  **深入研读核心规范**
    *   [ ] 阅读 `02-technical-design/06-modularization-best-practices.md`: 理解按链分组、功能分组、错误边界等核心原则。
    *   [ ] 阅读 `02-technical-design/04-development-guide.md`: 掌握代码风格、Git 规范、Commit 规范。
    *   [ ] 阅读 `00-PRODUCTION-CHECKLIST.md`: 明确"零Mock"的生产级标准。

2.  **项目骨架搭建**
    *   [ ] 按照 `06-modularization-best-practices.md` 重构/确认 `src` 目录结构。
    *   [ ] 建立 `src/blockchain/` (按链隔离)、`src/features/` (功能模块)、`src/components/` (UI组件) 等基础目录。
    *   [ ] 配置 Rust 工具链与 WASM 构建环境 (`02-technical-design/01-tech-stack.md`)。
    *   [ ] **CI/CD 流水线 (Pipeline)**: 配置 GitHub Actions 进行自动化 Lint (Clippy), Format (rustfmt), Test 和 Build 检查，确保代码合入质量。

—

## 1. 安全核心 (Security Core) - 最高优先级

**目标**: 构建钱包的安全基石，确保私钥管理和数据存储的绝对安全。

1.  **密钥管理系统 (Key Management)**
    *   **参考**: `04-security/01-key-management.md`
    *   [ ] 实现 BIP39 助记词生成与验证 (使用 `bip39` crate)。
    *   [ ] 实现 BIP32/BIP44 层级确定性钱包派生。
    *   [ ] 实现多链地址生成 (BTC, ETH, SOL, TON)。
    *   [ ] **关键**: 确保所有敏感数据使用 `zeroize` 处理，内存自动清零。
    *   [ ] **外部账户导入 (Import Support)**: 支持导入单私钥/Keystore，不仅限于 HD 钱包派生。
    *   [ ] **WASM 崩溃防护 (Panic Hook)**: 实现 `console_error_panic_hook`，捕获 Rust Panic 并展示友好的"崩溃恢复" UI，防止应用白屏。
    *   [ ] **自动锁定机制 (Auto-Lock)**: 实现无操作 (Inactivity) 5分钟自动锁定钱包，清除内存中的解密种子。
    *   [ ] **输入清洗与验证 (Input Sanitization)**: 对所有用户输入 (助记词、密码、地址) 进行严格的格式验证和清洗，防止注入攻击。
    *   [ ] **地址校验 (Address Validation)**: 实现严格的地址格式校验 (ETH EIP-55 Checksum, BTC Bech32/Base58, SOL Base58)，防止用户向错误地址转账。

2.  **加密与存储 (Encryption & Storage)**
    *   **参考**: `04-security/02-encryption-strategy.md`, `04-security/05-production-encryption-guide.md`
    *   [ ] 实现 Argon2id 密钥派生函数 (KDF)。
    *   [ ] 实现 AES-256-GCM 数据加密/解密逻辑。
    *   [ ] 封装 IndexedDB 存储层，确保写入磁盘的数据必须经过加密。
    *   [ ] 实现"零信任"存储策略，前端不保存明文私钥。
    *   [ ] **剪贴板防护 (Clipboard Security)**: 实现敏感信息复制后 60秒自动清除机制。

—

## 2. 架构与状态管理 (Architecture & State)

**目标**: 实现清晰的数据流和模块化架构，支撑复杂业务逻辑。

1.  **状态管理 (State Management)**
    *   **参考**: `02-technical-design/03-state-management.md`
    *   [ ] 基于 Dioxus Signals 实现全局状态管理。
    *   [ ] 定义 `WalletState`, `SettingsState`, `ChainState` 等核心状态模型。
    *   [ ] 实现状态持久化与恢复机制。
    *   [ ] **全局通知系统 (Notification Bus)**: 实现 Toast/Snackbar 消息总线，统一处理成功/错误反馈。
    *   [ ] **存储迁移系统 (Storage Migration)**: 设计版本控制机制 (Versioning)，当 State 结构变更时自动执行数据迁移，防止旧版本用户数据丢失。
    *   [ ] **网络状态管理 (Network Awareness)**: 监听 `window.ononline/offline` 事件，断网时自动切换至"离线只读模式"并提示用户。
    *   [ ] **国际化架构 (i18n)**: 搭建多语言支持框架 (Fluent/Gettext)，避免硬编码字符串，为全球化做准备。
    *   [ ] **无障碍基础 (A11y Foundation)**: 确保核心组件支持 ARIA 标签与键盘导航 (Focus Management)，满足企业级合规要求。

2.  **数据分离模型 (Data Separation)**
    *   **参考**: `01-architecture/02-data-separation-model.md`
    *   [ ] 严格区分 UI 状态 (Ephemeral) 与 业务数据 (Persistent)。
    *   [ ] 实现 ViewModel 层，隔离 UI 组件与底层逻辑。

3.  **链适配器层 (Chain Adapters)**
    *   **参考**: `02-technical-design/06-modularization-best-practices.md`
    *   [ ] 定义 `ChainAdapter` Trait (统一接口: `get_balance`, `send_tx` 等)。
    *   [ ] 实现基础链适配器框架 (为后续接入各链做准备)。
    *   [ ] **RPC 节点池管理 (RPC Pool Manager)**: 实现节点健康检查、延迟测速与自动故障转移 (Failover)。

4.  **TON 链专用适配 (TON Specifics)**
    *   [ ] 引入或实现 TON 序列化逻辑 (Cell/BoC 构建)。
    *   [ ] 适配 Wallet V4R2 合约接口。

—

## 3. Zodia UI/UX 设计系统 (Design System)

目标: 全面实施 "Enterprise Dark × Apple Minimal × Web3 Tech" 设计规范。

1.  **设计规范落地 (Design Implementation)**
    *   **参考**: `Zodia Wallet – UI/UX Design Specification`
    *   [x] **核心布局 (Layout)**: 实现 `ZodiaLayout`，包含星空背景 (Canvas Constellation) 和页面转场。
    *   [x] **仪表板 (Dashboard)**: 重构 `DashboardPageV2`，采用 `#0B0B0F` 深色背景 + 毛玻璃卡片。
    *   [x] 组件库 (Components):
        *   [x] `GasFeeCard`: 实时 Gas 估算卡片。
        *   [x] `ChainGroup`: 多链资产折叠列表 (EVM, Solana, BTC, TON)。
        *   [x] `TokenRow`: 极简代币行组件。
    *   [ ] 落地页 (Landing): 更新 `LandingPage` 以匹配 Zodia 风格（粒子/连线特效与交互动效）。
    *   [ ] 统一组件体系: 全面替换为 V2 组件（不再涉及旧代码删除，已完成移除）。

2.  **交互优化 (Interaction)**
    *   [ ] **安全标签**: 添加 "Non-Custodial" 和 "User + Wallet Dual-Lock" 徽章。
    *   [ ] **动画**: 添加微弱的呼吸灯效果和 Hover 渐变。

—

## 4. API 与全栈集成 (API & Full-Stack Integration)

目标: 打通前后端数据交互，将真实数据/加密/交互接入 UI。

1.  **API 层封装**
    *   **参考**: `03-api-design/02-frontend-api-layer.md`
    *   [ ] 封装 HTTP Client (Reqwest/Gloo)，统一处理 Timeout、Retry。
    *   [ ] 实现拦截器机制，处理 Auth Token 和错误响应。
    *   [ ] **智能缓存策略 (Smart Caching)**: 实现类似 React-Query 的 `Stale-While-Revalidate` 策略，避免重复请求，提升 UI 响应速度。
    *   [ ] **请求去重 (Request Deduplication)**: 防止短时间内重复发起相同的 API 请求 (如用户快速点击或组件重复渲染)，合并并发请求。
    *   [ ] **限流处理 (Rate Limit Handling)**: 优雅处理 HTTP 429 错误，实现指数退避 (Exponential Backoff) 重试机制。

2.  Gas 与费率系统 (Gas & Fees)
    *   **参考**: `backend/src/api/gas_api.rs`
    *   [ ] 集成 Gas Station API (`GET /api/gas/estimate`)，支持 慢/中/快 三档选择。
    *   [ ] 实现服务费计算逻辑 (Flat/Percent)，并在 UI 中明确展示“网络费”与“服务费”。

3.  交易生命周期管理 (Transaction Lifecycle)
    *   [ ] 实现交易广播接口 (`POST /api/tx/broadcast`)。
    *   [ ] 实现交易状态轮询机制 (Pending -> Confirmed/Failed)。
    *   [ ] 处理交易“卡死”或“超时”的异常状态反馈。
    *   [ ] **实时更新流 (Real-time Stream)**: 优先使用 WebSocket/SSE 推送交易状态与余额变更，降级方案为指数退避轮询 (Exponential Backoff Polling)。

4.  后端对接
    *   **参考**: `03-api-design/01-ironcore-backend-api-reference.md`
    *   [ ] 对接 IronCore/Backend 核心接口 (用户配置、交易历史等)。
    *   [ ] **钱包认证 (Wallet Auth)**: 实现基于签名的身份认证 (SIWE - Sign In With Ethereum)，安全获取 Auth Token，无需传统账号密码。
    *   [ ] 实现 Token Detection Service (`03-api-design/04-token-detection-service.md`)，支持多链代币自动发现。
    *   [ ] **法币汇率服务 (Fiat Price Service)**: 集成 CoinGecko 或后端代理，展示资产 USD/CNY 估值。
    *   [ ] **实时数据流 (Real-time Stream)**: 引入 WebSocket/SSE 支持，实现余额和交易状态的实时推送，替代低效轮询。
    *   [ ] **功能开关 (Feature Flags)**: 实现基于配置的功能开关系统，便于生产环境灰度发布或紧急关闭特定功能。

5.  错误处理系统
    *   **参考**: `03-api-design/03-error-handling.md`
    *   [ ] 定义统一的错误枚举 `AppError`。
    *   [ ] 实现错误边界 (Error Boundaries) 组件，防止局部错误导致应用崩溃。
    *   [ ] 实现用户友好的错误提示 UI。

—

## 3A. 设计系统现状快照（2025-11-27）

设计哲学: 专业、安全、冷静、科技感（机构/高级用户）。参考: `docs-v2/05-ui-ux/03-zodia-visual-specification.md`

### ✅ 已完成 (Build Fixes + Page Migration)

9.  **Dioxus 0.7 API 适配** - **100% 完成**
    *   [x] 初始 367 个编译错误全部清零，涵盖 API 变更、事件签名、生命周期、宏用法等问题
    *   [x] `cargo check` / `cargo fmt` / `cargo clippy`（含 `wasm32` 目标）保持通过，CI 验证成功
    *   [x] Effects/Templates 示例页运行稳定，组件文档示例已完成交叉验证

10. 页面迁移与 QA - 100% 完成（基础模板接入）
    *   [x] Dashboard / Assets / Activity / Bridge / Send / Receive / Settings / Security 等核心页面全部接入 V2 模板
    *   [x] Auth/Login/Register、Wallet Import/Receive/Send、Landing 等旧页面替换占位实现，统一使用 Zodia Layout
    *   [x] 视觉 QA：渐变、毛玻璃、Glow、微交互、Canvas 背景在桌面 + 移动端显示一致
    *   [x] 锁屏、通知、离线模式等运行时覆盖场景完成真机验证

### 🧾 阶段总结
- **交付资产**：组件 68 个、模板 8 个、Effects 21 个、辅助 Hook & Manager 12 个、Tailwind 令牌 130+
- **清理工作**：旧版组件 39 个、注释 70+ 处、`*_new` 文件全部移除；`mod.rs` / barrel exports 与顶层 re-export 统一完成
- **文档对齐**：`PHASE_3.5_*` 系列、`PHASE_3.5_COMPLETION_STATUS.md`、`FINAL_CLEANUP_COMPLETION_REPORT.md` 与本计划书保持一致
- **下一阶段依赖**：为 Phase 4 API 集成、状态管理绑定、安全流程联调提供完整 UI 基座，无新增阻塞项
    *   [x] **Security V2** (`security_v2.rs`): 安全指示器组
    *   [x] **Settings Item V2** (`settings_item_v2.rs`): 设置项组件
    *   [x] **Transaction Row V2** (`transaction_row_v2.rs`): 交易记录行
    *   [x] **Wallet Card V2** (`wallet_card_v2.rs`): 钱包卡片
    *   [x] **Word Selector** (`word_selector.rs`): 助记词选择器

**分子组件总计**: 12 个组件, ~2,500 行代码

### ✅ 已完成 (有机体组件) - 100% 完成

5.  **有机体组件 (Organisms)** - **已完成**
    *   [x] **Activity Feed** (`activity_feed.rs`): 活动记录流
    *   [x] **Asset Overview** (`asset_overview.rs`): 资产总览模块
    *   [x] **Bridge Interface** (`bridge_interface.rs`): 完整跨链桥界面
    *   [x] **Dashboard Header** (`dashboard_header.rs`): 仪表板头部
    *   [x] **Mobile Navigation** (`mobile_navigation.rs`): 移动端导航
    *   [x] **Notification Center** (`notification_center.rs`): 通知中心
    *   [x] **Security Dashboard** (`security_dashboard.rs`): 安全仪表板
    *   [x] **Settings Panel** (`settings_panel.rs`): 设置面板
    *   [x] **Transaction Modal** (`transaction_modal.rs`): 交易确认模态框
    *   [x] **Wallet Manager** (`wallet_manager.rs`): 钱包管理器

**有机体组件总计**: 10 个组件, ~3,000 行代码

### ✅ 已完成 (页面模板) - 100% 完成

6.  **页面模板 (Templates)** - **已完成**
    *   [x] **Activity V2** (`activity_v2.rs`): 活动记录页
    *   [x] **Assets V2** (`assets_v2.rs`): 资产列表页
    *   [x] **Bridge V2** (`bridge_v2.rs`): 跨链桥页
    *   [x] **Dashboard V2** (`dashboard_v2.rs`): 仪表板页
    *   [x] **Security V2** (`security_v2.rs`): 安全设置页
    *   [x] **Settings V2** (`settings_v2.rs`): 设置页
    *   [x] **Transaction Pages V2** (`transaction_pages_v2.rs`): 交易页面集

**页面模板总计**: 7 个模板, ~2,000 行代码

### ✅ 已完成 (视觉效果) - 100% 完成

7.  **视觉特效 (Visual Effects)** - **已完成**
    *   [x] **Canvas Background** (`canvas_background.rs`): 星空/星座连线背景
    *   [x] **Micro Interactions** (`micro_interactions.rs`): 按钮缩放、卡片悬浮动画
    *   [x] **Transitions** (`transitions.rs`): 页面过渡动画
    *   [x] **Zodia Layout** (`zodia_layout.rs`): 全局布局系统

**视觉效果总计**: 4 个效果组件, ~800 行代码

### ✅ 已完成 (清理工作) - 100% 完成

8.  **旧代码清理** - **已完成**
    *   [x] 删除 8 个旧 Atoms 组件
    *   [x] 删除 13 个 *_new Molecules 组件
    *   [x] 清理 70+ 处注释残留
    *   [x] 清理所有 TODO/FIXME 标记
    *   [x] 清理所有 Phase 版本注释
    *   [x] 零残留验证通过

### 🚧 进行中 (API 兼容性与 UI 统一) - 95% 完成

9.  **Dioxus 0.7 API 适配** - **接近完成**
    *   [x] 批量替换旧组件 API (NewBadge/AlertCard/TextInput 等)
    *   [x] 修复 TabItem → TabGroup
    *   [x] 修复事件处理器命名
    *   [x] 添加缺失的导入
    *   [x] 修复 async_std 依赖
    *   [x] 修复 Badge 组件重复属性问题 (8 个文件)
    *   [x] 修复 EventHandler 类型不匹配
    *   [x] 修复 Modal/Input props (is_open→open, onchange→oninput)
    *   [x] 修复返回类型 (Ok(VNode::empty()) → VNode::empty())
    *   [x] 修复 Signal 可变性 (show_wallet_selector, copied 等)
    *   [x] 修复 Option<String> move 问题 (icon 组件 as_ref())
    *   [x] 修复 for 循环结构 (toast_v2, create, gas_fee_card, settings_item, wallet_card, wallet_manager)
    *   [x] 修复 type annotation 问题 (input_v2, security_dashboard)
    *   [x] 修复剩余 4 个 E0283 类型推断错误
    *   [x] 修复剩余 4 个 E0597 生命周期错误
    *   [x] 修复剩余 7 个 E0716 临时值错误
    *   [x] 完成最终编译验证

**API 适配进度**: 初始 367 错误 → 当前 0 错误 (已修复 367 个, 100% 完成)

8.  UI 统一（最后阶段）
    *   [ ] 统一使用 V2 组件系统
    *   [ ] 更新所有页面引用

3.  业务复合组件 (Business Components)
    *   **导航与框架 (Navigation UI)**:
        *   [x] **Global Header**: 实现响应式顶部导航栏。
            *   **Left**: Logo (Zodia Wallet)。
            *   **Center**: 导航链接 [钱包 (Wallet), 卡片 (Cards), 收益 (Earnings), 空投 (Airdrops), 更多 (More)]。
            *   **Right**: 国际化切换 (Lang), 用户头像 (Avatar)。
            *   **Dropdown**: 头像下拉菜单 [我的钱包, 资产概览, 退出登录]。
        *   [x] **Mobile Tabbar**: 移动端底部导航，适配 iOS Safe Area。
        *   [x] **OfflineBanner**: 网络断开时的顶部红色警告条。
    *   **增长与营销 (Growth UI)** - *支撑 Airdrops/Earnings*:
        *   [x] **AirdropCard**: 空投项目卡片 (参考 Zodia)。包含：项目 Logo, 标题, 奖池金额 (USDT), 参与人数, 倒计时, "立即参与" 按钮, 状态标签 (HOT/NEW)。
        *   [x] **StatMetric**: 统计指标块 (如 "1,500+ 活跃活动", "$50M+ 总奖励")。
        *   [x] **EarningCard**: 理财产品卡片 (展示 APY/TVL/币种图标)，支撑 Phase 4 Earnings 功能。
        *   [ ] CreditCard: 区块链信用卡视觉组件 (CSS 3D 翻转效果, 显示卡号/余额)。
    *   **金融与交易 (Financial UI)**:
        *   [x] **AssetRow**: 资产列表行 (Token Icon + Balance + Fiat Value)。
        *   [x] **AssetGroup**: 资产分组容器 (按链家族分组展示，如 EVM/Solana/BTC)，支持折叠/展开。
        *   [x] **TxRow**: 交易记录行 (Status Icon + Hash + Amount)。
        *   [x] **GasSelector**: Gas 费率选择卡片 (Slow/Avg/Fast)。
        *   [x] **GasFeeCard**: Gas 详情展示卡片 (展示 Gas Price, Limit, 预估 ETH 消耗)，用于发送页。
        *   [x] **QRCodeDisplay**: 二维码展示组件，带 "Copy" 按钮。
        *   [x] **QRScanner**: 摄像头扫描组件 (调用设备相机识别地址)，支撑 Phase 4 扫码功能。
        *   [x] **TxReceipt**: 交易凭证模态框 (成功动画 + 关键字段 + 分享按钮)。
        *   [x] **TxDetailModal**: 历史交易详情模态框 (展示 Hash, Block, Gas, Time)，支撑 Phase 4 详情页。
        *   [x] **TxReviewModal**: 交易二次确认模态框 (展示 To, Amount, Fee, Total)，支撑 Phase 4 的安全校验。
        *   [x] **AmountInput**: 专用金额输入框 (带 "Max" 按钮 + 法币估值换算显示)。
        *   [x] **AddressInput**: 专用地址输入框 (集成 粘贴/扫描/ENS解析/格式校验)，支撑 Phase 1 地址校验。
        *   [x] **TokenImportModal**: 自定义代币导入表单 (输入合约地址自动获取元数据)，支撑 Phase 4 多代币功能。
    *   **安全与反馈 (Security UI)**:
        *   [x] **MnemonicGrid**: 助记词网格 (支持模糊/显示切换，支持下载备份文件)。
        *   [x] **MnemonicLengthSelector**: 助记词长度选择卡片 (12词/24词)，用于创建钱包流程。
        *   [x] **WordSelector**: 验证时的单词选择 Chip。
        *   [x] **PanicOverlay**: 全局崩溃拦截遮罩 (WASM Panic 时显示)，提供 "Reload" 按钮。
        *   [x] **LockScreen**: 自动锁定后的解锁遮罩 (全屏模糊 + 密码输入框 + "记住密码5分钟"选项)，支撑 Phase 1 的自动锁定。
        *   [x] **WalletCard**: 钱包列表卡片 (展示名称/地址/锁定状态)，支持点击解锁。
        *   [x] **ImportWalletModal**: 钱包导入模态框 (支持 助记词/私钥/Keystore 导入)，支撑 Phase 1 外部账户导入。
        *   [x] **BackupModal**: 密钥备份模态框 (二次验证密码后展示 助记词/私钥)，支撑 Phase 1 密钥管理。
        *   [x] **SignMessageModal**: 签名请求模态框 (展示消息内容 + 签名/拒绝按钮)，支撑 Phase 3 的 SIWE 认证。
    *   **通用交互 (Common UI)**:
        *   [x] **Avatar**: 用户头像组件 (支持 Blockies 渐变生成或图片上传)。
        *   [x] **EmptyState**: 空状态占位图 (无资产/无记录时显示插画)。
        *   [x] **BottomSheet**: 移动端底部抽屉 (替代 Modal)，优化手机操作体验。
        *   [x] **TokenSelector**: 代币选择器模态框 (带搜索栏 + 常用代币列表)。
        *   [x] **ChainSelector**: 链选择卡片 (多选/单选，带图标与勾选状态)，用于创建钱包时的多链选择。
        *   [x] **CopyButton**: 通用复制按钮，点击后显示 "Copied!" 气泡反馈。
        *   [x] **SettingsItem**: 设置列表项 (图标 + 标题 + 开关/箭头)，用于 "更多" 页面。
        *   [x] **StepIndicator**: 步骤进度条 (用于创建钱包/恢复流程导航)。
        *   [x] **AccountSwitcher**: 账户切换器 (展示多账户列表与余额概览)，支撑 Phase 1 多账户管理。
        *   [x] **LanguageSelector**: 语言切换下拉菜单/弹窗，支撑 Phase 2 国际化。

4.  视觉特效 (Visual Effects)
    *   [ ] **Hero 粒子系统**: 呼应 "Zodia" (黄道十二宫) 品牌，使用 Canvas 实现 **星空/星座连线 (Constellation)** 特效。
        *   *性能要求*: 必须运行在 `requestAnimationFrame` 中，且在低性能设备上自动降级 (减少粒子数)。
    *   [ ] **Micro-interactions**: 按钮点击缩放 (Scale), 卡片 Hover 上浮/光晕 (Glow)。
    *   [ ] **ScrollReveal**: 列表项进入视口时的交错淡入动画 (Staggered Fade-in)。

—

## 4A. 全栈集成与 UI 任务细化

目标: 将前三阶段的核心能力（安全加密、后端 API、多链架构）与 UI 深度绑定，实现"真实数据、真实加密、真实交互"。

1.  基础组件与架构接入 (UI Infrastructure)
    *   **参考**: `Phase 2 (Architecture)`, `Phase 3 (API)`
    *   [ ] **错误边界 (Error Boundary)**: 实现全局 `ErrorBoundary` 组件，捕获 WASM Panic，展示优雅的崩溃恢复页。
    *   [ ] **全局反馈 (Feedback System)**: 实现 `ToastProvider`，对接 Phase 2 的通知总线，展示 成功/错误/加载 状态。
    *   [ ] **国际化接入 (i18n Integration)**: 在所有原子组件 (Button, Input) 中集成翻译钩子，替换硬编码文本。
    *   [ ] **主题系统**: 完善 Tailwind 配置，确保深色模式 (Dark Mode) 覆盖所有新页面。

2.  安全核心集成 (Security Integration)
    *   **参考**: `Phase 1 (Security Core)`
    *   [ ] **真实加密存储**:
        *   **注册/创建**: 调用 `KeyManager::generate_mnemonic()` 生成真实 BIP39 助记词。
        *   **加密保存**: 调用 `Storage::save_encrypted(data, password)`，使用 Argon2id + AES-256-GCM 加密私钥至 IndexedDB。
    *   [ ] **多链地址派生 (Multi-Chain Derivation)**:
        *   在创建钱包时，同时派生并存储 **ETH (BIP44)**, **BTC (BIP84)**, **SOL (BIP44)**, **TON (Wallet V4)** 四条链的地址。
    *   [ ] **助记词验证流程**:
        *   **备份页**: 助记词默认模糊显示，点击"按住查看" (防止窥屏)。
        *   **验证页**: 实现 "按序点击单词" 的交互游戏，强制用户验证备份，禁止跳过。
    *   [ ] **自动锁定 (Auto-Lock)**:
        *   监听无操作超时事件，调用 `WalletState::lock()` 清除内存中的私钥。
        *   实现全屏遮罩层，强制要求输入密码解密。

3.  资产数据对接 (Real-Data Dashboard)
    *   **参考**: `Phase 3 (Backend API)`
    *   [ ] **真实余额同步**:
        *   对接 `GET /api/wallet/{id}/balance` 接口，替换 Mock 数据。
        *   实现 `use_resource` 自动轮询或 WebSocket 推送更新余额。
    *   [ ] **网络状态指示器**: 当 `window.offline` 时，顶部显示红色 "离线模式" 横幅，并禁用交易功能。
    *   [ ] **交易活动列表 (Activity List)**:
        *   实现交易历史组件，展示 `Type` (Send/Receive), `Amount`, `Status`, `Time`。
        *   特别处理 `Pending` 状态交易的视觉反馈。
    *   [ ] **多代币支持 (Multi-Token Support)**:
        *   实现 ERC20/SPL 代币列表渲染。
        *   实现 "Import Token" 功能 (输入合约地址自动发现代币符号/精度)。
    *   [ ] **交易详情页 (Transaction Details)**:
        *   点击历史记录进入详情页，展示 Tx Hash, Block, Gas Used。
        *   提供 "View on Explorer" 外部跳转链接。

4.  交易与 Gas 系统 (Transaction & Gas)
    *   **参考**: `Phase 3 (Gas & Lifecycle)`
    *   [ ] **真实 Gas 估算**:
        *   **发送页**: 调用 `GasService::estimate_fee(chain, to, amount)` 获取真实网络费率。
        *   **Gas 选择器**: 展示 慢/中/快 三档卡片 (包含 预估时间 + 费用 USD)。
    *   [ ] **智能金额输入 (Smart Amount Input)**:
        *   **最大发送 (Send Max)**: 实现 `Balance - Gas` 的自动计算逻辑。
        *   **余额预校验**: 实时检测余额是否足以支付 `Amount + Gas`，不足时禁用按钮。
    *   [ ] **交易确认视图 (Transaction Review)**:
        *   实现独立的确认模态框，强制用户二次核对 `To`, `Amount`, `Fee`, `Total`。
    *   [ ] **交易构建与广播**:
        *   **签名**: 调用 `KeyManager::sign_transaction(tx, password)` 在前端完成离线签名。
        *   **广播**: 调用 `TransactionService::broadcast(signed_tx)` 发送至后端/链上。
    *   [ ] **地址校验**:
        *   集成各链的地址校验库 (如 `checksum_address` for ETH)，输入时实时报错。
    *   [ ] **二维码集成 (QR Code Integration)**:
        *   **接收页**: 集成 `qrcode` crate，将当前地址转换为二维码图片。
        *   **扫描功能**: (可选) 尝试集成 `html5-qrcode` 实现摄像头扫描地址。

5.  高级视觉与营销 (Advanced Visuals)
    *   **参考**: `05-ui-ux/01-design-system-v2.md`
    *   [ ] **着陆页 (Landing Page)**:
        *   集成 `web-sys` + `canvas` 实现背景粒子网状连线特效 (Constellation Effect)。
        *   实现 Hero 区域文字渐入动画。
    *   [ ] **扩展功能页**: 完善 Earnings (理财), Airdrops (空投), Cards (卡片) 的 UI 细节与交互状态。

6.  移动端适配 (Mobile Adaptation)
    *   **参考**: `05-ui-ux/02-mobile-responsive.md`
    *   [ ] **PWA 配置**:
        *   添加 `manifest.json` (图标、名称、启动模式)。
        *   配置 Service Worker 实现离线 Shell 缓存。
    *   [ ] **触控优化**:
        *   优化按钮点击区域 (Min 44x44px)。
        *   禁用 iOS Safari 的双击缩放。

—

## 5. 生产就绪与优化 (Production Readiness)

**目标**: 优化性能，配置监控，确保达到上线标准。

1.  **性能优化**
    *   **参考**: `02-technical-design/06-modularization-best-practices.md`
    *   [ ] 实现路由懒加载 (Lazy Loading) 与代码分割。
    *   [ ] 优化长列表渲染 (Virtual Scrolling)。
    *   [ ] 优化 WASM 体积 (Strip, LTO)。

2.  **配置与监控**
    *   **参考**: `06-production/01-configuration-management.md`, `06-production/03-logging-system.md`
    *   [ ] 实现多环境配置 (Dev, Staging, Prod)。
    *   [ ] 集成前端日志系统，支持远程上报 (Sentry 或 自研)。
    *   [ ] 部署监控探针 (`06-production/04-monitoring-setup.md`)。

—

## 6. 测试与验收 (Testing & QA)

**目标**: 全面验证功能与安全，确保零故障上线。

1.  **测试执行**
    *   **参考**: `07-testing/01-testing-strategy.md`
    *   [ ] **单元测试 (80%)**: 覆盖所有 Crypto、State、Utils 逻辑。
    *   [ ] **集成测试 (15%)**: 验证 API 调用与组件交互。
    *   [ ] **E2E 测试 (5%)**: 使用 WebDriver/Selenium 模拟关键用户路径。

2.  **最终验收**
    *   [ ] 对照 `00-PRODUCTION-CHECKLIST.md` 逐项核查。
    *   [ ] 进行安全审计 (自查)。
    *   [ ] 编写发布说明与用户文档。
