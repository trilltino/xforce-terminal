//! # Services Module
//!
//! External service integrations for the Solana DeFi Trading Terminal.
//! This module provides clients and utilities for communicating with external systems.
//!
//! ## Module Overview
//!
//! ```text
//! services/
//! ├── api.rs       - Backend HTTP API client
//! │                  (authentication, market data, swaps)
//! └── wallet.rs    - Solana wallet service
//!                    (keypair management, transaction signing)
//! ```
//!
//! ## Service Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                     Terminal TUI                        │
//! │                                                         │
//! │  ┌──────────────────┐       ┌──────────────────┐       │
//! │  │  ApiClient       │       │  WalletService   │       │
//! │  │  (api.rs)        │       │  (wallet.rs)     │       │
//! │  └────────┬─────────┘       └────────┬─────────┘       │
//! │           │                          │                 │
//! └───────────┼──────────────────────────┼─────────────────┘
//!             │                          │
//!             │ HTTP/JSON                │ Solana RPC
//!             ▼                          ▼
//! ┌─────────────────────┐    ┌─────────────────────────────┐
//! │  Backend API Server │    │     Solana Network          │
//! │  (Axum + Postgres)  │    │     (Devnet/Mainnet)        │
//! │                     │    │                             │
//! │  /api/auth/*        │    │  - Get balance              │
//! │  /api/market/*      │    │  - Send transactions        │
//! │  /api/swap/*        │    │  - Query accounts           │
//! │  /api/wallet/*      │    │  - Get blockhash            │
//! │  /api/transactions/*│    │                             │
//! └─────────────────────┘    └─────────────────────────────┘
//! ```
//!
//! ## ApiClient (api.rs)
//!
//! ### Purpose
//!
//! HTTP client for communicating with the backend API server.
//! Handles all REST API calls for:
//! - User authentication (login/signup)
//! - Market data (prices, token lists)
//! - Swap operations (quotes, execution, history)
//! - Wallet queries (balances, token accounts)
//! - Transaction submission
//!
//! ### Key Methods
//!
//! ```rust
//! // Authentication
//! api_client.login(username, password) -> Result<AuthResponse, String>
//! api_client.signup(username, email, password) -> Result<AuthResponse, String>
//!
//! // Market Data
//! api_client.get_prices(&["SOL", "USDC"]) -> Result<PriceResponse, String>
//! api_client.get_token_list() -> Result<Vec<TokenListItem>, String>
//!
//! // Swaps
//! api_client.get_swap_quote(input_mint, output_mint, amount, slippage) -> Result<SwapQuoteResponse, String>
//! api_client.execute_swap(...) -> Result<SwapExecuteResponse, String>
//! api_client.get_swap_history(jwt_token, limit) -> Result<Vec<SwapHistoryItem>, String>
//!
//! // Transactions
//! api_client.submit_transaction(signed_tx, metadata, jwt_token) -> Result<TransactionSubmitResponse, String>
//!
//! // Wallet
//! api_client.get_wallet_balance(address) -> Result<WalletBalance, String>
//! api_client.get_token_balances(address) -> Result<Vec<TokenBalance>, String>
//! ```
//!
//! ### Usage Pattern
//!
//! ```rust
//! let api_client = Arc::new(ApiClient::new());
//!
//! // Login example
//! spawn(async move {
//!     match api_client.login(username, password).await {
//!         Ok(response) => {
//!             // Store JWT token, navigate to terminal
//!         }
//!         Err(e) => {
//!             // Display error to user
//!         }
//!     }
//! });
//! ```
//!
//! ## WalletService (wallet.rs)
//!
//! ### Purpose
//!
//! Manages Solana wallet connections and transaction signing.
//! Handles:
//! - Keypair loading from file or generation
//! - Transaction signing with local private key
//! - Balance queries via Solana RPC
//! - Token account balance queries
//!
//! ### Key Methods
//!
//! ```rust
//! // Wallet Connection
//! wallet_service.load_keypair_from_file(path) -> Result<(), WalletError>
//! wallet_service.load_keypair_from_base58(key) -> Result<(), WalletError>
//! wallet_service.generate_new_keypair() -> String  // Returns pubkey
//! wallet_service.disconnect()
//!
//! // Transaction Signing
//! wallet_service.sign_transaction(&mut tx) -> Result<Signature, WalletError>
//!
//! // Balance Queries
//! wallet_service.get_balance() -> Result<f64, WalletError>
//! wallet_service.get_token_balance(mint_address) -> Result<f64, WalletError>
//!
//! // State
//! wallet_service.get_public_key() -> Option<String>
//! wallet_service.is_connected() -> bool
//! wallet_service.get_status() -> &WalletStatus
//! ```
//!
//! ### Usage Pattern
//!
//! ```rust
//! let rpc_url = "https://api.devnet.solana.com";
//! let mut wallet_service = WalletService::new(rpc_url);
//!
//! // Load from Solana CLI default location
//! wallet_service.load_keypair_from_file("~/.config/solana/id.json")?;
//!
//! // Sign a transaction
//! let mut transaction = /* ... unsigned transaction ... */;
//! let signature = wallet_service.sign_transaction(&mut transaction)?;
//!
//! // Query balance
//! let balance = wallet_service.get_balance().await?;
//! println!("Balance: {} SOL", balance);
//! ```
//!
//! ## Service Interaction Pattern
//!
//! ### Swap Execution Flow
//!
//! The services work together to execute swaps:
//!
//! ```text
//! 1. Get Quote
//!    └─> ApiClient.get_swap_quote() ──> Backend ──> Jupiter
//!
//! 2. Get Unsigned Transaction
//!    └─> ApiClient.execute_swap() ──> Backend ──> Jupiter
//!        └─> Returns base64-encoded unsigned transaction
//!
//! 3. Sign Transaction Locally
//!    └─> WalletService.sign_transaction()
//!        └─> Uses local keypair (never sent to server)
//!
//! 4. Submit Signed Transaction
//!    └─> ApiClient.submit_transaction() ──> Backend ──> Solana Network
//!        └─> Backend broadcasts and saves to database
//! ```
//!
//! ### Security Model
//!
//! - **Private keys NEVER leave the client**
//! - Backend receives only:
//!   - Public key (for transaction creation)
//!   - Signed transaction (already signed, can't be modified)
//! - JWT token for API authentication (separate from wallet key)
//!
//! ## Error Handling
//!
//! ### ApiClient Errors
//!
//! Returns `Result<T, String>` with user-friendly messages:
//! - Network errors: "Network error: {details}"
//! - Parse errors: "Failed to parse response: {details}"
//! - API errors: Extracted from ErrorResponse body
//!
//! ### WalletService Errors
//!
//! Returns `Result<T, WalletError>` with detailed error types:
//! - `WalletError::KeypairLoadError` - Failed to load keypair
//! - `WalletError::InvalidKeypair` - Invalid keypair format
//! - `WalletError::RpcError` - Solana RPC connection failed
//! - `WalletError::SigningError` - Transaction signing failed
//! - `WalletError::BalanceError` - Balance query failed
//!
//! ## Thread Safety
//!
//! Both services are designed for async usage:
//!
//! - **ApiClient**: Uses `reqwest::Client` (internally thread-safe)
//!   - Can be wrapped in `Arc` and shared across tasks
//!   - Connection pooling and HTTP/2 multiplexing
//!
//! - **WalletService**: NOT thread-safe (contains mutable state)
//!   - Stored in `AppState` behind `RwLock`
//!   - Accessed via lock guards
//!
//! ## Configuration
//!
//! ### ApiClient Configuration
//!
//! - Base URL: `http://127.0.0.1:3001` (hardcoded)
//! - HTTP client: `reqwest::Client` with default settings
//! - No timeout (relies on OS defaults)
//!
//! ### WalletService Configuration
//!
//! - RPC URL: Configurable (passed to `new()`)
//! - Commitment level: `CommitmentConfig::confirmed()`
//! - Default keypair path: `~/.config/solana/id.json`
//!
//! ## Testing
//!
//! Both modules include comprehensive unit tests:
//!
//! ```bash
//! # Test wallet service
//! cargo test --lib services::wallet::tests
//!
//! # Test API client (requires backend)
//! cargo test --lib services::api::tests
//! ```
//!
//! ## Future Enhancements
//!
//! ### ApiClient
//! - Configurable base URL (environment variable)
//! - Request timeout configuration
//! - Retry logic with exponential backoff
//! - Circuit breaker pattern
//! - Request/response logging
//!
//! ### WalletService
//! - Hardware wallet support (Ledger, Trezor)
//! - Multi-signature wallets
//! - Wallet encryption at rest
//! - Mnemonic phrase import/export
//! - Token account creation

pub mod api;
pub mod braid_client;
pub mod wallet;
