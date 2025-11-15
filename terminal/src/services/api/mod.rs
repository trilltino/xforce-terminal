//! # Backend API Client Module
//!
//! HTTP client for communicating with the Axum backend API server.
//! Handles authentication, market data, wallet queries, and swap execution.
//!
//! ## Module Structure
//!
//! ```text
//! api/
//! ├── mod.rs      - Module exports and documentation
//! ├── client.rs   - ApiClient struct and common functionality
//! ├── auth.rs     - Authentication endpoints (login, signup)
//! ├── market.rs   - Market data endpoints (prices, token list)
//! ├── wallet.rs   - Wallet query endpoints (balance, tokens, transactions)
//! └── swap.rs     - Swap endpoints (quote, execute, submit, history)
//! ```

pub mod auth;
pub mod client;
pub mod friends;
pub mod market;
pub mod swap;
pub mod wallet;
pub mod websocket;

// Re-export types for backward compatibility
pub use auth::*;
pub use client::ApiClient;
// pub use friends::*; // Unused for now
pub use market::*;
pub use swap::*;
pub use wallet::*;

