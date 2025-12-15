//! Router - 路由系统
//! 生产级路由实现，使用 Dioxus Router

use dioxus::prelude::*;

// 导入所有页面组件
// Dioxus Router的Routable宏会自动匹配Route枚举变体名称到同名的组件函数
// 组件必须在当前作用域中可见，所以需要显式导入
use crate::components::navbar::Navbar;
use crate::components::route_guard::AuthGuard;
use crate::pages::{
    Bridge, Buy, CreateWallet, Dashboard, ImportWallet, Landing, Login, MnemonicBackup,
    MnemonicVerify, NotFound, Orders, Receive, Register, Sell, Send, Swap, WalletCreated,
    WalletDetail,
};

/// 路由定义
/// 使用嵌套路由，所有路由都在AppLayout内部
#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Landing {},
    
    #[route("/login")]
    Login {},
    
    #[route("/register")]
    Register {},
    
    #[route("/dashboard")]
    Dashboard {},
    
    #[route("/wallet/create")]
    CreateWallet {},
    
    #[route("/wallet/mnemonic/backup/:phrase")]
    MnemonicBackup { phrase: String },
    
    #[route("/wallet/mnemonic/verify/:phrase")]
    MnemonicVerify { phrase: String },
    
    #[route("/wallet/created")]
    WalletCreated {},
    
    #[route("/wallet/import")]
    ImportWallet {},
    
    #[route("/wallet/:id")]
    WalletDetail { id: String },
    
    #[route("/send")]
    Send {},
    
    #[route("/receive")]
    Receive {},
    
    #[route("/swap")]
    Swap {},
    
    #[route("/buy")]
    Buy {},
    
    #[route("/sell")]
    Sell {},
    
    #[route("/orders")]
    Orders {},
    
    #[route("/bridge")]
    Bridge {},
    
    #[route("/..")]
    NotFound {},
}

/// 应用布局组件 - 包含Navbar和路由内容
/// 这个组件作为所有路由的父组件，提供Navbar
/// Navbar在Router内部，可以安全使用use_navigator()
#[component]
pub fn AppLayout() -> Element {
    rsx! {
        div {
            // 统一顶部导航栏（所有页面共享）
            Navbar {}

            // 路由内容
            Outlet::<Route> {}
        }
    }
}

/// Router 组件
/// 在Dioxus Router 0.7中，Router会自动渲染匹配的路由组件
#[component]
pub fn AppRouter() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

/// 受保护的路由包装器
/// 需要在页面组件内部使用 AuthGuard
#[component]
pub fn ProtectedRoute(children: Element) -> Element {
    rsx! {
        AuthGuard {
            {children}
        }
    }
}
