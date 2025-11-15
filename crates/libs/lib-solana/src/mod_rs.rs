//! # Solana Integration Module
//!
//! Provides comprehensive integration with the Solana blockchain ecosystem including
//! RPC operations, price oracles, token swaps, and SPL token management.
//!
//! ## Architecture Overview
//!
//! This module orchestrates multiple Solana services:
//!
//! ```text
//! ┌─────────────────┐
//! │  SolanaState    │  ← Main state container
//! └────────┬────────┘
//!          │
//!          ├─► SolanaClient   (RPC operations & network selection)
//!          ├─► JupiterClient  (DEX aggregation & token swaps)
//!          ├─► PythClient     (On-chain price oracle)
//!          ├─► PriceCache     (Intelligent price caching with fallback)
//!          └─► SplTokenClient (SPL token account queries)
//! ```
//!
//! ## Key Components
//!
//! - **Network Selection**: Support for Mainnet and Devnet with configurable RPC endpoints
//! - **RPC Client**: Low-level Solana blockchain operations (accounts, transactions, epochs)
//! - **Price Oracles**: Multi-source price feeds with automatic fallback (Pyth → Jupiter → Mock)
//! - **Swap Integration**: Jupiter Aggregator for optimal DEX routing
//! - **Token Management**: SPL token account queries and balances
//! - **Caching**: Intelligent price caching to reduce API calls and latency
//!
//! ## Network Configuration
//!
//! The module supports two networks with different RPC endpoints:
//!
//! ### Mainnet Configuration
//! - **With Helius API key**: `https://mainnet.helius-rpc.com/?api-key={key}` (recommended for production)
//! - **Without API key**: `https://api.mainnet-beta.solana.com` (public endpoint with rate limits)
//!
//! ### Devnet Configuration
//! - **RPC URL**: `https://api.devnet.solana.com` (free, no API key needed)
//!
//! ## Example Usage
//!
//! ```rust
//! use backend::solana::{SolanaState, Network};
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Initialize Solana integration for mainnet with Helius
//! let state = SolanaState::new(
//!     Network::Mainnet,
//!     Some("your-helius-api-key".to_string())
//! ).await?;
//!
//! // Get real-time price data
//! let sol_price = state.price_cache.get_price("SOL").await?;
//! println!("SOL price: ${:.2} (from {})", sol_price.price, sol_price.source);
//!
//! // Query token balance
//! let balance = state.spl_token.get_token_balance("wallet_address", "token_mint").await?;
//! println!("Balance: {} tokens", balance.ui_amount);
//!
//! // Get swap quote via Jupiter
//! let quote = state.jupiter.get_swap_quote(
//!     "So11111111111111111111111111111111111111112", // SOL mint
//!     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC mint
//!     1_000_000_000, // 1 SOL in lamports
//!     50, // 0.5% slippage
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Considerations
//!
//! - **Price Cache**: 10-second TTL reduces API calls by up to 90% for popular tokens
//! - **Background Refresh**: Automatic price updates for frequently traded tokens
//! - **Helius RPC**: Premium endpoint provides 10x higher rate limits than public RPC
//! - **Connection Pooling**: HTTP clients reuse connections for better performance
//!
//! ## Error Handling
//!
//! All operations return `anyhow::Result` with descriptive error messages:
//! - Network failures are logged and retried via fallback sources
//! - Invalid addresses or mints return clear validation errors
//! - Rate limit errors suggest upgrading to premium RPC endpoints

// Modules are declared in lib.rs, so we can use them directly
use crate::client;
use crate::jupiter;
use crate::pyth;
use crate::cache;
use crate::types;
use crate::spl_token;
use crate::contracts;
use crate::price_stream;
use crate::candle_aggregator;

// Re-export types for convenience
pub use client::{SolanaClient, Network};
pub use jupiter::{JupiterClient, JupiterPriceData, TokenInfo};
pub use pyth::PythClient;
pub use cache::PriceCache;
pub use types::{PriceData, PriceResponse, PriceQuery};
pub use spl_token::{SplTokenClient, TokenAccountInfo, TokenBalance};
pub use contracts::{ContractRegistry, PluginLoader, ContractPlugin};
pub use price_stream::PriceStreamServer;
pub use candle_aggregator::CandleAggregator;
use std::sync::Arc;

/// Main Solana integration state container.
///
/// This struct aggregates all Solana-related services into a single, cloneable state
/// that can be shared across the application (e.g., in Axum handlers via extractors).
///
/// All internal clients are Arc-wrapped for efficient cloning and sharing across
/// async tasks and request handlers.
///
/// # Fields
///
/// * `rpc` - Low-level RPC client for blockchain operations (accounts, transactions, epochs)
/// * `jupiter` - Jupiter Aggregator client for DEX swaps and token metadata
/// * `pyth` - Pyth Network oracle client for real-time price feeds
/// * `price_cache` - Intelligent caching layer with multi-source fallback
/// * `spl_token` - SPL token client for querying token accounts and balances
///
/// # Example
///
/// ```rust
/// use backend::solana::{SolanaState, Network};
/// use axum::{extract::State, Json};
///
/// # async fn example() -> anyhow::Result<()> {
/// // Initialize state
/// let state = SolanaState::new(Network::Mainnet, None).await?;
///
/// // Clone state for sharing (cheap - only Arc clones)
/// let state_clone = state.clone();
///
/// // Use in async handler
/// async fn get_price(State(state): State<SolanaState>) -> Json<f64> {
///     let price = state.price_cache.get_price("SOL").await.unwrap();
///     Json(price.price)
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct SolanaState {
    pub rpc: Arc<SolanaClient>,
    pub jupiter: Arc<JupiterClient>,
    pub pyth: Arc<PythClient>,
    pub price_cache: Arc<PriceCache>,
    pub spl_token: Arc<SplTokenClient>,
    pub contracts: Arc<ContractRegistry>,
}

impl SolanaState {
    /// Create a new Solana integration state with all services initialized.
    ///
    /// This initializes and connects all Solana services including RPC client,
    /// Jupiter API, Pyth oracle, price cache, and SPL token client. Each service
    /// is configured according to the selected network.
    ///
    /// # Network Configuration
    ///
    /// ## Mainnet
    /// - With Helius API key: Uses premium RPC endpoint (recommended)
    /// - Without API key: Uses public endpoint (subject to rate limits)
    ///
    /// ## Devnet
    /// - Always uses public devnet endpoint
    /// - No API key required
    /// - Suitable for testing and development
    ///
    /// # Arguments
    ///
    /// * `network` - Target network (Mainnet or Devnet)
    /// * `helius_api_key` - Optional Helius API key for premium RPC access
    ///
    /// # Returns
    ///
    /// * `Ok(SolanaState)` - Fully initialized Solana state ready for use
    /// * `Err(_)` - If any service fails to initialize (HTTP client errors, etc.)
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::solana::{SolanaState, Network};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// // Mainnet with Helius (recommended for production)
    /// let mainnet_state = SolanaState::new(
    ///     Network::Mainnet,
    ///     Some("your-helius-api-key".to_string())
    /// ).await?;
    ///
    /// // Devnet for testing (no API key needed)
    /// let devnet_state = SolanaState::new(
    ///     Network::Devnet,
    ///     None
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Initialization is async and takes ~100-500ms depending on network latency
    /// - Once initialized, state can be cloned cheaply (only Arc clones)
    /// - Consider initializing once at application startup and sharing via Axum State
    pub async fn new(network: Network, helius_api_key: Option<String>) -> anyhow::Result<Self> {
        let rpc = Arc::new(SolanaClient::new(network.clone(), helius_api_key.clone()));
        tracing::info!("Solana RPC client created ({:?})", network);

        let jupiter = Arc::new(JupiterClient::new()?);
        tracing::info!("Jupiter API client ready");

        let pyth = Arc::new(PythClient::new()?);
        tracing::info!("Pyth Network oracle client ready");

        let price_cache = Arc::new(PriceCache::new(jupiter.clone(), pyth.clone()));
        tracing::info!("Price cache initialized (Pyth + Jupiter fallback)");

        // Create RPC URL for SPL token client
        let rpc_url = match network {
            Network::Mainnet => {
                if let Some(key) = helius_api_key {
                    format!("https://mainnet.helius-rpc.com/?api-key={}", key)
                } else {
                    "https://api.mainnet-beta.solana.com".to_string()
                }
            }
            Network::Devnet => "https://api.devnet.solana.com".to_string(),
        };
        let spl_token = Arc::new(SplTokenClient::new(rpc_url));
        tracing::info!("SPL Token client ready");

        // Initialize contract registry
        let contracts = Arc::new(ContractRegistry::new());
        tracing::info!("Contract registry initialized");

        Ok(Self {
            rpc,
            jupiter,
            pyth,
            price_cache,
            spl_token,
            contracts,
        })
    }
}
