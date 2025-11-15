//! # Core Abstractions
//!
//! Core traits and error types for dependency injection and better testability.
//!
//! This module provides foundational abstractions used throughout the terminal application:
//!
//! - **Error Types**: Centralized error handling (see [`error`] module)
//! - **Service Traits**: Dependency injection traits for better testability (see [`service`] module)
//!
//! ## Modules
//!
//! - **[`error`]**: Application error types (`AppError`, `Result<T>`)
//! - **[`service`]**: Service traits for dependency injection (`ApiService`, `WalletService`)
//!
//! ## Error Handling
//!
//! All application errors use the centralized [`AppError`] type:
//!
//! ```rust,no_run
//! use terminal::core::error::{AppError, Result};
//!
//! fn validate_input(input: &str) -> Result<String> {
//!     if input.is_empty() {
//!         return Err(AppError::Validation("Input cannot be empty".to_string()));
//!     }
//!     Ok(input.to_string())
//! }
//! ```
//!
//! ## Dependency Injection
//!
//! Service traits enable dependency injection for testing:
//!
//! ```rust,no_run
//! use terminal::core::service::{ApiService, WalletService};
//!
//! // In production: use real implementations
//! let api: Arc<dyn ApiService> = Arc::new(terminal::services::api::ApiClient::new());
//!
//! // In tests: use mock implementations
//! let api: Arc<dyn ApiService> = Arc::new(MockApiService::new());
//! ```
//!
//! ## Re-exports
//!
//! Common types are re-exported for convenience:
//! - [`AppError`]: Application error type
//! - [`Result<T>`]: Convenience alias for `Result<T, AppError>`
//! - [`ApiService`]: API service trait
//! - [`WalletService`]: Wallet service trait

pub mod error;
pub mod service;

// Re-export commonly used types for convenience
// Note: These may be unused in the current implementation but are part of the public API
// for dependency injection and testing purposes
#[allow(unused_imports)]
pub use error::{AppError, Result};
#[allow(unused_imports)]
pub use service::{ApiService, WalletService as WalletServiceTrait};

