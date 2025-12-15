use anyhow::Result;
use async_trait::async_trait;

/// 交易结构体
/// 为未来扩展准备的通用交易格式
#[allow(dead_code)] // 为未来扩展准备
#[derive(Debug, Clone)]
pub struct Transaction {
    pub to: String,
    pub value: String, // Wei/Satoshi as string to avoid overflow
    pub data: Option<Vec<u8>>,
    pub nonce: Option<u64>,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<String>,
}

/// 交易回执结构体
/// 为未来扩展准备的通用交易回执格式
#[allow(dead_code)] // 为未来扩展准备
#[derive(Debug, Clone)]
pub struct TransactionReceipt {
    pub hash: String,
    pub status: u64, // 1 success, 0 fail
    pub block_number: u64,
}

/// 链适配器trait
/// 为未来扩展准备的统一区块链接口
#[allow(dead_code)] // 为未来扩展准备
#[async_trait(?Send)]
pub trait ChainAdapter {
    // Basic info
    fn chain_name(&self) -> &'static str;
    fn chain_id(&self) -> u64;

    // Balance
    async fn get_balance(&self, address: &str) -> Result<String>;

    // History
    async fn get_transactions(
        &self,
        address: &str,
        limit: usize,
    ) -> Result<Vec<TransactionReceipt>>;

    // Transaction
    async fn estimate_gas(&self, tx: &Transaction) -> Result<u64>;
    async fn broadcast_transaction(&self, signed_tx: &[u8]) -> Result<String>; // Returns tx hash
}
