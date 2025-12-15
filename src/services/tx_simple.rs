use serde::{Deserialize, Serialize};

use crate::shared::api::ApiClient;
use crate::shared::error::AppError;
use crate::shared::request::{CachePolicy, SmartRequestContext};
use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)] // 用于简单交易服务
pub struct SimpleTransactionResp {
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub chain: String,
    pub status: String,
    pub timestamp: String,
    pub platform_fee: Option<String>,
    pub fee_applied: bool,
}

#[derive(Clone, Copy)]
#[allow(dead_code)] // 用于简单交易服务
pub struct SimpleTxService {
    app_state: AppState,
}

impl SimpleTxService {
    #[allow(dead_code)] // 用于简单交易服务
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    #[allow(dead_code)] // 内部使用
    fn api(&self) -> ApiClient {
        self.app_state.get_api_client()
    }

    #[allow(dead_code)] // 内部使用
    fn context(&self) -> SmartRequestContext {
        SmartRequestContext::new(self.app_state)
    }

    /// Fetch recent transactions (authorized)
    /// 获取最近的交易列表
    /// 企业级标准：统一使用 /api/transactions 端点
    #[allow(dead_code)] // 用于简单交易服务
    pub async fn list_recent(&self, limit: usize) -> Result<Vec<SimpleTransactionResp>, AppError> {
        let key = format!("tx:recent:{}", limit);
        let path = format!("/api/v1/transactions?page=1&page_size={}", limit);
        let api = self.api();
        let ctx = self.context();

        // deserialize 方法已自动提取 data 字段
        // 后端返回统一格式: {code, message, data: {transactions: [...], total: ...}}
        #[derive(Debug, Deserialize)]
        struct TransactionListData {
            transactions: Vec<SimpleTransactionResp>,
            #[serde(default)]
            total: usize,
        }

        let response: Option<TransactionListData> = ctx
            .run(&key, CachePolicy::short(), move || {
                let api = api.clone();
                let path = path.clone();
                async move { api.get(&path).await }
            })
            .await?;

        let mut list = response.map(|d| d.transactions).unwrap_or_default();

        if list.len() > limit {
            list.truncate(limit);
        }
        Ok(list)
    }
}
