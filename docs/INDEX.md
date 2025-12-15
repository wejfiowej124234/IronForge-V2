# IronForge Documentation Hub

> 🚀 **Version**: 2.0  
> 📅 **Last Updated**: December 6, 2025  
> 🌐 **i18n**: 4 languages, 540+ translations ✅  
> 📊 **Status**: Production Ready (95% complete)  
> 📚 **Documentation**: 48 files, 12 README indexes ⭐

---

## 🎯 Quick Navigation

### 🆕 Latest Updates (December 2025)
- 📚 **[Latest Updates Index](./latest-updates/README.md)** - All recent changes in one place ⭐
- 🌍 **[I18N System Complete](./02-technical-design/I18N_COMPLETION_REPORT.md)** - 4 languages, 135+ keys
- 📖 **[I18N Keys Reference](./02-technical-design/I18N_KEYS_REFERENCE.md)** - Complete translation guide
- 💳 **[Payment Analysis](./03-api-design/PAYMENT_ANALYSIS.md)** - MoonPay integration
- 🔐 **[401 Diagnostic Guide](./04-security/AUTH_401_DIAGNOSTIC_GUIDE.md)** - Auth troubleshooting

### 📚 Core Documentation (All Categories Have README Indexes)

#### [00-overview/](./00-overview/) → [📖 README](./00-overview/README.md)
**项目概览** - 了解 IronForge 的核心理念和目标
- 项目愿景、目标定位
- V1 → V2 演进历程
- 核心价值主张

#### [01-architecture/](./01-architecture/) → [📖 README](./01-architecture/README.md)
**系统架构** - 非托管钱包的技术架构设计
- 整体架构设计（四层架构）
- 数据分离模型（前端私钥 + 后端元数据）
- 数据库架构（CockroachDB）

#### [02-technical-design/](./02-technical-design/) → [📖 README](./02-technical-design/README.md) ⭐
**技术设计** - 技术选型、开发实践、i18n 系统
- 技术栈详解（Dioxus + Rust + WASM）
- 响应式设计、状态管理
- 开发指南、模块化最佳实践
- **🆕 国际化系统** (Dec 5, 2025)
  - [完成报告](./02-technical-design/I18N_COMPLETION_REPORT.md)
  - [实现指南](./02-technical-design/I18N_GUIDE.md)
  - [Key 参考手册](./02-technical-design/I18N_KEYS_REFERENCE.md) (135+ keys)

#### [03-api-design/](./03-api-design/) → [📖 README](./03-api-design/README.md) ⭐
**API 设计** - 前后端 API 集成、错误处理
- IronCore 后端 API 参考（46+ endpoints）
- 前端 API 封装层
- 错误处理策略、代币检测服务
- **🆕 [支付系统分析](./03-api-design/PAYMENT_ANALYSIS.md)** (Dec 4, 2025)

#### [04-security/](./04-security/) → [📖 README](./04-security/README.md) ⭐
**安全架构** - 密钥管理、加密策略、零信任架构
- 密钥管理方案（BIP39/BIP32/BIP44）
- 加密策略（AES-256-GCM + Argon2id）
- 安全架构设计、生产加密指南
- **🆕 [401 诊断指南](./04-security/AUTH_401_DIAGNOSTIC_GUIDE.md)** (Dec 4, 2025)

#### [05-ui-ux/](./05-ui-ux/) → [📖 README](./05-ui-ux/README.md)
**UI/UX 设计** - 设计系统、Logo 规范
- [设计系统 V3](./05-ui-ux/DESIGN_SYSTEM_V3.md) - 科技风+苹果风格+质感
- [Logo 设计规范](./05-ui-ux/LOGO_DESIGN.md)
- [Logo 使用指南](./05-ui-ux/LOGO_USAGE.md)

#### [06-production/](./06-production/) → [📖 README](./06-production/README.md)
**生产部署** - 配置管理、监控、部署
- 配置管理、错误处理系统
- 日志系统、监控告警
- 部署指南（Docker + K8s）

#### [07-testing/](./07-testing/) → [📖 README](./07-testing/README.md)
**测试策略** - 单元测试、集成测试、E2E
- 测试金字塔（80/15/5）
- 单元测试指南
- 集成测试、E2E 测试

#### [08-development/](./08-development/) → [📖 README](./08-development/README.md)
**开发指南** - 组件使用、开发计划
- 组件使用说明
- 开发计划、实现清单
- 路由状态、重构计划

#### [09-archive/](./09-archive/) → [📖 README](./09-archive/README.md)
**归档文档** - 历史文档存档

#### [latest-updates/](./latest-updates/) → [📖 README](./latest-updates/README.md) 🔥
**最新更新** - 2025年12月最新功能和文档

---

## 📊 Documentation Status

| Category | Files | Status | Last Update |
|----------|-------|--------|-------------|
| Overview | 2 | ✅ Complete | Nov 2025 |
| Architecture | 3 | ✅ Complete | Nov 2025 |
| Technical Design | 9 | ✅ Complete | **Dec 5, 2025** |
| API Design | 7 | ✅ Complete | **Dec 4, 2025** |
| Security | 5 | ✅ Complete | **Dec 4, 2025** |
| UI/UX | 4 | ✅ Complete | Nov 2025 |
| Production | 6 | ✅ Complete | Nov 2025 |
| Testing | 2 | ✅ Complete | Nov 2025 |
| Development | 7 | 🚧 In Progress | Nov 2025 |

**Total**: 48 documentation files

---

## 🔥 Must-Read Documents

### For New Developers
1. **[Project Vision](./00-overview/01-project-vision.md)** - Understand the goals
2. **[System Architecture](./01-architecture/01-system-architecture.md)** - Big picture
3. **[Development Guide](./02-technical-design/04-development-guide.md)** - Start coding
4. **[Components Usage](./08-development/COMPONENTS_USAGE.md)** - Use existing components

### For Frontend Developers
1. **[Tech Stack](./02-technical-design/01-tech-stack.md)** - Dioxus + Rust + WASM
2. **[State Management](./02-technical-design/03-state-management.md)** - Signals & Context
3. **[Design System V3](./05-ui-ux/DESIGN_SYSTEM_V3.md)** - UI components & colors
4. **[I18N Guide](./02-technical-design/I18N_GUIDE.md)** - Adding translations

### For Backend Integration
1. **[IronCore API Reference](./03-api-design/01-ironcore-backend-api-reference.md)** - All endpoints
2. **[Frontend API Layer](./03-api-design/02-frontend-api-layer.md)** - How to call APIs
3. **[Error Handling](./03-api-design/03-error-handling.md)** - Proper error patterns

### For Security Auditors
1. **[Key Management](./04-security/01-key-management.md)** - Private key handling
2. **[Encryption Strategy](./04-security/02-encryption-strategy.md)** - AES-256-GCM
3. **[Security Architecture](./04-security/03-security-architecture.md)** - Non-custodial design
4. **[Production Encryption Guide](./04-security/05-production-encryption-guide.md)** - Best practices

### For DevOps
1. **[Configuration Management](./06-production/01-configuration-management.md)** - Env vars
2. **[Logging System](./06-production/03-logging-system.md)** - Structured logging
3. **[Monitoring Setup](./06-production/04-monitoring-setup.md)** - Prometheus + Grafana
4. **[Deployment Guide](./06-production/05-deployment-guide.md)** - Docker + K8s

---

## 🎨 Project Features

### ✅ Completed Features
- 🔐 **Non-Custodial Wallet** - Client-side key management
- 🌍 **Multi-Chain Support** - ETH, BSC, Polygon, Bitcoin
- 🌐 **Internationalization** - 4 languages (中文/English/日本語/한국어)
- 💱 **Token Swap** - DEX integration
- 💳 **Buy Stablecoin** - Fiat to crypto
- 💰 **Withdraw/Sell** - Crypto to fiat
- 📤 **Send Tokens** - Multi-chain transfers
- 📥 **Receive Tokens** - QR codes
- 🔒 **Auto Logout** - 401 error handling
- 🎨 **Modern UI** - Apple-style design system

### 🚧 In Progress
- 📊 **Limit Orders** (80% complete)
- 📜 **Transaction History** (80% complete)
- ⚙️ **Advanced Settings** (60% complete)

### 📋 Planned
- 🔗 **Solana Integration** (Q1 2026)
- 🔐 **Hardware Wallet Support** (Q1 2026)
- 📱 **Mobile PWA** (Q2 2026)

---

## 🚀 Quick Start

### Development Server
```bash
cd IronForge
trunk serve --port 8081 --open
```

### Build for Production
```bash
trunk build --release
```

### Run Tests
```bash
cargo test --workspace
```

---

## 📝 Contributing to Documentation

### Adding New Docs
1. Choose appropriate category (00-08)
2. Use numbered prefix for ordering
3. Follow existing markdown style
4. Update this README with link
5. Add to category README if exists

### Updating Existing Docs
1. Update date in document header
2. Add changelog section if major changes
3. Update "Last Update" in status table above
4. Mark as **NEW** or **Updated** in TOC

### Documentation Standards
- ✅ Use clear, descriptive titles
- ✅ Include code examples where relevant
- ✅ Add diagrams for complex concepts
- ✅ Keep language consistent (EN/CN sections)
- ✅ Update timestamps on changes

---

## 🔗 Related Resources

### Internal Links
- [Main Project README](../README.md)
- [IronCore Backend Docs](../../IronCore/docs/)
- [Testing Guide](../tests/README.md)
- [Scripts Documentation](../scripts/README.md)

### External Links
- [Dioxus Documentation](https://dioxuslabs.com/learn/0.5/)
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [Tailwind CSS](https://tailwindcss.com/docs)

---

## 📞 Support

- 💬 **Issues**: GitHub Issues
- 📧 **Email**: team@ironforge.dev
- 📚 **Docs**: You're here!

---

**Documentation Maintained By**: IronForge Development Team  
**Last Full Review**: December 5, 2025  
**Next Review**: January 2026
