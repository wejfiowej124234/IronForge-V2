//! Settings Page - 设置页面（已废弃）
//! 为了简化用户体验，设置页已从导航中移除。
//! 保留一个空组件占位，避免旧链接导致编译错误。

use dioxus::prelude::*;

/// Deprecated Settings Page
#[component]
pub fn Settings() -> Element {
    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center",
            p { "此版本中设置页面已移除。" }
        }
    }
}
