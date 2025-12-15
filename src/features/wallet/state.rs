use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountType {
    Derived,  // From HD Seed
    Imported, // From Private Key
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub address: String,
    pub chain: String, // e.g., "ethereum", "bitcoin"
    /// 公钥（✅ 非托管钱包可以存储公开信息）
    ///
    /// # 为什么可以存储公钥？
    /// - 公钥是公开信息，不涉及资产控制
    /// - 后端需要公钥来查询余额、交易历史
    /// - 公钥用于验证地址的合法性
    /// - 与用户绑定，方便多设备同步
    ///
    /// # 安全说明
    /// - ❌ 私钥控制资产，永不存储
    /// - ✅ 公钥仅用于查询，可以安全存储
    #[serde(default)]
    pub public_key: String,
    pub derivation_path: Option<String>, // None for Imported accounts
    pub account_type: AccountType,
    #[serde(default)]
    pub balance: String,
}

impl Account {
    /// Returns a user-friendly label for the chain
    pub fn chain_label(&self) -> &str {
        match self.chain.to_lowercase().as_str() {
            "ethereum" | "eth" => "Ethereum",
            "bitcoin" | "btc" => "Bitcoin",
            "solana" | "sol" => "Solana",
            "ton" => "TON",
            "polygon" | "matic" => "Polygon",
            "bsc" | "binance" => "BNB Chain",
            _ => "Unknown",
        }
    }

    /// Returns a shortened address for display (e.g., "0x45d5...6ba")
    pub fn short_address(&self) -> String {
        if self.address.len() <= 10 {
            return self.address.clone();
        }
        format!(
            "{}...{}",
            &self.address[..6],
            &self.address[self.address.len() - 4..]
        )
    }
}

/// 单个钱包的数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Wallet {
    pub id: String,             // 钱包唯一ID（UUID）
    pub name: String,           // 钱包名称
    pub is_locked: bool,        // 是否锁定（交易签名需要密码解锁）
    pub created_at: String,     // 创建时间
    pub accounts: Vec<Account>, // 账户列表
    pub selected_account_index: Option<usize>,
}

impl Wallet {
    pub fn new(id: String, name: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            name,
            is_locked: true, // 默认锁定
            created_at: now,
            accounts: Vec::new(),
            selected_account_index: None,
        }
    }
}

/// 钱包状态（多钱包设计）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WalletState {
    #[serde(default = "default_version")]
    #[allow(dead_code)] // 用于版本控制
    pub version: u32,
    pub wallets: Vec<Wallet>,               // 钱包列表
    pub selected_wallet_id: Option<String>, // 当前选中的钱包ID
}

fn default_version() -> u32 {
    3 // 新版本：支持多钱包
}

impl Default for WalletState {
    fn default() -> Self {
        Self {
            version: 3,
            wallets: Vec::new(),
            selected_wallet_id: None,
        }
    }
}

impl WalletState {
    /// 加载钱包状态（从LocalStorage）
    pub async fn load() -> Self {
        // 尝试加载当前版本
        if let Ok(mut stored) = LocalStorage::get::<WalletState>("wallet_state") {
            if stored.version < 3 {
                // 迁移旧版本数据（如果有）
                stored.version = 3;
                let _ = LocalStorage::set("wallet_state", &stored);
            }
            return stored;
        }

        Self::default()
    }

    /// 保存钱包状态（到LocalStorage）
    pub fn save(&self) -> Result<(), gloo_storage::errors::StorageError> {
        LocalStorage::set("wallet_state", self)
    }

    /// 获取当前选中的钱包
    pub fn get_selected_wallet(&self) -> Option<&Wallet> {
        self.selected_wallet_id
            .as_ref()
            .and_then(|id| self.wallets.iter().find(|w| w.id == *id))
    }

    /// 获取当前选中的钱包（可变引用）
    #[allow(dead_code)] // 用于修改钱包状态
    pub fn get_selected_wallet_mut(&mut self) -> Option<&mut Wallet> {
        self.selected_wallet_id
            .as_ref()
            .and_then(|id| self.wallets.iter_mut().find(|w| w.id == *id))
    }

    /// 添加钱包
    pub fn add_wallet(&mut self, wallet: Wallet) {
        self.wallets.push(wallet);
    }

    /// 根据ID获取钱包
    pub fn get_wallet(&self, wallet_id: &str) -> Option<&Wallet> {
        self.wallets.iter().find(|w| w.id == wallet_id)
    }

    /// 根据ID获取钱包（可变引用）
    pub fn get_wallet_mut(&mut self, wallet_id: &str) -> Option<&mut Wallet> {
        self.wallets.iter_mut().find(|w| w.id == wallet_id)
    }

    /// 删除钱包
    pub fn remove_wallet(&mut self, wallet_id: &str) -> bool {
        if let Some(pos) = self.wallets.iter().position(|w| w.id == wallet_id) {
            self.wallets.remove(pos);
            // 如果删除的是选中的钱包，清空选中状态
            if self.selected_wallet_id.as_ref() == Some(&wallet_id.to_string()) {
                self.selected_wallet_id = None;
            }
            true
        } else {
            false
        }
    }

    /// 检查是否有钱包
    pub fn has_wallets(&self) -> bool {
        !self.wallets.is_empty()
    }

    /// 检查是否有已初始化的钱包（兼容旧代码）
    pub fn is_initialized(&self) -> bool {
        self.has_wallets()
    }

    /// 获取钱包名称（兼容旧代码）
    pub fn name(&self) -> String {
        self.get_selected_wallet()
            .map(|w| w.name.clone())
            .unwrap_or_else(|| "My Wallet".to_string())
    }

    /// 获取账户列表（兼容旧代码）
    pub fn accounts(&self) -> Vec<Account> {
        self.get_selected_wallet()
            .map(|w| w.accounts.clone())
            .unwrap_or_default()
    }

    /// 检查是否锁定（兼容旧代码 - 检查当前选中的钱包）
    pub fn is_locked(&self) -> bool {
        self.get_selected_wallet()
            .map(|w| w.is_locked)
            .unwrap_or(true)
    }

    /// 获取选中的账户索引（兼容旧代码）
    #[allow(dead_code)] // 用于账户索引查询
    pub fn selected_account_index(&self) -> Option<usize> {
        self.get_selected_wallet()
            .and_then(|w| w.selected_account_index)
    }
}
