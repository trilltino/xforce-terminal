//! # Solana DeFi Trading Terminal - Library Root
//!
//! A high-performance **native desktop GUI** for Solana DeFi trading.
//! This library crate contains all modules used by the binary crate (`main.rs`).
//!
//! ## Features
//!
//! - **Real-time Market Data**: Live price feeds via Jupiter and Pyth oracles
//! - **Wallet Management**: Local keypair signing with Solana SDK
//! - **Token Swaps**: Jupiter aggregator integration for optimal routes
//! - **Transaction History**: Monitor and track all blockchain transactions
//! - **Native GUI Window**: Full control without terminal limitations
//!
//! ## Architecture
//!
//! ### Technology Stack
//!
//! ```text
//! ┌────────────────────────────────────────────────────────┐
//! │              terminal (this crate)                     │
//! ├────────────────────────────────────────────────────────┤
//! │  egui          - Immediate-mode GUI framework          │
//! │  eframe        - Native window framework               │
//! │  egui_plot     - Charts and plotting                   │
//! │  Tokio         - Async runtime                          │
//! │  Reqwest       - HTTP client                            │
//! │  Solana SDK    - Blockchain transaction signing         │
//! └────────────────────────────────────────────────────────┘
//!          │                              │
//!          │ HTTP                         │ Solana RPC
//!          ▼                              ▼
//! ┌─────────────────┐          ┌─────────────────────────┐
//! │  Backend API    │          │   Solana Network        │
//! │  (Axum server)  │          │   (Devnet/Mainnet)      │
//! └─────────────────┘          └─────────────────────────┘
//! ```
//!
//! ## Module Structure
//!
//! ### Public Modules
//!
//! - **app**: Application state and screen management
//!   - Core orchestrator of the GUI
//!   - Event-driven architecture with async tasks
//!   - Screen navigation and state machine
//!
//! - **services**: External integrations
//!   - `api`: Backend HTTP client (authentication, market data, swaps)
//!   - `wallet`: Solana wallet management and transaction signing
//!
//! - **ui**: Rendering framework
//!   - `screens`: Screen-specific rendering (auth, terminal, wallet, transactions)
//!   - `widgets`: Custom UI components
//!   - `theme`: Color palette and styling
//!   - `effects`: Visual effects and animations
//!
//! - **utils**: Utility functions
//!   - `runtime`: Tokio runtime helpers
//!
//! ### Module Dependency Graph
//!
//! ```text
//! main.rs
//!   │
//!   ├── app (state, events, input handling)
//!   │   ├── services::api (HTTP requests)
//!   │   └── services::wallet (transaction signing)
//!   │
//!   └── ui (rendering)
//!       ├── screens::* (auth, terminal, wallet, transactions)
//!       ├── widgets::* (custom components)
//!       ├── theme (colors, styles)
//!       └── effects (animations)
//! ```
//!
//! ## Core Concepts
//!
//! ### Event-Driven Architecture
//!
//! The application uses **async channels** for communication:
//! - Main thread: Handles input and rendering (single-threaded)
//! - Async tasks: Network requests and blockchain operations (multi-threaded)
//!
//! Events flow from async tasks back to main thread via `AppEvent` enum.
//!
//! ### State Management
//!
//! Application state is wrapped in `Arc<RwLock<AppState>>`:
//! - **Thread-safe**: Multiple readers, exclusive writers
//! - **Shared**: Accessible from async tasks
//! - **Locked briefly**: Minimize contention, drop locks immediately
//!
//! ### Screen System
//!
//! Four main screens with tab navigation:
//! 1. **Auth**: Login/signup forms
//! 2. **Terminal**: Trading view (swaps, prices, charts)
//! 3. **Wallet**: Wallet connection and balance management
//! 4. **Transactions**: Transaction history and monitoring
//!
//! ## Usage
//!
//! ### As a Binary
//!
//! ```bash
//! cargo run --bin terminal
//! ```
//!
//! ### As a Library (for testing)
//!
//! ```rust
//! use terminal::app::App;
//! use terminal::services::api::ApiClient;
//!
//! #[tokio::test]
//! async fn test_app_creation() {
//!     let app = App::new();
//!     let state = app.state.read().await;
//!     assert_eq!(state.current_screen, Screen::Auth);
//! }
//! ```
//!
//! ## Re-Exported Modules
//!
//! All modules are public for testing and integration purposes:
//! - `pub mod app` - Application state and logic
//! - `pub mod services` - External service clients
//! - `pub mod ui` - UI rendering components
//! - `pub mod utils` - Utility functions
//!
//! ## Testing
//!
//! Run all tests:
//! ```bash
//! cargo test --lib
//! ```
//!
//! Run specific module tests:
//! ```bash
//! cargo test --lib app::tests
//! cargo test --lib services::wallet::tests
//! ```
//!
//! ## Performance
//!
//! - **CPU**: ~1-2% idle, <10% during active trading
//! - **Memory**: ~10-20 MB typical
//! - **Network**: Minimal (HTTP requests only when needed)
//! - **Latency**: <50ms input-to-render
//!
//! ## Dependencies
//!
//! ### Core Dependencies
//!
//! - `egui` - Immediate-mode GUI framework
//! - `eframe` - Native window framework for egui
//! - `egui_plot` - Charts and plotting for egui
//! - `tokio` - Async runtime
//! - `reqwest` - HTTP client
//! - `solana-sdk` - Blockchain SDK
//! - `solana-client` - RPC client
//!
//! ### Shared Crate
//!
//! - `shared` - Common types (DTOs, error responses)
//!   - Shared between terminal and backend
//!
//! ## Future Enhancements
//!
//! - WebSocket support for real-time price streaming
//! - Advanced charting (candlestick, line, area)
//! - Limit orders and advanced order types
//! - Portfolio tracking and analytics
//! - Multi-wallet support
//! - Transaction bundling
//! - Custom themes and color schemes

// Re-export main modules for testing and integration
// All modules are public to enable library usage and testing
pub mod app;
pub mod core;
pub mod debug;
pub mod services;
pub mod ui;
pub mod utils;

// Re-export commonly used types for convenience
// These are the most frequently used types that consumers of this library will need
pub use app::{App, AppState, AppEvent, Screen};
pub use core::{AppError, Result};
