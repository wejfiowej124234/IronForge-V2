//! Route Guard - 路由守卫
//! 用于保护需要认证的路由

use crate::router::Route;
use crate::shared::state::AppState;
use dioxus::prelude::*;

/// 路由守卫组件 - 检查用户是否已登录
#[component]
pub fn AuthGuard(children: Element) -> Element {
    let app_state = use_context::<AppState>();
    let navigator = use_navigator();

    let is_authenticated = app_state.user.read().is_authenticated;

    // 检查登录状态并跳转
    if !is_authenticated {
        use_effect(move || {
            let nav = navigator;
            spawn(async move {
                nav.push(Route::Login {});
            });
        });

        return rsx! {
            div {
                class: "min-h-screen flex items-center justify-center",
                style: "background: #0A0A0F;",
                div {
                    class: "text-center",
                    p {
                        style: "color: #E5E7EB;",
                        "正在跳转到登录页面..."
                    }
                }
            }
        };
    }

    rsx! { {children} }
}
