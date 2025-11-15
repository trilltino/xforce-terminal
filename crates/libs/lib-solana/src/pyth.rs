//! # Pyth Network Price Oracle Client
//!
//! This module provides integration with Pyth Network, a high-fidelity oracle network
//! that provides real-time, on-chain price data for cryptocurrencies and other assets.
//!
//! ## Features
//! - Real-time price feeds from Pyth Hermes API
//! - Confidence intervals for price data quality
//! - Sub-second price updates
//! - Support for major crypto assets
//!
//! ## Price Feed Architecture
//! Pyth aggregates prices from multiple data providers (exchanges, market makers)
//! and publishes them on-chain. This client fetches the latest prices via the
//! Hermes HTTP API, which provides historical and latest price feed data.
//!
//! ## Example
//! ```no_run
//! let client = PythClient::new()?;
//! let sol_price = client.get_price("SOL").await?;
//! println!("SOL price from Pyth: ${:.2}", sol_price);
//! ```
//!
//! ## Documentation
//! - Pyth Network: https://pyth.network/
//! - Hermes API: https://docs.pyth.network/price-feeds/api-instances-and-providers/hermes

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Client for Pyth Network price oracle via Hermes HTTP API.
///
/// Pyth provides high-fidelity, real-time price feeds for cryptocurrencies
/// and other assets. Prices are aggregated from multiple institutional sources
/// and published on-chain.
pub struct PythClient {
    http: Client,
    hermes_url: String,
}

/// Pyth Hermes API response containing price feed data.
#[derive(Debug, Deserialize)]
struct ParsedPrice {
    /// Unique price feed identifier
    #[allow(dead_code)] // Deserialized but not used (only price.price is needed)
    id: String,
    /// Current spot price
    price: PythPriceData,
    /// Exponential moving average price (smoothed)
    #[serde(rename = "ema_price")]
    #[allow(dead_code)] // Deserialized but not used (only price.price is needed)
    ema_price: PythPriceData,
}

/// Individual price data point from Pyth.
///
/// Prices are encoded as strings to preserve precision, with separate
/// exponent field for scale.
#[derive(Debug, Deserialize)]
struct PythPriceData {
    /// Raw price as string (e.g., "14550")
    price: String,
    /// Confidence interval as string
    #[allow(dead_code)] // Deserialized but not currently used (may be used in future)
    conf: String,
    /// Price exponent (e.g., -2 means divide by 100)
    expo: i32,
    /// Unix timestamp of price publication
    #[allow(dead_code)] // Deserialized but not currently used (may be used in future)
    publish_time: i64,
}

impl PythClient {
    /// Create a new Pyth Network API client.
    ///
    /// # Returns
    /// * `Ok(PythClient)` - Successfully initialized client
    /// * `Err(_)` - Failed to build HTTP client (rare, only on system issues)
    ///
    /// # Example
    /// ```no_run
    /// let client = PythClient::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;

        Ok(Self {
            http,
            hermes_url: "https://hermes.pyth.network".to_string(),
        })
    }

    /// Convert token symbol to Pyth Network price feed ID.
    ///
    /// Each asset on Pyth has a unique price feed identified by a 32-byte hex string.
    /// These IDs are stable and published by Pyth Network.
    ///
    /// # Arguments
    /// * `symbol` - Token symbol (case-insensitive)
    ///
    /// # Returns
    /// * `Some(feed_id)` - Pyth price feed ID for known symbols
    /// * `None` - Unknown symbol or no Pyth feed available
    ///
    /// # Supported Assets
    /// SOL, BTC/WBTC, ETH/WETH, USDC, USDT
    ///
    /// # Price Feed IDs
    /// Feed IDs can be found at: https://pyth.network/developers/price-feed-ids
    fn symbol_to_price_feed_id(&self, symbol: &str) -> Option<&str> {
        // TODO: Move these mappings to configuration or fetch from Pyth API
        match symbol.to_uppercase().as_str() {
            "SOL" => Some("0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d"),
            "BTC" | "WBTC" => Some("0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43"),
            "ETH" | "WETH" => Some("0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace"),
            "USDC" => Some("0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a"),
            "USDT" => Some("0x2b89b9dc8fdf9f34709a5b106b472f0f39bb6ca9ce04b0fd7f2e971688e2e53b"),
            _ => None,
        }
    }

    /// Fetch the latest price for a token from Pyth Network.
    ///
    /// This queries the Pyth Hermes API for the most recent price update. Pyth prices
    /// are typically updated sub-second and include confidence intervals.
    ///
    /// # Price Encoding
    /// Pyth encodes prices as `raw_price * 10^expo` where:
    /// - `raw_price` is an integer (e.g., 14550)
    /// - `expo` is typically negative (e.g., -2)
    /// - Final price = 14550 * 10^(-2) = $145.50
    ///
    /// # Arguments
    /// * `symbol` - Token symbol (e.g., "SOL", "BTC")
    ///
    /// # Returns
    /// * `Ok(price)` - Current price in USD
    /// * `Err(_)` - Unknown symbol, API failure, or parse error
    ///
    /// # Example
    /// ```no_run
    /// let price = client.get_price("SOL").await?;
    /// println!("SOL: ${:.2}", price);
    /// ```
    pub async fn get_price(&self, symbol: &str) -> Result<f64> {
        let feed_id = self
            .symbol_to_price_feed_id(symbol)
            .ok_or_else(|| anyhow::anyhow!("No Pyth feed for symbol: {}", symbol))?;

        let url = format!("{}/api/latest_price_feeds?ids[]={}", self.hermes_url, feed_id);

        debug!("Fetching Pyth price for {} (feed: {})", symbol, &feed_id[..8]);

        let response: Vec<ParsedPrice> = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                warn!("Pyth Hermes API request failed for {}: {}", symbol, e);
                anyhow::anyhow!("Pyth API request failed: {}", e)
            })?
            .json()
            .await
            .map_err(|e| {
                warn!("Pyth Hermes API parse failed for {}: {}", symbol, e);
                anyhow::anyhow!("Pyth API parse failed: {}", e)
            })?;

        let parsed = response
            .first()
            .ok_or_else(|| anyhow::anyhow!("No price data in Pyth response"))?;

        // Convert price string to f64 and apply exponent
        // Pyth prices are encoded as: price = raw_price * 10^expo
        let price_raw: i64 = parsed.price.price.parse()?;
        let expo = parsed.price.expo;
        let price = (price_raw as f64) * 10_f64.powi(expo);

        debug!("Pyth LIVE: {} = ${:.4} (raw: {}, expo: {})", symbol, price, price_raw, expo);
        Ok(price)
    }

    /// Fetch prices for multiple tokens, returning only successful fetches.
    ///
    /// This method fetches prices sequentially for each symbol. Failed fetches
    /// are logged as warnings but don't affect other symbols.
    ///
    /// # Arguments
    /// * `symbols` - Slice of token symbols to fetch
    ///
    /// # Returns
    /// HashMap with successful price fetches. Failed fetches are omitted.
    ///
    /// # Example
    /// ```no_run
    /// let prices = client.get_prices(&["SOL", "BTC", "ETH"]).await;
    /// println!("Fetched {} prices from Pyth", prices.len());
    /// ```
    pub async fn get_prices(&self, symbols: &[&str]) -> HashMap<String, f64> {
        let mut prices = HashMap::new();

        for symbol in symbols {
            match self.get_price(symbol).await {
                Ok(price) => {
                    prices.insert(symbol.to_string(), price);
                }
                Err(e) => {
                    warn!("Failed to get Pyth price for {}: {}", symbol, e);
                }
            }
        }

        prices
    }
}

impl Default for PythClient {
    fn default() -> Self {
        // Safe to unwrap here because we want the default to panic if HTTP client fails
        // This is acceptable for Default trait as it indicates a fundamental system issue
        Self::new().expect("Failed to create default PythClient")
    }
}
