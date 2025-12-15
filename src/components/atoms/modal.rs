//! Modal Component - 模态框组件
//! 生产级模态框组件，提升层毛玻璃效果

use crate::components::atoms::icon::{Icon, IconSize};
use crate::shared::design_tokens::{Radius, Shadows};
use dioxus::prelude::*;

/// Modal 组件
///
/// # 示例
///
/// ```rust
/// rsx! {
///     Modal {
///         open: show_modal,
///         onclose: move |_| { show_modal.set(false); },
///         title: "标题",
///         children: rsx! {
///             p { "内容" }
///         }
///     }
/// }
/// ```
#[component]
pub fn Modal(
    /// 是否打开
    open: bool,
    /// 关闭事件
    onclose: EventHandler<MouseEvent>,
    /// 标题
    #[props(default)]
    title: Option<String>,
    /// 是否显示关闭按钮
    #[props(default = true)]
    show_close: bool,
    /// 自定义类名
    #[props(default)]
    class: Option<String>,
    /// 模态框内容
    children: Element,
) -> Element {
    if !open {
        return rsx! { div { style: "display: none;", } };
    }

    let overlay_style = format!(
        "position: fixed; \
         inset: 0; \
         background: rgba(0, 0, 0, 0.6); \
         backdrop-filter: blur(4px); \
         z-index: 50; \
         display: flex; \
         align-items: center; \
         justify-content: center; \
         padding: 20px;"
    );

    let modal_style = format!(
        "background: rgba(36, 36, 47, 0.8); \
         backdrop-filter: blur(40px) saturate(220%); \
         -webkit-backdrop-filter: blur(40px) saturate(220%); \
         border: 1px solid rgba(255, 255, 255, 0.2); \
         border-radius: {}; \
         box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6), {}; \
         max-width: 500px; \
         width: 100%; \
         max-height: 90vh; \
         overflow-y: auto;",
        Radius::XL,
        Shadows::INNER_GLOW
    );

    rsx! {
        div {
            class: "modal-overlay",
            style: "{overlay_style}",
            div {
                id: "modal-overlay",
                class: "absolute inset-0",
                onclick: move |e| {
                    onclose.call(e);
                },
            }
            div {
                class: class.unwrap_or_default(),
                style: "{modal_style}",
                onclick: move |e| { e.stop_propagation(); },
                // 标题栏
                if title.is_some() || show_close {
                    div {
                        class: "flex items-center justify-between p-6 border-b",
                        style: "border-color: rgba(255, 255, 255, 0.08);",
                        if let Some(title_text) = title {
                            h2 {
                                class: "text-xl font-semibold",
                                style: "color: #FFFFFF;",
                                {title_text}
                            }
                        }
                        if show_close {
                            button {
                                class: "p-2 rounded-lg hover:bg-white/10 transition-colors",
                                onclick: move |e| { onclose.call(e); },
                                style: "color: #E5E7EB;",
                                Icon {
                                    name: "close".to_string(),
                                    size: IconSize::MD,
                                }
                            }
                        }
                    }
                }
                // 内容区域
                div {
                    class: "p-6",
                    {children}
                }
            }
        }
    }
}
