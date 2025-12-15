//! NotFound Page - 404页面

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// NotFound Page 组件
#[component]
pub fn NotFound() -> Element {
    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center",
            style: format!("background: {};", Colors::BG_PRIMARY),
            div {
                class: "text-center",
                h1 {
                    class: "text-6xl font-bold mb-4",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    "404"
                }
                p {
                    class: "text-xl mb-8",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    "页面未找到"
                }
            }
        }
    }
}
