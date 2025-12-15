//! Select Component - 选择框组件
//! 生产级选择框组件，基于设计系统 V3

use crate::shared::design_tokens::{Colors, Radius, Spacing};
use dioxus::prelude::*;

/// Select选项
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    #[allow(dead_code)] // 用于未来扩展
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Select 组件
///
/// # 示例
///
/// ```rust
/// rsx! {
///     Select {
///         label: Some("选择链".to_string()),
///         value: selected_chain,
///         options: vec![
///             SelectOption::new("ethereum", "Ethereum"),
///             SelectOption::new("polygon", "Polygon"),
///         ],
///         onchange: move |e| { selected_chain.set(e.value()); },
///     }
/// }
/// ```
#[component]
pub fn Select(
    /// 选择值
    value: Option<String>,
    /// 选项列表
    options: Vec<SelectOption>,
    /// 标签文本
    #[props(default)]
    label: Option<String>,
    /// 占位符
    #[props(default)]
    placeholder: Option<String>,
    /// 是否禁用
    #[props(default = false)]
    disabled: bool,
    /// 是否必填
    #[props(default = false)]
    required: bool,
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
    let base_style = format!(
        "width: 100%; \
         background: {}; \
         border: 1px solid {}; \
         border-radius: {}; \
         padding: {} {}; \
         color: {}; \
         font-size: 16px; \
         transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); \
         appearance: none; \
         background-image: url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23E5E7EB' d='M6 9L1 4h10z'/%3E%3C/svg%3E\"); \
         background-repeat: no-repeat; \
         background-position: right {} center; \
         padding-right: 40px;",
        Colors::BG_SECONDARY,
        Colors::BORDER_PRIMARY,
        Radius::MD,
        Spacing::MD,
        Spacing::MD,
        Colors::TEXT_PRIMARY,
        Spacing::MD
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

    let focus_style = "focus:outline-none focus:border-[#6366F1] focus:shadow-[0_0_0_3px_rgba(99,102,241,0.1),0_4px_16px_rgba(99,102,241,0.3),0_8px_32px_rgba(99,102,241,0.2),0_0_40px_rgba(99,102,241,0.1)]";

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
            select {
                value: value.as_deref().unwrap_or(""),
                disabled: disabled,
                required: required,
                class: "{class_clone.as_deref().unwrap_or_default()} {focus_style}",
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
                if let Some(ref placeholder_text) = placeholder {
                    option {
                        value: "",
                        disabled: true,
                        selected: value.is_none() || value.as_ref().unwrap().is_empty(),
                        {placeholder_text.clone()}
                    }
                }
                for option in options.iter() {
                    option {
                        value: "{option.value}",
                        disabled: option.disabled,
                        selected: value.as_ref().map(|v| v == &option.value).unwrap_or(false),
                        {option.label.clone()}
                    }
                }
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
