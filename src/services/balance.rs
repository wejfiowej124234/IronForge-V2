use serde::{Deserialize, Serialize};

use crate::shared::api::ApiClient;
use crate::shared::error::AppError;
use crate::shared::request::{CachePolicy, SmartRequestContext};
use crate::shared::state::AppState;

// URL编码辅助函数
fn encode_uri_component(s: &str) -> String {
    // 使用JavaScript的encodeURIComponent进行URL编码
    let encoded = js_sys::Reflect::get(&js_sys::global(), &"encodeURIComponent".into())
        .ok()
        .and_then(|f| {
            js_sys::Function::from(f)
                .call1(&js_sys::global(), &s.into())
                .ok()
        })
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| s.to_string());
    encoded
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance: String,
    pub chain_id: u64,
    pub confirmed: bool,
}

#[derive(Clone, Copy)]
pub struct BalanceService {
    app_state: AppState,
}

impl BalanceService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    fn api(&self) -> ApiClient {
        self.app_state.get_api_client()
    }

    fn context(&self) -> SmartRequestContext {
        SmartRequestContext::new(self.app_state)
    }

    pub async fn get_balance(
        &self,
        address: &str,
        chain_id: u64,
    ) -> Result<BalanceResponse, AppError> {
        let key = format!("balance:{}:{}", chain_id, address.to_lowercase());
        // URL编码地址，因为某些地址（如TON）包含特殊字符（如:）
        let encoded_address = encode_uri_component(address);
        let path = format!(
            "/api/v1/wallets/{}/balance?chain_id={}",
            encoded_address, chain_id
        );
        let ctx = self.context();
        let api = self.api();

        // deserialize 方法已自动提取 data 字段
        let response: BalanceResponse = ctx
            .run(&key, CachePolicy::medium(), move || {
                let api = api.clone();
                let path = path.clone();
                async move { api.get(&path).await }
            })
            .await
            .map_err(AppError::from)?;

        Ok(response)
    }
}
