use crate::features::fees::service::FeeService;
use crate::services::fee::PlatformFeeQuote;
use crate::shared::error::AppError;
use crate::shared::state::AppState;

/// 计算平台手续费
/// 为未来扩展准备的函数
#[allow(dead_code)] // 为未来扩展准备
pub async fn calculate_platform_fee(
    app_state: AppState,
    chain: &str,
    operation: &str,
    amount: f64,
) -> Result<PlatformFeeQuote, AppError> {
    let service = FeeService::new(app_state);
    service.calculate(chain, operation, amount).await
}
