//! Mnemonic Backup Page - 助记词备份页面
//! 显示助记词，要求用户备份

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::router::Route;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use js_sys;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Blob, HtmlElement, Url};

/// Mnemonic Backup Page - 助记词备份页面
///
/// 显示助记词，要求用户：
/// 1. 按住查看（防窥屏）
/// 2. 确认已备份
/// 3. 进入验证步骤
#[component]
pub fn MnemonicBackup(
    /// 助记词短语（通过路由参数传递）
    phrase: String,
) -> Element {
    let is_revealed = use_signal(|| false);
    let is_confirmed = use_signal(|| false);
    let navigator = use_navigator();
    let app_state = use_context::<AppState>();

    // 将助记词分割成单词数组
    let words: Vec<String> = phrase.split_whitespace().map(|s| s.to_string()).collect();

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center p-4",
            style: format!("background: {};", Colors::BG_PRIMARY),

            Card {
                variant: crate::components::atoms::card::CardVariant::Base,
                padding: Some("32px".to_string()),
                children: rsx! {
                    // 标题
                    div {
                        class: "text-center mb-6",
                        h1 {
                            class: "text-2xl font-bold mb-2",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "备份助记词"
                        }
                        p {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "请按顺序抄写以下12个单词，并妥善保管"
                        }
                    }

                    // 安全提示
                    div {
                        class: "mb-6 p-4 rounded-lg",
                        style: format!("background: rgba(239, 68, 68, 0.1); border: 1px solid {};", Colors::PAYMENT_ERROR),
                        div {
                            class: "flex items-start gap-2",
                            span {
                                class: "text-xl",
                                style: format!("color: {};", Colors::PAYMENT_ERROR),
                                "⚠️"
                            }
                            div {
                                p {
                                    class: "font-semibold mb-1",
                                    style: format!("color: {};", Colors::PAYMENT_ERROR),
                                    "重要提示"
                                }
                                ul {
                                    class: "text-sm space-y-1 list-disc list-inside",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    li { "助记词是恢复钱包的唯一方式，请务必妥善保管" }
                                    li { "不要截图或拍照保存，避免泄露" }
                                    li { "不要将助记词存储在联网设备上" }
                                    li { "丢失助记词将无法恢复钱包资产" }
                                }
                            }
                        }
                    }

                    // 助记词网格
                    if is_revealed() {
                        div {
                            class: "mb-6",
                            div {
                                class: "grid grid-cols-3 gap-3",
                                for (index, word) in words.iter().enumerate() {
                                    div {
                                        class: "p-3 rounded-lg border",
                                        style: format!("background: {}; border-color: {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                                        div {
                                            class: "text-xs mb-1",
                                            style: format!("color: {};", Colors::TEXT_TERTIARY),
                                            "{index + 1}"
                                        }
                                        div {
                                            class: "font-semibold",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            {word.clone()}
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // 模糊显示
                        div {
                            class: "mb-6",
                            div {
                                class: "grid grid-cols-3 gap-3",
                                for i in 0..12 {
                                    div {
                                        class: "p-3 rounded-lg border",
                                        style: format!("background: {}; border-color: {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                                        div {
                                            class: "text-xs mb-1",
                                            style: format!("color: {};", Colors::TEXT_TERTIARY),
                                            "{i + 1}"
                                        }
                                        div {
                                            class: "font-semibold blur-sm",
                                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                                            "••••"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 显示/隐藏按钮
                    if !is_revealed() {
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            class: Some("w-full mb-4".to_string()),
                            onclick: {
                                let mut is_revealed = is_revealed;
                                move |_| {
                                    is_revealed.set(true);
                                }
                            },
                            "按住查看助记词"
                        }
                    }

                    // 备份操作按钮（仅在显示助记词时显示）
                    if is_revealed() {
                        div {
                            class: "mb-6 flex flex-col gap-3",
                            div {
                                class: "flex gap-2",
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    size: ButtonSize::Medium,
                                    class: Some("flex-1".to_string()),
                                    onclick: {
                                        let phrase = phrase.clone();
                                        let app_state = app_state;
                                        move |_| {
                                            let phrase_clone = phrase.clone();
                                            let toasts = app_state.toasts;
                                            spawn(async move {
                                                // 复制到剪贴板
                                                if let Some(window) = window() {
                                                    let clipboard = window.navigator().clipboard();
                                                    let promise = clipboard.write_text(&phrase_clone);
                                                    if JsFuture::from(promise).await.is_ok() {
                                                        AppState::show_success(toasts, "助记词已复制到剪贴板".to_string());
                                                    } else {
                                                        AppState::show_error(toasts, "复制失败，请手动复制".to_string());
                                                    }
                                                }
                                            });
                                        }
                                    },
                                    "📋 复制助记词"
                                }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    size: ButtonSize::Medium,
                                    class: Some("flex-1".to_string()),
                                    onclick: {
                                        let phrase = phrase.clone();
                                        let app_state = app_state;
                                        move |_| {
                                            let phrase_clone = phrase.clone();
                                            let toasts = app_state.toasts;
                                            spawn(async move {
                                                // 下载TXT文件
                                                if let Some(window) = window() {
                                                    let document = window.document().expect("无法获取document");

                                                    // 创建文件内容
                                                    use chrono::Utc;
                                                    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
                                                    let filename = format!("wallet_mnemonic_{}.txt", timestamp);
                                                    let content = format!(
                                                        "IronForge 钱包助记词备份\n\
                                                        ======================\n\
                                                        \n\
                                                        创建时间: {}\n\
                                                        \n\
                                                        重要提示：\n\
                                                        - 这是您钱包的助记词，请妥善保管\n\
                                                        - 不要将助记词存储在联网设备上\n\
                                                        - 不要截图或拍照保存\n\
                                                        - 丢失助记词将无法恢复钱包资产\n\
                                                        \n\
                                                        助记词（12个单词）：\n\
                                                        {}\n\
                                                        \n\
                                                        ======================\n\
                                                        请妥善保管此文件，建议打印后存放在安全的地方。\n",
                                                        Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                                                        phrase_clone
                                                    );

                                                    // 创建Blob
                                                    let blob_parts = js_sys::Array::new();
                                                    let uint8_array = js_sys::Uint8Array::new_with_length(content.len() as u32);
                                                    let bytes = content.as_bytes();
                                                    for (i, &byte) in bytes.iter().enumerate() {
                                                        uint8_array.set_index(i as u32, byte as u8);
                                                    }
                                                    blob_parts.push(&uint8_array);

                                                    // 使用简单的Blob创建方法
                                                    if let Ok(blob) = Blob::new_with_u8_array_sequence(&blob_parts) {
                                                        if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                                                            // 创建下载链接
                                                            if let Ok(link) = document.create_element("a") {
                                                                let link_element = link.dyn_ref::<HtmlElement>()
                                                                    .expect("无法转换为HtmlElement");

                                                                // 使用set_attribute设置属性
                                                                link_element.set_attribute("href", &url).ok();
                                                                link_element.set_attribute("download", &filename).ok();
                                                                link_element.set_attribute("style", "display: none").ok();

                                                                if let Some(body) = document.body() {
                                                                    if let Err(_) = body.append_child(&link) {
                                                                        AppState::show_error(toasts, "下载失败".to_string());
                                                                        let _ = Url::revoke_object_url(&url);
                                                                        return;
                                                                    }

                                                                    // 触发下载 - 使用click方法（通过js_sys调用）
                                                                    if let Ok(click_fn) = js_sys::Reflect::get(link_element.as_ref(), &"click".into()) {
                                                                        if let Some(click_method) = click_fn.dyn_ref::<js_sys::Function>() {
                                                                            let _ = click_method.call0(link_element.as_ref());
                                                                        }
                                                                    }

                                                                    // 清理
                                                                    let _ = body.remove_child(&link);
                                                                    let _ = Url::revoke_object_url(&url);

                                                                    AppState::show_success(toasts, format!("助记词已下载为 {}", filename));
                                                                } else {
                                                                    let _ = Url::revoke_object_url(&url);
                                                                    AppState::show_error(toasts, "下载失败".to_string());
                                                                }
                                                            } else {
                                                                AppState::show_error(toasts, "下载失败".to_string());
                                                            }
                                                        } else {
                                                            AppState::show_error(toasts, "创建下载链接失败".to_string());
                                                        }
                                                    } else {
                                                        AppState::show_error(toasts, "创建文件失败".to_string());
                                                    }
                                                }
                                            });
                                        }
                                    },
                                    "📥 下载备份文件"
                                }
                            }
                        }
                    }

                    // 确认复选框
                    if is_revealed() {
                        div {
                            class: "mb-6",
                            label {
                                class: "flex items-center gap-3 cursor-pointer",
                                input {
                                    r#type: "checkbox",
                                    checked: is_confirmed(),
                                    onclick: {
                                        let mut is_confirmed = is_confirmed;
                                        move |_e: dioxus::events::MouseEvent| {
                                            // 切换checkbox状态
                                            is_confirmed.set(!is_confirmed());
                                        }
                                    },
                                    class: "w-5 h-5 rounded",
                                    style: format!("accent-color: {};", Colors::TECH_PRIMARY),
                                }
                                span {
                                    class: "text-sm",
                                    style: format!("color: {};", Colors::TEXT_SECONDARY),
                                    "我已确认已安全备份助记词"
                                }
                            }
                        }
                    }

                    // 操作按钮
                    div {
                        class: "flex gap-4",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            disabled: !is_revealed() || !is_confirmed(),
                            onclick: move |_| {
                                // 导航到验证页面，传递助记词
                                navigator.push(Route::MnemonicVerify { phrase: phrase.clone() });
                            },
                            "下一步：验证助记词"
                        }
                        if is_revealed() {
                            Button {
                                variant: ButtonVariant::Secondary,
                                size: ButtonSize::Large,
                                onclick: move |_| {
                                    navigator.go_back();
                                },
                                "返回"
                            }
                        }
                    }
                }
            }
        }
    }
}
