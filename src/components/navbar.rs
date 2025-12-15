//! Navbar - 统一顶部导航组件
//! 响应式设计，支持移动端和桌面端

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::logo::LogoPlanet;
use crate::features::auth::hooks::use_auth;
use crate::router::Route;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use gloo_storage::Storage;

/// 统一顶部导航栏组件
#[component]
pub fn Navbar() -> Element {
    let app_state = use_context::<AppState>();
    let navigator = use_navigator();
    let auth_controller = use_auth();
    let user_state = app_state.user.read();
    let is_authenticated = user_state.is_authenticated;
    let mut show_mobile_menu = use_signal(|| false);

    // 获取翻译函数
    let t = crate::i18n::use_translation();

    rsx! {
        nav {
            class: "sticky top-0 z-50 w-full",
            style: format!("background: {}; border-bottom: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
            div {
                class: "container mx-auto px-4 sm:px-6 lg:px-8",
                div {
                    class: "flex items-center justify-between h-16",
                    // Logo和品牌
                    div {
                        class: "flex items-center gap-3",
                        div {
                            class: "cursor-pointer",
                            onclick: move |_| {
                                if is_authenticated {
                                    navigator.push(Route::Dashboard {});
                                } else {
                                    navigator.push(Route::Landing {});
                                }
                            },
                            LogoPlanet {
                                size: crate::components::logo::LogoSize::Medium,
                                variant: crate::components::logo::LogoVariant::Glowing,
                            }
                        }
                        span {
                            class: "hidden sm:block text-xl font-bold",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "IronForge"
                        }
                    }

                    // 桌面端导航菜单
                    div {
                        class: "hidden md:flex items-center gap-1",
                        if is_authenticated {
                            // 已登录用户的导航
                            NavLink {
                                route: Route::Dashboard {},
                                label: t("nav.dashboard"),
                                icon: "dashboard".to_string(),
                            }
                            NavLink {
                                route: Route::Send {},
                                label: t("nav.send"),
                                icon: "send".to_string(),
                            }
                            NavLink {
                                route: Route::Receive {},
                                label: t("nav.receive"),
                                icon: "receive".to_string(),
                            }
                            NavLink {
                                route: Route::Swap {},
                                label: t("nav.swap"),
                                icon: "swap".to_string(),
                            }
                        }
                        // 未登录用户不显示额外导航项，只显示Logo
                    }

                    // 右侧操作区
                    div {
                        class: "flex items-center gap-2",
                        // 语言切换器
                        LanguageSwitcher {}

                        if is_authenticated {
                            // 用户信息
                            div {
                                class: "hidden sm:flex items-center gap-3 mr-2",
                                div {
                                    class: "text-right",
                                    p {
                                        class: "text-sm font-medium",
                                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                                        {user_state.email.as_ref().unwrap_or(&"用户".to_string()).clone()}
                                    }
                                    p {
                                        class: "text-xs",
                                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                                        "IronForge 钱包"
                                    }
                                }
                                img {
                                    src: user_state.get_avatar_url(),
                                    alt: "Avatar",
                                    class: "w-8 h-8 rounded-full border",
                                    style: format!("border-color: {};", Colors::TECH_PRIMARY),
                                }
                            }
                            // 登出按钮
                            Button {
                                variant: ButtonVariant::Secondary,
                                size: ButtonSize::Small,
                                onclick: move |_| {
                                    let auth_ctrl = auth_controller;
                                    let nav = navigator;
                                    spawn(async move {
                                        // 调用异步登出方法（清除本地状态并调用后端API）
                                        let _ = auth_ctrl.logout().await;
                                        nav.push(Route::Landing {});
                                    });
                                },
                                {t("common.logout")}
                            }
                        } else {
                            // 登录/注册按钮
                            Button {
                                variant: ButtonVariant::Secondary,
                                size: ButtonSize::Small,
                                onclick: move |_| {
                                    navigator.push(Route::Login {});
                                },
                                {t("common.login")}
                            }
                            Button {
                                variant: ButtonVariant::Primary,
                                size: ButtonSize::Small,
                                onclick: move |_| {
                                    navigator.push(Route::Register {});
                                },
                                {t("common.register")}
                            }
                        }

                        // 移动端菜单按钮
                        button {
                            class: "md:hidden p-2 rounded-lg",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            onclick: move |_| {
                                show_mobile_menu.set(!show_mobile_menu());
                            },
                            if show_mobile_menu() {
                                "✕"
                            } else {
                                "☰"
                            }
                        }
                    }
                }
            }

            // 移动端下拉菜单
            if show_mobile_menu() {
                div {
                    class: "md:hidden border-t",
                    style: format!("border-color: {}; background: {};", Colors::BORDER_PRIMARY, Colors::BG_SECONDARY),
                    div {
                        class: "px-4 py-2 space-y-1",
                        if is_authenticated {
                            MobileNavLink {
                                route: Route::Dashboard {},
                                label: "仪表盘".to_string(),
                                icon: "dashboard".to_string(),
                                on_click: move |_| {
                                    show_mobile_menu.set(false);
                                },
                            }
                            MobileNavLink {
                                route: Route::Send {},
                                label: "发送".to_string(),
                                icon: "send".to_string(),
                                on_click: move |_| {
                                    show_mobile_menu.set(false);
                                },
                            }
                            MobileNavLink {
                                route: Route::Receive {},
                                label: "接收".to_string(),
                                icon: "receive".to_string(),
                                on_click: move |_| {
                                    show_mobile_menu.set(false);
                                },
                            }
                            MobileNavLink {
                                route: Route::Swap {},
                                label: "交换".to_string(),
                                icon: "swap".to_string(),
                                on_click: move |_| {
                                    show_mobile_menu.set(false);
                                },
                            }
                        } else {
                            MobileNavLink {
                                route: Route::Login {},
                                label: "登录".to_string(),
                                icon: "login".to_string(),
                                on_click: move |_| {
                                    show_mobile_menu.set(false);
                                },
                            }
                            MobileNavLink {
                                route: Route::Register {},
                                label: "注册".to_string(),
                                icon: "register".to_string(),
                                on_click: move |_| {
                                    show_mobile_menu.set(false);
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 桌面端导航链接组件
#[component]
fn NavLink(route: Route, label: String, icon: String) -> Element {
    let navigator = use_navigator();

    rsx! {
        button {
            class: "px-4 py-2 rounded-lg text-sm font-medium transition-all hover:opacity-80",
            style: format!("color: {};", Colors::TEXT_SECONDARY),
            onclick: move |_| {
                navigator.push(route.clone());
            },
            {label}
        }
    }
}

/// 移动端导航链接组件
#[component]
fn MobileNavLink(route: Route, label: String, icon: String, on_click: EventHandler<()>) -> Element {
    let navigator = use_navigator();

    rsx! {
        button {
            class: "w-full text-left px-4 py-3 rounded-lg text-base font-medium transition-all hover:opacity-80",
            style: format!("color: {}; background: transparent;", Colors::TEXT_SECONDARY),
            onclick: move |_| {
                navigator.push(route.clone());
                on_click.call(());
            },
            div {
                class: "flex items-center gap-3",
                span { {label} }
            }
        }
    }
}

/// 语言切换器组件
#[component]
fn LanguageSwitcher() -> Element {
    let mut app_state = use_context::<AppState>();
    let current_lang = app_state.language.read().clone();
    let mut show_menu = use_signal(|| false);

    let languages = vec![
        ("zh", "中文", "🇨🇳"),
        ("en", "English", "🇺🇸"),
        ("ja", "日本語", "🇯🇵"),
        ("ko", "한국어", "🇰🇷"),
    ];

    let current_flag = languages
        .iter()
        .find(|(code, _, _)| *code == current_lang)
        .map(|(_, _, flag)| *flag)
        .unwrap_or("🌐");

    rsx! {
        div {
            class: "relative",
            // 语言按钮
            button {
                class: "flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-all hover:opacity-80",
                style: format!("color: {}; background: {};", Colors::TEXT_SECONDARY, Colors::BG_SECONDARY),
                onclick: move |_| {
                    show_menu.set(!show_menu());
                },
                span { class: "text-base", {current_flag} }
                span { class: "hidden sm:inline", {current_lang.to_uppercase()} }
            }

            // 下拉菜单
            if show_menu() {
                div {
                    class: "absolute right-0 mt-2 py-2 rounded-lg shadow-xl z-50 min-w-[160px]",
                    style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                    for (code, name, flag) in languages {
                        button {
                            class: "w-full text-left px-4 py-2 text-sm transition-all hover:opacity-80 flex items-center gap-3",
                            style: if code == current_lang {
                                format!("color: {}; background: {};", Colors::TECH_PRIMARY, "rgba(99, 102, 241, 0.1)")
                            } else {
                                format!("color: {};", Colors::TEXT_SECONDARY)
                            },
                            onclick: move |_| {
                                let mut lang = app_state.language.write();
                                *lang = code.to_string();
                                // 保存到 LocalStorage
                                let _ = gloo_storage::LocalStorage::set("app_language", code);
                                show_menu.set(false);
                            },
                            span { class: "text-base", {flag} }
                            span { {name} }
                            if code == current_lang {
                                span { class: "ml-auto text-xs", "✓" }
                            }
                        }
                    }
                }
            }
        }
    }
}
