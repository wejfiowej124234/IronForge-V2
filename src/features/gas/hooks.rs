use crate::features::gas::service::GasService;
use crate::services::gas::GasEstimateResponse;
use crate::shared::error::AppError;
use crate::shared::state::AppState;
use dioxus::prelude::*;

/// Hook to get gas estimates for a chain
/// Dioxus 0.6 语法兼容
/// 获取Gas费估算（响应式）
///
/// 注意：此函数当前未使用，但保留用于未来扩展
#[allow(dead_code)]
pub fn use_gas_estimate(chain: &str) -> Signal<Option<Result<GasEstimateResponse, AppError>>> {
    let app_state = use_context::<AppState>();
    let gas_data = use_signal(|| None::<Result<GasEstimateResponse, AppError>>);

    {
        let chain = chain.to_string();
        use_effect(move || {
            let chain_clone = chain.clone();
            let mut gas_data_clone = gas_data;
            let app_state_clone = app_state;
            spawn(async move {
                let service = GasService::new(app_state_clone);
                let result = service.estimate_all(&chain_clone).await;
                gas_data_clone.set(Some(result));
            });
        });
    }

    gas_data
}
