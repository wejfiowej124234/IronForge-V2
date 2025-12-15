//! AmountInput - 金额输入组件
//! 支持MAX按钮和余额检查

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::input::{Input, InputType};
use crate::features::wallet::state::Account;
use dioxus::events::FormEvent;
use dioxus::prelude::*;

/// 金额输入组件
#[component]
pub fn AmountInput(
    /// 金额值
    value: Signal<String>,
    /// 当前账户（用于MAX按钮和余额显示）
    current_account: ReadSignal<Option<Account>>,
    /// Gas费估算（用于计算可用余额）
    gas_estimate: Signal<Option<crate::services::gas::GasEstimate>>,
    /// 标签文本
    #[props(default = "金额".to_string())]
    label: String,
    /// 占位符
    #[props(default = "0.0".to_string())]
    placeholder: String,
    /// 值变化回调
    onchange: Option<EventHandler<FormEvent>>,
    /// 显示余额信息
    #[props(default = true)]
    show_balance: bool,
) -> Element {
    rsx! {
        div {
            class: "mb-6",
            div {
                class: "flex items-end gap-2",
                div {
                    class: "flex-1",
                    Input {
                        input_type: InputType::Number,
                        label: Some(label),
                        placeholder: Some(placeholder),
                        value: Some(value.read().clone()),
                        onchange: {
                            let mut value_sig = value;
                            let onchange_cb = onchange;
                            Some(EventHandler::new(move |e: FormEvent| {
                                value_sig.set(e.value());

                                // 调用外部回调
                                if let Some(ref cb) = onchange_cb {
                                    cb.call(e);
                                }
                            }))
                        },
                    }
                }
                {
                    if let Some(account) = current_account.read().as_ref() {
                        let account_clone = account.clone();
                        rsx! {
                            Button {
                                variant: ButtonVariant::Secondary,
                                size: ButtonSize::Medium,
                                onclick: {
                                    let account = account_clone;
                                    let gas_est = gas_estimate;
                                    let mut value_sig = value;
                                    move |_| {
                                        // 设置最大金额（余额 - Gas费）
                                        let balance: f64 = account.balance.parse().unwrap_or(0.0);
                                        let gas_cost = gas_est.read().as_ref().map(|g| {
                                            g.max_fee_per_gas_gwei * 21000.0 / 1e9
                                        }).unwrap_or(0.0);
                                        let max_amount = (balance - gas_cost).max(0.0);
                                        value_sig.set(format!("{:.6}", max_amount));
                                    }
                                },
                                "MAX"
                            }
                        }
                    } else {
                        rsx! { div {} }
                    }
                }
            }

            // 余额显示
            if show_balance {
                if let Some(account) = current_account.read().as_ref() {
                    div {
                        class: "mt-2 text-sm text-gray-400",
                        "可用余额: {account.balance}"
                    }
                }
            }
        }
    }
}
