//! Card Component - 卡片组件
//! 生产级卡片组件，支持基础版和强化版毛玻璃效果

use crate::shared::design_tokens::{Colors, Radius, Shadows, Spacing};
use dioxus::prelude::*;

/// 卡片变体
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CardVariant {
    /// 基础毛玻璃 - 功能页面用
    Base,
    /// 强化毛玻璃 - 营销页面用
    Strong,
    /// 提升层毛玻璃 - 模态框用
    #[allow(dead_code)]
    Elevated,
    /// 纯色背景 - 无毛玻璃
    #[allow(dead_code)]
    Solid,
}

/// Card 组件
///
/// # 示例
///
/// ```rust
/// rsx! {
///     Card {
///         variant: CardVariant::Base,
///         padding: Some("24px"),
///         children: rsx! {
///             h3 { "标题" }
///             p { "内容" }
///         }
///     }
/// }
/// ```
#[component]
pub fn Card(
    /// 卡片变体
    #[props(default = CardVariant::Base)]
    variant: CardVariant,
    /// 内边距
    #[props(default)]
    padding: Option<String>,
    /// 是否可点击
    #[props(default = false)]
    clickable: bool,
    /// 点击事件
    onclick: Option<EventHandler<MouseEvent>>,
    /// 自定义类名
    #[props(default)]
    class: Option<String>,
    /// 卡片内容
    children: Element,
) -> Element {
    let padding_value = padding.unwrap_or_else(|| Spacing::LG.to_string());
    let border_radius = Radius::LG;

    let (bg_style, border_style, shadow_style, hover_style): (String, String, String, String) = match variant {
        CardVariant::Base => (
            "background: rgba(18, 18, 26, 0.6); backdrop-filter: blur(20px) saturate(180%); -webkit-backdrop-filter: blur(20px) saturate(180%);".to_string(),
            format!("border: 1px solid {};", Colors::BORDER_PRIMARY),
            format!("box-shadow: {};", Shadows::APPLE),
            if clickable { "hover:bg-[rgba(18,18,26,0.8)] hover:scale-[1.02] hover:shadow-[0_12px_48px_rgba(0,0,0,0.5)] cursor-pointer transition-all duration-300".to_string() } else { "".to_string() },
        ),
        CardVariant::Strong => (
            "background: rgba(26, 26, 36, 0.7); backdrop-filter: blur(30px) saturate(200%); -webkit-backdrop-filter: blur(30px) saturate(200%);".to_string(),
            "border: 1px solid rgba(255, 255, 255, 0.15);".to_string(),
            format!("box-shadow: 0 12px 48px rgba(0, 0, 0, 0.5), {}, 0 0 40px rgba(99, 102, 241, 0.1);", Shadows::INNER_GLOW),
            if clickable { "hover:bg-[rgba(26,26,36,0.9)] hover:scale-[1.03] hover:shadow-[0_16px_64px_rgba(0,0,0,0.6),0_0_60px_rgba(99,102,241,0.15)] cursor-pointer transition-all duration-300".to_string() } else { "".to_string() },
        ),
        CardVariant::Elevated => (
            "background: rgba(36, 36, 47, 0.8); backdrop-filter: blur(40px) saturate(220%); -webkit-backdrop-filter: blur(40px) saturate(220%);".to_string(),
            "border: 1px solid rgba(255, 255, 255, 0.2);".to_string(),
            format!("box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6), {};", Shadows::INNER_GLOW),
            "".to_string(),
        ),
        CardVariant::Solid => (
            format!("background: {};", Colors::BG_SECONDARY),
            format!("border: 1px solid {};", Colors::BORDER_PRIMARY),
            format!("box-shadow: {};", Shadows::APPLE),
            if clickable { "hover:bg-[#1A1A24] hover:scale-[1.02] cursor-pointer transition-all duration-300".to_string() } else { "".to_string() },
        ),
    };

    let base_class = class.unwrap_or_default();

    rsx! {
        div {
            class: "{base_class} {hover_style}",
            style: "{bg_style} {border_style} border-radius: {border_radius}; padding: {padding_value}; {shadow_style}",
            onclick: move |e| {
                if clickable {
                    if let Some(handler) = onclick.as_ref() {
                        handler.call(e);
                    }
                }
            },
            {children}
        }
    }
}
