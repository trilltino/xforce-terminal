//! # Services Layer
//!
//! This module contains business logic services that orchestrate operations
//! across the data layer (database) and integration layer (Solana, external APIs).
//!
//! ## Architecture
//!
//! Services follow the Service Layer pattern, providing a clean separation
//! between HTTP handlers (presentation layer) and business logic:
//!
//! ```text
//! Handlers (HTTP) → Services (Business Logic) → Repository/Database/External APIs
//! ```
//!
//! ## Module Organization
//!
//! - [`market`] - Market data services (prices, token lists)
//! - [`swap`] - Token swap services (quotes, execution)
//! - [`wallet`] - Wallet operation services (balances, token accounts)
//! - [`transaction`] - Transaction services (history, submission)
//! - [`staking`] - Staking services (staking info, positions)
//!
//! ## Service Pattern
//!
//! Services are structs that hold dependencies (like `Arc<SolanaState>`, `DbPool`)
//! and provide async methods for business operations:
//!
//! ```rust,no_run
//! use backend::services::market::MarketService;
//! use backend::solana::SolanaState;
//! use std::sync::Arc;
//!
//! let service = MarketService::new(solana);
//! let prices = service.get_prices(&["SOL", "USDC"]).await?;
//! ```
//!
//! ## Error Handling
//!
//! All services return `Result<T, AppError>` where `AppError` is the centralized
//! error type from `backend::error`. Services should convert lower-level errors
//! (from Solana, database, etc.) into appropriate `AppError` variants.
//!
//! ## Testing
//!
//! Services can be tested independently by providing mock dependencies:
//!
//! ```rust,ignore
//! #[tokio::test]
//! async fn test_market_service() {
//!     let mock_solana = create_mock_solana_state();
//!     let service = MarketService::new(mock_solana);
//!     let prices = service.get_prices(&["SOL"]).await.unwrap();
//!     assert_eq!(prices.len(), 1);
//! }
//! ```

pub mod market;
pub mod swap;
pub mod wallet;
pub mod transaction;
pub mod staking;

// Re-export services for convenience
pub use market::MarketService;
pub use swap::SwapService;
pub use wallet::WalletService;
pub use transaction::TransactionService;
pub use staking::StakingService;

