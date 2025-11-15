//! # Candle Aggregator Service
//!
//! Aggregates real-time price updates into OHLC (Open, High, Low, Close) candlestick data
//! for multiple timeframes. This service receives price updates from the websocket stream
//! and automatically creates/updates candles based on the timeframe.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{trace, debug, info};

/// Timeframe for candle aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timeframe {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    FourHours,
    OneDay,
}

impl Timeframe {
    /// Get timeframe duration in seconds
    pub fn seconds(&self) -> u64 {
        match self {
            Timeframe::OneMinute => 60,
            Timeframe::FiveMinutes => 300,
            Timeframe::FifteenMinutes => 900,
            Timeframe::OneHour => 3600,
            Timeframe::FourHours => 14400,
            Timeframe::OneDay => 86400,
        }
    }

    /// Get timeframe label
    pub fn label(&self) -> &'static str {
        match self {
            Timeframe::OneMinute => "1m",
            Timeframe::FiveMinutes => "5m",
            Timeframe::FifteenMinutes => "15m",
            Timeframe::OneHour => "1h",
            Timeframe::FourHours => "4h",
            Timeframe::OneDay => "1d",
        }
    }
}

/// OHLC candle data
#[derive(Debug, Clone)]
pub struct Candle {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Current candle being built for a symbol/timeframe
#[derive(Debug, Clone)]
struct CurrentCandle {
    timeframe: Timeframe,
    start_time: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    price_count: u64,
}

impl CurrentCandle {
    fn new(timeframe: Timeframe, timestamp: u64, price: f64) -> Self {
        Self {
            timeframe,
            start_time: timestamp,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 0.0,
            price_count: 1,
        }
    }

    fn update(&mut self, price: f64) {
        self.close = price;
        if price > self.high {
            self.high = price;
        }
        if price < self.low {
            self.low = price;
        }
        self.price_count += 1;
        // Volume is approximated as price * number of updates
        self.volume = self.price_count as f64 * price;
    }

    fn to_candle(&self) -> Candle {
        Candle {
            timestamp: self.start_time,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
        }
    }

    fn should_close(&self, current_time: u64) -> bool {
        let duration = self.timeframe.seconds();
        current_time >= self.start_time + duration
    }
}

/// Candle storage for a single symbol
struct SymbolCandles {
    /// Current candles being built for each timeframe
    current: HashMap<Timeframe, CurrentCandle>,
    /// Completed candles for each timeframe (limited history)
    completed: HashMap<Timeframe, Vec<Candle>>,
    max_candles: usize,
}

impl SymbolCandles {
    fn new(max_candles: usize) -> Self {
        Self {
            current: HashMap::new(),
            completed: HashMap::new(),
            max_candles,
        }
    }

    fn add_price_update(&mut self, price: f64, timestamp: u64, symbol: &str) {
        // Update all timeframes
        for timeframe in &[
            Timeframe::OneMinute,
            Timeframe::FiveMinutes,
            Timeframe::FifteenMinutes,
            Timeframe::OneHour,
            Timeframe::FourHours,
            Timeframe::OneDay,
        ] {
            let duration = timeframe.seconds();
            let candle_start = (timestamp / duration) * duration;

            // Check if we need to close current candle and start new one
            let should_start_new = if let Some(current) = self.current.get(timeframe) {
                current.start_time != candle_start
            } else {
                true
            };

            if should_start_new {
                // Close previous candle if exists
                if let Some(prev_candle) = self.current.remove(timeframe) {
                    let completed_candle = prev_candle.to_candle();
                    let completed = self.completed.entry(*timeframe).or_default();
                    completed.push(completed_candle.clone());
                    
                    // Log candle completion
                    info!(
                        symbol = %symbol,
                        timeframe = %timeframe.label(),
                        timestamp = completed_candle.timestamp,
                        open = completed_candle.open,
                        high = completed_candle.high,
                        low = completed_candle.low,
                        close = completed_candle.close,
                        volume = completed_candle.volume,
                        "Candle completed"
                    );
                    
                    // Limit history
                    if completed.len() > self.max_candles {
                        completed.remove(0);
                    }
                }

                // Start new candle
                let new_candle = CurrentCandle::new(*timeframe, candle_start, price);
                debug!(
                    symbol = %symbol,
                    timeframe = %timeframe.label(),
                    timestamp = candle_start,
                    open = price,
                    "New candle created"
                );
                self.current.insert(*timeframe, new_candle);
            } else {
                // Update current candle
                if let Some(current) = self.current.get_mut(timeframe) {
                    current.update(price);
                    trace!(
                        symbol = %symbol,
                        timeframe = %timeframe.label(),
                        price = price,
                        "Updated current candle"
                    );
                }
            }
        }
    }

    fn get_candles(&self, timeframe: Timeframe, limit: usize) -> Vec<Candle> {
        let mut result = Vec::new();
        
        // Get completed candles
        if let Some(completed) = self.completed.get(&timeframe) {
            let start = completed.len().saturating_sub(limit);
            result.extend_from_slice(&completed[start..]);
        }

        // Add current candle if it exists
        if let Some(current) = self.current.get(&timeframe) {
            result.push(current.to_candle());
        }

        result
    }

    fn get_latest_candle(&self, timeframe: Timeframe) -> Option<Candle> {
        // Prefer current candle if it exists
        if let Some(current) = self.current.get(&timeframe) {
            return Some(current.to_candle());
        }

        // Otherwise get last completed candle
        self.completed
            .get(&timeframe)
            .and_then(|candles| candles.last())
            .cloned()
    }
}

/// Candle aggregator that converts price updates into OHLC candles
pub struct CandleAggregator {
    /// Candle data per symbol
    candles: Arc<RwLock<HashMap<String, SymbolCandles>>>,
    /// Maximum number of candles to keep per symbol/timeframe
    max_candles: usize,
}

impl CandleAggregator {
    /// Create a new candle aggregator
    ///
    /// # Arguments
    /// * `max_candles` - Maximum number of candles to keep per symbol/timeframe (default: 500)
    pub fn new(max_candles: usize) -> Self {
        info!(max_candles = max_candles, "Candle aggregator created");
        Self {
            candles: Arc::new(RwLock::new(HashMap::new())),
            max_candles,
        }
    }

    /// Add a price update and update/create candles
    ///
    /// # Arguments
    /// * `symbol` - Token symbol (e.g., "SOL", "USDC")
    /// * `price` - Current price
    /// * `timestamp` - Unix timestamp in seconds
    pub async fn add_price_update(&self, symbol: &str, price: f64, timestamp: u64) {
        let symbol_upper = symbol.to_uppercase();
        let mut candles = self.candles.write().await;
        
        let is_new_symbol = !candles.contains_key(&symbol_upper);
        let symbol_candles = candles
            .entry(symbol_upper.clone())
            .or_insert_with(|| SymbolCandles::new(self.max_candles));
        
        if is_new_symbol {
            info!(symbol = %symbol_upper, "First price update received for symbol, initializing candles");
        }
        
        symbol_candles.add_price_update(price, timestamp, &symbol_upper);
        
        trace!(symbol = %symbol_upper, price = price, timestamp = timestamp, "Updated candles for symbol");
    }

    /// Get candles for a symbol and timeframe
    ///
    /// # Arguments
    /// * `symbol` - Token symbol
    /// * `timeframe` - Candle timeframe
    /// * `limit` - Maximum number of candles to return
    ///
    /// # Returns
    /// Vector of candles in chronological order (oldest first)
    pub async fn get_candles(&self, symbol: &str, timeframe: Timeframe, limit: usize) -> Vec<Candle> {
        let symbol_upper = symbol.to_uppercase();
        let candles = self.candles.read().await;
        
        candles
            .get(&symbol_upper)
            .map(|sc| sc.get_candles(timeframe, limit))
            .unwrap_or_default()
    }

    /// Get the latest candle for a symbol and timeframe
    ///
    /// # Arguments
    /// * `symbol` - Token symbol
    /// * `timeframe` - Candle timeframe
    ///
    /// # Returns
    /// Latest candle if available
    pub async fn get_latest_candle(&self, symbol: &str, timeframe: Timeframe) -> Option<Candle> {
        let symbol_upper = symbol.to_uppercase();
        let candles = self.candles.read().await;
        
        candles
            .get(&symbol_upper)
            .and_then(|sc| sc.get_latest_candle(timeframe))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_candle_aggregation() {
        let aggregator = CandleAggregator::new(100);
        let base_time = 1000000;

        // Add price updates within same minute
        aggregator.add_price_update("SOL", 100.0, base_time).await;
        aggregator.add_price_update("SOL", 101.0, base_time + 10).await;
        aggregator.add_price_update("SOL", 99.0, base_time + 20).await;
        aggregator.add_price_update("SOL", 102.0, base_time + 30).await;

        let candles = aggregator.get_candles("SOL", Timeframe::OneMinute, 10).await;
        assert_eq!(candles.len(), 1);
        let candle = &candles[0];
        assert_eq!(candle.open, 100.0);
        assert_eq!(candle.high, 102.0);
        assert_eq!(candle.low, 99.0);
        assert_eq!(candle.close, 102.0);
    }

    #[tokio::test]
    async fn test_multiple_timeframes() {
        let aggregator = CandleAggregator::new(100);
        let base_time = 1000000;

        aggregator.add_price_update("SOL", 100.0, base_time).await;
        
        // Check that all timeframes have candles
        let one_min = aggregator.get_latest_candle("SOL", Timeframe::OneMinute).await;
        let one_hour = aggregator.get_latest_candle("SOL", Timeframe::OneHour).await;
        
        assert!(one_min.is_some());
        assert!(one_hour.is_some());
    }
}

