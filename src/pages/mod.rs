//! Pages - 页面模块
//! 所有页面组件都在这里

pub mod bridge;
pub mod buy;
pub mod dashboard;
pub mod dashboard_balance;
pub mod dashboard_transactions;
pub mod import_wallet;
pub mod landing;
pub mod login;
pub mod mnemonic_backup;
pub mod mnemonic_verify;
pub mod not_found;
pub mod orders;
pub mod receive;
pub mod register;
pub mod sell;
pub mod send;
pub mod settings;
pub mod swap;
pub mod wallet;
pub mod wallet_created;
pub mod wallet_detail;

// 路由页面导出
pub use bridge::Bridge;
pub use buy::Buy;
pub use dashboard::Dashboard;
pub use import_wallet::ImportWallet;
pub use landing::Landing;
pub use login::Login;
pub use mnemonic_backup::MnemonicBackup;
pub use mnemonic_verify::MnemonicVerify;
pub use not_found::NotFound;
pub use orders::Orders;
pub use receive::Receive;
pub use register::Register;
pub use sell::Sell;
pub use send::Send;
pub use settings::Settings;
pub use swap::Swap;
pub use wallet::CreateWallet;
pub use wallet_created::WalletCreated;
pub use wallet_detail::WalletDetail;
