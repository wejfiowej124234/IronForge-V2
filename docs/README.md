# IronForge Frontend Documentation

> **IronForge** - Enterprise-grade Non-Custodial Multi-Chain Cryptocurrency Wallet Web Frontend
> 
> Built with Dioxus 0.7 + Rust WASM + Tailwind CSS

---

## 📚 Documentation Structure

### 🏗️ Architecture
- [Security Architecture](./architecture/SECURITY_ARCHITECTURE.md) - 安全架构设计

### ✨ Features
- **Internationalization (i18n)**
  - [I18N Completion Report](./features/I18N_COMPLETION_REPORT.md) - 国际化完成报告
  - [I18N Keys Reference](./features/I18N_KEYS_REFERENCE.md) - 翻译 Key 完整参考
  - [I18N Implementation Guide](./features/I18N_GUIDE.md) - 国际化实现指南

- **Swap & Exchange**
  - [Swap Page Navigation](./features/SWAP_PAGE_NAVIGATION.md) - Swap 页面导航设计
  - [Swap Page Refactor](./features/REFACTOR_SWAP_PAGE.md) - Swap 页面重构
  - [Payment Analysis](./features/PAYMENT_ANALYSIS.md) - 支付系统分析

- **Send & Receive**
  - [Send Page Success Status](./features/SEND_PAGE_STATUS_SUCCESS.md) - 发送页面实现完成

### 📖 Guides
- [401 Auto Logout Implementation](./guides/401_AUTO_LOGOUT_IMPLEMENTATION_COMPLETE.md) - 401 自动登出实现
- [401 Safety Verification](./guides/401_SAFETY_VERIFICATION_REPORT.md) - 401 安全验证报告
- [Auth 401 Diagnostic Guide](./guides/AUTH_401_DIAGNOSTIC_GUIDE.md) - 401 错误诊断指南

### 🗄️ Deprecated
Historical documents moved to [deprecated/](./deprecated/) folder.

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

## 🌍 Internationalization (i18n)

IronForge supports **4 languages**:
- 🇨🇳 中文 (Chinese)
- 🇺🇸 English
- 🇯🇵 日本語 (Japanese)
- 🇰🇷 한국어 (Korean)

**Translation Coverage**: 135+ keys × 4 languages = **540+ translations**

See [I18N_KEYS_REFERENCE.md](./features/I18N_KEYS_REFERENCE.md) for all available translation keys.

---

## 📦 Project Structure

```
IronForge/
├── src/
│   ├── api/              # API client layer
│   ├── components/       # Reusable UI components
│   │   ├── atoms/        # Basic components (Button, Input, etc.)
│   │   ├── molecules/    # Composite components (Card, Modal, etc.)
│   │   └── organisms/    # Complex components (Navbar, Wallet, etc.)
│   ├── pages/            # Page-level components
│   │   ├── dashboard.rs  # Dashboard page
│   │   ├── send.rs       # Send page
│   │   ├── receive.rs    # Receive page
│   │   └── swap.rs       # Swap page (7000+ lines, main feature)
│   ├── i18n/             # Internationalization
│   │   ├── mod.rs        # i18n hooks
│   │   └── translations.rs # Translation dictionary (540+ entries)
│   ├── services/         # Business logic layer
│   ├── shared/           # Shared utilities
│   │   ├── state.rs      # Global app state
│   │   ├── design_tokens.rs # Design system colors
│   │   └── types.rs      # Shared types
│   └── main.rs           # Application entry point
├── public/               # Static assets
├── docs/                 # Documentation (this folder)
└── tests/                # Integration tests
```

---

## 🔐 Security Features

- ✅ **Non-Custodial**: Private keys never touch backend, 100% client-side encryption
- ✅ **Multi-Chain Support**: ETH, BSC, Polygon, Bitcoin (Solana coming soon)
- ✅ **Auto Logout**: Automatic session termination on 401 errors
- ✅ **Encrypted Storage**: IndexedDB with AES-256-GCM encryption
- ✅ **Memory Safety**: Rust's memory safety guarantees + zeroize for sensitive data

See [SECURITY_ARCHITECTURE.md](./architecture/SECURITY_ARCHITECTURE.md) for details.

---

## 🎨 Design System

- **Framework**: Dioxus 0.7 (React-like UI in Rust)
- **Styling**: Tailwind CSS v3
- **Colors**: Centralized design tokens in `src/shared/design_tokens.rs`
- **Components**: Atomic Design Pattern (Atoms → Molecules → Organisms)

---

## 🧪 Testing Strategy

- **Unit Tests**: Component-level testing
- **Integration Tests**: API integration testing
- **E2E Tests**: Selenium WebDriver for full user flows

See [tests/README.md](../tests/README.md) for testing guidelines.

---

## 📝 Contributing

1. Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
2. Use `cargo fmt` before committing
3. Ensure `cargo clippy` passes with no warnings
4. Add tests for new features
5. Update documentation

---

## 📄 License

This project is part of the IronGuard-AI ecosystem. See root README for license information.

---

## 🔗 Related Projects

- **IronCore**: Backend API (Axum + CockroachDB)
- **IronLink**: Mobile wallet (Android/iOS)
- **IronVault-XR**: AR/VR wallet interface
- **IronGuard-AI**: AI security layer

---

**Last Updated**: December 5, 2025  
**Status**: ✅ Production Ready (95% complete)  
**i18n Coverage**: 🌍 100% (4 languages, 540+ translations)
