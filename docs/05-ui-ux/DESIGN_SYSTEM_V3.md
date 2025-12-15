# IronForge 设计系统 V3 - 专业版

> **版本**: 3.0  
> **设计理念**: 苹果风格 + 未来科技 + 智能支付 + 眼镜支付 + 层次感 + 质感  
> **参考**: Juno Network, Apple Design, 现代区块链钱包  
> **更新日期**: 2025-11-27

---

## 🎨 设计理念

### 核心风格定位

1. **苹果风格** - 极简、优雅、精致、注重细节
2. **未来科技** - 数字化、数据可视化、智能交互
3. **智能支付** - 流畅、安全、专业、可信
4. **眼镜支付** - AR/VR 元素、空间感、沉浸式体验
5. **层次感** - 深度、立体、光影、材质
6. **质感** - 细腻、高级、专业、金融级

### 设计原则

- **营销页面**: 视觉冲击力强，展示品牌和产品优势
- **功能页面**: 干净整洁，专注服务，提供优质体验
- **一致性**: 统一的设计语言，但允许不同场景的差异化

---

## 🎨 颜色系统

### 主色调（深色科技风格）

```css
/* 背景色系 - 深色科技感 */
--bg-primary: #0A0A0F;           /* 深空黑（主背景） */
--bg-secondary: #12121A;          /* 深灰蓝（卡片背景） */
--bg-tertiary: #1A1A24;          /* 中灰蓝（悬浮卡片） */
--bg-elevated: #24242F;          /* 提升层（模态框） */

/* 科技蓝紫渐变系统 */
--tech-primary: #6366F1;         /* 靛蓝（主色） */
--tech-secondary: #8B5CF6;       /* 紫色（辅助色） */
--tech-accent: #06B6D4;           /* 青色（强调色） */
--tech-glow: #A78BFA;             /* 光晕色 */

/* 智能支付色系 */
--payment-primary: #10B981;       /* 支付绿 */
--payment-success: #34D399;       /* 成功绿 */
--payment-warning: #F59E0B;       /* 警告橙 */
--payment-error: #EF4444;         /* 错误红 */

/* 中性色 */
--text-primary: #FFFFFF;           /* 主文本 */
--text-secondary: #E5E7EB;       /* 次要文本 */
--text-tertiary: #9CA3AF;         /* 三级文本 */
--text-disabled: #6B7280;         /* 禁用文本 */

/* 边框与分割线 */
--border-primary: rgba(255, 255, 255, 0.1);
--border-secondary: rgba(255, 255, 255, 0.05);
--divider: rgba(255, 255, 255, 0.08);
```

### 渐变系统

```css
/* 主渐变 - 科技蓝紫 */
--gradient-primary: linear-gradient(135deg, #6366F1 0%, #8B5CF6 50%, #06B6D4 100%);
--gradient-secondary: linear-gradient(135deg, #8B5CF6 0%, #A78BFA 100%);
--gradient-accent: linear-gradient(135deg, #06B6D4 0%, #6366F1 100%);

/* 智能支付渐变 */
--gradient-payment: linear-gradient(135deg, #10B981 0%, #34D399 100%);
--gradient-success: linear-gradient(135deg, #34D399 0%, #10B981 100%);

/* 背景渐变（营销页面用） */
--gradient-bg-hero: radial-gradient(ellipse at top, rgba(99, 102, 241, 0.15) 0%, transparent 50%);
--gradient-bg-card: linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(139, 92, 246, 0.1) 100%);
```

### 光晕与发光效果

```css
/* 科技光晕 */
--glow-primary: 0 0 20px rgba(99, 102, 241, 0.4);
--glow-secondary: 0 0 30px rgba(139, 92, 246, 0.3);
--glow-accent: 0 0 40px rgba(6, 182, 212, 0.2);

/* 智能支付光晕 */
--glow-payment: 0 0 25px rgba(16, 185, 129, 0.5);
--glow-success: 0 0 30px rgba(52, 211, 153, 0.4);
```

---

## 🏗️ 材质系统

### 毛玻璃效果（Glassmorphism）

```css
/* 基础毛玻璃 - 功能页面 */
.glass-base {
  background: rgba(18, 18, 26, 0.6);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 
    0 8px 32px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

/* 强化毛玻璃 - 营销页面 */
.glass-strong {
  background: rgba(26, 26, 36, 0.7);
  backdrop-filter: blur(30px) saturate(200%);
  -webkit-backdrop-filter: blur(30px) saturate(200%);
  border: 1px solid rgba(255, 255, 255, 0.15);
  box-shadow: 
    0 12px 48px rgba(0, 0, 0, 0.5),
    inset 0 1px 0 rgba(255, 255, 255, 0.15),
    0 0 40px rgba(99, 102, 241, 0.1);
}

/* 提升层毛玻璃 - 模态框 */
.glass-elevated {
  background: rgba(36, 36, 47, 0.8);
  backdrop-filter: blur(40px) saturate(220%);
  -webkit-backdrop-filter: blur(40px) saturate(220%);
  border: 1px solid rgba(255, 255, 255, 0.2);
  box-shadow: 
    0 20px 60px rgba(0, 0, 0, 0.6),
    inset 0 1px 0 rgba(255, 255, 255, 0.2);
}
```

### 质感阴影系统

```css
/* 苹果风格多层次阴影 */
.shadow-apple {
  box-shadow: 
    0 2px 8px rgba(0, 0, 0, 0.2),
    0 8px 24px rgba(0, 0, 0, 0.3),
    0 16px 48px rgba(0, 0, 0, 0.2);
}

/* 科技光晕阴影 */
.shadow-tech {
  box-shadow: 
    0 4px 16px rgba(99, 102, 241, 0.3),
    0 8px 32px rgba(99, 102, 241, 0.2),
    0 0 40px rgba(99, 102, 241, 0.1);
}

/* 智能支付阴影 */
.shadow-payment {
  box-shadow: 
    0 4px 20px rgba(16, 185, 129, 0.4),
    0 8px 40px rgba(16, 185, 129, 0.2);
}

/* 内发光 */
.inner-glow {
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.15);
}

/* 外发光 */
.outer-glow {
  box-shadow: 
    0 0 20px rgba(99, 102, 241, 0.5),
    0 0 40px rgba(99, 102, 241, 0.3);
}
```

### 层次感系统

```css
/* 层次1 - 背景层 */
.layer-bg {
  z-index: 0;
  background: var(--bg-primary);
}

/* 层次2 - 内容层 */
.layer-content {
  z-index: 10;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
}

/* 层次3 - 悬浮层 */
.layer-floating {
  z-index: 20;
  background: var(--bg-tertiary);
  box-shadow: var(--shadow-apple);
}

/* 层次4 - 模态层 */
.layer-modal {
  z-index: 30;
  background: var(--bg-elevated);
  box-shadow: var(--shadow-tech);
}
```

---

## 📐 间距系统

### 间距单位（8px 基准）

```css
--spacing-xs: 4px;      /* 极小间距 */
--spacing-sm: 8px;      /* 小间距 */
--spacing-md: 16px;     /* 中等间距 */
--spacing-lg: 24px;     /* 大间距 */
--spacing-xl: 32px;     /* 超大间距 */
--spacing-2xl: 48px;    /* 2倍超大 */
--spacing-3xl: 64px;    /* 3倍超大 */
--spacing-4xl: 96px;    /* 4倍超大（营销页面用） */
```

### 圆角系统

```css
--radius-xs: 6px;       /* 极小圆角 */
--radius-sm: 8px;       /* 小圆角 */
--radius-md: 12px;      /* 中等圆角 */
--radius-lg: 16px;      /* 大圆角 */
--radius-xl: 20px;      /* 超大圆角 */
--radius-2xl: 24px;     /* 2倍超大圆角 */
--radius-full: 9999px;  /* 完全圆形 */
```

---

## 🔤 字体系统

### 字体族

```css
/* 主字体 - 苹果系统字体 */
font-family: -apple-system, BlinkMacSystemFont, 
  "SF Pro Display", "SF Pro Text", 
  "Helvetica Neue", "Segoe UI", 
  "Roboto", sans-serif;

/* 等宽字体 - 地址、代码 */
font-family: "SF Mono", "Monaco", 
  "Menlo", "Consolas", monospace;
```

### 字重

```css
--font-light: 300;      /* 细体 */
--font-normal: 400;     /* 常规 */
--font-medium: 500;      /* 中等 */
--font-semibold: 600;   /* 半粗 */
--font-bold: 700;       /* 粗体 */
```

### 字号系统

```css
/* 营销页面 */
--text-hero: 64px;      /* Hero 标题 */
--text-display: 48px;   /* 展示标题 */
--text-h1: 36px;        /* 一级标题 */
--text-h2: 28px;        /* 二级标题 */
--text-h3: 24px;        /* 三级标题 */
--text-h4: 20px;        /* 四级标题 */
--text-body-lg: 18px;   /* 大正文 */
--text-body: 16px;      /* 正文 */
--text-body-sm: 14px;   /* 小正文 */
--text-caption: 12px;   /* 说明文字 */

/* 功能页面 */
--text-h1: 28px;        /* 一级标题 */
--text-h2: 24px;        /* 二级标题 */
--text-h3: 20px;        /* 三级标题 */
--text-body: 16px;      /* 正文 */
--text-body-sm: 14px;   /* 小正文 */
--text-caption: 12px;   /* 说明文字 */
```

---

## 🎭 组件设计规范

### 营销页面组件

#### Hero 区域
- **背景**: 深色渐变 + 科技光晕
- **标题**: 大字号（64px+），渐变文字
- **按钮**: 大尺寸，渐变背景，光晕效果
- **视觉**: 3D 卡片、粒子效果、动画

#### 功能卡片
- **背景**: 强化毛玻璃 + 渐变边框
- **图标**: 大尺寸，发光效果
- **悬停**: 上浮 + 光晕增强

#### CTA 按钮
- **尺寸**: 大（56px 高度）
- **背景**: 渐变 + 光晕
- **动画**: 悬停放大 + 光晕增强

### 功能页面组件

#### 卡片组件
- **背景**: 基础毛玻璃
- **边框**: 细边框（1px）
- **圆角**: 16px
- **阴影**: 苹果风格多层次阴影
- **内边距**: 24px

#### 按钮组件

**主要按钮**:
```css
.btn-primary {
  background: var(--gradient-primary);
  color: white;
  padding: 12px 24px;
  border-radius: 12px;
  font-weight: 600;
  box-shadow: var(--shadow-tech);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: 
    var(--shadow-tech),
    var(--glow-primary);
}
```

**次要按钮**:
```css
.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-primary);
  padding: 12px 24px;
  border-radius: 12px;
  font-weight: 500;
}
```

**智能支付按钮**:
```css
.btn-payment {
  background: var(--gradient-payment);
  color: white;
  padding: 14px 28px;
  border-radius: 12px;
  font-weight: 600;
  box-shadow: var(--shadow-payment);
}
```

#### 输入框组件
```css
.input-field {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 12px;
  padding: 12px 16px;
  color: var(--text-primary);
  font-size: 16px;
  transition: all 0.3s;
}

.input-field:focus {
  border-color: var(--tech-primary);
  box-shadow: 
    0 0 0 3px rgba(99, 102, 241, 0.1),
    var(--glow-primary);
  outline: none;
}
```

---

## 🎬 动画系统

### 过渡动画

```css
/* 标准过渡 */
transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);

/* 快速过渡 */
transition: all 0.2s ease-out;

/* 慢速过渡 */
transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1);
```

### 微交互

```css
/* 按钮点击 */
.btn:active {
  transform: scale(0.98);
}

/* 卡片悬停 */
.card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-tech);
}

/* 输入聚焦 */
.input:focus {
  transform: scale(1.02);
}
```

### 页面过渡

```css
/* 页面进入 */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 页面退出 */
@keyframes fadeOutDown {
  from {
    opacity: 1;
    transform: translateY(0);
  }
  to {
    opacity: 0;
    transform: translateY(20px);
  }
}
```

---

## 🎯 页面类型设计规范

### 营销页面设计

**特点**:
- 视觉冲击力强
- 大量使用渐变、光晕、动画
- 大标题、大按钮
- 丰富的视觉元素

**适用页面**:
- 首页（Landing）
- 卡片页面（Card）
- 收益页面（Earnings）
- 空投页面（Airdrops）

**设计元素**:
- Hero 区域：全屏背景 + 3D 效果
- 功能展示：大卡片 + 图标 + 动画
- CTA 区域：渐变按钮 + 光晕效果
- 数据展示：大数字 + 渐变文字

### 功能页面设计

**特点**:
- 干净整洁
- 专注服务
- 信息层次清晰
- 操作流畅

**适用页面**:
- 登录/注册
- 钱包列表
- 钱包详情
- 发送/接收
- 仪表盘
- 设置

**设计元素**:
- 简洁的导航
- 清晰的卡片布局
- 统一的按钮样式
- 友好的错误提示

---

## 📱 响应式设计

### 断点系统

```css
--breakpoint-sm: 640px;   /* 手机 */
--breakpoint-md: 768px;   /* 平板 */
--breakpoint-lg: 1024px;  /* 小桌面 */
--breakpoint-xl: 1280px;  /* 大桌面 */
--breakpoint-2xl: 1536px; /* 超大桌面 */
```

### 适配原则

- **移动端**: 单列布局，大触摸区域（44px+）
- **平板**: 保持桌面布局，优化间距
- **桌面**: 充分利用空间，多列布局

---

## 🎨 视觉特效

### 背景粒子系统（营销页面）

```css
/* 粒子背景 */
.particle-bg {
  position: absolute;
  width: 100%;
  height: 100%;
  background: radial-gradient(
    circle at 20% 50%,
    rgba(99, 102, 241, 0.1) 0%,
    transparent 50%
  ),
  radial-gradient(
    circle at 80% 80%,
    rgba(139, 92, 246, 0.1) 0%,
    transparent 50%
  );
}
```

### 3D 卡片效果

```css
.card-3d {
  transform-style: preserve-3d;
  transition: transform 0.3s;
}

.card-3d:hover {
  transform: rotateY(5deg) rotateX(5deg);
}
```

### 光晕动画

```css
@keyframes glow-pulse {
  0%, 100% {
    box-shadow: 0 0 20px rgba(99, 102, 241, 0.4);
  }
  50% {
    box-shadow: 0 0 40px rgba(99, 102, 241, 0.6);
  }
}

.glow-animated {
  animation: glow-pulse 2s ease-in-out infinite;
}
```

---

## ✅ 设计检查清单

### 营销页面
- [ ] 使用大标题和渐变文字
- [ ] 添加光晕和动画效果
- [ ] 使用强化毛玻璃
- [ ] 大尺寸 CTA 按钮
- [ ] 丰富的视觉层次

### 功能页面
- [ ] 干净整洁的布局
- [ ] 统一的设计语言
- [ ] 清晰的信息层次
- [ ] 友好的交互反馈
- [ ] 专业的视觉呈现

---

**最后更新**: 2025-11-27  
**设计系统版本**: v3.0

