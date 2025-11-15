//! # Shared Data Transfer Objects Library
//!
//! This library defines the contract between the frontend (terminal/web) and backend API.
//! All DTOs use JSON serialization via `serde` for API communication.
//!
//! ## Structure
//!
//! - **[`dto`]**: Data Transfer Objects for API communication
//!   - **[`dto::auth`]**: Authentication and user management DTOs
//!   - **[`dto::market`]**: Market data and charting DTOs
//! - **[`utils`]**: Shared utility functions
//!   - **[`utils::format_address`]**: Format wallet addresses for display
//!   - **[`utils::truncate_address`]**: Truncate addresses with ellipsis
//!
//! ## Wire Format
//!
//! All DTOs serialize to JSON using the default `serde` behavior:
//! - Field names use **snake_case** in Rust, which maps to **snake_case** in JSON by default
//! - Optional fields are omitted from JSON when `None` (using `#[serde(skip_serializing_if = "Option::is_none")]`)
//! - All structs implement both `Serialize` and `Deserialize` for bidirectional communication
//!
//! ## Usage in Backend
//!
//! ```rust,no_run
//! use shared::dto::auth::{LoginRequest, AuthResponse};
//! use shared::utils::format_address;
//! use axum::Json;
//!
//! async fn login(Json(request): Json<LoginRequest>) -> Json<AuthResponse> {
//!     // Request is automatically deserialized from JSON
//!     // Response is automatically serialized to JSON
//!     # todo!()
//! }
//!
//! fn display_address(address: &str) -> String {
//!     format_address(address, 4, 4)
//! }
//! ```
//!
//! ## Usage in Frontend
//!
//! ```rust,no_run
//! use shared::dto::auth::{LoginRequest, AuthResponse};
//! use shared::utils::truncate_address;
//!
//! let request = LoginRequest {
//!     email_or_username: "alice".to_string(),
//!     password: "secret".to_string(),
//! };
//!
//! let response: AuthResponse = reqwest::Client::new()
//!     .post("http://localhost:3001/api/auth/login")
//!     .json(&request)
//!     .send()
//!     .await?
//!     .json()
//!     .await?;
//!
//! let display = truncate_address(&response.user.wallet_address.unwrap_or_default());
//! ```

pub mod dto;
pub mod utils;

// Re-export commonly used types for convenience
// Note: Wildcard re-exports are used here since shared is a DTO library
// where all exports are meant to be public API
pub use dto::*;
pub use utils::*;
