//! QR Code Display - 二维码显示组件
//! 显示地址的二维码，支持复制功能

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::shared::design_tokens::Colors;
use crate::shared::security;
use dioxus::prelude::*;
use qrcode::render::svg;
use qrcode::QrCode;
use wasm_bindgen_futures;

/// 生成二维码SVG
fn generate_qr_code(data: &str) -> String {
    match QrCode::new(data) {
        Ok(qr) => {
            let svg = qr
                .render::<svg::Color>()
                .min_dimensions(256, 256)
                .max_dimensions(256, 256)
                .build();
            svg
        }
        Err(_) => {
            format!(
                r#"<svg width="256" height="256" xmlns="http://www.w3.org/2000/svg">
                    <rect width="256" height="256" fill="white"/>
                    <text x="128" y="128" text-anchor="middle" font-size="12" fill="black">QR Code Error</text>
                </svg>"#
            )
        }
    }
}

/// 复制到剪贴板
async fn copy_to_clipboard(text: &str) -> Result<(), String> {
    use web_sys::window;

    let window = window().ok_or("No window")?;
    let navigator = window.navigator();
    let clipboard = navigator.clipboard();

    wasm_bindgen_futures::JsFuture::from(clipboard.write_text(text))
        .await
        .map_err(|_| "Failed to copy to clipboard".to_string())?;

    Ok(())
}

/// 二维码显示组件
#[component]
pub fn QrCodeDisplay(address: String, show_copy_button: Option<bool>) -> Element {
    let copy_success = use_signal(|| false);
    let show_copy = show_copy_button.unwrap_or(true);

    // 安全验证和清理地址
    let sanitized_address = security::sanitize_qr_data(&address);

    // 验证地址格式
    if !security::validate_address(&sanitized_address, None) {
        return rsx! {
            div {
                class: "p-4 rounded-lg bg-red-500/10 border border-red-500/20",
                "Invalid address format"
            }
        };
    }

    // 生成二维码SVG（使用清理后的地址）
    let address_for_qr = sanitized_address.clone();
    let qr_code_svg = use_memo(move || generate_qr_code(&address_for_qr));

    // 复制地址到剪贴板（使用清理后的地址）
    let address_for_copy = sanitized_address.clone();
    let handle_copy = {
        let address_clone = address_for_copy.clone();
        let copy_success = copy_success;

        move |_| {
            let addr = address_clone.clone();
            let mut success = copy_success;
            spawn(async move {
                if copy_to_clipboard(&addr).await.is_ok() {
                    success.set(true);
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                    success.set(false);
                }
            });
        }
    };

    rsx! {
        div {
            class: "flex flex-col items-center gap-6",
            // 二维码显示 - 增强视觉
            div {
                class: "p-6 rounded-2xl shadow-xl",
                style: format!("background: white; border: 3px solid {}; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);", Colors::TECH_PRIMARY),
                div {
                    class: "w-64 h-64",
                    dangerous_inner_html: qr_code_svg.read().clone(),
                }
            }

            // 地址显示和复制按钮
            div {
                class: "w-full space-y-3",
                // 地址显示区域
                div {
                    class: "space-y-2",
                    // 地址标签
                    div {
                        class: "flex items-center gap-2 text-xs font-semibold uppercase tracking-wide",
                        style: format!("color: {};", Colors::TEXT_SECONDARY),
                        span { "🔑" }
                        span { "钱包地址" }
                    }
                    // 地址内容
                    div {
                        class: "p-4 rounded-xl font-mono text-sm break-all border-2 transition-all duration-200 hover:border-indigo-400",
                        style: format!("background: {}; color: {}; border-color: {}; line-height: 1.8;", 
                            "rgba(99, 102, 241, 0.05)", Colors::TEXT_PRIMARY, Colors::BORDER_PRIMARY),
                        {security::escape_for_display(&sanitized_address)}
                    }
                }

                if show_copy {
                    button {
                        class: "w-full py-4 px-6 rounded-xl font-semibold text-base transition-all duration-300 transform hover:scale-[1.02] active:scale-[0.98]",
                        style: if *copy_success.read() {
                            "background: linear-gradient(135deg, #10b981 0%, #059669 100%); color: white; box-shadow: 0 4px 12px rgba(16, 185, 129, 0.4);"
                        } else {
                            format!("background: linear-gradient(135deg, {} 0%, #4f46e5 100%); color: white; box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);", Colors::TECH_PRIMARY)
                        },
                        onclick: handle_copy,
                        if *copy_success.read() {
                            "✅ 已复制到剪贴板"
                        } else {
                            "📋 复制地址"
                        }
                    }
                }
            }
        }
    }
}
