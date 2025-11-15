//! # Market Data Transfer Objects
//!
//! Defines structures for market data, price charts, and OHLC (candlestick) data.
//!
//! ## Overview
//!
//! This module contains DTOs for:
//! - **OHLC data**: Candlestick chart data (Open, High, Low, Close, Volume)
//! - **Timeframes**: Chart timeframe selection (1M, 5M, 1H, 1D, etc.)
//! - **Market requests**: Requesting chart data from the API
//!
//! ## Endpoints Using These DTOs
//!
//! - `GET /api/market/prices` - Get current token prices
//! - `GET /api/market/ohlc?symbol=SOL/USDC&timeframe=OneHour&limit=100` - Get OHLC chart data
//!
//! ## Wire Format
//!
//! All DTOs use **snake_case** field names in JSON (default serde behavior).
//! Enums use custom serialization formats as noted in their documentation.
//!
//! ## Example Chart Data Request
//!
//! ```text
//! GET /api/market/ohlc?symbol=SOL/USDC&timeframe=OneHour&limit=100
//! ```
//!
//! Response:
//! ```json
//! {
//!   "symbol": "SOL/USDC",
//!   "timeframe": "OneHour",
//!   "data": [
//!     {
//!       "timestamp": 1704067200,
//!       "open": 100.50,
//!       "high": 101.20,
//!       "low": 100.30,
//!       "close": 100.95,
//!       "volume": 125000.0
//!     },
//!     {
//!       "timestamp": 1704070800,
//!       "open": 100.95,
//!       "high": 102.00,
//!       "low": 100.80,
//!       "close": 101.50,
//!       "volume": 150000.0
//!     }
//!   ]
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// OHLC (Open, High, Low, Close) candlestick data for charting.
///
/// Standard candlestick format used for financial charts.
/// Each candle represents price action over a specific time period (defined by [`Timeframe`]).
///
/// ## Usage
///
/// Used for:
/// - Candlestick charts in the terminal UI
/// - Technical analysis (trend detection, pattern recognition)
/// - Historical price data visualization
///
/// ## Helper Methods
///
/// - [`OHLC::is_bullish`] - Check if close > open (green candle)
/// - [`OHLC::is_bearish`] - Check if close < open (red candle)
/// - [`OHLC::body_size`] - Get candle body size (|close - open|)
/// - [`OHLC::total_range`] - Get total price range (high - low)
/// - [`OHLC::datetime`] - Convert timestamp to DateTime
///
/// ## JSON Example
///
/// ```json
/// {
///   "timestamp": 1704067200,
///   "open": 100.50,
///   "high": 101.20,
///   "low": 100.30,
///   "close": 100.95,
///   "volume": 125000.0
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLC {
    /// Unix timestamp in seconds (epoch time).
    ///
    /// Represents the start of the candle period.
    /// Use [`OHLC::datetime`] to convert to `DateTime<Utc>`.
    ///
    /// Example: `1704067200` = 2024-01-01 00:00:00 UTC
    pub timestamp: i64,

    /// Opening price at the start of the period.
    ///
    /// The first trade price in this time period.
    pub open: f64,

    /// Highest price reached during the period.
    ///
    /// The maximum price across all trades in this candle.
    pub high: f64,

    /// Lowest price reached during the period.
    ///
    /// The minimum price across all trades in this candle.
    pub low: f64,

    /// Closing price at the end of the period.
    ///
    /// The last trade price in this time period.
    /// When `close > open`, the candle is bullish (green).
    /// When `close < open`, the candle is bearish (red).
    pub close: f64,

    /// Trading volume during the period.
    ///
    /// Total amount of asset traded in this time period.
    /// Higher volume indicates more market activity.
    pub volume: f64,
}

impl OHLC {
    pub fn new(timestamp: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    /// Get datetime from timestamp.
    ///
    /// Returns a `DateTime<Utc>` representation of the timestamp.
    /// If the timestamp is invalid, returns Unix epoch (1970-01-01 00:00:00 UTC).
    pub fn datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.timestamp, 0)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).expect("Unix epoch should be valid"))
    }

    /// Check if candle is bullish (close > open)
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Check if candle is bearish (close < open)
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    /// Get body size (absolute difference between open and close)
    pub fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Get wick size (total range)
    pub fn total_range(&self) -> f64 {
        self.high - self.low
    }
}

/// Chart timeframe for aggregating OHLC data.
///
/// Defines the time period that each OHLC candle represents.
/// Smaller timeframes show more granular data, larger timeframes show longer trends.
///
/// ## Serialization
///
/// Serializes to **PascalCase** variant names in JSON (e.g., "OneMinute", "FiveMinutes").
/// This is the default serde behavior for enums without `#[serde(rename_all)]`.
///
/// ## Helper Methods
///
/// - [`Timeframe::duration_secs`] - Get duration in seconds
/// - [`Timeframe::label`] - Get short display label (e.g., "1M", "1H", "1D")
///
/// ## JSON Examples
///
/// ```json
/// "OneMinute"
/// ```
/// ```json
/// "OneHour"
/// ```
/// ```json
/// "OneDay"
/// ```
///
/// ## Usage in Requests
///
/// ```text
/// GET /api/market/ohlc?symbol=SOL/USDC&timeframe=OneHour&limit=100
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Timeframe {
    /// 1 minute candles (60 seconds).
    ///
    /// Best for: Day trading, scalping, very short-term analysis.
    /// Each candle = 1 minute of trading activity.
    OneMinute,

    /// 5 minute candles (300 seconds).
    ///
    /// Best for: Short-term trading, intraday analysis.
    /// Each candle = 5 minutes of trading activity.
    FiveMinutes,

    /// 15 minute candles (900 seconds).
    ///
    /// Best for: Swing trading, short-term trends.
    /// Each candle = 15 minutes of trading activity.
    FifteenMinutes,

    /// 1 hour candles (3600 seconds).
    ///
    /// Best for: Medium-term analysis, daily trading.
    /// Each candle = 1 hour of trading activity.
    OneHour,

    /// 4 hour candles (14400 seconds).
    ///
    /// Best for: Swing trading, multi-day trends.
    /// Each candle = 4 hours of trading activity.
    FourHours,

    /// 1 day candles (86400 seconds).
    ///
    /// Best for: Long-term analysis, weekly/monthly trends.
    /// Each candle = 1 day (24 hours) of trading activity.
    OneDay,

    /// 1 week candles (604800 seconds).
    ///
    /// Best for: Very long-term analysis, major trend identification.
    /// Each candle = 1 week (7 days) of trading activity.
    OneWeek,
}

impl Timeframe {
    /// Get duration in seconds
    pub fn duration_secs(&self) -> i64 {
        match self {
            Timeframe::OneMinute => 60,
            Timeframe::FiveMinutes => 300,
            Timeframe::FifteenMinutes => 900,
            Timeframe::OneHour => 3600,
            Timeframe::FourHours => 14400,
            Timeframe::OneDay => 86400,
            Timeframe::OneWeek => 604800,
        }
    }

    /// Get display label for Bloomberg-style UI
    pub fn label(&self) -> &'static str {
        match self {
            Timeframe::OneMinute => "1M",
            Timeframe::FiveMinutes => "5M",
            Timeframe::FifteenMinutes => "15M",
            Timeframe::OneHour => "1H",
            Timeframe::FourHours => "4H",
            Timeframe::OneDay => "1D",
            Timeframe::OneWeek => "1W",
        }
    }
}

/// Request for OHLC chart data.
///
/// Used to request historical candlestick data for charting.
/// Typically sent as query parameters to `GET /api/market/ohlc`.
///
/// # Fields
///
/// * `symbol` - Trading pair symbol (e.g., "SOL/USDC", "BTC/USD")
/// * `timeframe` - Candle period (see [`Timeframe`])
/// * `limit` - Optional max number of candles (default 100, max varies by endpoint)
///
/// # Query Parameter Example
///
/// ```text
/// GET /api/market/ohlc?symbol=SOL/USDC&timeframe=OneHour&limit=200
/// ```
///
/// # JSON Example (if sent as body)
///
/// ```json
/// {
///   "symbol": "SOL/USDC",
///   "timeframe": "OneHour",
///   "limit": 200
/// }
/// ```
///
/// # Limit Behavior
///
/// - If `limit` is `None`, server defaults to 100 candles
/// - Server may enforce a maximum limit (e.g., 1000 candles)
/// - Candles are returned in chronological order (oldest first)
#[derive(Debug, Serialize, Deserialize)]
pub struct OHLCRequest {
    pub symbol: String,
    pub timeframe: Timeframe,
    /// Number of candles to return (default 100, max varies by server).
    ///
    /// When `None`, server uses default (typically 100).
    /// Server may cap this to prevent excessive data transfer.
    pub limit: Option<usize>,
}

/// Response containing OHLC chart data.
///
/// Returned by `GET /api/market/ohlc` with the requested candlestick data.
///
/// # Fields
///
/// * `symbol` - Trading pair symbol that was requested
/// * `timeframe` - Candle period that was requested
/// * `data` - Array of OHLC candles in chronological order
///
/// # Data Ordering
///
/// Candles in `data` are ordered chronologically (oldest first).
/// This makes it easy to render charts left-to-right.
///
/// # JSON Example
///
/// ```json
/// {
///   "symbol": "SOL/USDC",
///   "timeframe": "OneHour",
///   "data": [
///     {
///       "timestamp": 1704067200,
///       "open": 100.50,
///       "high": 101.20,
///       "low": 100.30,
///       "close": 100.95,
///       "volume": 125000.0
///     },
///     {
///       "timestamp": 1704070800,
///       "open": 100.95,
///       "high": 102.00,
///       "low": 100.80,
///       "close": 101.50,
///       "volume": 150000.0
///     }
///   ]
/// }
/// ```
///
/// # Usage Example
///
/// ```rust,ignore
/// let response: OHLCResponse = reqwest::get(
///     "http://localhost:3001/api/market/ohlc?symbol=SOL/USDC&timeframe=OneHour&limit=100"
/// )
/// .await?
/// .json()
/// .await?;
///
/// for candle in response.data {
///     println!("Close: {}, Volume: {}", candle.close, candle.volume);
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct OHLCResponse {
    pub symbol: String,
    pub timeframe: Timeframe,
    pub data: Vec<OHLC>,
}
