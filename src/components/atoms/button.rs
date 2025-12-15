//! Button Component - 按钮组件
//! 生产级按钮组件，基于设计系统 V3

use crate::shared::design_tokens::{Colors, Gradients, Radius, Shadows};
use dioxus::prelude::*;

/// 按钮变体
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ButtonVariant {
    /// 主要按钮 - 科技渐变
    Primary,
    /// 次要按钮 - 毛玻璃效果
    Secondary,
    /// 智能支付按钮 - 支付绿渐变
    #[allow(dead_code)]
    Payment,
    /// 成功按钮 - 成功绿渐变
    Success,
    /// 警告按钮 - 警告橙
    #[allow(dead_code)]
    Warning,
    /// 错误按钮 - 错误红
    #[allow(dead_code)]
    Error,
    /// 文本按钮 - 无背景
    #[allow(dead_code)]
    Text,
}

/// 按钮尺寸
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ButtonSize {
    /// 小尺寸
    Small,
    /// 中等尺寸（默认）
    Medium,
    /// 大尺寸
    Large,
    /// 超大尺寸（营销页面用）
    XLarge,
}

impl ButtonSize {
    fn padding(&self) -> &'static str {
        match self {
            ButtonSize::Small => "8px 16px",
            ButtonSize::Medium => "12px 24px",
            ButtonSize::Large => "14px 28px",
            ButtonSize::XLarge => "16px 32px",
        }
    }

    fn font_size(&self) -> &'static str {
        match self {
            ButtonSize::Small => "14px",
            ButtonSize::Medium => "16px",
            ButtonSize::Large => "18px",
            ButtonSize::XLarge => "20px",
        }
    }
}

/// Button 组件
///
/// # 示例
///
/// ```rust
/// rsx! {
///     Button {
///         variant: ButtonVariant::Primary,
///         size: ButtonSize::Medium,
///         onclick: move |_| { /* 处理点击 */ },
///         "点击我"
///     }
/// }
/// ```
#[component]
pub fn Button(
    /// 按钮变体
    #[props(default = ButtonVariant::Primary)]
    variant: ButtonVariant,
    /// 按钮尺寸
    #[props(default = ButtonSize::Medium)]
    size: ButtonSize,
    /// 是否禁用
    #[props(default = false)]
    disabled: bool,
    /// 是否加载中
    #[props(default = false)]
    loading: bool,
    /// 点击事件
    onclick: Option<EventHandler<MouseEvent>>,
    /// 自定义类名
    #[props(default)]
    class: Option<String>,
    /// 按钮内容
    children: Element,
) -> Element {
    let base_class = "inline-flex items-center justify-center font-semibold rounded-lg transition-all duration-300 cursor-pointer";
    let disabled_class = if disabled || loading {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    };
    let custom_class = class.unwrap_or_default();

    let (bg_style, text_color, shadow_style, hover_style): (String, &str, String, String) = match variant {
        ButtonVariant::Primary => (
            format!("background: {};", Gradients::PRIMARY),
            Colors::TEXT_PRIMARY,
            format!("box-shadow: {};", Shadows::TECH),
            "hover:scale-[1.02] hover:shadow-[0_8px_32px_rgba(99,102,241,0.4),0_0_50px_rgba(99,102,241,0.2)]".to_string(),
        ),
        ButtonVariant::Secondary => (
            format!("background: rgba(26, 26, 36, 0.6); backdrop-filter: blur(20px) saturate(180%); border: 1px solid {};", Colors::BORDER_PRIMARY),
            Colors::TEXT_PRIMARY,
            format!("box-shadow: {};", Shadows::APPLE),
            "hover:bg-[rgba(26,26,36,0.8)] hover:scale-[1.02]".to_string(),
        ),
        ButtonVariant::Payment => (
            format!("background: {};", Gradients::PAYMENT),
            Colors::TEXT_PRIMARY,
            format!("box-shadow: {};", Shadows::PAYMENT),
            "hover:scale-[1.02] hover:shadow-[0_8px_40px_rgba(16,185,129,0.5),0_0_50px_rgba(16,185,129,0.3)]".to_string(),
        ),
        ButtonVariant::Success => (
            format!("background: {};", Gradients::SUCCESS),
            Colors::TEXT_PRIMARY,
            format!("box-shadow: {};", Shadows::PAYMENT),
            "hover:scale-[1.02]".to_string(),
        ),
        ButtonVariant::Warning => (
            format!("background: {};", Colors::PAYMENT_WARNING),
            Colors::TEXT_PRIMARY,
            "box-shadow: 0 4px 16px rgba(245, 158, 11, 0.3);".to_string(),
            "hover:scale-[1.02] hover:shadow-[0_8px_32px_rgba(245,158,11,0.4)]".to_string(),
        ),
        ButtonVariant::Error => (
            format!("background: {};", Colors::PAYMENT_ERROR),
            Colors::TEXT_PRIMARY,
            "box-shadow: 0 4px 16px rgba(239, 68, 68, 0.3);".to_string(),
            "hover:scale-[1.02] hover:shadow-[0_8px_32px_rgba(239,68,68,0.4)]".to_string(),
        ),
        ButtonVariant::Text => (
            "background: transparent;".to_string(),
            Colors::TEXT_PRIMARY,
            "".to_string(),
            "hover:bg-white/5".to_string(),
        ),
    };

    let padding = size.padding();
    let font_size = size.font_size();
    let border_radius = Radius::MD;

    rsx! {
        button {
            class: "{base_class} {disabled_class} {custom_class} {hover_style}",
            style: "{bg_style} color: {text_color}; padding: {padding}; font-size: {font_size}; border-radius: {border_radius}; {shadow_style}",
            disabled: disabled || loading,
            onclick: move |e| {
                if !disabled && !loading {
                    if let Some(handler) = onclick.as_ref() {
                        handler.call(e);
                    }
                }
            },
            onmousedown: move |_| {
                if !disabled && !loading {
                    // 点击缩放效果
                }
            },
            if loading {
                span {
                    class: "inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2",
                }
            }
            {children}
        }
    }
}
