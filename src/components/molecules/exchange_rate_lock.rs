//! Exchange Rate Lock - 汇率锁定倒计时组件
//! 显示汇率锁定剩余时间（30秒有效期）

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;

/// 汇率锁定倒计时组件
#[component]
pub fn ExchangeRateLockCountdown(
    /// 锁定开始时间（Unix timestamp，秒）
    lock_start_time: u64,
    /// 锁定有效期（秒）
    lock_duration: u64,
    /// 过期回调
    on_expired: Option<EventHandler<()>>,
) -> Element {
    let current_time = use_signal(|| js_sys::Date::now() as u64 / 1000);
    let expired = use_signal(|| false);

    // 每秒更新一次倒计时
    use_effect({
        let current_time_sig = current_time;
        let expired_sig = expired;
        let lock_start = lock_start_time;
        let lock_dur = lock_duration;
        let expired_handler = on_expired.clone();

        move || {
            let _interval_id = gloo_timers::callback::Interval::new(1000, {
                let mut current_time_sig = current_time_sig;
                let mut expired_sig = expired_sig;
                let lock_start = lock_start;
                let lock_dur = lock_dur;
                let expired_handler = expired_handler.clone();

                move || {
                    let now = js_sys::Date::now() as u64 / 1000;
                    current_time_sig.set(now);

                    let elapsed = now.saturating_sub(lock_start);
                    if elapsed >= lock_dur && !*expired_sig.read() {
                        expired_sig.set(true);
                        if let Some(handler) = expired_handler.as_ref() {
                            handler.call(());
                        }
                    }
                }
            });

            // 注意：Dioxus 0.7 的 use_effect 不直接支持清理函数
            // 定时器会在组件卸载时自动停止（因为闭包被丢弃）
        }
    });

    let now = *current_time.read();
    let elapsed = now.saturating_sub(lock_start_time);
    let remaining = lock_duration.saturating_sub(elapsed);

    if *expired.read() || remaining == 0 {
        return rsx! {
            div {
                class: "p-3 rounded-lg",
                style: format!("background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3);"),
                div {
                    class: "flex items-center gap-2 text-sm",
                    span { "⚠️" }
                    span {
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "汇率已过期，请重新获取报价"
                    }
                }
            }
        };
    }

    let minutes = remaining / 60;
    let seconds = remaining % 60;
    let progress = (remaining as f64 / lock_duration as f64) * 100.0;

    // 根据剩余时间显示不同颜色
    let bg_color = if remaining <= 10 {
        "rgba(239, 68, 68, 0.1)"
    } else if remaining <= 20 {
        "rgba(251, 191, 36, 0.1)"
    } else {
        "rgba(34, 197, 94, 0.1)"
    };

    let border_color = if remaining <= 10 {
        "rgba(239, 68, 68, 0.3)"
    } else if remaining <= 20 {
        "rgba(251, 191, 36, 0.3)"
    } else {
        "rgba(34, 197, 94, 0.3)"
    };

    rsx! {
        div {
            class: "p-3 rounded-lg",
            style: format!("background: {}; border: 1px solid {};", bg_color, border_color),
            div {
                class: "flex items-center justify-between mb-2",
                div {
                    class: "flex items-center gap-2 text-sm",
                    span { "🔒" }
                    span {
                        style: format!("color: {};", Colors::TEXT_PRIMARY),
                        "汇率锁定中"
                    }
                }
                div {
                    class: "text-lg font-bold",
                    style: format!("color: {};", Colors::TEXT_PRIMARY),
                    "{minutes:02}:{seconds:02}"
                }
            }
            // 进度条
            div {
                class: "w-full h-1 rounded-full overflow-hidden",
                style: format!("background: {};", Colors::BG_PRIMARY),
                div {
                    class: "h-full transition-all duration-1000",
                    style: format!(
                        "width: {}%; background: {};",
                        progress,
                        if remaining <= 10 {
                            "rgba(239, 68, 68, 0.8)"
                        } else if remaining <= 20 {
                            "rgba(251, 191, 36, 0.8)"
                        } else {
                            "rgba(34, 197, 94, 0.8)"
                        }
                    ),
                }
            }
        }
    }
}
