//! # Data Transfer Objects (DTOs)
//!
//! This module contains all data structures used for communication between
//! the frontend and backend via the REST API.
//!
//! ## Module Organization
//!
//! - [`auth`] - Authentication, signup, login, and wallet auth DTOs
//! - [`market`] - Market data, OHLC charts, and price information
//!
//! ## Serialization Format
//!
//! All DTOs use `serde_json` for JSON serialization:
//!
//! - **Field naming**: snake_case (default serde behavior)
//! - **Optional fields**: Omitted when `None` using `#[serde(skip_serializing_if = "Option::is_none")]`
//! - **Enums**: Serialize to lowercase strings using `#[serde(rename_all = "lowercase")]`
//! - **All types**: Implement both `Serialize` and `Deserialize`
//!
//! ## Example JSON Communication
//!
//! ### Request/Response Pair
//!
//! ```text
//! POST /api/auth/login
//! Content-Type: application/json
//!
//! {
//!   "email_or_username": "alice",
//!   "password": "MyPassword123!"
//! }
//! ```
//!
//! ```text
//! HTTP/1.1 200 OK
//! Content-Type: application/json
//!
//! {
//!   "user": {
//!     "id": "1",
//!     "username": "alice",
//!     "email": "alice@example.com",
//!     "created_at": "2024-01-01T00:00:00Z"
//!   },
//!   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
//!   "message": "Login successful"
//! }
//! ```

pub mod auth;
pub mod market;
pub mod messaging;

pub use auth::*;
pub use market::*;
pub use messaging::*;
