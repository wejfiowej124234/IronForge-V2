//! Icon Component - 图标组件
//! 生产级图标组件，支持SVG图标

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 图标尺寸
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IconSize {
    #[allow(dead_code)]
    XS, // 12px
    #[allow(dead_code)]
    SM, // 16px
    MD, // 20px
    #[allow(dead_code)]
    LG, // 24px
    XL, // 32px
    XXL, // 48px
}

impl IconSize {
    fn to_pixels(&self) -> &'static str {
        match self {
            IconSize::XS => "12px",
            IconSize::SM => "16px",
            IconSize::MD => "20px",
            IconSize::LG => "24px",
            IconSize::XL => "32px",
            IconSize::XXL => "48px",
        }
    }
}

/// Icon 组件
///
/// # 示例
///
/// ```rust
/// rsx! {
///     Icon {
///         name: "wallet",
///         size: IconSize::MD,
///     }
/// }
/// ```
#[component]
pub fn Icon(
    /// 图标名称（SVG路径或名称）
    name: String,
    /// 图标尺寸
    #[props(default = IconSize::MD)]
    size: IconSize,
    /// 图标颜色
    #[props(default)]
    color: Option<String>,
    /// 自定义类名
    #[props(default)]
    class: Option<String>,
) -> Element {
    let size_px = size.to_pixels();
    let icon_color = color.unwrap_or_else(|| Colors::TEXT_PRIMARY.to_string());
    let custom_class = class.unwrap_or_default();

    // 常用图标SVG路径
    let icon_path = match name.as_str() {
        "wallet" => "M21 12a2.25 2.25 0 0 0-2.25-2.25H15a3 3 0 1 1-6 0H5.25A2.25 2.25 0 0 0 3 12m18 0a2.25 2.25 0 0 1-2.25 2.25H15a3 3 0 1 0-6 0H5.25A2.25 2.25 0 0 1 3 12m18 0v6a2.25 2.25 0 0 1-2.25 2.25H5.25A2.25 2.25 0 0 1 3 18v-6",
        "send" => "M6 12L3.269 3.126A1.125 1.125 0 0 1 4.224 2h15.552a1.125 1.125 0 0 1 .955 1.126L18 12m0 0l-3 9m3-9l-3 9M9 12l3 9",
        "receive" => "M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3",
        "settings" => "M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 0 1 0 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 0 1 0-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281Z",
        "security" => "M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 0 1 3.598 6 11.99 11.99 0 0 0 3 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285Z",
        "close" => "M6 18L18 6M6 6l12 12",
        "check" => "M4.5 12.75l6 6 9-13.5",
        "arrow-right" => "M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3",
        "arrow-left" => "M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18",
        _ => "", // 默认空路径，可以扩展
    };

    rsx! {
        svg {
            class: "{custom_class}",
            style: "width: {size_px}; height: {size_px};",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{icon_color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path {
                d: "{icon_path}",
            }
        }
    }
}
