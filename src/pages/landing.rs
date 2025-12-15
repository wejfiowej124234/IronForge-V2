//! Landing Page - 营销首页
//! 参考 Juno Network 设计，灵活现代的布局
//! 融入钱包特性，视觉冲击力强

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::logo::LogoPlanet;
use crate::router::Route;
use crate::shared::design_tokens::{Colors, Glass, Gradients};
use dioxus::prelude::*;

/// Landing Page 组件
#[component]
pub fn Landing() -> Element {
    let navigator = use_navigator();

    rsx! {
        div {
            class: "min-h-screen overflow-x-hidden",
            style: format!("background: {}; background-image: {};", Colors::BG_PRIMARY, Gradients::BG_HERO),

            // Hero Section - 更灵活的大胆设计
            section {
                class: "container mx-auto px-6 py-16 md:py-24",
                div {
                    class: "max-w-5xl mx-auto",
                    // Logo - 居中但更显眼
                    div {
                        class: "flex justify-center mb-8",
                        LogoPlanet {
                            size: crate::components::logo::LogoSize::XLarge,
                            variant: crate::components::logo::LogoVariant::Glowing,
                        }
                    }

                    // 主标题 - 更大更醒目
                    div {
                        class: "text-center mb-8",
                        h1 {
                            class: "text-5xl md:text-7xl lg:text-8xl font-bold mb-6 leading-tight",
                            style: format!("background: {}; -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;", Gradients::PRIMARY),
                            "The Gateway to"
                        }
                        h1 {
                            class: "text-5xl md:text-7xl lg:text-8xl font-bold mb-6 leading-tight",
                            style: format!("background: {}; -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;", Gradients::PRIMARY),
                            "Web3 Wallets"
                        }
                        p {
                            class: "text-lg sm:text-xl md:text-2xl lg:text-3xl mb-4",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "Non-Custodial × Multi-Chain × DeFi × Fiat Gateway"
                        }
                        p {
                            class: "text-sm sm:text-base md:text-lg mb-6 sm:mb-8 max-w-2xl mx-auto px-4",
                            style: format!("color: {};", Colors::TEXT_TERTIARY),
                            "下一代非托管企业级 Web3 钱包 | 您的私钥，您完全掌控 | 安全、高效、多链支持 | DeFi + 法币兑换一站式体验"
                        }
                        div {
                            class: "flex flex-wrap justify-center gap-2 sm:gap-4 mb-8 px-4",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            span {
                                class: "text-xs sm:text-sm px-3 py-1 rounded-full",
                                style: format!("background: rgba(99, 102, 241, 0.1); border: 1px solid {};", Colors::TECH_PRIMARY),
                                "🔒 非托管"
                            }
                            span {
                                class: "text-xs sm:text-sm px-3 py-1 rounded-full",
                                style: format!("background: rgba(99, 102, 241, 0.1); border: 1px solid {};", Colors::TECH_PRIMARY),
                                "🌐 多链支持"
                            }
                            span {
                                class: "text-xs sm:text-sm px-3 py-1 rounded-full",
                                style: format!("background: rgba(99, 102, 241, 0.1); border: 1px solid {};", Colors::TECH_PRIMARY),
                                "💸 DeFi 集成"
                            }
                            span {
                                class: "text-xs sm:text-sm px-3 py-1 rounded-full",
                                style: format!("background: rgba(99, 102, 241, 0.1); border: 1px solid {};", Colors::TECH_PRIMARY),
                                "💳 法币兑换"
                            }
                            span {
                                class: "text-xs sm:text-sm px-3 py-1 rounded-full",
                                style: format!("background: rgba(99, 102, 241, 0.1); border: 1px solid {};", Colors::TECH_PRIMARY),
                                "⚡ 企业级"
                            }
                        }
                    }

                    // CTA 按钮 - 更突出的设计，移动端优化
                    div {
                        class: "flex flex-col sm:flex-row gap-3 sm:gap-4 justify-center items-center mb-12 sm:mb-16 px-4",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            class: Some("w-full sm:w-auto".to_string()),
                            onclick: move |_| {
                                navigator.push(Route::Register {});
                            },
                            "注册账户 →"
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Large,
                            class: Some("w-full sm:w-auto".to_string()),
                            onclick: move |_| {
                                navigator.push(Route::Login {});
                            },
                            "登录账户"
                        }
                    }
                }
            }

            // Quick Start Guide Section - 参考Juno的设计
            section {
                class: "container mx-auto px-6 py-16",
                div {
                    class: "max-w-6xl mx-auto",
                    div {
                        class: "text-center mb-12",
                        h2 {
                            class: "text-3xl md:text-4xl font-bold mb-4",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "快速开始"
                        }
                        p {
                            class: "text-lg",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "三种方式开始使用 IronForge"
                        }
                    }

                    // Quick Start Cards - 灵活的3列布局
                    div {
                        class: "grid grid-cols-1 md:grid-cols-3 gap-6 mb-16",
                        QuickStartCard {
                            title: "创建钱包",
                            description: "生成新的多链钱包，支持 Bitcoin, Ethereum, Solana, TON",
                            icon: "wallet",
                            action: "开始创建",
                            route: Route::CreateWallet {},
                        }
                        QuickStartCard {
                            title: "导入钱包",
                            description: "使用助记词、私钥或Keystore恢复现有钱包",
                            icon: "wallet",
                            action: "导入钱包",
                            route: Route::ImportWallet {},
                        }
                        QuickStartCard {
                            title: "查看仪表盘",
                            description: "查看资产、交易历史和钱包详情",
                            icon: "wallet",
                            action: "进入仪表盘",
                            route: Route::Dashboard {},
                        }
                    }
                }
            }

            // 核心特性 Section - 灵活的非对称布局
            section {
                class: "container mx-auto px-4 sm:px-6 py-12 sm:py-16",
                div {
                    class: "max-w-6xl mx-auto",
                    div {
                        class: "text-center mb-8 sm:mb-12",
                        h2 {
                            class: "text-2xl sm:text-3xl md:text-4xl font-bold mb-3 sm:mb-4",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "核心特性"
                        }
                    }

                    // 灵活的非对称网格布局，移动端优化
                    div {
                        class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6",
                        // 大卡片 - 占据2列
                        div {
                            class: "md:col-span-2 lg:col-span-2",
                            FeatureCardLarge {
                                title: "🔒 非托管安全架构",
                                description: "您的私钥，您完全掌控。零信任架构，内存安全保证。使用 Argon2id KDF 和 AES-256-GCM 加密，私钥永不离开本地设备。自动锁定机制、双锁保护（账户锁+钱包锁），全方位保护您的数字资产。",
                                icon: "security",
                                gradient: "from-[#6366F1] to-[#8B5CF6]",
                            }
                        }
                        // 小卡片
                        FeatureCardSmall {
                            title: "🌐 多链原生支持",
                            description: "Bitcoin, Ethereum, Solana, TON - 一个钱包管理所有链",
                            icon: "wallet",
                        }
                        FeatureCardSmall {
                            title: "💸 DeFi 一站式",
                            description: "跨链桥接、代币交换、NFT管理",
                            icon: "send",
                        }
                        FeatureCardSmall {
                            title: "💳 法币兑换",
                            description: "加密货币直接提现到银行卡，多支付方式支持",
                            icon: "wallet",
                        }
                        // 另一个大卡片
                        div {
                            class: "md:col-span-2 lg:col-span-2",
                            FeatureCardLarge {
                                title: "⚡ 企业级性能",
                                description: "基于 Rust 构建，内存安全、高性能、并发安全。智能 Gas 费优化，自动选择最优网络。实时交易状态追踪，多设备同步（查看余额），新设备安全恢复。",
                                icon: "settings",
                                gradient: "from-[#8B5CF6] to-[#06B6D4]",
                            }
                        }
                        FeatureCardSmall {
                            title: "🔐 企业API集成",
                            description: "RESTful API，支持企业级应用集成",
                            icon: "settings",
                        }
                        FeatureCardSmall {
                            title: "📱 响应式设计",
                            description: "完美适配桌面、平板、移动设备",
                            icon: "wallet",
                        }
                    }
                }
            }

            // 多链支持可视化 Section
            section {
                class: "container mx-auto px-4 sm:px-6 py-12 sm:py-16",
                div {
                    class: "max-w-6xl mx-auto",
                    div {
                        class: "text-center mb-8 sm:mb-12",
                        h2 {
                            class: "text-2xl sm:text-3xl md:text-4xl font-bold mb-3 sm:mb-4",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "多链支持"
                        }
                        p {
                            class: "text-base sm:text-lg",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "原生支持主流的区块链网络"
                        }
                    }

                    // 链展示卡片 - 移动端2列，桌面端4列
                    div {
                        class: "grid grid-cols-2 sm:grid-cols-2 md:grid-cols-4 gap-3 sm:gap-4",
                        ChainCard {
                            name: "Bitcoin",
                            symbol: "BTC",
                            color: "#F7931A",
                        }
                        ChainCard {
                            name: "Ethereum",
                            symbol: "ETH",
                            color: "#627EEA",
                        }
                        ChainCard {
                            name: "Solana",
                            symbol: "SOL",
                            color: "#9945FF",
                        }
                        ChainCard {
                            name: "TON",
                            symbol: "TON",
                            color: "#0088CC",
                        }
                    }
                }
            }

            // 技术优势 Section
            section {
                class: "container mx-auto px-4 sm:px-6 py-12 sm:py-16",
                div {
                    class: "max-w-6xl mx-auto",
                    div {
                        class: "text-center mb-8 sm:mb-12",
                        h2 {
                            class: "text-2xl sm:text-3xl md:text-4xl font-bold mb-3 sm:mb-4",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "技术优势"
                        }
                        p {
                            class: "text-base sm:text-lg",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "基于 Rust 的现代化技术栈"
                        }
                    }

                    // 技术特性网格 - 移动端单列，平板2列，桌面3列
                    div {
                        class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6",
                        TechFeatureCard {
                            title: "Rust 构建",
                            description: "内存安全、高性能、并发安全，零成本抽象",
                        }
                        TechFeatureCard {
                            title: "Dioxus 框架",
                            description: "现代化的 Web 框架，类似 React，性能卓越",
                        }
                        TechFeatureCard {
                            title: "非托管架构",
                            description: "私钥本地加密存储，服务端仅存储公钥",
                        }
                        TechFeatureCard {
                            title: "BIP39/BIP44",
                            description: "标准化的助记词和密钥派生，兼容所有主流钱包",
                        }
                        TechFeatureCard {
                            title: "IndexedDB 存储",
                            description: "浏览器本地加密存储，数据永不离开设备",
                        }
                        TechFeatureCard {
                            title: "双锁机制",
                            description: "账户锁（邮箱+密码）+ 钱包锁（密码+私钥）",
                        }
                        TechFeatureCard {
                            title: "跨链桥接",
                            description: "集成 LiFi API，支持多链资产桥接",
                        }
                        TechFeatureCard {
                            title: "DEX 聚合",
                            description: "集成 1inch API，最优价格代币交换",
                        }
                        TechFeatureCard {
                            title: "NFT 管理",
                            description: "集成 Alchemy API，支持 ERC721/ERC1155",
                        }
                        TechFeatureCard {
                            title: "法币兑换",
                            description: "集成 MoonPay API，支持银行卡/PayPal/Apple Pay",
                        }
                    }
                }
            }

            // CTA Section - 最后的行动号召
            section {
                class: "container mx-auto px-4 sm:px-6 py-12 sm:py-20",
                div {
                    class: "max-w-4xl mx-auto text-center",
                    style: format!("{}", Glass::strong()),
                    class: "rounded-2xl sm:rounded-3xl p-6 sm:p-12",
                    h2 {
                        class: "text-3xl md:text-4xl font-bold mb-4",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "准备开始了吗？"
                    }
                    p {
                        class: "text-lg mb-8",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        "立即创建您的 Web3 钱包，体验下一代区块链技术"
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::XLarge,
                        onclick: move |_| {
                            navigator.push(Route::CreateWallet {});
                        },
                        "创建钱包 →"
                    }
                }
            }
        }
    }
}

/// Quick Start 卡片组件
#[component]
fn QuickStartCard(
    title: String,
    description: String,
    icon: String,
    action: String,
    route: Route,
) -> Element {
    let navigator = use_navigator();

    rsx! {
        Card {
            variant: crate::components::atoms::card::CardVariant::Strong,
            padding: Some("32px".to_string()),
            children: rsx! {
                div {
                    class: "text-center h-full flex flex-col",
                    div {
                        class: "flex justify-center mb-4",
                        crate::components::atoms::icon::Icon {
                            name: icon.clone(),
                            size: crate::components::atoms::icon::IconSize::XXL,
                        }
                    }
                    h3 {
                        class: "text-xl font-semibold mb-2",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {title}
                    }
                    p {
                        class: "text-sm mb-6 flex-grow",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        {description}
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Medium,
                        class: Some("w-full".to_string()),
                        onclick: {
                            let route_clone = route.clone();
                            move |_| {
                                navigator.push(route_clone.clone());
                            }
                        },
                        {action}
                    }
                }
            }
        }
    }
}

/// 大特性卡片组件
#[component]
fn FeatureCardLarge(title: String, description: String, icon: String, gradient: String) -> Element {
    rsx! {
        Card {
            variant: crate::components::atoms::card::CardVariant::Strong,
            padding: Some("48px".to_string()),
            children: rsx! {
                div {
                    class: "flex flex-col md:flex-row items-center gap-6",
                    div {
                        class: "flex-shrink-0",
                        div {
                            class: format!("w-20 h-20 rounded-2xl bg-gradient-to-br {} flex items-center justify-center", gradient),
                            crate::components::atoms::icon::Icon {
                                name: icon.clone(),
                                size: crate::components::atoms::icon::IconSize::XXL,
                                color: Some("#FFFFFF".to_string()),
                            }
                        }
                    }
                    div {
                        class: "flex-grow",
                        h3 {
                            class: "text-2xl font-bold mb-3",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            {title}
                        }
                        p {
                            class: "text-base leading-relaxed",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            {description}
                        }
                    }
                }
            }
        }
    }
}

/// 小特性卡片组件
#[component]
fn FeatureCardSmall(title: String, description: String, icon: String) -> Element {
    rsx! {
        Card {
            variant: crate::components::atoms::card::CardVariant::Strong,
            padding: Some("32px".to_string()),
            children: rsx! {
                div {
                    class: "text-center h-full flex flex-col",
                    div {
                        class: "flex justify-center mb-4",
                        crate::components::atoms::icon::Icon {
                            name: icon.clone(),
                            size: crate::components::atoms::icon::IconSize::XL,
                        }
                    }
                    h3 {
                        class: "text-xl font-semibold mb-2",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {title}
                    }
                    p {
                        class: "text-sm",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        {description}
                    }
                }
            }
        }
    }
}

/// 链卡片组件
#[component]
fn ChainCard(name: String, symbol: String, color: String) -> Element {
    rsx! {
        Card {
            variant: crate::components::atoms::card::CardVariant::Base,
            padding: Some("24px".to_string()),
            children: rsx! {
                div {
                    class: "text-center",
                    div {
                        class: "w-16 h-16 rounded-full mx-auto mb-3 flex items-center justify-center",
                        style: format!("background: {};", color),
                        span {
                            class: "text-2xl font-bold text-white",
                            {symbol.clone()}
                        }
                    }
                    h3 {
                        class: "text-lg font-semibold mb-1",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {name}
                    }
                    span {
                        class: "text-sm",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        {symbol}
                    }
                }
            }
        }
    }
}

/// 技术特性卡片组件
#[component]
fn TechFeatureCard(title: String, description: String) -> Element {
    rsx! {
        Card {
            variant: crate::components::atoms::card::CardVariant::Base,
            padding: Some("24px".to_string()),
            children: rsx! {
                h3 {
                    class: "text-lg font-semibold mb-2",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    {title}
                }
                p {
                    class: "text-sm leading-relaxed",
                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                    {description}
                }
            }
        }
    }
}
