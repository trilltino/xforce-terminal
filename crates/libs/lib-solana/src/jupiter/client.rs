//! # Jupiter HTTP Client
//!
//! HTTP client wrapper and token caching for Jupiter API.

use super::types::TokenInfo;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Cached token list data structure
pub struct TokenCache {
    /// Map of uppercase symbol to mint address
    symbol_to_mint: HashMap<String, String>,
    /// Full list of tokens
    tokens: Vec<TokenInfo>,
    /// When the cache was last refreshed
    last_refresh: std::time::Instant,
}

/// HTTP client wrapper for Jupiter API
pub struct JupiterHttpClient {
    pub http: Client,
    pub price_api_base: String,
    pub token_api_base: String,
    /// Cached token list with symbol→mint mapping
    pub token_cache: Arc<RwLock<Option<TokenCache>>>,
}

impl JupiterHttpClient {
    /// Create a new HTTP client with timeout configuration
    pub fn new() -> anyhow::Result<Self> {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;

        Ok(Self {
            http,
            price_api_base: "https://price.jup.ag/v6".into(),
            token_api_base: "https://token.jup.ag".into(),
            token_cache: Arc::new(RwLock::new(None)),
        })
    }

    /// Load and cache token list from Jupiter API
    pub async fn load_token_list(&self) -> anyhow::Result<()> {
        let tokens = self.get_token_list().await?;
        
        let mut symbol_to_mint = HashMap::new();
        
        for token in &tokens {
            let symbol_upper = token.symbol.to_uppercase();
            symbol_to_mint.insert(symbol_upper, token.address.clone());
        }
        
        let cache = TokenCache {
            symbol_to_mint,
            tokens,
            last_refresh: std::time::Instant::now(),
        };
        
        *self.token_cache.write().await = Some(cache);
        info!("Token list cached ({} tokens)", self.token_cache.read().await.as_ref().map(|c| c.tokens.len()).unwrap_or(0));
        
        Ok(())
    }

    /// Get mint address for a token symbol
    pub async fn get_mint_for_symbol(&self, symbol: &str) -> Option<String> {
        let needs_refresh = {
            let cache = self.token_cache.read().await;
            if let Some(ref cache) = *cache {
                if cache.last_refresh.elapsed() > std::time::Duration::from_secs(3600) {
                    true
                } else {
                    return cache.symbol_to_mint.get(&symbol.to_uppercase()).cloned();
                }
            } else {
                false
            }
        };
        
        if needs_refresh {
            let client = self.clone_for_refresh();
            tokio::spawn(async move {
                if let Err(e) = client.load_token_list().await {
                    tracing::warn!("Failed to refresh token list: {}", e);
                }
            });
            return self.token_cache.read().await.as_ref()
                .and_then(|c| c.symbol_to_mint.get(&symbol.to_uppercase()))
                .cloned();
        }
        
        None
    }

    /// Get all cached tokens
    pub async fn get_all_tokens(&self) -> Option<Vec<TokenInfo>> {
        let cache = self.token_cache.read().await;
        cache.as_ref().map(|c| c.tokens.clone())
    }

    /// Clone self for async refresh task
    fn clone_for_refresh(&self) -> Self {
        Self {
            http: self.http.clone(),
            price_api_base: self.price_api_base.clone(),
            token_api_base: self.token_api_base.clone(),
            token_cache: Arc::clone(&self.token_cache),
        }
    }

    /// Fetch complete token list with metadata from Jupiter
    pub async fn get_token_list(&self) -> anyhow::Result<Vec<TokenInfo>> {
        let url = format!("{}/all", self.token_api_base);

        self.http
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Jupiter token list request failed: {}", e))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Jupiter token list parse failed: {}", e))
    }
}

