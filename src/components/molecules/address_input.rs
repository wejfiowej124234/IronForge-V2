//! AddressInput - 地址输入组件
//! 支持多链地址验证和实时错误提示

use crate::components::atoms::input::{Input, InputType};
use crate::shared::validation;
use dioxus::events::FormEvent;
use dioxus::prelude::*;

/// 地址输入组件
#[component]
pub fn AddressInput(
    /// 地址值
    value: Signal<String>,
    /// 选中的链
    selected_chain: Signal<String>,
    /// 错误消息
    error: Signal<Option<String>>,
    /// 标签文本
    #[props(default = "接收地址".to_string())]
    label: String,
    /// 占位符
    #[props(default = "请输入接收地址".to_string())]
    placeholder: String,
    /// 值变化回调
    onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    // 内部地址验证
    let internal_error = use_signal(|| Option::<String>::None);

    rsx! {
        Input {
            input_type: InputType::Text,
            label: Some(label),
            placeholder: Some(placeholder),
            value: Some(value.read().clone()),
            error: {
                let err = error.read();
                let int_err = internal_error.read();
                err.clone().or_else(|| int_err.clone())
            },
            onchange: {
                let mut value_sig = value;
                let chain_sig = selected_chain;
                let mut error_sig = error;
                let mut internal_err = internal_error;
                let onchange_cb = onchange;
                Some(EventHandler::new(move |e: FormEvent| {
                    value_sig.set(e.value());
                    error_sig.set(None);
                    internal_err.set(None);

                    // 实时验证地址
                    let addr_val = value_sig.read().trim().to_string();
                    if !addr_val.is_empty() {
                        let chain_val = chain_sig.read().clone();
                        let validation_result = match chain_val.as_str() {
                            "ethereum" | "eth" => validation::validate_eth_address(&addr_val),
                            "bitcoin" | "btc" => validation::validate_btc_address(&addr_val),
                            "solana" | "sol" => validation::validate_sol_address(&addr_val),
                            "ton" => validation::validate_ton_address(&addr_val),
                            _ => Ok(()),
                        };
                        if let Err(e) = validation_result {
                            internal_err.set(Some(e.to_string()));
                        }
                    }

                    // 调用外部回调
                    if let Some(ref cb) = onchange_cb {
                        cb.call(e);
                    }
                }))
            },
        }
    }
}
