# Logo 使用指南

> **组件位置**: `src/components/logo.rs`  
> **设计文档**: `LOGO_DESIGN.md`  
> **更新日期**: 2025-11-27

---

## 🚀 快速开始

### 基础使用

```rust
use crate::components::logo::{LogoPlanet, LogoSize, LogoVariant};

// 标准 Logo
rsx! {
    LogoPlanet {
        size: LogoSize::Large,
        variant: LogoVariant::Standard,
    }
}
```

### 带文字的 Logo

```rust
use crate::components::logo::{LogoWithText, LogoSize, LogoVariant};

rsx! {
    LogoWithText {
        size: LogoSize::Large,
        variant: LogoVariant::Standard,
    }
}
```

---

## 📐 尺寸选项

```rust
// 小尺寸 - Favicon, 小图标
LogoPlanet { size: LogoSize::Small, .. }

// 中等尺寸 - 移动端导航栏
LogoPlanet { size: LogoSize::Medium, .. }

// 大尺寸 - 桌面端导航栏（默认）
LogoPlanet { size: LogoSize::Large, .. }

// 超大尺寸 - 营销页面 Hero
LogoPlanet { size: LogoSize::XLarge, .. }

// 自定义尺寸
LogoPlanet { size: LogoSize::Custom(96), .. }
```

---

## 🎨 变体选项

### Standard - 标准版

纯白星球，适合通用场景。

```rust
LogoPlanet {
    size: LogoSize::Large,
    variant: LogoVariant::Standard,
}
```

**使用场景**:
- 导航栏
- 页面标题
- 通用展示

### Gradient - 渐变版

白色到浅蓝渐变，增强科技感。

```rust
LogoPlanet {
    size: LogoSize::XLarge,
    variant: LogoVariant::Gradient,
}
```

**使用场景**:
- 营销页面
- Hero 区域
- 强调展示

### Glowing - 发光版

强化光晕效果，视觉冲击力强。

```rust
LogoPlanet {
    size: LogoSize::XLarge,
    variant: LogoVariant::Glowing,
}
```

**使用场景**:
- 营销页面 Hero
- CTA 按钮
- 重要位置

### Minimal - 简化版

减少细节，适合小尺寸。

```rust
LogoPlanet {
    size: LogoSize::Small,
    variant: LogoVariant::Minimal,
}
```

**使用场景**:
- Favicon
- 小图标
- 移动端导航栏

---

## 💡 使用示例

### 导航栏 Logo

```rust
use crate::components::logo::{LogoWithText, LogoSize, LogoVariant};

rsx! {
    nav {
        class: "flex items-center gap-4 px-6 py-4",
        LogoWithText {
            size: LogoSize::Medium,
            variant: LogoVariant::Standard,
            text_size: Some("text-lg".to_string()),
            text_color: Some("text-white".to_string()),
        }
    }
}
```

### 营销页面 Hero

```rust
use crate::components::logo::{LogoPlanet, LogoSize, LogoVariant};

rsx! {
    section {
        class: "flex flex-col items-center justify-center min-h-screen",
        LogoPlanet {
            size: LogoSize::XLarge,
            variant: LogoVariant::Glowing,
        }
        h1 {
            class: "text-5xl font-bold mt-8",
            "IronForge"
        }
    }
}
```

### 加载动画

```rust
use crate::components::logo::{LogoPlanet, LogoSize, LogoVariant};

rsx! {
    div {
        class: "flex flex-col items-center justify-center min-h-screen",
        LogoPlanet {
            size: LogoSize::Large,
            variant: LogoVariant::Standard,
        }
        p {
            class: "mt-4 text-white/60",
            "加载中..."
        }
    }
}
```

### Favicon

```rust
use crate::components::logo::{LogoPlanet, LogoSize, LogoVariant};

rsx! {
    LogoPlanet {
        size: LogoSize::Small,
        variant: LogoVariant::Minimal,
        class: Some("favicon".to_string()),
    }
}
```

---

## 🎨 自定义样式

### 添加自定义类名

```rust
LogoPlanet {
    size: LogoSize::Large,
    variant: LogoVariant::Standard,
    class: Some("my-custom-class".to_string()),
}
```

### 自定义文字样式

```rust
LogoWithText {
    size: LogoSize::Large,
    variant: LogoVariant::Standard,
    text_size: Some("text-2xl font-bold".to_string()),
    text_color: Some("text-blue-400".to_string()),
}
```

---

## 🔧 技术细节

### 动画性能

- 使用 CSS 动画，性能优秀
- 旋转动画：20秒/圈，60fps
- 光晕脉冲：3秒周期
- 坑洼动画：随机延迟，营造自然感

### 响应式适配

Logo 会自动适配不同尺寸：
- **小尺寸** (< 48px): 简化坑洼，减少光晕
- **中等尺寸** (48-128px): 标准配置
- **大尺寸** (> 128px): 完整细节

### 浏览器兼容性

- 现代浏览器：完整支持
- SVG 动画：Chrome, Firefox, Safari, Edge
- 降级方案：静态 Logo（无动画）

---

## 📝 注意事项

1. **性能优化**: Logo 使用 SVG，矢量缩放，性能优秀
2. **动画控制**: 可以通过 CSS 控制动画播放/暂停
3. **可访问性**: Logo 包含 `aria-label`，支持屏幕阅读器
4. **SEO**: 建议在 Logo 周围添加适当的语义化标签

---

## 🎯 最佳实践

1. **导航栏**: 使用 `LogoSize::Medium` + `LogoVariant::Standard`
2. **营销页面**: 使用 `LogoSize::XLarge` + `LogoVariant::Glowing`
3. **Favicon**: 使用 `LogoSize::Small` + `LogoVariant::Minimal`
4. **加载动画**: 使用 `LogoSize::Large` + `LogoVariant::Standard`

---

**最后更新**: 2025-11-27

