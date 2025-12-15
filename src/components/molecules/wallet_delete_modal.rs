//! Wallet Delete Modal - 删除钱包确认弹窗

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::modal::Modal;
use crate::features::wallet::hooks::WalletController;
use crate::router::Route;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use dioxus_router::use_navigator;

/// 删除钱包确认弹窗（与全局主题风格一致）
#[component]
pub fn WalletDeleteModal(
    open: bool,
    wallet_id: String,
    wallet_name: String,
    app_state: Signal<AppState>,
    wallet_controller: Signal<WalletController>,
    on_close: EventHandler<()>,
) -> Element {
    let navigator = use_navigator();

    let app_state = *app_state.read();
    let wallet_controller = *wallet_controller.read();

    rsx! {
        Modal {
            open: open,
            onclose: move |_| {
                on_close.call(());
            },
            title: Some("确认删除钱包".to_string()),
            children: rsx! {
                div {
                    class: "space-y-4",
                    // 主提示：提高字号和对比度
                    p {
                        class: "text-base font-medium",
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        {format!("确定要删除钱包 \"{}\" 吗？", wallet_name)}
                    }
                    // 风险说明：稍大字号，突出红色警告
                    p {
                        class: "text-sm",
                        style: format!("color: {};", Colors::PAYMENT_ERROR),
                        "删除后将无法恢复：包括私钥、账户配置和本地交易记录。请确保已安全备份助记词。"
                    }
                    // 说明列表：字号提升一档，保持次要颜色
                    ul {
                        class: "text-sm list-disc list-inside",
                        style: format!("color: {};", Colors::TEXT_TERTIARY),
                        li { "此操作只影响本设备，不会删除区块链上的历史交易。" }
                        li { "如需在新设备继续使用，请先确认已备份助记词。" }
                    }
                    div {
                        class: "flex gap-3 mt-6",
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Small,
                            class: Some("flex-1".to_string()),
                            onclick: move |_| {
                                on_close.call(());
                            },
                            "取消"
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Small,
                            class: Some("flex-1".to_string()),
                            onclick: move |_| {
                                let wallet_id_clone = wallet_id.clone();
                                let wallet_name_clone = wallet_name.clone();
                                let app_state_clone = app_state;
                                let wallet_controller_clone = wallet_controller;
                                let navigator_clone = navigator;
                                let on_close = on_close;
                                spawn(async move {
                                    match wallet_controller_clone.delete_wallet(&wallet_id_clone).await {
                                        Ok(_) => {
                                            AppState::show_success(
                                                app_state_clone.toasts,
                                                format!("钱包 \"{}\" 已删除", wallet_name_clone)
                                            );
                                            on_close.call(());
                                            navigator_clone.push(Route::Dashboard {});
                                        }
                                        Err(e) => {
                                            AppState::show_error(
                                                app_state_clone.toasts,
                                                format!("删除钱包失败: {}", e)
                                            );
                                            on_close.call(());
                                        }
                                    }
                                });
                            },
                            "确认删除"
                        }
                    }
                }
            }
        }
    }
}
