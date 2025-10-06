use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Stellar RPC error: {0}")]
    StellarRpc(String),

    #[error("Account error: {0}")]
    Account(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("XDR encoding error: {0}")]
    XdrEncoding(String),

    #[error("XDR decoding error: {0}")]
    XdrDecoding(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
