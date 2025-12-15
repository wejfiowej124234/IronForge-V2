//! Wallet Created Page - 钱包创建成功页面
//! 显示创建成功信息，引导用户进入Dashboard

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::router::Route;
use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// Wallet Created Page - 钱包创建成功页面
#[component]
pub fn WalletCreated() -> Element {
    let navigator = use_navigator();

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center p-4",
            style: format!("background: {};", Colors::BG_PRIMARY),

            Card {
                variant: crate::components::atoms::card::CardVariant::Base,
                padding: Some("32px".to_string()),
                children: rsx! {
                    // 成功图标和标题
                    div {
                        class: "text-center mb-6",
                        div {
                            class: "text-6xl mb-4",
                            "🎉"
                        }
                        h1 {
                            class: "text-2xl font-bold mb-2",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "钱包创建成功！"
                        }
                        p {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "您的钱包已准备就绪，可以开始使用了"
                        }
                    }

                    // 成功信息卡片
                    div {
                        class: "mb-6 p-4 rounded-lg",
                        style: format!("background: rgba(34, 197, 94, 0.1); border: 1px solid #22c55e;"),
                        div {
                            class: "space-y-2 text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            div {
                                class: "flex items-center gap-2",
                                span { "✅" }
                                span { "钱包已创建并加密保存" }
                            }
                            div {
                                class: "flex items-center gap-2",
                                span { "✅" }
                                span { "多链地址已生成（ETH, BTC, SOL, TON）" }
                            }
                            div {
                                class: "flex items-center gap-2",
                                span { "✅" }
                                span { "助记词已备份验证" }
                            }
                        }
                    }

                    // 安全提示
                    div {
                        class: "mb-6 p-4 rounded-lg",
                        style: format!("background: rgba(99, 102, 241, 0.1); border: 1px solid {};", Colors::TECH_PRIMARY),
                        h3 {
                            class: "font-semibold mb-2",
                            style: format!("color: {};", Colors::TECH_PRIMARY),
                            "💡 温馨提示"
                        }
                        ul {
                            class: "text-sm space-y-1 list-disc list-inside",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            li { "钱包将在5分钟无操作后自动锁定" }
                            li { "请妥善保管您的助记词，这是恢复钱包的唯一方式" }
                            li { "建议定期备份钱包数据" }
                        }
                    }

                    // 操作按钮
                    div {
                        class: "flex gap-4",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            class: Some("flex-1".to_string()),
                            onclick: move |_| {
                                navigator.push(Route::Dashboard {});
                            },
                            "进入钱包"
                        }
                    }
                }
            }
        }
    }
}
