# IronForge Logo 设计规范

> **版本**: 1.0  
> **设计理念**: 旋转的白色星球 + 真实质感 + 科技感  
> **更新日期**: 2025-11-27

---

## 🎨 设计概念

### 核心视觉元素

**旋转的白色星球** - 象征：
- **区块链网络** - 星球表面的坑坑洼洼代表节点和连接
- **未来科技** - 白色代表纯净、专业、科技感
- **智能支付** - 旋转动画代表流动性和活力
- **层次感** - 3D 质感营造深度和立体感

### 设计原则

1. **简洁而有力** - 符合苹果风格
2. **科技感** - 未来感、数字化
3. **质感** - 真实星球的坑坑洼洼纹理
4. **动态** - 缓慢旋转，优雅流畅
5. **可扩展** - 适配各种尺寸和场景

---

## 🌍 视觉设计

### 基础形态

```
         ╭─────────╮
        ╱           ╲
       │             │
      │   ●  ●  ●    │  ← 表面节点（坑洼）
     │  ●       ●   │
    │   ●  ●  ●    │
     │             │
      ╲           ╱
       ╰─────────╯
```

### 颜色系统

**主色**:
- **星球本体**: `#FFFFFF` (纯白) 或 `#F8F9FA` (柔和白)
- **阴影区域**: `rgba(0, 0, 0, 0.3)` (深色阴影)
- **高光区域**: `rgba(255, 255, 255, 0.8)` (亮部高光)

**科技光晕**:
- **外圈光晕**: `rgba(99, 102, 241, 0.2)` (科技蓝)
- **内圈光晕**: `rgba(139, 92, 246, 0.15)` (科技紫)
- **智能支付光晕**: `rgba(16, 185, 129, 0.1)` (支付绿，可选)

### 尺寸规范

| 使用场景 | 尺寸 | 说明 |
|---------|------|------|
| Favicon | 32×32px | 最小尺寸 |
| 移动端 Logo | 48×48px | 导航栏 |
| 桌面端 Logo | 64×64px | 导航栏 |
| 营销页面 | 128×128px | Hero 区域 |
| 大屏展示 | 256×256px | 落地页 |

---

## 🎬 动画设计

### 旋转动画

**基础旋转**:
- **速度**: 20秒完成一圈（缓慢优雅）
- **方向**: 顺时针
- **缓动**: `linear` (匀速)
- **循环**: 无限循环

**高级旋转** (可选):
- **3D 旋转**: 绕 Y 轴旋转，营造立体感
- **轻微摆动**: 添加轻微的上下浮动（呼吸效果）

### 光晕动画

**脉冲光晕**:
- **周期**: 3秒
- **效果**: 光晕大小和透明度轻微变化
- **缓动**: `ease-in-out`

### 表面纹理动画

**节点闪烁** (可选):
- **随机闪烁**: 表面节点随机闪烁
- **频率**: 每 2-3 秒一个节点闪烁
- **效果**: 代表网络活动

---

## 🛠️ 实现方案

### 方案 1: CSS + SVG (推荐)

**优点**:
- 性能好，矢量缩放
- 易于实现动画
- 文件小

**实现**:
```html
<svg class="logo-planet" viewBox="0 0 200 200">
  <!-- 外圈光晕 -->
  <circle class="glow-outer" cx="100" cy="100" r="90" />
  <!-- 内圈光晕 -->
  <circle class="glow-inner" cx="100" cy="100" r="80" />
  <!-- 星球主体 -->
  <circle class="planet-body" cx="100" cy="100" r="70" />
  <!-- 表面节点（坑洼） -->
  <circle class="crater" cx="80" cy="80" r="4" />
  <circle class="crater" cx="120" cy="90" r="3" />
  <circle class="crater" cx="90" cy="120" r="5" />
  <!-- 更多节点... -->
</svg>
```

```css
.logo-planet {
  animation: rotate 20s linear infinite;
}

@keyframes rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.planet-body {
  fill: #FFFFFF;
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.3));
}

.crater {
  fill: rgba(0, 0, 0, 0.2);
  animation: pulse 3s ease-in-out infinite;
}
```

### 方案 2: Canvas (3D 效果)

**优点**:
- 可以实现真正的 3D 效果
- 更真实的星球质感
- 可以添加光照效果

**实现思路**:
- 使用 Canvas 2D API 绘制球体
- 使用渐变和阴影营造 3D 效果
- 添加表面纹理（噪声或点阵）
- 使用 `requestAnimationFrame` 实现旋转

### 方案 3: WebGL (最真实)

**优点**:
- 最真实的 3D 效果
- 可以实现复杂的光照和材质
- 性能优秀

**缺点**:
- 实现复杂
- 文件较大

---

## 📐 详细设计规范

### 星球主体

**形状**: 正圆形

**尺寸比例**:
- 主体半径: 70% (相对于容器)
- 光晕外圈: 90%
- 光晕内圈: 80%

**表面纹理**:
- **坑洼数量**: 8-12 个
- **坑洼大小**: 随机，2-6px
- **坑洼分布**: 不均匀分布，模拟真实星球
- **坑洼深度**: 使用阴影营造深度感

### 光晕系统

**外圈光晕**:
- **颜色**: 科技蓝紫渐变
- **透明度**: 0.2
- **模糊**: 20px
- **动画**: 轻微脉冲

**内圈光晕**:
- **颜色**: 科技紫
- **透明度**: 0.15
- **模糊**: 10px

### 阴影系统

**星球阴影**:
- **位置**: 右下角
- **偏移**: (4px, 4px)
- **模糊**: 8px
- **颜色**: rgba(0, 0, 0, 0.3)

**内阴影** (营造深度):
- **位置**: 左上角
- **颜色**: rgba(255, 255, 255, 0.3)
- **模糊**: 4px

---

## 🎨 变体设计

### 标准版 (白色星球)

- **主色**: 纯白 (#FFFFFF)
- **背景**: 透明或深色
- **用途**: 通用场景

### 渐变版 (科技感)

- **主色**: 白色到浅蓝渐变
- **效果**: 顶部亮，底部暗
- **用途**: 营销页面

### 发光版 (强调)

- **主色**: 纯白
- **光晕**: 强化科技蓝紫光晕
- **用途**: CTA 按钮、重要位置

### 简化版 (小尺寸)

- **细节**: 减少坑洼数量
- **光晕**: 简化或移除
- **用途**: Favicon、小图标

---

## 💻 代码实现

### Rust/Dioxus 实现

```rust
use dioxus::prelude::*;

#[component]
pub fn LogoPlanet(
    size: Option<u32>,
    variant: Option<LogoVariant>,
) -> Element {
    let size = size.unwrap_or(64);
    let variant = variant.unwrap_or(LogoVariant::Standard);
    
    rsx! {
        div {
            class: "logo-planet-container",
            style: "width: {size}px; height: {size}px;",
            svg {
                class: "logo-planet",
                view_box: "0 0 200 200",
                // 外圈光晕
                circle {
                    class: "glow-outer",
                    cx: "100",
                    cy: "100",
                    r: "90",
                    fill: "rgba(99, 102, 241, 0.2)",
                    filter: "blur(20px)",
                }
                // 内圈光晕
                circle {
                    class: "glow-inner",
                    cx: "100",
                    cy: "100",
                    r: "80",
                    fill: "rgba(139, 92, 246, 0.15)",
                    filter: "blur(10px)",
                }
                // 星球主体
                circle {
                    class: "planet-body",
                    cx: "100",
                    cy: "100",
                    r: "70",
                    fill: "#FFFFFF",
                    filter: "drop-shadow(0 4px 8px rgba(0, 0, 0, 0.3))",
                }
                // 表面坑洼
                {render_craters()}
            }
        }
    }
}

fn render_craters() -> Vec<Element> {
    // 定义坑洼位置和大小
    let craters = vec![
        (80.0, 80.0, 4.0),
        (120.0, 90.0, 3.0),
        (90.0, 120.0, 5.0),
        (110.0, 70.0, 3.5),
        (70.0, 110.0, 4.5),
        (130.0, 110.0, 3.0),
        (85.0, 95.0, 2.5),
        (115.0, 105.0, 4.0),
    ];
    
    craters.iter().map(|(x, y, r)| {
        rsx! {
            circle {
                class: "crater",
                cx: "{x}",
                cy: "{y}",
                r: "{r}",
                fill: "rgba(0, 0, 0, 0.2)",
            }
        }
    }).collect()
}

#[derive(Clone, Copy, PartialEq)]
pub enum LogoVariant {
    Standard,    // 标准版
    Gradient,    // 渐变版
    Glowing,     // 发光版
    Minimal,     // 简化版
}
```

### CSS 样式

```css
/* Logo 容器 */
.logo-planet-container {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

/* 旋转动画 */
.logo-planet {
  width: 100%;
  height: 100%;
  animation: rotate-planet 20s linear infinite;
  transform-origin: center center;
}

@keyframes rotate-planet {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* 星球主体 */
.planet-body {
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.3));
}

/* 坑洼动画 */
.crater {
  animation: crater-pulse 3s ease-in-out infinite;
  animation-delay: var(--delay, 0s);
}

@keyframes crater-pulse {
  0%, 100% {
    opacity: 0.2;
    transform: scale(1);
  }
  50% {
    opacity: 0.4;
    transform: scale(1.1);
  }
}

/* 光晕脉冲 */
.glow-outer {
  animation: glow-pulse 3s ease-in-out infinite;
}

@keyframes glow-pulse {
  0%, 100% {
    opacity: 0.2;
    transform: scale(1);
  }
  50% {
    opacity: 0.3;
    transform: scale(1.05);
  }
}

/* 3D 旋转效果（可选） */
.logo-planet-3d {
  transform-style: preserve-3d;
  animation: rotate-3d 20s linear infinite;
}

@keyframes rotate-3d {
  from {
    transform: rotateY(0deg);
  }
  to {
    transform: rotateY(360deg);
  }
}
```

---

## 🎯 使用场景

### 1. 导航栏 Logo

```rust
<LogoPlanet size={32} variant={LogoVariant::Minimal} />
<span class="logo-text">IronForge</span>
```

### 2. 营销页面 Hero

```rust
<LogoPlanet size={128} variant={LogoVariant::Glowing} />
```

### 3. Favicon

```rust
<LogoPlanet size={32} variant={LogoVariant::Minimal} />
```

### 4. 加载动画

```rust
<LogoPlanet size={64} variant={LogoVariant::Standard} />
// 添加加载文字
```

---

## 🎨 设计细节

### 坑洼分布算法

为了营造真实感，坑洼应该：
1. **不均匀分布** - 避免对称
2. **大小随机** - 2-6px 随机
3. **深度变化** - 使用不同透明度
4. **位置优化** - 避免过于集中

### 光照效果

**高光区域** (左上):
- 颜色: rgba(255, 255, 255, 0.8)
- 位置: 30% 左上角
- 形状: 椭圆形

**阴影区域** (右下):
- 颜色: rgba(0, 0, 0, 0.3)
- 位置: 30% 右下角
- 形状: 椭圆形

### 科技感增强

**可选元素**:
- **连接线**: 表面节点之间的细线（代表网络）
- **数据流**: 节点间的流动效果
- **粒子**: 围绕星球的粒子效果

---

## 📱 响应式适配

### 小尺寸 (< 48px)

- 简化坑洼数量（4-6 个）
- 减少光晕效果
- 保持核心旋转动画

### 中等尺寸 (48-128px)

- 标准坑洼数量（8-10 个）
- 完整光晕效果
- 所有动画效果

### 大尺寸 (> 128px)

- 增加坑洼细节（12+ 个）
- 强化光晕效果
- 添加更多动画细节

---

## ✅ 设计检查清单

- [ ] 旋转动画流畅（20秒/圈）
- [ ] 坑洼分布自然
- [ ] 光晕效果适中
- [ ] 阴影营造深度感
- [ ] 适配各种尺寸
- [ ] 性能优化（60fps）
- [ ] 符合品牌调性

---

## 🚀 实现优先级

### Phase 1: 基础版本
1. SVG 基础形状
2. 旋转动画
3. 基础坑洼纹理

### Phase 2: 增强版本
1. 光晕效果
2. 阴影系统
3. 坑洼动画

### Phase 3: 高级版本
1. 3D 效果
2. 粒子系统
3. 交互效果

---

**最后更新**: 2025-11-27  
**设计版本**: v1.0

