use crate::services::gas::{
    GasEstimate, GasEstimateResponse, GasService as CoreGasService, GasSpeed,
};
use crate::shared::error::AppError;
use crate::shared::state::AppState;

#[derive(Clone)]
pub struct GasService {
    inner: CoreGasService,
}

impl GasService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            inner: CoreGasService::new(app_state),
        }
    }

    /// 估算Gas费（单个速度）
    /// 为未来扩展准备的方法
    #[allow(dead_code)] // 为未来扩展准备
    pub async fn estimate(&self, chain: &str, speed: GasSpeed) -> Result<GasEstimate, AppError> {
        self.inner.estimate(chain, speed).await
    }

    pub async fn estimate_all(&self, chain: &str) -> Result<GasEstimateResponse, AppError> {
        self.inner.estimate_all(chain).await
    }
}
