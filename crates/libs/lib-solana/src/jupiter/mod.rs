//! # Jupiter Aggregator Client
//!
//! Integration with Jupiter Aggregator for swap routing and price data.

// region: --- Modules
pub mod types;
pub mod client;
pub mod quote;
pub mod swap;
pub mod price;
// endregion: --- Modules

// region: --- Main Client
use client::JupiterHttpClient;

/// Builder for configuring JupiterClient.
///
/// Allows fluent configuration of client settings before building.
#[derive(Debug, Clone)]
pub struct JupiterClientBuilder {
    timeout: Option<std::time::Duration>,
    price_api_base: Option<String>,
    token_api_base: Option<String>,
}

impl Default for JupiterClientBuilder {
    fn default() -> Self {
        Self {
            timeout: Some(std::time::Duration::from_secs(10)),
            price_api_base: Some("https://price.jup.ag/v6".to_string()),
            token_api_base: Some("https://token.jup.ag".to_string()),
        }
    }
}

impl JupiterClientBuilder {
    /// Set the HTTP request timeout.
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the price API base URL.
    pub fn price_api_base(mut self, url: String) -> Self {
        self.price_api_base = Some(url);
        self
    }

    /// Set the token API base URL.
    pub fn token_api_base(mut self, url: String) -> Self {
        self.token_api_base = Some(url);
        self
    }

    /// Build the JupiterClient with configured settings.
    pub fn build(self) -> anyhow::Result<JupiterClient> {
        let http = reqwest::Client::builder()
            .timeout(self.timeout.unwrap_or_else(|| std::time::Duration::from_secs(10)))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;

        let inner = JupiterHttpClient {
            http,
            price_api_base: self.price_api_base.unwrap_or_else(|| "https://price.jup.ag/v6".to_string()),
            token_api_base: self.token_api_base.unwrap_or_else(|| "https://token.jup.ag".to_string()),
            token_cache: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
        };

        Ok(JupiterClient { inner })
    }
}

/// Client for Jupiter Aggregator API
pub struct JupiterClient {
    inner: JupiterHttpClient,
}

impl JupiterClient {
    /// Create a new Jupiter API client with default settings.
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            inner: JupiterHttpClient::new()?,
        })
    }

    /// Create a new Jupiter client using a builder for configuration.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use lib_solana::jupiter::JupiterClient;
    ///
    /// let client = JupiterClient::builder()
    ///     .timeout(std::time::Duration::from_secs(30))
    ///     .price_api_base("https://price.jup.ag/v6".to_string())
    ///     .build()?;
    /// ```
    pub fn builder() -> JupiterClientBuilder {
        JupiterClientBuilder::default()
    }

    // Delegate methods to inner client
    pub async fn load_token_list(&self) -> anyhow::Result<()> {
        self.inner.load_token_list().await
    }

    pub async fn get_mint_for_symbol(&self, symbol: &str) -> Option<String> {
        self.inner.get_mint_for_symbol(symbol).await
    }

    pub async fn get_all_tokens(&self) -> Option<Vec<types::TokenInfo>> {
        self.inner.get_all_tokens().await
    }

    /// Fetch complete token list with metadata from Jupiter (direct API call, not cached)
    pub async fn get_token_list(&self) -> anyhow::Result<Vec<types::TokenInfo>> {
        self.inner.get_token_list().await
    }

    pub async fn get_swap_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> anyhow::Result<types::QuoteResponse> {
        self.inner.get_swap_quote(input_mint, output_mint, amount, slippage_bps).await
    }

    pub async fn get_swap_transaction(
        &self,
        quote_response: &types::QuoteResponse,
        user_public_key: &str,
    ) -> anyhow::Result<types::SwapTransactionResponse> {
        self.inner.get_swap_transaction(quote_response, user_public_key).await
    }

    pub async fn get_prices(&self, symbols: &[&str]) -> anyhow::Result<std::collections::HashMap<String, f64>> {
        self.inner.get_prices(symbols).await
    }

    pub async fn get_price(&self, symbol: &str) -> anyhow::Result<f64> {
        self.inner.get_price(symbol).await
    }
}

impl Default for JupiterClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default JupiterClient")
    }
}
// endregion: --- Main Client

// Re-export commonly used types
pub use types::*;

