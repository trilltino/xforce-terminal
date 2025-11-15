//! # Price Caching Module
//!
//! This module provides intelligent price caching with multiple data sources.
//! It implements a fallback strategy: Pyth Network (oracle) → Jupiter API → Mock data.
//!
//! ## Features
//! - Automatic cache expiration (configurable TTL)
//! - Background refresh for popular tokens
//! - Multi-source fallback for reliability
//! - Thread-safe concurrent access
//!
//! ## Example
//! ```no_run
//! let cache = PriceCache::new(jupiter_client, pyth_client);
//! let price = cache.get_price("SOL").await?;
//! println!("SOL price: ${}", price.price);
//! ```

use crate::jupiter::JupiterClient;
use crate::pyth::PythClient;
use crate::types::PriceData;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Represents a cached price entry with metadata and expiration.
struct CachedPrice {
    /// Current price in USD
    price: f64,
    /// Confidence interval (only available from some sources)
    confidence: Option<f64>,
    /// 24-hour price change percentage
    change_24h: Option<f64>,
    /// Data source identifier (e.g., "pyth", "jupiter")
    source: String,
    /// When this price was cached
    timestamp: Instant,
    /// How long until this cache entry expires
    ttl: Duration,
}

/// Thread-safe price cache with automatic expiration and multi-source fallback.
///
/// The cache uses a read-write lock to allow concurrent reads while serializing writes.
/// Expired entries are automatically refreshed on next access.
pub struct PriceCache {
    cache: Arc<RwLock<HashMap<String, CachedPrice>>>,
    jupiter: Arc<JupiterClient>,
    pyth: Arc<PythClient>,
}

/// Helper function to get current Unix timestamp safely.
///
/// # Returns
/// Current time as seconds since Unix epoch, or 0 if system time is before epoch
/// (which should never happen on real systems).
fn get_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_else(|e| {
            warn!("System time is before Unix epoch: {}. Using 0.", e);
            0
        })
}

impl PriceCache {
    pub fn new(jupiter: Arc<JupiterClient>, pyth: Arc<PythClient>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            jupiter,
            pyth,
        }
    }


    /// Get the current price for a token symbol, using cache if available.
    ///
    /// This function first checks the cache. If the cached price is still valid (not expired),
    /// it returns immediately. Otherwise, it fetches fresh data from external sources.
    ///
    /// # Arguments
    /// * `symbol` - Token symbol (e.g., "SOL", "USDC", "BTC")
    ///
    /// # Returns
    /// * `Ok(PriceData)` - Price data with source and timestamp
    /// * `Err(_)` - If all data sources fail
    ///
    /// # Example
    /// ```no_run
    /// let price = cache.get_price("SOL").await?;
    /// println!("SOL: ${:.2}", price.price);
    /// ```
    pub async fn get_price(&self, symbol: &str) -> anyhow::Result<PriceData> {
        // 1. Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(symbol) {
                if cached.timestamp.elapsed() < cached.ttl {
                    debug!("Cache hit for {}: ${:.4}", symbol, cached.price);
                    return Ok(PriceData {
                        price: cached.price,
                        confidence: cached.confidence,
                        source: cached.source.clone(),
                        change_24h: cached.change_24h,
                        last_updated: get_unix_timestamp(),
                    });
                } else {
                    debug!("Cache expired for {}", symbol);
                }
            }
        }

        // 2. Cache miss - fetch fresh data
        let price_data = self.fetch_fresh(symbol).await?;

        // 3. Update cache with new data
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                symbol.to_string(),
                CachedPrice {
                    price: price_data.price,
                    confidence: price_data.confidence,
                    change_24h: price_data.change_24h,
                    source: price_data.source.clone(),
                    timestamp: Instant::now(),
                    // TODO: Make TTL configurable via config system
                    ttl: Duration::from_secs(10),
                },
            );
        }

        Ok(price_data)
    }

    /// Fetch fresh price data from external sources.
    ///
    /// Implements a fallback strategy:
    /// 1. Try Pyth Network (on-chain oracle - most reliable)
    /// 2. Fall back to Jupiter API (aggregator)
    /// 3. Return error if both fail
    ///
    /// # Arguments
    /// * `symbol` - Token symbol to fetch price for
    ///
    /// # Returns
    /// * `Ok(PriceData)` - Fresh price data from first available source
    /// * `Err(_)` - If all sources fail
    async fn fetch_fresh(&self, symbol: &str) -> anyhow::Result<PriceData> {
        // Priority 1: Pyth Network (real on-chain oracle data)
        // This is the most reliable source as it's an on-chain oracle
        match self.pyth.get_price(symbol).await {
            Ok(pyth_price) => {
                info!("REAL LIVE DATA - Pyth: {} = ${:.4}", symbol, pyth_price);
                return Ok(PriceData {
                    price: pyth_price,
                    confidence: None,
                    source: "pyth".into(),
                    change_24h: None,
                    last_updated: get_unix_timestamp(),
                });
            }
            Err(e) => {
                debug!("Pyth failed for {}, trying Jupiter: {}", symbol, e);
            }
        }

        // Priority 2: Jupiter API fallback
        // Jupiter aggregates from multiple DEXes
        match self.jupiter.get_price(symbol).await {
            Ok(jup_price) => {
                debug!("Jupiter price for {}: ${:.4}", symbol, jup_price);
                Ok(PriceData {
                    price: jup_price,
                    confidence: None,
                    source: "jupiter".into(),
                    change_24h: None,
                    last_updated: get_unix_timestamp(),
                })
            }
            Err(e) => {
                warn!("All APIs failed for {}: {}", symbol, e);
                Err(anyhow::anyhow!("No price data available for {}", symbol))
            }
        }
    }

    /// Get multiple prices at once, returning only successful fetches.
    ///
    /// This method fetches prices for multiple symbols concurrently. If any individual
    /// fetch fails, it's logged as a warning but doesn't affect other symbols.
    ///
    /// # Arguments
    /// * `symbols` - Slice of token symbols to fetch
    ///
    /// # Returns
    /// HashMap with successful price fetches. Failed fetches are omitted.
    ///
    /// # Example
    /// ```no_run
    /// let prices = cache.get_prices(&["SOL", "USDC", "BTC"]).await;
    /// println!("Fetched {} prices", prices.len());
    /// ```
    pub async fn get_prices(&self, symbols: &[&str]) -> HashMap<String, PriceData> {
        let mut prices = HashMap::new();

        for symbol in symbols {
            match self.get_price(symbol).await {
                Ok(price_data) => {
                    prices.insert(symbol.to_string(), price_data);
                }
                Err(e) => {
                    warn!("Failed to get price for {}: {}", symbol, e);
                }
            }
        }

        prices
    }

    /// Start a background task that continuously refreshes popular token prices.
    ///
    /// This spawns a tokio task that refreshes prices every 10 seconds to keep
    /// the cache warm for frequently requested tokens. This reduces latency for
    /// common price queries.
    ///
    /// The task runs indefinitely until the program exits.
    ///
    /// # Arguments
    /// * `self` - Arc-wrapped self to enable sharing across async context
    ///
    /// # Popular Tokens
    /// Currently refreshes: SOL, USDC, BTC, ETH, JUP, RAY, ORCA
    ///
    /// # Example
    /// ```no_run
    /// let cache = Arc::new(PriceCache::new(jupiter, pyth));
    /// cache.clone().start_background_refresh().await;
    /// ```
    pub async fn start_background_refresh(self: Arc<Self>) {
        // TODO: Make this list configurable via config system
        let popular_tokens = vec!["SOL", "USDC", "BTC", "ETH", "JUP", "RAY", "ORCA"];

        tokio::spawn(async move {
            // TODO: Make interval configurable
            let mut interval = tokio::time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                debug!("Background refresh starting...");

                for symbol in &popular_tokens {
                    if let Err(e) = self.get_price(symbol).await {
                        warn!("Background refresh failed for {}: {}", symbol, e);
                    }
                }

                debug!("Background refresh completed");
            }
        });
    }
}
