# IronForge V2 - 响应式布局设计

> 📅 创建日期: 2025-11-25  
> 📱 版本: 2.0  
> 🎯 目标: 移动优先，全端适配

---

## 📋 目录

- [设计原则](#设计原则)
- [断点系统](#断点系统)
- [布局模式](#布局模式)
- [Dioxus 实现](#dioxus-实现)
- [最佳实践](#最佳实践)

---

## 🎯 设计原则

### 移动优先 (Mobile First)

```rust
// ✅ 正确：先写移动端样式，再用媒体查询扩展
rsx! {
    div {
        class: "card",
        style: "
            /* 移动端默认样式 */
            padding: 1rem;
            font-size: 0.875rem;
            
            /* 平板及以上 */
            @media (min-width: 768px) {{
                padding: 1.5rem;
                font-size: 1rem;
            }}
            
            /* 桌面及以上 */
            @media (min-width: 1024px) {{
                padding: 2rem;
                font-size: 1.125rem;
            }}
        ",
        "Content"
    }
}
```

### 流式布局 (Fluid Layout)

- ✅ 使用相对单位 (`rem`, `em`, `%`, `vw`, `vh`)
- ✅ 使用 Flexbox / Grid
- ❌ 避免固定像素宽度

### 触摸友好 (Touch Friendly)

- ✅ 按钮最小点击区域：44x44px (Apple) / 48x48px (Material)
- ✅ 足够的间距，避免误触
- ✅ 支持手势操作

---

## 📐 断点系统

### 标准断点

```rust
// src/presentation/styles/breakpoints.rs
pub const BREAKPOINT_MOBILE: u32 = 480;     // 移动端
pub const BREAKPOINT_TABLET: u32 = 768;     // 平板
pub const BREAKPOINT_DESKTOP: u32 = 1024;   // 桌面
pub const BREAKPOINT_WIDE: u32 = 1440;      // 宽屏

pub enum DeviceType {
    Mobile,     // < 768px
    Tablet,     // 768px - 1024px
    Desktop,    // 1024px - 1440px
    Wide,       // > 1440px
}

impl DeviceType {
    pub fn from_width(width: f64) -> Self {
        match width as u32 {
            w if w < BREAKPOINT_TABLET => DeviceType::Mobile,
            w if w < BREAKPOINT_DESKTOP => DeviceType::Tablet,
            w if w < BREAKPOINT_WIDE => DeviceType::Desktop,
            _ => DeviceType::Wide,
        }
    }
}
```

### 检测设备类型

```rust
use dioxus::prelude::*;
use gloo_utils::window;

pub fn use_device_type(cx: &ScopeState) -> &Signal<DeviceType> {
    let device_type = use_signal(cx, || {
        let width = window().inner_width()
            .ok()
            .and_then(|w| w.as_f64())
            .unwrap_or(1024.0);
        
        DeviceType::from_width(width)
    });
    
    // 监听窗口大小变化
    use_effect(cx, (), |_| async move {
        let window = window();
        let closure = Closure::wrap(Box::new(move |_: Event| {
            let width = window.inner_width()
                .ok()
                .and_then(|w| w.as_f64())
                .unwrap_or(1024.0);
            
            device_type.set(DeviceType::from_width(width));
        }) as Box<dyn FnMut(_)>);
        
        window.add_event_listener_with_callback(
            "resize",
            closure.as_ref().unchecked_ref()
        ).ok();
        
        closure.forget();
    });
    
    device_type
}
```

---

## 🎨 布局模式

### 1. Container 容器

```rust
// src/presentation/components/layout/container.rs
use dioxus::prelude::*;

#[derive(Props)]
pub struct ContainerProps<'a> {
    children: Element<'a>,
    #[props(default = false)]
    fluid: bool,
}

pub fn Container<'a>(cx: Scope<'a, ContainerProps<'a>>) -> Element {
    let class = if cx.props.fluid {
        "container-fluid"
    } else {
        "container"
    };
    
    rsx! {
        div {
            class: "{class}",
            style: "
                width: 100%;
                margin: 0 auto;
                padding: 0 1rem;
                
                @media (min-width: 640px) {{
                    max-width: 640px;
                }}
                
                @media (min-width: 768px) {{
                    max-width: 768px;
                    padding: 0 1.5rem;
                }}
                
                @media (min-width: 1024px) {{
                    max-width: 1024px;
                    padding: 0 2rem;
                }}
                
                @media (min-width: 1280px) {{
                    max-width: 1280px;
                }}
            ",
            &cx.props.children
        }
    }
}
```

### 2. Grid 网格布局

```rust
// src/presentation/components/layout/grid.rs
#[derive(Props)]
pub struct GridProps<'a> {
    children: Element<'a>,
    #[props(default = 1)]
    cols: u8,
    #[props(default = "1rem")]
    gap: &'a str,
}

pub fn Grid<'a>(cx: Scope<'a, GridProps<'a>>) -> Element {
    let cols = cx.props.cols;
    let gap = cx.props.gap;
    
    rsx! {
        div {
            style: "
                display: grid;
                gap: {gap};
                
                /* 移动端：1 列 */
                grid-template-columns: 1fr;
                
                /* 平板：2 列 */
                @media (min-width: 768px) {{
                    grid-template-columns: repeat(2, 1fr);
                }}
                
                /* 桌面：指定列数 */
                @media (min-width: 1024px) {{
                    grid-template-columns: repeat({cols}, 1fr);
                }}
            ",
            &cx.props.children
        }
    }
}

// 使用示例
fn WalletList(cx: Scope) -> Element {
    rsx! {
        Container {
            Grid {
                cols: 3,
                gap: "1.5rem",
                
                // 钱包卡片
                WalletCard { name: "Wallet 1" }
                WalletCard { name: "Wallet 2" }
                WalletCard { name: "Wallet 3" }
            }
        }
    }
}
```

### 3. Flex 弹性布局

```rust
// src/presentation/components/layout/flex.rs
#[derive(Props)]
pub struct FlexProps<'a> {
    children: Element<'a>,
    #[props(default = "row")]
    direction: &'a str,
    #[props(default = "flex-start")]
    justify: &'a str,
    #[props(default = "stretch")]
    align: &'a str,
    #[props(default = "0")]
    gap: &'a str,
}

pub fn Flex<'a>(cx: Scope<'a, FlexProps<'a>>) -> Element {
    rsx! {
        div {
            style: "
                display: flex;
                flex-direction: {cx.props.direction};
                justify-content: {cx.props.justify};
                align-items: {cx.props.align};
                gap: {cx.props.gap};
                flex-wrap: wrap;
            ",
            &cx.props.children
        }
    }
}

// 使用示例：响应式导航栏
fn Navbar(cx: Scope) -> Element {
    rsx! {
        Flex {
            direction: "row",
            justify: "space-between",
            align: "center",
            gap: "1rem",
            
            style: "
                /* 移动端：垂直布局 */
                @media (max-width: 768px) {{
                    flex-direction: column;
                }}
            ",
            
            div { class: "logo", "IronForge" }
            div { class: "nav-links", "Links" }
            div { class: "actions", "Actions" }
        }
    }
}
```

### 4. 条件渲染

```rust
// 根据设备类型渲染不同组件
fn ResponsiveHeader(cx: Scope) -> Element {
    let device = use_device_type(cx);
    
    match device.read() {
        DeviceType::Mobile => rsx! { MobileHeader {} },
        DeviceType::Tablet => rsx! { TabletHeader {} },
        _ => rsx! { DesktopHeader {} },
    }
}

// 或者使用条件样式
fn Header(cx: Scope) -> Element {
    let device = use_device_type(cx);
    let is_mobile = matches!(device.read(), DeviceType::Mobile);
    
    rsx! {
        header {
            style: "
                padding: {if is_mobile { '0.5rem' } else { '1rem' }};
                font-size: {if is_mobile { '0.875rem' } else { '1rem' }};
            ",
            "Header Content"
        }
    }
}
```

---

## 💻 Dioxus 实现示例

### 完整的响应式页面

```rust
// src/presentation/pages/wallet_list.rs
use dioxus::prelude::*;

pub fn WalletListPage(cx: Scope) -> Element {
    let device = use_device_type(cx);
    let wallets = use_signal(cx, Vec::new);
    
    // 响应式列数
    let grid_cols = match device.read() {
        DeviceType::Mobile => 1,
        DeviceType::Tablet => 2,
        _ => 3,
    };
    
    rsx! {
        div {
            class: "wallet-list-page",
            
            // 页面标题
            div {
                class: "page-header",
                style: "
                    padding: 1rem;
                    
                    @media (min-width: 768px) {{
                        padding: 1.5rem;
                    }}
                    
                    @media (min-width: 1024px) {{
                        padding: 2rem;
                    }}
                ",
                
                h1 {
                    style: "
                        font-size: 1.5rem;
                        
                        @media (min-width: 768px) {{
                            font-size: 1.875rem;
                        }}
                        
                        @media (min-width: 1024px) {{
                            font-size: 2.25rem;
                        }}
                    ",
                    "My Wallets"
                }
            }
            
            // 钱包网格
            Container {
                Grid {
                    cols: grid_cols,
                    gap: "1rem",
                    
                    for wallet in wallets.read().iter() {
                        WalletCard {
                            key: "{wallet.id}",
                            wallet: wallet.clone(),
                        }
                    }
                }
            }
            
            // 浮动操作按钮（移动端）
            if matches!(device.read(), DeviceType::Mobile) {
                rsx! {
                    button {
                        class: "fab",
                        style: "
                            position: fixed;
                            bottom: 1rem;
                            right: 1rem;
                            width: 56px;
                            height: 56px;
                            border-radius: 50%;
                            background: #8B5CF6;
                            border: none;
                            box-shadow: 0 4px 12px rgba(139, 92, 246, 0.4);
                            cursor: pointer;
                            z-index: 100;
                        ",
                        "+"
                    }
                }
            }
        }
    }
}
```

### 响应式卡片组件

```rust
// src/presentation/components/wallet_card.rs
#[component]
pub fn WalletCard(cx: Scope, wallet: Wallet) -> Element {
    let device = use_device_type(cx);
    let is_mobile = matches!(device.read(), DeviceType::Mobile);
    
    rsx! {
        div {
            class: "wallet-card",
            style: "
                background: rgba(17, 17, 27, 0.95);
                border-radius: 12px;
                padding: 1rem;
                border: 1px solid rgba(139, 92, 246, 0.2);
                transition: all 0.3s ease;
                
                @media (min-width: 768px) {{
                    padding: 1.5rem;
                    border-radius: 16px;
                }}
                
                @media (min-width: 1024px) {{
                    padding: 2rem;
                }}
                
                &:hover {{
                    border-color: rgba(139, 92, 246, 0.5);
                    transform: translateY(-4px);
                    box-shadow: 0 8px 24px rgba(139, 92, 246, 0.2);
                }}
            ",
            
            // 钱包名称
            h3 {
                style: "
                    font-size: 1rem;
                    margin-bottom: 0.5rem;
                    
                    @media (min-width: 768px) {{
                        font-size: 1.125rem;
                    }}
                ",
                "{wallet.name}"
            }
            
            // 地址（移动端截断）
            p {
                style: "
                    font-size: 0.75rem;
                    color: #9CA3AF;
                    font-family: monospace;
                    
                    @media (min-width: 768px) {{
                        font-size: 0.875rem;
                    }}
                ",
                if is_mobile {
                    format!("{}...{}", 
                        &wallet.address[..6], 
                        &wallet.address[wallet.address.len()-4..]
                    )
                } else {
                    wallet.address.clone()
                }
            }
            
            // 余额
            div {
                style: "
                    margin-top: 1rem;
                    font-size: 1.25rem;
                    font-weight: 600;
                    color: #8B5CF6;
                    
                    @media (min-width: 768px) {{
                        font-size: 1.5rem;
                    }}
                ",
                "{wallet.balance} ETH"
            }
        }
    }
}
```

---

## 📱 触摸优化

### 点击区域优化

```rust
// 确保按钮有足够的点击区域
rsx! {
    button {
        style: "
            /* 视觉大小 */
            padding: 0.5rem 1rem;
            
            /* 点击区域（通过伪元素扩大） */
            position: relative;
            
            &::before {{
                content: '';
                position: absolute;
                inset: -0.5rem;  /* 扩大 8px */
            }}
        ",
        "Button"
    }
}
```

### 手势支持

```rust
use web_sys::TouchEvent;

fn SwipeableCard(cx: Scope) -> Element {
    let start_x = use_signal(cx, || 0.0);
    let current_x = use_signal(cx, || 0.0);
    
    let on_touch_start = move |evt: Event<TouchData>| {
        if let Some(touch) = evt.touches().get(0) {
            start_x.set(touch.client_x());
        }
    };
    
    let on_touch_move = move |evt: Event<TouchData>| {
        if let Some(touch) = evt.touches().get(0) {
            current_x.set(touch.client_x());
        }
    };
    
    let on_touch_end = move |_| {
        let delta = current_x.read() - start_x.read();
        if delta.abs() > 50.0 {
            // 左滑或右滑
            if delta > 0 {
                println!("Swipe right");
            } else {
                println!("Swipe left");
            }
        }
        current_x.set(0.0);
    };
    
    rsx! {
        div {
            ontouchstart: on_touch_start,
            ontouchmove: on_touch_move,
            ontouchend: on_touch_end,
            "Swipeable Content"
        }
    }
}
```

---

## ✅ 最佳实践

### 1. 图片响应式

```rust
rsx! {
    img {
        src: "/logo.png",
        style: "
            max-width: 100%;
            height: auto;
            display: block;
        ",
        // 使用 srcset 提供多尺寸
        srcset: "/logo-320w.png 320w,
                 /logo-640w.png 640w,
                 /logo-1280w.png 1280w",
        sizes: "(max-width: 768px) 100vw,
                (max-width: 1024px) 50vw,
                33vw",
    }
}
```

### 2. 字体响应式

```rust
// 使用 clamp() 实现流式字体
rsx! {
    h1 {
        style: "
            /* 最小 1.5rem, 理想 4vw, 最大 3rem */
            font-size: clamp(1.5rem, 4vw, 3rem);
        ",
        "Responsive Title"
    }
}
```

### 3. 避免水平滚动

```rust
// 全局样式
style: "
    html, body {{
        overflow-x: hidden;
        width: 100%;
    }}
    
    * {{
        max-width: 100%;
    }}
"
```

### 4. 性能优化

```rust
// 使用 will-change 提示浏览器优化
rsx! {
    div {
        style: "
            transition: transform 0.3s;
            will-change: transform;
        ",
        "Animated content"
    }
}
```

---

## 📊 测试清单

### 设备测试
- [ ] iPhone SE (375x667)
- [ ] iPhone 12/13 (390x844)
- [ ] iPhone 14 Pro Max (430x932)
- [ ] iPad (768x1024)
- [ ] iPad Pro (1024x1366)
- [ ] Desktop 1080p (1920x1080)
- [ ] Desktop 4K (3840x2160)

### 功能测试
- [ ] 横屏/竖屏切换
- [ ] 缩放测试（50% - 200%）
- [ ] 触摸操作
- [ ] 键盘导航
- [ ] 屏幕阅读器

---

**下一步**: 阅读 [组件库设计](../05-ui-ux/03-component-library.md)

**最后更新**: 2025-11-25
