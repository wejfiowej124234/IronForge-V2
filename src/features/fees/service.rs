use crate::services::fee::{FeeService as CoreFeeService, PlatformFeeQuote};
use crate::shared::error::AppError;
use crate::shared::state::AppState;

/// 费用服务（包装层）
///
/// 注意：此结构体当前未使用，但保留用于未来扩展
#[allow(dead_code)]
#[derive(Clone)]
pub struct FeeService {
    inner: CoreFeeService,
}

impl FeeService {
    /// 创建费用服务实例
    ///
    /// 注意：此方法当前未使用，但保留用于未来扩展
    #[allow(dead_code)]
    pub fn new(app_state: AppState) -> Self {
        Self {
            inner: CoreFeeService::new(app_state),
        }
    }

    /// 计算平台服务费（企业级实现）
    ///
    /// 注意：这是平台服务费（钱包服务商收取的服务费用），与Gas费用（区块链网络费用）完全独立！
    ///
    /// Gas费用由区块链网络收取，用于执行交易（gas_used * gas_price）
    /// 平台服务费由钱包服务商收取，用于提供钱包服务
    ///
    /// 这两个费用是完全独立的，不能混淆！
    ///
    /// 注意：此方法当前未使用，但保留用于未来扩展
    #[allow(dead_code)]
    pub async fn calculate(
        &self,
        chain: &str,
        operation: &str,
        amount: f64,
    ) -> Result<PlatformFeeQuote, AppError> {
        self.inner.calculate(chain, operation, amount).await
    }
}
