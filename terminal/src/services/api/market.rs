//! # Market Data Endpoints
//!
//! Handles market data queries (prices, token lists).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::client::ApiClient;

/// Get Solana token prices.
#[tracing::instrument(skip(client), fields(symbols = ?symbols))]
pub async fn get_prices(
    client: &ApiClient,
    symbols: &[&str],
) -> Result<PriceResponse, String> {
    let start = std::time::Instant::now();
    let symbols_param = symbols.join(",");
    let url = format!("{}/api/market/prices?symbols={}", ApiClient::base_url(), symbols_param);

    tracing::debug!("Fetching prices");

    let response = client
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Price fetch network error");
            format!("Network error: {}", e)
        })?;

    let duration = start.elapsed();

    if response.status().is_success() {
        let result = response
            .json::<PriceResponse>()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Price response parse error");
                format!("Failed to parse response: {}", e)
            });

        if let Ok(ref prices) = result {
            tracing::debug!(
                duration_ms = duration.as_millis(),
                price_count = prices.prices.len(),
                "Prices fetched successfully"
            );
        }
        result
    } else {
        let status = response.status();
        tracing::warn!(
            status = status.as_u16(),
            duration_ms = duration.as_millis(),
            "Price fetch failed"
        );
        Err(format!("Failed to fetch prices: {}", status))
    }
}

/// Get available token list for swapping.
pub async fn get_token_list(
    client: &ApiClient,
) -> Result<Vec<TokenListItem>, String> {
    let url = format!("{}/api/market/tokens", ApiClient::base_url());

    let response = client
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json::<TokenListResponse>()
            .await
            .map(|resp| resp.tokens)
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        Err(format!("Failed to fetch token list: {}", response.status()))
    }
}

/// Get OHLC candlestick data for a token.
#[tracing::instrument(skip(client), fields(symbol = %symbol, timeframe = %timeframe))]
pub async fn get_candles(
    client: &ApiClient,
    symbol: &str,
    timeframe: &str,
    limit: usize,
) -> Result<Vec<shared::dto::market::OHLC>, String> {
    let url = format!(
        "{}/api/market/candles?symbol={}&timeframe={}&limit={}",
        ApiClient::base_url(),
        symbol,
        timeframe,
        limit
    );

    let start = std::time::Instant::now();
    tracing::debug!(
        url = %url,
        symbol = %symbol,
        timeframe = %timeframe,
        limit = limit,
        "Fetching candles from API"
    );

    let response = client
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            let duration = start.elapsed();
            tracing::error!(
                error = %e,
                url = %url,
                symbol = %symbol,
                timeframe = %timeframe,
                duration_ms = duration.as_millis(),
                "Candle fetch network error"
            );
            format!("Network error: {}", e)
        })?;

    let status = response.status();
    let duration = start.elapsed();

    if status.is_success() {
        let candles = response
            .json::<Vec<shared::dto::market::OHLC>>()
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    url = %url,
                    symbol = %symbol,
                    timeframe = %timeframe,
                    duration_ms = duration.as_millis(),
                    "Candle response parse error"
                );
                format!("Failed to parse response: {}", e)
            })?;
        
        tracing::debug!(
            url = %url,
            symbol = %symbol,
            timeframe = %timeframe,
            count = candles.len(),
            status = status.as_u16(),
            duration_ms = duration.as_millis(),
            "Candles fetched successfully"
        );
        
        Ok(candles)
    } else {
        tracing::warn!(
            url = %url,
            symbol = %symbol,
            timeframe = %timeframe,
            status = status.as_u16(),
            duration_ms = duration.as_millis(),
            "Candle fetch failed with non-success status"
        );
        Err(format!("Failed to fetch candles: {}", status))
    }
}

// ==================== MARKET DATA TYPES ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_24h: Option<f64>,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResponse {
    pub prices: HashMap<String, PriceData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenListItem {
    pub symbol: String,
    pub name: String,
    pub mint: String,
    pub decimals: u8,
    pub logo_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenListResponse {
    pub tokens: Vec<TokenListItem>,
}

