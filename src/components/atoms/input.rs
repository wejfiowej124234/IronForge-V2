//! Input Component - 输入框组件
//! 生产级输入框组件，基于设计系统 V3

use crate::shared::design_tokens::{Colors, Radius, Spacing};
use dioxus::prelude::*;

/// Input 类型
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputType {
    Text,
    Password,
    #[allow(dead_code)]
    Email,
    Number,
    #[allow(dead_code)]
    Tel,
    #[allow(dead_code)]
    Url,
}

/// Input 组件
///
/// # 示例
///
/// ```rust
/// rsx! {
///     Input {
///         input_type: InputType::Text,
///         placeholder: "请输入...",
///         value: input_value,
///         onchange: move |e| { input_value.set(e.value()); },
///     }
/// }
/// ```
#[component]
pub fn Input(
    /// 输入类型
    #[props(default = InputType::Text)]
    input_type: InputType,
    /// 输入值
    value: Option<String>,
    /// 占位符
    #[props(default)]
    placeholder: Option<String>,
    /// 是否禁用
    #[props(default = false)]
    disabled: bool,
    /// 是否必填
    #[props(default = false)]
    required: bool,
    /// 标签文本
    #[props(default)]
    label: Option<String>,
    /// 错误信息
    #[props(default)]
    error: Option<String>,
    /// 帮助文本
    #[props(default)]
    help_text: Option<String>,
    /// 值变化事件
    #[props(default)]
    onchange: Option<EventHandler<FormEvent>>,
    /// 聚焦事件
    onfocus: Option<EventHandler<FocusEvent>>,
    /// 失焦事件
    onblur: Option<EventHandler<FocusEvent>>,
    /// 自定义类名
    #[props(default)]
    class: Option<String>,
) -> Element {
    let input_type_str = match input_type {
        InputType::Text => "text",
        InputType::Password => "password",
        InputType::Email => "email",
        InputType::Number => "number",
        InputType::Tel => "tel",
        InputType::Url => "url",
    };

    let base_style = format!(
        "width: 100%; \
         background: {}; \
         border: 1px solid {}; \
         border-radius: {}; \
         padding: {} {}; \
         color: {}; \
         font-size: 16px; \
         transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);",
        Colors::BG_SECONDARY,
        Colors::BORDER_PRIMARY,
        Radius::MD,
        Spacing::MD,
        Spacing::MD,
        Colors::TEXT_PRIMARY
    );

    let error_style = if error.is_some() {
        format!("border-color: {};", Colors::PAYMENT_ERROR)
    } else {
        String::new()
    };

    let disabled_style = if disabled {
        "opacity: 0.5; cursor: not-allowed;"
    } else {
        ""
    };

    let label_clone = label.clone();
    let error_clone = error.clone();
    let help_text_clone = help_text.clone();
    let onchange_clone = onchange;
    let onfocus_clone = onfocus;
    let onblur_clone = onblur;
    let class_clone = class.clone();

    rsx! {
        div {
            class: "w-full",
            if let Some(ref label_text) = label_clone {
                label {
                    class: "block text-sm font-medium mb-2",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    {label_text.clone()}
                    if required {
                        span {
                            style: format!("color: {};", Colors::PAYMENT_ERROR),
                            " *"
                        }
                    }
                }
            }
            input {
                r#type: "{input_type_str}",
                value: value.as_deref().unwrap_or(""),
                placeholder: placeholder.as_deref().unwrap_or(""),
                disabled: disabled,
                required: required,
                class: "{class_clone.as_deref().unwrap_or_default()} focus:outline-none focus:border-[#6366F1] focus:shadow-[0_0_0_3px_rgba(99,102,241,0.1),0_4px_16px_rgba(99,102,241,0.3),0_8px_32px_rgba(99,102,241,0.2),0_0_40px_rgba(99,102,241,0.1)]",
                style: "{base_style}{error_style}{disabled_style}",
                onchange: move |e| {
                    if let Some(handler) = onchange_clone.as_ref() {
                        handler.call(e);
                    }
                },
                onfocus: move |e| {
                    if let Some(handler) = onfocus_clone.as_ref() {
                        handler.call(e);
                    }
                },
                onblur: move |e| {
                    if let Some(handler) = onblur_clone.as_ref() {
                        handler.call(e);
                    }
                },
            }
            if let Some(ref error_text) = error_clone {
                p {
                    class: "mt-2 text-sm",
                    style: format!("color: {};", Colors::PAYMENT_ERROR),
                    {error_text.clone()}
                }
            }
            if let Some(ref help) = help_text_clone {
                p {
                    class: "mt-2 text-sm",
                    style: format!("color: {};", Colors::TEXT_TERTIARY),
                    {help.clone()}
                }
            }
        }
    }
}
