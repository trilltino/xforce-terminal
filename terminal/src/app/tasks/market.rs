//! # Market Data Tasks
//!
//! Async tasks for fetching market data including prices and token lists.

use crate::app::state::{AppState, PriceData, TokenInfo};
use crate::app::events::AppEvent;
use crate::core::service::ApiService;
use async_channel::Sender;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::spawn;
use tracing::{info, debug, warn};

/// Fetch prices from backend API
///
/// Internal task function - spawns async task to fetch prices and send results via event channel.
pub(crate) fn fetch_prices(
    state: Arc<RwLock<AppState>>,
    event_tx: Sender<AppEvent>,
) {
    // Check if already fetching and get API client with minimal lock duration
    let should_fetch = {
        let mut state = state.write();

        // Skip if already fetching (prevents task pileup)
        if state.terminal.fetching_prices {
            return;
        }

        state.terminal.fetching_prices = true;
        state.terminal.last_price_update = std::time::Instant::now();
        state.api_client.clone()
    }; // Lock released here

    if let Some(api_client) = should_fetch {
        let state_arc = Arc::clone(&state);

        spawn(async move {
            let symbols = ["SOL", "USDC", "BTC", "ETH", "USDT", "JUP", "RAY"];
            let result = api_client.get_prices(&symbols).await;

            // Always reset fetching flag when done
            // CRITICAL: Release lock immediately to prevent deadlock with main thread
            {
                let mut state = state_arc.write();
                state.terminal.fetching_prices = false;
                // Lock released here automatically
            }

            match result {
                Ok(response) => {
                    let prices: Vec<PriceData> = response
                        .prices
                        .iter()
                        .map(|(symbol, data)| PriceData {
                            symbol: symbol.clone(),
                            price: data.price,
                            change_24h: data.change_24h.unwrap_or(0.0),
                            previous_price: None, // Will be set when event is processed
                            source: Some(data.source.clone()),
                        })
                        .collect();
                    tracing::info!(
                        price_count = prices.len(),
                        symbols = ?prices.iter().map(|p| p.symbol.clone()).collect::<Vec<_>>(),
                        "REST API: Fetched prices successfully - sending to event channel"
                    );
                    let _ = event_tx.send(AppEvent::PricesUpdated(prices)).await;
                }
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        "REST API: Failed to fetch prices - will retry on next fallback trigger"
                    );
                    // Silently fail - keep showing last known prices
                }
            }
        });
    }
}

/// Fetch token list from backend API
///
/// Internal task function - spawns async task to fetch token list and send results via event channel.
pub(crate) fn fetch_token_list(
    state: Arc<RwLock<AppState>>,
    event_tx: Sender<AppEvent>,
) {
    let api_client = {
        let state = state.read();
        state.api_client.clone()
    };

    if let Some(api_client) = api_client {
        spawn(async move {
            let result = api_client.get_token_list().await;

            match result {
                Ok(token_list) => {
                    let tokens: Vec<TokenInfo> = token_list
                        .iter()
                        .map(|token| TokenInfo {
                            symbol: token.symbol.clone(),
                            name: token.name.clone(),
                            mint: token.mint.clone(),
                            price: 0.0, // Price will be populated from price feed
                            balance: 0.0,
                            change_24h: 0.0,
                            is_favorite: false,
                        })
                        .collect();
                    let _ = event_tx.send(AppEvent::TokenListResult(Ok(tokens))).await;
                }
                Err(e) => {
                    let _ = event_tx.send(AppEvent::TokenListResult(Err(e))).await;
                }
            }
        });
    }
}

/// Fetch OHLC candlestick data for a token.
///
/// Internal task function - spawns async task to fetch candles and send results via event channel.
pub(crate) fn fetch_candles(
    state: Arc<RwLock<AppState>>,
    event_tx: Sender<AppEvent>,
    symbol: String,
    timeframe: shared::dto::market::Timeframe,
) {
    let api_client = {
        let state = state.read();
        state.api_client.clone()
    };

    if let Some(api_client) = api_client {
        let timeframe_str = match timeframe {
            shared::dto::market::Timeframe::OneMinute => "1m",
            shared::dto::market::Timeframe::FiveMinutes => "5m",
            shared::dto::market::Timeframe::FifteenMinutes => "15m",
            shared::dto::market::Timeframe::OneHour => "1h",
            shared::dto::market::Timeframe::FourHours => "4h",
            shared::dto::market::Timeframe::OneDay => "1d",
            shared::dto::market::Timeframe::OneWeek => "1w", // Not supported by backend, but handle gracefully
        };
        
        info!(
            symbol = %symbol,
            timeframe = %timeframe_str,
            limit = 100,
            "Fetching candles from API"
        );
        
        spawn(async move {
            let start = std::time::Instant::now();
            let result = api_client.get_candles(&symbol, timeframe_str, 100).await;
            let duration = start.elapsed();
            
            match &result {
                Ok(candles) => {
                    debug!(
                        symbol = %symbol,
                        timeframe = %timeframe_str,
                        count = candles.len(),
                        duration_ms = duration.as_millis(),
                        "Candles fetched successfully"
                    );
                }
                Err(e) => {
                    warn!(
                        symbol = %symbol,
                        timeframe = %timeframe_str,
                        error = %e,
                        duration_ms = duration.as_millis(),
                        "Failed to fetch candles"
                    );
                }
            }
            
            let _ = event_tx.send(AppEvent::CandlesResult(result)).await;
        });
    }
}
