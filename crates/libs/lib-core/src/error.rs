//! # Centralized Error Handling
//!
//! This module defines the application-wide error type [`AppError`] used consistently
//! across all backend modules. It follows the `thiserror` pattern for ergonomic error handling.
//!
//! ## Design Philosophy
//!
//! - **Single Error Type**: All modules use `AppError` for consistency
//! - **Descriptive Messages**: Each variant includes a context string
//! - **HTTP Mapping**: Errors map naturally to HTTP status codes
//! - **Type Safety**: Compiler ensures all errors are handled
//!
//! ## Error Categories
//!
//! Errors are categorized by their source/nature:
//!
//! 1. **Client Errors** (4xx) - User/input issues
//!    - [`InvalidInput`](AppError::InvalidInput) → 400 Bad Request
//!    - [`NotFound`](AppError::NotFound) → 404 Not Found
//!
//! 2. **Server Errors** (5xx) - Internal/system issues
//!    - [`Config`](AppError::Config) → 500 Internal Server Error
//!    - [`Rpc`](AppError::Rpc) → 502 Bad Gateway (external service)
//!    - [`Internal`](AppError::Internal) → 500 Internal Server Error
//!
//! 3. **Domain Errors** - Business logic issues
//!    - [`Account`](AppError::Account) → Context-dependent
//!    - [`Transaction`](AppError::Transaction) → Context-dependent
//!    - [`Encoding`](AppError::Encoding) / [`Decoding`](AppError::Decoding) → 500
//!
//! ## Usage Example
//!
//! ```rust
//! use lib_core::error::{AppError, Result};
//!
//! fn parse_address(addr: &str) -> Result<String> {
//!     if addr.len() < 32 {
//!         return Err(AppError::InvalidInput(
//!             "Address must be at least 32 characters".to_string()
//!         ));
//!     }
//!     Ok(addr.to_string())
//! }
//! ```
//!
//! ## Error Conversion
//!
//! The error module provides conversion traits and implementations for common error types:
//! - `From<anyhow::Error>` - Convert anyhow errors to AppError
//! - `From<sqlx::Error>` - Convert database errors to AppError
//! - `From<serde_json::Error>` - Convert JSON errors to AppError

use thiserror::Error;
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

/// Convenience type alias for `Result<T, AppError>`.
///
/// Use this throughout the codebase for consistent error handling:
/// ```rust
/// use lib_core::error::Result;
///
/// fn operation() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, AppError>;

/// Application-wide error type covering all error scenarios.
///
/// Each variant includes a descriptive `String` for context. The `#[error]` attribute
/// from `thiserror` provides automatic `Display` implementation.
#[derive(Debug, Error)]
pub enum AppError {
    /// Configuration error during startup or environment loading.
    ///
    /// **HTTP Status**: 500 Internal Server Error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Solana RPC client error (network, rate limit, node issues).
    ///
    /// **HTTP Status**: 502 Bad Gateway (external service failure)
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Solana account error (not found, invalid state, insufficient funds).
    ///
    /// **HTTP Status**: 404 Not Found or 400 Bad Request (context-dependent)
    #[error("Account error: {0}")]
    Account(String),

    /// Transaction error (building, signing, simulation, submission).
    ///
    /// **HTTP Status**: 400 Bad Request or 500 Internal Server Error
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Data encoding error (base58, base64, bincode serialization).
    ///
    /// **HTTP Status**: 500 Internal Server Error
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Data decoding error (base58, base64, bincode deserialization).
    ///
    /// **HTTP Status**: 400 Bad Request (if user-provided) or 500 (if internal)
    #[error("Decoding error: {0}")]
    Decoding(String),

    /// Invalid user input validation error.
    ///
    /// **HTTP Status**: 400 Bad Request
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Internal server error (unexpected failures).
    ///
    /// **HTTP Status**: 500 Internal Server Error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Requested resource not found.
    ///
    /// **HTTP Status**: 404 Not Found
    #[error("Not found: {0}")]
    NotFound(String),
}

impl AppError {
    /// Get the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Account(_) => StatusCode::NOT_FOUND,
            AppError::Transaction(_) => StatusCode::BAD_REQUEST,
            AppError::Rpc(_) => StatusCode::BAD_GATEWAY,
            AppError::Config(_) | AppError::Internal(_) | AppError::Encoding(_) | AppError::Decoding(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    /// Get a user-friendly error message.
    ///
    /// For internal errors, returns a generic message to avoid exposing implementation details.
    pub fn user_message(&self) -> String {
        match self {
            AppError::InvalidInput(msg) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::Account(msg) => msg.clone(),
            AppError::Transaction(msg) => msg.clone(),
            AppError::Rpc(_) => "Service temporarily unavailable".to_string(),
            AppError::Config(_) | AppError::Internal(_) | AppError::Encoding(_) | AppError::Decoding(_) => {
                "An internal error occurred".to_string()
            }
        }
    }
}

/// Implement Axum's `IntoResponse` for automatic error handling.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.user_message();
        
        // Log error details (full error message for server logs)
        match status {
            StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => {
                tracing::debug!("Client error: {}", self);
            }
            StatusCode::BAD_GATEWAY | StatusCode::INTERNAL_SERVER_ERROR => {
                tracing::error!("Server error: {}", self);
            }
            _ => {
                tracing::warn!("Unexpected error: {}", self);
            }
        }

        // Extract error variant name for error code
        let error_code = match self {
            AppError::Config(_) => "Config",
            AppError::Rpc(_) => "Rpc",
            AppError::Account(_) => "Account",
            AppError::Transaction(_) => "Transaction",
            AppError::Encoding(_) => "Encoding",
            AppError::Decoding(_) => "Decoding",
            AppError::InvalidInput(_) => "InvalidInput",
            AppError::Internal(_) => "Internal",
            AppError::NotFound(_) => "NotFound",
        };
        
        let body = Json(json!({
            "error": message,
            "code": error_code,
        }));

        (status, body).into_response()
    }
}

/// Convert `anyhow::Error` to `AppError`.
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

/// Convert `sqlx::Error` to `AppError`.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Database record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                AppError::Internal(format!("Database error: {}", db_err.message()))
            }
            _ => AppError::Internal(format!("Database error: {}", err)),
        }
    }
}

/// Convert `serde_json::Error` to `AppError`.
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Decoding(format!("JSON error: {}", err))
    }
}
