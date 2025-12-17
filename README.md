# IronForge - Enterprise Web3 Wallet Frontend

> 🚀 **版本**: 2.0  
> 📅 **最后更新**: 2025-12-06  
> 🎯 **目标**: 构建下一代企业级 Web3 钱包前端  
> 🌐 **i18n**: 4+ languages (see docs) ✅  
> 📚 **Documentation**: 57 files, 27,437 lines, 12 README indexes ⭐

---

## 📚 完整文档系统 (Enterprise-Grade Documentation)

**➡️ [📖 Documentation Hub](./docs/INDEX.md)** - 中心索引，一站式导航 ⭐

### 🔥 最新更新 (2025年12月)
- 🌍 **[国际化系统完成](./docs/02-technical-design/I18N_COMPLETION_REPORT.md)** - 4 语言, 135+ keys, 540+ 翻译
- 📚 **[文档深度整理完成](./docs/latest-updates/DEEP_DOCUMENTATION_OPTIMIZATION_REPORT.md)** - 12 README 索引, 三层导航
- 💳 **[MoonPay 支付集成](./docs/03-api-design/PAYMENT_ANALYSIS.md)** - 法币购买加密货币
- 🔐 **[401 错误诊断指南](./docs/04-security/AUTH_401_DIAGNOSTIC_GUIDE.md)** - 认证问题快速排查

### 📂 文档分类导航 (All Categories Have README Indexes)

| 分类 | 简介 | 文档数 | 快速链接 |
|------|------|--------|----------|
| **[00-overview](./docs/00-overview/)** | 项目概览 | 2 | [📖 README](./docs/00-overview/README.md) |
| **[01-architecture](./docs/01-architecture/)** | 系统架构 | 3 | [📖 README](./docs/01-architecture/README.md) |
| **[02-technical-design](./docs/02-technical-design/)** | 技术设计 ⭐ | 8 | [📖 README](./docs/02-technical-design/README.md) |
| **[03-api-design](./docs/03-api-design/)** | API 设计 ⭐ | 7 | [📖 README](./docs/03-api-design/README.md) |
| **[04-security](./docs/04-security/)** | 安全架构 ⭐ | 5 | [📖 README](./docs/04-security/README.md) |
| **[05-ui-ux](./docs/05-ui-ux/)** | UI/UX 设计 | 4 | [📖 README](./docs/05-ui-ux/README.md) |
| **[06-production](./docs/06-production/)** | 生产部署 | 6 | [📖 README](./docs/06-production/README.md) |
| **[07-testing](./docs/07-testing/)** | 测试策略 | 2 | [📖 README](./docs/07-testing/README.md) |
| **[08-development](./docs/08-development/)** | 开发指南 | 7 | [📖 README](./docs/08-development/README.md) |
| **[latest-updates](./docs/latest-updates/)** | 最新更新 🔥 | 3 | [📖 README](./docs/latest-updates/README.md) |

**总计**: 57 个文档, 27,437 行, 100% 覆盖 ✅

---

## 📖 项目简介

IronForge 是一个基于 Rust + Dioxus 构建的企业级 Web3 钱包前端应用，支持多链（Bitcoin、Ethereum、Solana、TON）资产管理，提供安全、高效、现代化的用户体验。

### 核心特性

- 🔐 **安全第一**: 零信任架构，内存安全保证，完善的密钥管理
- ⚡ **高性能**: WASM 优化，虚拟滚动，智能缓存
- 🎨 **现代 UI**: 苹果风格设计系统，毛玻璃效果，流畅动画
- 🌍 **国际化**: 支持多语言（以 `docs/02-technical-design/` 的 i18n 文档为准）
- 📱 **响应式**: Mobile-First 设计，完美适配各种设备

---

## 🚀 快速开始

### 环境要求

- Rust stable (推荐使用 rustup)
- Node.js 20+（用于 Tailwind CSS；CI 使用 Node 20）
- Trunk 0.21.14（CI 固定版本；建议保持一致）

### 安装依赖

```bash
# 安装 Rust 依赖
cargo build

# 安装 Node.js 依赖
npm ci
```

### 开发模式

```bash
# 启动开发服务器（自动热重载）
# 说明：Trunk build hook 会自动执行 `npm run build:css`
trunk serve

# 监听 Tailwind CSS 变化
npm run watch:css
```

### 构建生产版本

```bash
# 构建 WASM
trunk build --release

# 构建 CSS
npm run build:css
```

---

## 🚀 生产部署（当前实现）

本仓库已接入 GitHub Actions 自动部署：

- GitHub Pages：push 到 `main` 会发布 `dist/`
- Fly.io：push 到 `main` 会通过 `flyctl deploy` 部署到 `oxidevault-ironforge-v2`

### 必要配置

- `FLY_API_TOKEN`：GitHub 仓库 Actions Secret（必需；缺失会导致 Fly 部署失败）
- `API_BASE_URL`：可选 GitHub Actions Variable（用于编译期注入后端 API Base URL）

相关文件：

- `.github/workflows/deploy.yml`
- `fly.toml`
- `Dockerfile`

---

## 🎯 必读文档推荐

### 👨‍💻 新人开发者
1. **[项目愿景](./docs/00-overview/01-project-vision.md)** - 5 分钟了解 IronForge
2. **[系统架构](./docs/01-architecture/01-system-architecture.md)** - 理解整体架构
3. **[开发指南](./docs/02-technical-design/04-development-guide.md)** - 快速上手开发
4. **[i18n 实现指南](./docs/02-technical-design/I18N_GUIDE.md)** - 如何添加翻译

### 🎨 前端工程师
1. **[技术栈选型](./docs/02-technical-design/01-tech-stack.md)** - Dioxus + Rust + WASM
2. **[状态管理](./docs/02-technical-design/03-state-management.md)** - Signal & Context 使用
3. **[设计系统 V3](./docs/05-ui-ux/DESIGN_SYSTEM_V3.md)** - 苹果风格 UI 组件
4. **[API 封装层](./docs/03-api-design/02-frontend-api-layer.md)** - 如何调用后端 API

### 🔐 安全审计人员
1. **[密钥管理](./docs/04-security/01-key-management.md)** - 私钥生成、存储、派生
2. **[加密策略](./docs/04-security/02-encryption-strategy.md)** - AES-256-GCM + Argon2id
3. **[安全架构](./docs/04-security/03-security-architecture.md)** - 非托管零信任架构
4. **[数据分离模型](./docs/01-architecture/02-data-separation-model.md)** - 前后端数据分离

### 🚀 DevOps / SRE
1. **[配置管理](./docs/06-production/01-configuration-management.md)** - 环境变量配置
2. **[监控告警](./docs/06-production/04-monitoring-setup.md)** - Prometheus + Grafana
3. **[部署指南](./docs/06-production/05-deployment-guide.md)** - Docker + K8s 部署
4. **[日志系统](./docs/06-production/03-logging-system.md)** - 结构化日志


---

## 🏗️ 项目结构

```
IronForge/
├── src/                    # Rust 源代码
│   ├── blockchain/        # 区块链集成（BTC, ETH, SOL, TON）
│   ├── components/        # UI 组件（正在重构中）
│   ├── features/          # 功能模块（业务逻辑）
│   ├── services/          # 业务服务层
│   ├── shared/            # 共享工具与状态
│   ├── crypto/            # 加密与密钥管理
│   └── archive/           # 旧UI代码备份
├── docs/                  # 📚 完整文档目录
├── scripts/               # 脚本文件
├── public/                # 静态资源
└── Cargo.toml            # Rust 依赖配置
```

---

## 🛠️ 技术栈

- **前端框架**: Dioxus 0.7 (Rust Web Framework)
- **样式**: Tailwind CSS (wasm-css)
- **构建工具**: Trunk
- **加密**: AES-256-GCM, Argon2id
- **区块链**: 多链支持（Bitcoin, Ethereum, Solana, TON）

---

## 📝 开发规范

### 代码风格

- 遵循 Rust 官方代码风格（rustfmt）
- 使用 Clippy 进行代码检查
- 所有公共 API 必须有文档注释

### Git 提交规范

```
<type>(<scope>): <subject>

<body>

<footer>
```

类型：
- `feat`: 新功能
- `fix`: 修复
- `docs`: 文档
- `style`: 格式
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具

---

## 🤝 贡献指南

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'feat: Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 📄 许可证

本项目采用 [MIT License](./LICENSE) 许可证。

---

## 📞 联系方式

- **问题反馈**: [GitHub Issues](https://github.com/wejfiowej124234/IronForge-V2/issues)
- **项目文档**: [📖 Documentation Hub](./docs/INDEX.md)

---

**最后更新**: 2025-11-27  
**文档版本**: v2.0.0

