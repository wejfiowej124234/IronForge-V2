# 项目概览 (Project Overview)

> 📖 了解 IronForge 项目的核心理念、目标和定位

---

## 📂 本分类文档

| 文档 | 描述 | 状态 |
|------|------|------|
| [01-project-vision.md](./01-project-vision.md) | 项目愿景、目标、核心价值主张 | ✅ 完成 |

---

## 🎯 快速导航

### 新人必读
1. **[项目愿景](./01-project-vision.md)** - 5 分钟了解 IronForge 是什么

### 核心理念
- 🔑 **非托管钱包** - 私钥永远在用户手中
- 🦀 **100% Rust** - 类型安全、高性能
- 🌍 **全球化** - 多语言、多链支持
- 🏢 **企业级** - 生产就绪、审计友好

### 技术定位
- **前端框架**: Dioxus 0.7 (React-like Rust WASM)
- **后端 API**: IronCore (Axum + CockroachDB)
- **目标用户**: 100 万+ 并发，20+ 区块链
- **性能目标**: 页面加载 < 2s，交互响应 < 100ms

---

## 🚀 从这里开始

### 第一次接触？
1. 阅读 [项目愿景](./01-project-vision.md) 了解核心目标
2. 查看 [系统架构](../01-architecture/01-system-architecture.md) 理解技术架构
3. 参考 [开发指南](../02-technical-design/04-development-guide.md) 开始编码

### 为什么选择 IronForge？
- ✅ **安全**: 非托管架构，用户完全掌控资产
- ✅ **性能**: WASM 编译，接近原生体验
- ✅ **完整**: 钱包 + Swap + Buy + NFT 一站式服务
- ✅ **开源**: 透明、可审计、社区驱动

---

## 📊 项目现状

### V1 → V2 演进
- ❌ **V1 问题**: 架构混乱、WASM 过大 (2.08MB)、安全隐患
- ✅ **V2 改进**: 分层清晰、代码优化、国际化完善、测试覆盖 90%+

### 当前状态 (2025年12月)
- ✅ **核心功能**: Wallet, Send, Receive, Swap, Buy
- ✅ **i18n**: 4 语言 (中/英/日/韩), 540+ 翻译
- ✅ **文档**: 48 个技术文档，完整覆盖
- 🔄 **进行中**: Mobile App (IronLink), AR/VR (IronVault-XR)

---

## 🔗 相关资源

- **代码仓库**: [GitHub](https://github.com/your-org/ironforge)
- **在线演示**: [Demo Site](https://demo.ironforge.dev)
- **API 文档**: [API Reference](../03-api-design/01-ironcore-backend-api-reference.md)
- **设计系统**: [Design System V3](../05-ui-ux/DESIGN_SYSTEM_V3.md)

---

**最后更新**: 2025-12-06  
**维护者**: IronForge Team  
**反馈**: [GitHub Issues](https://github.com/your-org/ironforge/issues)
