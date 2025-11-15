//! # Jupiter Price API
//!
//! Price fetching with fallback to CoinGecko and mock data.

use super::client::JupiterHttpClient;
use super::types::JupiterPriceResponse;
use std::collections::HashMap;
use tracing::{debug, warn};

impl JupiterHttpClient {
    /// Convert a token symbol to its Solana mint address
    async fn symbol_to_mint(&self, symbol: &str) -> Option<String> {
        // Try cached token list first
        if let Some(mint) = self.get_mint_for_symbol(symbol).await {
            return Some(mint);
        }
        
        // Fallback to hardcoded mappings for common tokens
        match symbol.to_uppercase().as_str() {
            "SOL" => Some("So11111111111111111111111111111111111111112".to_string()),
            "USDC" => Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
            "USDT" => Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string()),
            "BTC" | "WBTC" => Some("3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh".to_string()),
            "ETH" | "WETH" => Some("7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs".to_string()),
            "JUP" => Some("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".to_string()),
            "RAY" => Some("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string()),
            "ORCA" => Some("orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE".to_string()),
            "BONK" => Some("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string()),
            "WIF" => Some("EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm".to_string()),
            _ => None,
        }
    }

    /// Fetch prices for multiple tokens in a single API call
    pub async fn get_prices(&self, symbols: &[&str]) -> anyhow::Result<HashMap<String, f64>> {
        // Convert symbols to mint addresses
        let mut symbol_to_mint = HashMap::new();
        let mut mints = Vec::new();
        
        for symbol in symbols {
            if let Some(mint) = self.symbol_to_mint(symbol).await {
                symbol_to_mint.insert(symbol.to_string(), mint.clone());
                mints.push(mint);
            } else {
                warn!("Unknown symbol: {}, skipping", symbol);
            }
        }
        
        if mints.is_empty() {
            return Err(anyhow::anyhow!("No valid symbols found"));
        }
        
        // Jupiter API accepts comma-separated mint addresses
        let ids = mints.join(",");
        let url = format!("{}/price?ids={}", self.price_api_base, ids);

        debug!("Fetching Jupiter prices for {} tokens", symbols.len());

        let response: JupiterPriceResponse = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                warn!("Jupiter API request failed: {}", e);
                anyhow::anyhow!("Jupiter API request failed: {}", e)
            })?
            .json()
            .await
            .map_err(|e| {
                warn!("Jupiter API parse failed: {}", e);
                anyhow::anyhow!("Jupiter API parse failed: {}", e)
            })?;

        // Map mint addresses back to symbols
        let mut prices: HashMap<String, f64> = HashMap::new();
        for (symbol, mint) in &symbol_to_mint {
            if let Some(price_data) = response.data.get(mint) {
                prices.insert(symbol.clone(), price_data.price);
            }
        }

        debug!("Jupiter returned {} prices", prices.len());

        Ok(prices)
    }

    /// Get the current price for a single token with automatic fallback
    pub async fn get_price(&self, symbol: &str) -> anyhow::Result<f64> {
        let mint = self
            .symbol_to_mint(symbol)
            .await
            .ok_or_else(|| anyhow::anyhow!("Unknown symbol: {}", symbol))?;

        let url = format!("{}/price?ids={}", self.price_api_base, mint);

        debug!("Fetching Jupiter price for {} ({})", symbol, mint);

        // Priority 1: Try Jupiter API
        match self.http.get(&url).send().await {
            Ok(response) => match response.json::<JupiterPriceResponse>().await {
                Ok(price_response) => {
                    if let Some(data) = price_response.data.into_values().next() {
                        debug!("Jupiter API: {} = ${:.4}", symbol, data.price);
                        return Ok(data.price);
                    }
                }
                Err(e) => {
                    warn!("Jupiter API parse failed for {}: {}", symbol, e);
                }
            },
            Err(e) => {
                warn!("Jupiter API request failed for {}: {}", symbol, e);
            }
        }

        // Priority 2: Try CoinGecko API
        debug!("Trying CoinGecko API for {}...", symbol);
        match self.get_coingecko_price(symbol).await {
            Ok(price) => {
                debug!("🦎 CoinGecko API: {} = ${:.4}", symbol, price);
                Ok(price)
            }
            Err(e) => {
                warn!("CoinGecko API failed for {}: {} - using mock data", symbol, e);
                // Priority 3: Use mock data as last resort
                self.get_mock_price(symbol)
            }
        }
    }

    /// Fetch price from CoinGecko API as fallback
    async fn get_coingecko_price(&self, symbol: &str) -> anyhow::Result<f64> {
        let coin_id = match symbol.to_uppercase().as_str() {
            "SOL" => "solana",
            "USDC" => "usd-coin",
            "USDT" => "tether",
            "BTC" | "WBTC" => "bitcoin",
            "ETH" | "WETH" => "ethereum",
            "JUP" => "jupiter-exchange-solana",
            "RAY" => "raydium",
            "ORCA" => "orca",
            "BONK" => "bonk",
            "WIF" => "dogwifcoin",
            _ => return Err(anyhow::anyhow!("Unknown symbol for CoinGecko: {}", symbol)),
        };

        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            coin_id
        );

        #[derive(serde::Deserialize)]
        struct CoinGeckoResponse {
            #[serde(flatten)]
            prices: HashMap<String, CoinPrice>,
        }

        #[derive(serde::Deserialize)]
        struct CoinPrice {
            usd: f64,
        }

        let response: CoinGeckoResponse = self
            .http
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        response
            .prices
            .get(coin_id)
            .map(|p| p.usd)
            .ok_or_else(|| anyhow::anyhow!("No price data from CoinGecko for {}", symbol))
    }

    /// Generate mock prices for development/testing
    fn get_mock_price(&self, symbol: &str) -> anyhow::Result<f64> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let base_price = match symbol.to_uppercase().as_str() {
            "SOL" => 145.50,
            "USDC" => 1.0,
            "USDT" => 1.0,
            "BTC" | "WBTC" => 64250.00,
            "ETH" | "WETH" => 3100.00,
            "JUP" => 1.25,
            "RAY" => 2.85,
            "ORCA" => 0.95,
            "BONK" => 0.00002150,
            "WIF" => 2.15,
            _ => 0.0,
        };

        if base_price <= 0.0 {
            return Err(anyhow::anyhow!("Unknown symbol: {}", symbol));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_else(|e| {
                warn!("System time before Unix epoch: {}. Using 0 for mock price seed.", e);
                0
            });

        let seed = (now / 2) + symbol.len() as u64;
        let fluctuation_factor = ((seed * 16807) % 100) as f64 / 100.0;

        let volatility = if symbol == "USDC" || symbol == "USDT" {
            0.001
        } else {
            0.02
        };

        let change_percent = (fluctuation_factor - 0.5) * 2.0 * volatility;
        let price = base_price * (1.0 + change_percent);

        debug!("Using dynamic mock price for {}: ${:.6} ({:+.2}%)",
               symbol, price, change_percent * 100.0);
        Ok(price)
    }
}

