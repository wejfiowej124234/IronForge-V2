//! IronForge Logo Component
//! 旋转的白色星球 Logo
//!
//! 设计理念: 区块链网络 + 未来科技 + 智能支付

use dioxus::prelude::*;

/// Logo 变体
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LogoVariant {
    /// 标准版 - 纯白星球
    Standard,
    /// 渐变版 - 白色到浅蓝渐变
    Gradient,
    /// 发光版 - 强化光晕效果
    Glowing,
    /// 简化版 - 减少细节，用于小尺寸
    #[allow(dead_code)] // 为未来扩展准备
    Minimal,
}

/// Logo 尺寸预设
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LogoSize {
    /// 32px - Favicon, 小图标
    #[allow(dead_code)] // 为未来扩展准备
    Small,
    /// 48px - 移动端导航栏
    #[allow(dead_code)] // 为未来扩展准备
    Medium,
    /// 64px - 桌面端导航栏
    Large,
    /// 128px - 营销页面 Hero
    XLarge,
    /// 自定义尺寸
    #[allow(dead_code)] // 为未来扩展准备
    Custom(u32),
}

impl LogoSize {
    pub fn to_pixels(&self) -> u32 {
        match self {
            LogoSize::Small => 32,
            LogoSize::Medium => 48,
            LogoSize::Large => 64,
            LogoSize::XLarge => 128,
            LogoSize::Custom(size) => *size,
        }
    }
}

/// 坑洼定义（位置和大小）
struct Crater {
    x: f64,
    y: f64,
    radius: f64,
    opacity: f64,
}

impl Crater {
    fn new(x: f64, y: f64, radius: f64, opacity: f64) -> Self {
        Self {
            x,
            y,
            radius,
            opacity,
        }
    }
}

/// 渲染坑洼列表
fn get_craters(variant: LogoVariant) -> Vec<Crater> {
    match variant {
        LogoVariant::Minimal => {
            // 简化版：4-6 个坑洼
            vec![
                Crater::new(80.0, 80.0, 4.0, 0.2),
                Crater::new(120.0, 90.0, 3.0, 0.25),
                Crater::new(90.0, 120.0, 5.0, 0.2),
                Crater::new(110.0, 70.0, 3.5, 0.22),
            ]
        }
        _ => {
            // 标准版：8-12 个坑洼
            vec![
                Crater::new(80.0, 80.0, 4.0, 0.2),
                Crater::new(120.0, 90.0, 3.0, 0.25),
                Crater::new(90.0, 120.0, 5.0, 0.2),
                Crater::new(110.0, 70.0, 3.5, 0.22),
                Crater::new(70.0, 110.0, 4.5, 0.2),
                Crater::new(130.0, 110.0, 3.0, 0.25),
                Crater::new(85.0, 95.0, 2.5, 0.23),
                Crater::new(115.0, 105.0, 4.0, 0.2),
                Crater::new(95.0, 75.0, 3.0, 0.24),
                Crater::new(105.0, 125.0, 4.5, 0.21),
            ]
        }
    }
}

/// IronForge Logo 组件
///
/// # 示例
///
/// ```rust
/// // 标准 Logo
/// rsx! {
///     LogoPlanet {
///         size: LogoSize::Large,
///         variant: LogoVariant::Standard,
///     }
/// }
///
/// // 营销页面发光版
/// rsx! {
///     LogoPlanet {
///         size: LogoSize::XLarge,
///         variant: LogoVariant::Glowing,
///     }
/// }
/// ```
#[component]
pub fn LogoPlanet(
    /// Logo 尺寸
    #[props(default = LogoSize::Large)]
    size: LogoSize,
    /// Logo 变体
    #[props(default = LogoVariant::Standard)]
    variant: LogoVariant,
    /// 自定义类名
    #[props(default)]
    class: Option<String>,
) -> Element {
    let size_px = size.to_pixels();
    let craters = get_craters(variant);
    let class_str = class.as_deref().unwrap_or("");
    let container_class = format!("logo-planet-container {}", class_str);
    let container_style = format!("width: {}px; height: {}px;", size_px, size_px);
    let container_class_clone = container_class.clone();

    rsx! {
        div {
            class: "{container_class_clone}",
            style: "{container_style}",
            svg {
                class: "logo-planet logo-planet-horizontal",
                view_box: "0 0 200 200",
                xmlns: "http://www.w3.org/2000/svg",

                // 外圈光晕（发光版和标准版）
                if variant == LogoVariant::Glowing || variant == LogoVariant::Standard {
                    circle {
                        class: "glow-outer",
                        cx: "100",
                        cy: "100",
                        r: "90",
                        fill: "rgba(99, 102, 241, 0.2)",
                        style: "filter: blur(20px);",
                    }
                }

                // 内圈光晕
                if variant == LogoVariant::Glowing || variant == LogoVariant::Standard {
                    circle {
                        class: "glow-inner",
                        cx: "100",
                        cy: "100",
                        r: "80",
                        fill: "rgba(139, 92, 246, 0.15)",
                        style: "filter: blur(10px);",
                    }
                }

                // 星球主体
                if variant == LogoVariant::Gradient {
                    // 渐变版：白色到浅蓝渐变
                    defs {
                        linearGradient {
                            id: "planet-gradient",
                            x1: "0%",
                            y1: "0%",
                            x2: "100%",
                            y2: "100%",
                            stop {
                                offset: "0%",
                                stop_color: "#FFFFFF",
                            }
                            stop {
                                offset: "100%",
                                stop_color: "#E0E7FF",
                            }
                        }
                    }
                    circle {
                        class: "planet-body",
                        cx: "100",
                        cy: "100",
                        r: "70",
                        fill: "url(#planet-gradient)",
                        style: "filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.3));",
                    }
                } else {
                    // 标准版和发光版：纯白
                    circle {
                        class: "planet-body",
                        cx: "100",
                        cy: "100",
                        r: "70",
                        fill: "#FFFFFF",
                        style: "filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.3));",
                    }
                }

                // 表面坑洼
                for (index, crater) in craters.iter().enumerate() {
                    circle {
                        class: "crater",
                        cx: "{crater.x}",
                        cy: "{crater.y}",
                        r: "{crater.radius}",
                        fill: "rgba(0, 0, 0, {crater.opacity})",
                        style: format!("animation-delay: {}s;", index as f64 * 0.3),
                    }
                }
            }
        }
    }
}

/// 带文字的 Logo（Logo + 品牌名）
#[component]
pub fn LogoWithText(
    /// Logo 尺寸
    #[props(default = LogoSize::Large)]
    size: LogoSize,
    /// Logo 变体
    #[props(default = LogoVariant::Standard)]
    variant: LogoVariant,
    /// 文字大小
    #[props(default = "text-xl")]
    text_size: Option<String>,
    /// 文字颜色
    #[props(default = "text-white")]
    text_color: Option<String>,
) -> Element {
    let text_size = text_size.unwrap_or_else(|| "text-xl".to_string());
    let text_color = text_color.unwrap_or_else(|| "text-white".to_string());

    rsx! {
        div {
            class: "flex items-center gap-3",
            LogoPlanet {
                size: size,
                variant: variant,
            }
            span {
                class: "{text_size} {text_color} font-semibold tracking-tight",
                "IronForge"
            }
        }
    }
}
