//! # Solana Integration Type Definitions
//!
//! Common data structures used across the Solana integration layer.
//!
//! ## Overview
//!
//! This module defines shared types for:
//! - Price data from various sources (Pyth, Jupiter, CoinGecko)
//! - API request/response structures
//! - Query parameters for HTTP endpoints
//!
//! ## Type Categories
//!
//! ### Price Data Types
//! - `PriceData`: Complete price information with metadata
//! - `PriceResponse`: API response wrapper for multiple prices
//! - `PriceQuery`: Query parameters for price requests
//!
//! ## Example
//!
//! ```rust
//! use backend::solana::types::{PriceData, PriceResponse};
//! use std::collections::HashMap;
//!
//! // Create price data
//! let sol_price = PriceData {
//!     price: 145.50,
//!     confidence: Some(0.05),
//!     source: "pyth".to_string(),
//!     change_24h: Some(5.2),
//!     last_updated: 1234567890,
//! };
//!
//! // Build response with multiple prices
//! let mut prices = HashMap::new();
//! prices.insert("SOL".to_string(), sol_price);
//! let response = PriceResponse { prices };
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Price data for a single token with metadata.
///
/// Contains the current price along with optional confidence intervals,
/// 24-hour change data, and source information. This struct is used across
/// all price sources (Pyth, Jupiter, CoinGecko, mocks) for consistency.
///
/// # Fields
///
/// * `price` - Current price in USD
/// * `confidence` - Optional confidence interval (±value in USD). Only available from some sources like Pyth.
/// * `source` - Data source identifier (e.g., "pyth", "jupiter", "coingecko", "mock")
/// * `change_24h` - Optional 24-hour price change percentage (positive = increase, negative = decrease)
/// * `last_updated` - Unix timestamp of when this price was last updated
///
/// # Serialization
///
/// The `confidence` and `change_24h` fields are skipped when `None` during JSON serialization
/// to produce cleaner API responses.
///
/// # Example
///
/// ```rust
/// use backend::solana::types::PriceData;
///
/// // Price from Pyth (includes confidence)
/// let pyth_price = PriceData {
///     price: 145.32,
///     confidence: Some(0.05),  // ±$0.05 confidence
///     source: "pyth".to_string(),
///     change_24h: Some(5.2),   // +5.2% in 24h
///     last_updated: 1234567890,
/// };
///
/// // Price from Jupiter (no confidence data)
/// let jupiter_price = PriceData {
///     price: 145.28,
///     confidence: None,
///     source: "jupiter".to_string(),
///     change_24h: None,
///     last_updated: 1234567890,
/// };
///
/// // Display price with source
/// println!("SOL: ${:.2} (from {})", pyth_price.price, pyth_price.source);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    /// Current price in USD
    pub price: f64,

    /// Optional confidence interval in USD (e.g., 0.05 = ±$0.05)
    ///
    /// Only available from oracle sources like Pyth. Represents the uncertainty
    /// in the price measurement. Lower values indicate higher confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,

    /// Data source identifier
    ///
    /// Common values: "pyth", "jupiter", "coingecko", "mock"
    pub source: String,

    /// 24-hour price change percentage
    ///
    /// Positive values indicate price increase, negative values indicate decrease.
    /// For example, 5.2 means +5.2% change in the last 24 hours.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_24h: Option<f64>,

    /// Unix timestamp (seconds since epoch) of last price update
    pub last_updated: u64,
}

/// API response containing prices for multiple tokens.
///
/// Used by HTTP handlers to return batch price queries. The response maps
/// token symbols to their corresponding price data.
///
/// # Example
///
/// ```rust
/// use backend::solana::types::{PriceData, PriceResponse};
/// use std::collections::HashMap;
/// use axum::Json;
///
/// # async fn example() -> Json<PriceResponse> {
/// let mut prices = HashMap::new();
///
/// prices.insert("SOL".to_string(), PriceData {
///     price: 145.32,
///     confidence: Some(0.05),
///     source: "pyth".to_string(),
///     change_24h: Some(5.2),
///     last_updated: 1234567890,
/// });
///
/// prices.insert("USDC".to_string(), PriceData {
///     price: 1.0,
///     confidence: Some(0.001),
///     source: "pyth".to_string(),
///     change_24h: Some(0.0),
///     last_updated: 1234567890,
/// });
///
/// let response = PriceResponse { prices };
/// Json(response)
/// # }
/// ```
///
/// # JSON Format
///
/// ```json
/// {
///   "prices": {
///     "SOL": {
///       "price": 145.32,
///       "confidence": 0.05,
///       "source": "pyth",
///       "change_24h": 5.2,
///       "last_updated": 1234567890
///     },
///     "USDC": {
///       "price": 1.0,
///       "source": "jupiter",
///       "last_updated": 1234567890
///     }
///   }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct PriceResponse {
    /// Map of token symbol to price data
    pub prices: HashMap<String, PriceData>,
}

/// Query parameters for price requests.
///
/// Used to parse HTTP query parameters in price API endpoints. Supports
/// requesting multiple token prices in a single request via comma-separated symbols.
///
/// # Example
///
/// ```rust
/// use backend::solana::types::PriceQuery;
/// use axum::extract::Query;
///
/// # async fn handler(Query(query): Query<PriceQuery>) {
/// // Parse comma-separated symbols
/// let symbols: Vec<&str> = query.symbols.split(',').collect();
///
/// for symbol in symbols {
///     println!("Fetching price for: {}", symbol);
/// }
/// # }
/// ```
///
/// # URL Format
///
/// ```text
/// GET /api/prices?symbols=SOL,USDC,BTC
/// ```
///
/// This would be parsed into:
/// ```rust
/// # use backend::solana::types::PriceQuery;
/// let query = PriceQuery {
///     symbols: "SOL,USDC,BTC".to_string()
/// };
/// ```
#[derive(Debug, Deserialize)]
pub struct PriceQuery {
    /// Comma-separated token symbols (e.g., "SOL,USDC,BTC")
    pub symbols: String,
}
