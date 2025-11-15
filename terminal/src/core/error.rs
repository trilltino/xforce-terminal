//! # Common Error Types
//!
//! Consolidated error handling for the terminal application.
//!
//! This module provides a centralized error type [`AppError`] that covers all error
//! scenarios in the terminal application.
//! 
//! ## Error Categories
//! Errors are categorized by their source:
//!
//! - **Api**: Backend API communication errors (network, HTTP, JSON parsing)
//! - **Wallet**: Solana wallet operations (connection, signing, balance queries)
//! - **State**: Application state management errors (lock failures, invalid state)
//! - **Validation**: Input validation errors (invalid format, missing fields)
//!
//! ## Usage Pattern
//!
//! ```rust,no_run
//! use terminal::core::error::AppError;
//!
//! fn validate_amount(amount: f64) -> Result<f64, AppError> {
//!     if amount <= 0.0 {
//!         return Err(AppError::Validation("Amount must be positive".to_string()));
//!     }
//!     Ok(amount)
//! }
//! ```
//!
//! ## Error Conversion
//!
//! Common error types automatically convert to `AppError`:
//!
//! - `String` → `AppError::Api`
//! - `WalletError` → `AppError::Wallet`
//!
//! ## Related Types
//!
//! - [`crate::services::wallet::WalletError`]: Wallet-specific errors

use thiserror::Error;

/// Application-wide error type covering all error scenarios in the terminal.
///
/// Each variant includes a descriptive `String` message for context. The `#[error]`
/// attribute from `thiserror` provides automatic `Display` and `Error` implementations.
///
/// # Error Variants
///
/// - **Api**: Backend API communication failures
///   - Network errors (connection refused, timeout)
///   - HTTP errors (4xx, 5xx status codes)
///   - JSON parsing errors
///   - Authentication failures
///
/// - **Wallet**: Solana wallet operation failures
///   - Connection errors (keypair load, wallet not found)
///   - Signing errors (transaction signing failures)
///   - Balance query errors (RPC failures)
///   - Wallet state errors (not connected, invalid state)
///
/// - **State**: Application state management failures
///   - Lock contention (rare, indicates design issue)
///   - Invalid state transitions
///   - State corruption (should never happen)
///
/// - **Validation**: Input validation failures
///   - Invalid format (amount, address, etc.)
///   - Missing required fields
///   - Out of range values
///
/// # Example
///
/// ```rust
/// use terminal::core::error::AppError;
///
/// let api_err = AppError::Api("Connection timeout".to_string());
/// let wallet_err = AppError::Wallet("Keypair file not found".to_string());
/// let validation_err = AppError::Validation("Amount must be positive".to_string());
///
/// assert_eq!(api_err.to_string(), "API error: Connection timeout");
/// assert_eq!(wallet_err.to_string(), "Wallet error: Keypair file not found");
/// assert_eq!(validation_err.to_string(), "Validation error: Amount must be positive");
/// ```
/// Application-wide error type covering all error scenarios in the terminal.
///
/// Note: This type is exported for public API use and dependency injection.
/// It may appear unused in internal code but is part of the public interface.
#[derive(Debug, Error)]
#[allow(dead_code)] // Exported for public API and future use
pub enum AppError {
    /// Backend API communication error.
    ///
    /// Used for errors during HTTP requests to the backend:
    /// - Network failures (connection refused, timeout, DNS errors)
    /// - HTTP errors (4xx client errors, 5xx server errors)
    /// - JSON parsing errors (malformed responses)
    /// - Authentication failures (invalid JWT, expired token)
    ///
    /// # Example
    ///
    /// ```rust
    /// use terminal::core::error::AppError;
    ///
    /// let err = AppError::Api("Connection timeout: backend not responding".to_string());
    /// ```
    #[error("API error: {0}")]
    Api(String),

    /// Solana wallet operation error.
    ///
    /// Used for errors during wallet operations:
    /// - Keypair loading failures (file not found, invalid format)
    /// - Transaction signing failures (invalid transaction, signing error)
    /// - Balance query failures (RPC errors, account not found)
    /// - Wallet connection failures (not connected, disconnected)
    ///
    /// # Example
    ///
    /// ```rust
    /// use terminal::core::error::AppError;
    ///
    /// let err = AppError::Wallet("Keypair file not found: ~/.config/solana/id.json".to_string());
    /// ```
    #[error("Wallet error: {0}")]
    Wallet(String),

    /// Application state management error.
    ///
    /// Used for errors related to state management:
    /// - Lock contention (rare, indicates potential deadlock risk)
    /// - Invalid state transitions (e.g., navigating to screen requiring auth without token)
    /// - State corruption (should never happen in normal operation)
    ///
    /// # Example
    ///
    /// ```rust
    /// use terminal::core::error::AppError;
    ///
    /// let err = AppError::State("Failed to acquire write lock: lock contention".to_string());
    /// ```
    #[error("State error: {0}")]
    State(String),

    /// Input validation error.
    ///
    /// Used for user input validation failures:
    /// - Invalid format (amount must be numeric, address must be base58)
    /// - Missing required fields (username, password, amount)
    /// - Out of range values (amount too large, negative amount)
    /// - Business rule violations (insufficient balance, invalid swap pair)
    ///
    /// # Example
    ///
    /// ```rust
    /// use terminal::core::error::AppError;
    ///
    /// let err = AppError::Validation("Amount must be greater than 0".to_string());
    /// ```
    #[error("Validation error: {0}")]
    Validation(String),
}

// Convenience type alias for Result<T, AppError>
/// Convenience type alias for `Result<T, AppError>`.
///
/// Use this throughout the terminal crate for consistent error handling:
///
/// ```rust
/// use terminal::core::error::Result;
///
/// fn operation() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
///
/// Note: This type is exported for public API use. It may appear unused
/// in internal code but is part of the public interface.
#[allow(dead_code)] // Exported for public API and future use
pub type Result<T> = std::result::Result<T, AppError>;

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Api(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Api(msg.to_string())
    }
}

impl From<crate::services::wallet::WalletError> for AppError {
    fn from(err: crate::services::wallet::WalletError) -> Self {
        AppError::Wallet(err.to_string())
    }
}

