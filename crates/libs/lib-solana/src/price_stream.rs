//! # Real-Time Price Stream Server
//!
//! WebSocket server that streams real-time price updates for all tokens from Jupiter API.
//!
//! ## Features
//! - Sub-second price updates (500ms-1s polling)
//! - Supports all tokens from Jupiter token list
//! - Broadcasts updates to all connected WebSocket clients
//! - Automatic reconnection handling
//! - Rate limiting to respect Jupiter API limits

use crate::jupiter::JupiterClient;
use crate::candle_aggregator::CandleAggregator;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::Duration;
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};

/// Price update message sent to WebSocket clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdateMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: PriceUpdateData,
}

/// Price update data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdateData {
    pub symbol: String,
    pub mint: String,
    pub price: f64,
    pub source: String,
    pub timestamp: u64,
}

/// Price stream server that polls Jupiter API and broadcasts updates
pub struct PriceStreamServer {
    /// Jupiter client for fetching prices
    jupiter: Arc<JupiterClient>,
    /// Broadcast channel for price updates
    price_tx: broadcast::Sender<PriceUpdateMessage>,
    /// Tracked token symbols
    tracked_symbols: Arc<RwLock<Vec<String>>>,
    /// Update interval in milliseconds
    update_interval_ms: u64,
    /// Candle aggregator for OHLC data
    candle_aggregator: Arc<CandleAggregator>,
}

impl PriceStreamServer {
    /// Create a new price stream server.
    ///
    /// # Arguments
    /// * `jupiter` - Jupiter client for API calls
    /// * `update_interval_ms` - How often to poll Jupiter API (default: 500ms)
    ///
    /// # Returns
    /// New PriceStreamServer instance
    pub fn new(jupiter: Arc<JupiterClient>, update_interval_ms: u64) -> Self {
        let (price_tx, _) = broadcast::channel(1000); // Buffer up to 1000 messages
        let candle_aggregator = Arc::new(CandleAggregator::new(500)); // Keep last 500 candles per timeframe
        
        Self {
            jupiter,
            price_tx,
            tracked_symbols: Arc::new(RwLock::new(Vec::new())),
            update_interval_ms,
            candle_aggregator,
        }
    }

    /// Get reference to candle aggregator
    pub fn candle_aggregator(&self) -> Arc<CandleAggregator> {
        Arc::clone(&self.candle_aggregator)
    }

    /// Get a receiver for price updates (used by WebSocket handlers)
    pub fn subscribe(&self) -> broadcast::Receiver<PriceUpdateMessage> {
        self.price_tx.subscribe()
    }

    /// Start the price streaming service.
    ///
    /// This spawns a background task that:
    /// 1. Loads all tokens from Jupiter token list (with retry logic)
    /// 2. Polls Jupiter API every `update_interval_ms` for all token prices
    /// 3. Broadcasts updates to all connected WebSocket clients
    ///
    /// # Arguments
    /// * `self` - Arc-wrapped self for sharing across async context
    pub async fn start(self: Arc<Self>) -> anyhow::Result<()> {
        // Try to load token list with retry logic
        info!("Loading Jupiter token list for price streaming...");
        
        // Retry logic: try up to 3 times with exponential backoff
        for attempt in 1..=3 {
            match self.jupiter.load_token_list().await {
                Ok(()) => {
                    info!("Successfully loaded Jupiter token list (attempt {})", attempt);
                    break;
                }
                Err(e) => {
                    if attempt < 3 {
                        let delay_ms = 1000 * attempt; // 1s, 2s, 3s
                        warn!("Failed to load token list (attempt {}): {}. Retrying in {}ms...", 
                              attempt, e, delay_ms);
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    } else {
                        warn!("Failed to load token list after {} attempts: {}. Price stream will start with empty token list and retry in background.", attempt, e);
                    }
                }
            }
        }
        
        // Get all tokens and extract symbols (if available)
        let tokens = self.jupiter.get_all_tokens().await;
        
        let symbols: Vec<String> = if let Some(tokens) = tokens {
            tokens.iter()
                .map(|t| t.symbol.clone())
                .collect()
        } else {
            warn!("Token list not available. Price stream will start with empty token list.");
            Vec::new()
        };
        
        // Update tracked symbols
        *self.tracked_symbols.write().await = symbols.clone();
        if !symbols.is_empty() {
            info!("Tracking {} tokens for real-time price updates", symbols.len());
        } else {
            info!("Price stream started with empty token list. Will retry loading tokens in background.");
            
            // Spawn background task to retry loading token list
            let jupiter_clone = Arc::clone(&self.jupiter);
            let symbols_clone = Arc::clone(&self.tracked_symbols);
            tokio::spawn(async move {
                let mut retry_interval = tokio::time::interval(Duration::from_secs(30));
                retry_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                
                loop {
                    retry_interval.tick().await;
                    
                    if let Ok(()) = jupiter_clone.load_token_list().await {
                        if let Some(tokens) = jupiter_clone.get_all_tokens().await {
                            let new_symbols: Vec<String> = tokens.iter()
                                .map(|t| t.symbol.clone())
                                .collect();
                            
                            *symbols_clone.write().await = new_symbols.clone();
                            info!("Successfully loaded {} tokens in background retry", new_symbols.len());
                            break; // Stop retrying once successful
                        }
                    }
                }
            });
        }
        
        // Spawn background polling task
        // This task will run even if token list is empty (it will just skip until tokens are loaded)
        let server = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(server.update_interval_ms));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                interval.tick().await;
                
                let symbols = server.tracked_symbols.read().await.clone();
                if symbols.is_empty() {
                    // No tokens loaded yet - skip this cycle but continue running
                    // The background retry task will load tokens eventually
                    continue;
                }
                
                debug!("Fetching prices for {} tokens...", symbols.len());
                
                // Fetch prices in batches to avoid API rate limits
                // Jupiter API can handle up to ~100 tokens per request
                const BATCH_SIZE: usize = 100;
                for chunk in symbols.chunks(BATCH_SIZE) {
                    let symbol_refs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();
                    
                    match server.jupiter.get_prices(&symbol_refs).await {
                        Ok(prices) => {
                            // Broadcast each price update
                            for (symbol, price) in prices {
                                // Get mint address for this symbol
                                if let Some(mint) = server.jupiter.get_mint_for_symbol(&symbol).await {
                                    let timestamp = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    
                                    // Update candle aggregator (non-blocking, errors are logged but don't stop the stream)
                                    let candle_agg = Arc::clone(&server.candle_aggregator);
                                    let symbol_clone = symbol.clone();
                                    tokio::spawn(async move {
                                        candle_agg.add_price_update(&symbol_clone, price, timestamp).await;
                                    });
                                    
                                    let update = PriceUpdateMessage {
                                        message_type: "price_update".to_string(),
                                        data: PriceUpdateData {
                                            symbol: symbol.clone(),
                                            mint,
                                            price,
                                            source: "jupiter".to_string(),
                                            timestamp,
                                        },
                                    };
                                    
                                    // Broadcast to all subscribers (non-blocking)
                                    // If send fails (no subscribers), that's fine - just continue
                                    if server.price_tx.send(update).is_err() {
                                        debug!("No active WebSocket subscribers for price updates");
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to fetch prices: {}", e);
                            // Continue to next batch - don't stop the entire stream
                        }
                    }
                    
                    // Small delay between batches to respect rate limits
                    if chunk.len() == BATCH_SIZE {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
                
                debug!("Price update cycle completed");
            }
        });
        
        info!("Price stream server started ({}ms interval)", self.update_interval_ms);
        Ok(())
    }

    /// Add tokens to track (dynamically add new tokens)
    pub async fn add_tokens(&self, symbols: &[&str]) {
        let mut tracked = self.tracked_symbols.write().await;
        for symbol in symbols {
            let symbol_upper = symbol.to_uppercase();
            if !tracked.contains(&symbol_upper) {
                tracked.push(symbol_upper);
            }
        }
        info!("Now tracking {} tokens", tracked.len());
    }

    /// Remove tokens from tracking
    pub async fn remove_tokens(&self, symbols: &[&str]) {
        let mut tracked = self.tracked_symbols.write().await;
        for symbol in symbols {
            let symbol_upper = symbol.to_uppercase();
            tracked.retain(|s| s != &symbol_upper);
        }
        info!("Now tracking {} tokens", tracked.len());
    }
}


