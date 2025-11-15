//! # Market Handlers
//!
//! HTTP endpoints for fetching real-time market data including token prices and token lists.
//!
//! ## Endpoints
//!
//! - `GET /api/market/prices` - Get real-time prices for Solana tokens
//! - `GET /api/market/tokens` - Get list of available tokens with metadata
//! - `GET /api/market/candles` - Get OHLC candlestick data for charting
//!
//! ## Authentication
//!
//! These endpoints are public and do not require authentication.
//!
//! ## Request Examples
//!
//! ```bash
//! # Get prices for multiple tokens
//! curl "http://localhost:3001/api/market/prices?symbols=SOL,USDC,BTC"
//!
//! # Get full token list
//! curl http://localhost:3001/api/market/tokens
//! ```
//!
//! ## Data Sources
//!
//! - Price data is cached from Solana price feeds and Jupiter aggregator
//! - Token metadata is fetched from Jupiter token list
//! - Prices are refreshed periodically by the price cache service

use crate::services::market::MarketService;
use lib_solana::{SolanaState, types::PriceQuery, candle_aggregator::Timeframe};
use lib_solana::price_stream::PriceStreamServer;
use axum::{extract::{Query, State}, http::StatusCode, Json};
use lib_core::dto::{ErrorResponse, market::OHLC};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error, info, warn, instrument};

/// Get real-time prices for multiple Solana tokens.
///
/// **Route**: `GET /api/market/prices`
///
/// # Parameters
///
/// - `symbols` (query) - Comma-separated list of token symbols (e.g., "SOL,USDC,BTC")
///
/// # Returns
///
/// Success (200): `Json<PriceResponse>` - Map of token symbols to price data including:
/// - `price`: Current price in USD
/// - `source`: Price data source (e.g., "jupiter", "pyth")
/// - `last_updated`: Timestamp of last price update
///
/// Error (404): Token not found or no prices available
/// Error (500): Internal server error fetching prices
///
/// # Example
///
/// ```bash
/// curl "http://localhost:3001/api/market/prices?symbols=SOL,USDC"
/// ```
///
/// Response:
/// ```json
/// {
///   "prices": {
///     "SOL": {
///       "price": 24.50,
///       "source": "jupiter",
///       "last_updated": "2025-10-25T12:00:00Z"
///     },
///     "USDC": {
///       "price": 1.00,
///       "source": "jupiter",
///       "last_updated": "2025-10-25T12:00:00Z"
///     }
///   }
/// }
/// ```
#[instrument(skip(solana), fields(symbols = %params.symbols))]
pub async fn get_prices(
    State(solana): State<Arc<SolanaState>>,
    Query(params): Query<PriceQuery>,
) -> Result<(StatusCode, Json<lib_solana::types::PriceResponse>), (StatusCode, Json<ErrorResponse>)> {
    info!("[MARKET] Symbols: {}", params.symbols);
    
    let symbols: Vec<&str> = params.symbols.split(',').collect();
    let service = MarketService::new(solana);
    
    let response = service.get_prices(&symbols).await.map_err(|e| {
        error!("[MARKET] Failed to get prices: {}", e);
        let status = if e.to_string().contains("No prices available") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(ErrorResponse {
            error: e.to_string(),
        }))
    })?;
    
    info!("[MARKET] Returning {} prices", response.prices.len());
    Ok((StatusCode::OK, Json(response)))
}

/// Get list of available tokens with metadata.
///
/// **Route**: `GET /api/market/tokens`
///
/// # Parameters
///
/// None
///
/// # Returns
///
/// Success (200): `Json<Vec<TokenInfo>>` - Array of token metadata including:
/// - `address`: Token mint address on Solana
/// - `symbol`: Token trading symbol (e.g., "SOL", "USDC")
/// - `name`: Full token name
/// - `decimals`: Number of decimal places
/// - `logoURI`: URL to token logo image (optional)
///
/// Error (500): Failed to fetch token list from Jupiter
///
/// # Example
///
/// ```bash
/// curl http://localhost:3001/api/market/tokens
/// ```
///
/// Response:
/// ```json
/// [
///   {
///     "address": "So11111111111111111111111111111111111111112",
///     "symbol": "SOL",
///     "name": "Wrapped SOL",
///     "decimals": 9,
///     "logoURI": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png"
///   },
///   {
///     "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
///     "symbol": "USDC",
///     "name": "USD Coin",
///     "decimals": 6,
///     "logoURI": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png"
///   }
/// ]
/// ```
#[instrument(skip(solana))]
pub async fn get_token_list(
    State(solana): State<Arc<SolanaState>>,
) -> Result<(StatusCode, Json<Vec<lib_solana::jupiter::TokenInfo>>), (StatusCode, Json<ErrorResponse>)> {
    info!("[MARKET] Token list request");
    
    let service = MarketService::new(solana);
    match service.get_token_list().await {
        Ok(tokens) => {
            info!("[MARKET] Fetched {} tokens", tokens.len());
            Ok((StatusCode::OK, Json(tokens)))
        }
        Err(e) => {
            warn!("[MARKET] Failed to fetch token list: {}. Returning empty list.", e);
            // Return empty list instead of error to allow frontend to work
            // Frontend can handle empty token list gracefully
            Ok((StatusCode::OK, Json(Vec::new())))
        }
    }
}

/// Query parameters for candle endpoint
#[derive(Debug, Deserialize)]
pub struct CandleQuery {
    /// Token symbol (e.g., "SOL", "USDC")
    pub symbol: String,
    /// Timeframe: "1m", "5m", "15m", "1h", "4h", "1d"
    pub timeframe: String,
    /// Maximum number of candles to return (default: 100)
    #[serde(default = "default_candle_limit")]
    pub limit: usize,
}

fn default_candle_limit() -> usize {
    100
}

/// Parse timeframe string to enum
fn parse_timeframe(s: &str) -> Result<Timeframe, String> {
    match s.to_lowercase().as_str() {
        "1m" => Ok(Timeframe::OneMinute),
        "5m" => Ok(Timeframe::FiveMinutes),
        "15m" => Ok(Timeframe::FifteenMinutes),
        "1h" => Ok(Timeframe::OneHour),
        "4h" => Ok(Timeframe::FourHours),
        "1d" => Ok(Timeframe::OneDay),
        _ => Err(format!("Invalid timeframe: {}. Must be one of: 1m, 5m, 15m, 1h, 4h, 1d", s)),
    }
}

/// Get OHLC candlestick data for a token.
///
/// **Route**: `GET /api/market/candles`
///
/// # Parameters
///
/// - `symbol` (query, required) - Token symbol (e.g., "SOL", "USDC")
/// - `timeframe` (query, required) - Candle timeframe: "1m", "5m", "15m", "1h", "4h", "1d"
/// - `limit` (query, optional) - Maximum number of candles to return (default: 100, max: 500)
///
/// # Returns
///
/// Success (200): `Json<Vec<OHLC>>` - Array of OHLC candles in chronological order (oldest first)
///
/// Error (400): Invalid timeframe or missing symbol
/// Error (404): No candles available for symbol
/// Error (500): Internal server error
///
/// # Example
///
/// ```bash
/// curl "http://localhost:3001/api/market/candles?symbol=SOL&timeframe=1h&limit=100"
/// ```
///
/// Response:
/// ```json
/// [
///   {
///     "timestamp": 1704067200,
///     "open": 100.50,
///     "high": 101.20,
///     "low": 100.30,
///     "close": 100.95,
///     "volume": 125000.0
///   },
///   {
///     "timestamp": 1704070800,
///     "open": 100.95,
///     "high": 101.50,
///     "low": 100.80,
///     "close": 101.25,
///     "volume": 150000.0
///   }
/// ]
/// ```
#[instrument(skip(price_stream), fields(symbol = %params.symbol, timeframe = ?params.timeframe))]
pub async fn get_candles(
    State(price_stream): State<Arc<PriceStreamServer>>,
    Query(params): Query<CandleQuery>,
) -> Result<(StatusCode, Json<Vec<OHLC>>), (StatusCode, Json<ErrorResponse>)> {
    debug!(
        symbol = %params.symbol,
        timeframe = %params.timeframe,
        limit = params.limit,
        "Candle request received"
    );
    
    // Parse timeframe
    let timeframe = parse_timeframe(&params.timeframe).map_err(|e| {
        error!(
            symbol = %params.symbol,
            timeframe = %params.timeframe,
            error = %e,
            "Invalid timeframe in candle request"
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )
    })?;
    
    // Limit maximum candles
    let limit = params.limit.min(500);
    
    // Get candles from aggregator
    let aggregator = price_stream.candle_aggregator();
    let candles = aggregator.get_candles(&params.symbol, timeframe, limit).await;
    
    debug!(
        symbol = %params.symbol,
        timeframe = %params.timeframe,
        requested_limit = params.limit,
        actual_limit = limit,
        candle_count = candles.len(),
        "Candles retrieved from aggregator"
    );
    
    if candles.is_empty() {
        warn!(
            symbol = %params.symbol,
            timeframe = %params.timeframe,
            "No candles available for symbol"
        );
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("No candles available for symbol: {}", params.symbol),
            }),
        ));
    }
    
    // Convert internal Candle to shared OHLC
    let ohlc_data: Vec<OHLC> = candles
        .into_iter()
        .map(|c| OHLC {
            timestamp: c.timestamp as i64,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        })
        .collect();
    
    info!(
        symbol = %params.symbol,
        timeframe = %params.timeframe,
        count = ohlc_data.len(),
        "Returning candles to client"
    );
    Ok((StatusCode::OK, Json(ohlc_data)))
}
