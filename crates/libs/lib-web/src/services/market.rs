//! # Market Service
//!
//! Business logic for fetching real-time market data including token prices and token lists.
//!
//! ## Overview
//!
//! This service orchestrates market data operations by coordinating with the Solana
//! integration layer (price cache, Jupiter client) to provide up-to-date market information.
//!
//! ## Features
//!
//! - **Price Fetching**: Get real-time prices for multiple tokens via cached price feeds
//! - **Token Lists**: Fetch available tokens with metadata from Jupiter
//! - **Error Handling**: Comprehensive error handling with user-friendly messages
//!
//! ## Usage
//!
//! ```rust,no_run
//! use backend::services::market::MarketService;
//! use backend::solana::SolanaState;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let solana = Arc::new(SolanaState::new(/* ... */).await?);
//! let service = MarketService::new(solana);
//!
//! // Get prices for multiple tokens
//! let prices = service.get_prices(&["SOL", "USDC", "BTC"]).await?;
//!
//! // Get token list
//! let tokens = service.get_token_list().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All methods return `Result<T, AppError>`. Common errors:
//! - `AppError::NotFound` - No prices available for requested symbols
//! - `AppError::Internal` - Failed to fetch data from external sources
//!
//! ## Architecture
//!
//! The service uses the price cache for efficient price lookups:
//!
//! ```text
//! MarketService → PriceCache → (Pyth → Jupiter → Mock)
//!              → JupiterClient → Token List API
//! ```

use lib_core::AppError;
use lib_solana::{SolanaState, types::PriceResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn, instrument};

/// Service for market data operations.
///
/// This service provides business logic for fetching token prices and token lists.
/// It coordinates with the Solana integration layer to retrieve market data.
pub struct MarketService {
    solana: Arc<SolanaState>,
}

impl MarketService {
    /// Create a new market service.
    ///
    /// # Arguments
    ///
    /// * `solana` - Shared Solana state containing price cache and Jupiter client
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::market::MarketService;
    /// use backend::solana::SolanaState;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let service = MarketService::new(solana);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(solana: Arc<SolanaState>) -> Self {
        Self { solana }
    }

    /// Get real-time prices for multiple tokens.
    ///
    /// Fetches prices from the price cache, which aggregates data from multiple
    /// sources (Pyth, Jupiter, mocks) with automatic fallback.
    ///
    /// # Arguments
    ///
    /// * `symbols` - Slice of token symbols to fetch prices for (e.g., &["SOL", "USDC"])
    ///
    /// # Returns
    ///
    /// * `Ok(PriceResponse)` - Map of token symbols to price data
    /// * `Err(AppError::NotFound)` - No prices available for any requested symbol
    /// * `Err(AppError::Internal)` - Failed to fetch prices from cache
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::market::MarketService;
    /// use backend::solana::SolanaState;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let service = MarketService::new(solana);
    ///
    /// let response = service.get_prices(&["SOL", "USDC"]).await?;
    ///
    /// // Access prices
    /// if let Some(sol_price) = response.prices.get("SOL") {
    ///     println!("SOL price: ${:.2}", sol_price.price);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Notes
    ///
    /// - Prices are fetched from the cache, which is refreshed periodically
    /// - Missing prices for individual symbols are logged but don't fail the request
    /// - If no prices are available for any symbol, returns `NotFound` error
    /// - Prices may come from different sources (Pyth, Jupiter, mocks)
    #[instrument(skip(self), fields(symbols = ?symbols.iter().collect::<Vec<_>>()))]
    pub async fn get_prices(&self, symbols: &[&str]) -> Result<PriceResponse, AppError> {
        let mut prices = HashMap::new();

        for symbol in symbols {
            debug!("Fetching price for {}...", symbol);
            match self.solana.price_cache.get_price(symbol).await {
                Ok(price_data) => {
                    debug!(
                        "{}: ${:.4} (source: {})",
                        symbol, price_data.price, price_data.source
                    );
                    prices.insert(symbol.to_string(), price_data);
                }
                Err(e) => {
                    warn!("Failed to get price for {}: {}", symbol, e);
                    // Continue with other symbols even if one fails
                }
            }
        }

        if prices.is_empty() {
            return Err(AppError::NotFound(
                "No prices available for requested symbols".to_string(),
            ));
        }

        Ok(PriceResponse { prices })
    }

    /// Get list of available tokens with metadata.
    ///
    /// Fetches token metadata from Jupiter's token list API, which includes
    /// token addresses, symbols, names, decimals, and logo URIs.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<TokenInfo>)` - List of tokens with metadata
    /// * `Err(AppError::Internal)` - Failed to fetch token list from Jupiter
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::market::MarketService;
    /// use backend::solana::SolanaState;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let service = MarketService::new(solana);
    ///
    /// let tokens = service.get_token_list().await?;
    ///
    /// println!("Found {} tokens", tokens.len());
    /// for token in &tokens {
    ///     println!("{}: {}", token.symbol, token.address);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Notes
    ///
    /// - Token list is fetched directly from Jupiter API (not cached)
    /// - May take a few seconds for large token lists
    /// - Token metadata includes addresses, symbols, names, decimals, and logos
    pub async fn get_token_list(&self) -> Result<Vec<lib_solana::jupiter::TokenInfo>, AppError> {
        self.solana
            .jupiter
            .get_token_list()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch token list: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests would require mocking SolanaState
    // For now, we'll add integration tests in the handlers

    #[tokio::test]
    #[ignore] // Requires SolanaState setup
    async fn test_get_prices() {
        // TODO: Add test with mock SolanaState
    }

    #[tokio::test]
    #[ignore] // Requires SolanaState setup
    async fn test_get_token_list() {
        // TODO: Add test with mock SolanaState
    }
}

