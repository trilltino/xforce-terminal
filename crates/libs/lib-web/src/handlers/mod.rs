//! # HTTP Request Handlers
//!
//! This module contains all Axum HTTP request handlers organized by feature domain.
//!
//! Each handler module follows the **single responsibility principle**, handling
//! all HTTP endpoints for a specific feature area. Handlers delegate business logic
//! to services in the [`crate::services`] module.
//!
//! ## Handler Modules
//!
//! - **[`auth`]**: User authentication endpoints (signup, login)
//!   - `POST /api/auth/signup` - Create new user account
//!   - `POST /api/auth/login` - Authenticate with email/password
//!
//! - **[`wallet_auth`]**: Wallet-based authentication endpoints
//!   - `GET /api/wallet/setup/validate` - Validate wallet setup token
//!   - `POST /api/wallet/setup/complete` - Complete wallet setup with signature
//!   - `POST /api/wallet/login` - Authenticate with wallet signature
//!
//! - **[`market`]**: Market data endpoints (prices, token lists, charts)
//!   - `GET /api/market/prices` - Get token prices
//!   - `GET /api/market/tokens` - Get available tokens
//!   - `GET /api/market/ohlc` - Get OHLC chart data
//!
//! - **[`wallet`]**: Wallet query endpoints
//!   - `GET /api/wallet/balance` - Get SOL balance
//!   - `GET /api/wallet/tokens` - Get SPL token balances
//!
//! - **[`swap`]**: Token swap operation endpoints
//!   - `GET /api/swap/quote` - Get swap quote from Jupiter
//!   - `POST /api/swap/execute` - Get unsigned swap transaction
//!   - `GET /api/swap/history` - Get user's swap history
//!
//! - **[`transaction`]**: Transaction management endpoints
//!   - `POST /api/transaction/submit` - Submit signed transaction
//!   - `GET /api/transaction/history` - Get transaction history
//!
//! - **[`staking`]**: Staking operation endpoints
//!   - `GET /api/staking/info` - Get staking information
//!   - `GET /api/staking/rewards` - Get staking rewards
//!
//! - **[`contracts`]**: Contract plugin endpoints
//!   - `GET /api/contracts` - List registered contracts
//!   - Contract-specific routes (defined by plugins)
//!
//! ## Handler Architecture
//!
//! All handlers follow Axum's extractor pattern:
//!
//! ```rust,ignore
//! async fn handler(
//!     State(db): State<DbPool>,              // Shared state
//!     Extension(claims): Extension<Claims>,  // JWT auth
//!     Json(payload): Json<RequestBody>,      // Request body
//! ) -> Result<Json<Response>, (StatusCode, String)> {
//!     // Handler logic...
//!     Ok(Json(response))
//! }
//! ```
//!
//! ## Authentication
//!
//! Protected endpoints use `Extension<Claims>` to extract JWT claims.
//! The auth middleware validates tokens before handlers execute.
//!
//! Public endpoints (signup, login, health check) don't require auth.
//!
//! ## Error Handling
//!
//! Handlers return `Result<T, E>` where:
//! - `Ok(T)` - Success response (typically `Json<Data>`)
//! - `Err(E)` - Error tuple `(StatusCode, String)` or `(StatusCode, Json<ErrorResponse>)`
//!
//! Example error responses:
//! ```rust,ignore
//! // Simple string error
//! Err((StatusCode::BAD_REQUEST, "Invalid input".to_string()))
//!
//! // Structured JSON error
//! Err((StatusCode::NOT_FOUND, Json(ErrorResponse {
//!     error: "User not found".to_string(),
//!     code: "USER_NOT_FOUND".to_string(),
//! })))
//! ```
//!
//! ## Request/Response Flow
//!
//! ```text
//! Client Request
//!     ↓
//! CORS Middleware (tower-http)
//!     ↓
//! Auth Middleware (JWT validation)
//!     ↓
//! Handler (business logic)
//!     ↓
//! Response (JSON serialization)
//!     ↓
//! Client Response
//! ```
//!
//! ## Testing Handlers
//!
//! Use Axum's testing utilities:
//! ```rust,ignore
//! use axum::http::StatusCode;
//! use axum_test_helper::TestClient;
//!
//! #[tokio::test]
//! async fn test_handler() {
//!     let app = Router::new().route("/test", get(handler));
//!     let client = TestClient::new(app);
//!
//!     let res = client.get("/test").send().await;
//!     assert_eq!(res.status(), StatusCode::OK);
//! }
//! ```
//!
//! ## API Documentation
//!
//! See [`crate`] module docs for complete endpoint listing with curl examples.

pub mod auth;
pub mod friends;
pub mod market;
pub mod wallet;
pub mod transaction;
pub mod staking;
pub mod swap;
pub mod wallet_auth;
pub mod contracts;
pub mod websocket;

// Note: Individual handler functions are not re-exported here to avoid
// ambiguous glob re-exports. Import specific handlers from their modules:
// use backend::handlers::auth::{signup, login};
// use backend::handlers::market::get_prices;
// etc.
