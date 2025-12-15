use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum AppError {
    #[error("Wallet error: {0}")]
    Wallet(#[from] WalletError),

    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),

    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    #[allow(dead_code)]
    Internal(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[derive(Error, Debug, Clone, PartialEq)]
#[allow(dead_code)] // 错误类型定义，用于未来功能
pub enum WalletError {
    #[error("Wallet not initialized")]
    #[allow(dead_code)]
    NotInitialized,
    #[error("Wallet locked")]
    #[allow(dead_code)]
    Locked,
    #[error("Account not found")]
    #[allow(dead_code)]
    AccountNotFound,
    #[error("Invalid mnemonic")]
    #[allow(dead_code)]
    InvalidMnemonic,
}

#[derive(Error, Debug, Clone, PartialEq)]
#[allow(dead_code)] // 错误类型定义，用于未来功能
pub enum TransactionError {
    #[error("Insufficient funds")]
    #[allow(dead_code)]
    InsufficientFunds,
    #[error("Invalid address")]
    #[allow(dead_code)]
    InvalidAddress,
    #[error("Gas estimation failed")]
    #[allow(dead_code)]
    GasEstimationFailed,
}

#[derive(Error, Debug, Clone, PartialEq)]
#[allow(dead_code)] // 错误类型定义，用于未来功能
pub enum SecurityError {
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    #[allow(dead_code)]
    DecryptionFailed,
    #[error("Invalid password")]
    #[allow(dead_code)]
    InvalidPassword,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Response error: {0}")]
    ResponseError(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Timeout")]
    Timeout,
}

#[derive(Error, Debug, Clone, PartialEq)]
#[allow(dead_code)] // 错误类型定义，用于未来功能
pub enum StorageError {
    #[error("Save failed: {0}")]
    #[allow(dead_code)]
    SaveFailed(String),
    #[error("Load failed: {0}")]
    #[allow(dead_code)]
    LoadFailed(String),
    #[error("Delete failed: {0}")]
    #[allow(dead_code)]
    DeleteFailed(String),
}

#[derive(Error, Debug, Clone, PartialEq)]
#[allow(dead_code)] // 错误类型定义，用于未来功能
pub enum NetworkError {
    #[error("Connection failed")]
    #[allow(dead_code)]
    ConnectionFailed,
    #[error("Timeout")]
    #[allow(dead_code)]
    Timeout,
}

// Implement conversion from anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Unknown(err.to_string())
    }
}
