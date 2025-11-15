//! # WebSocket Client for Real-Time Price Updates
//!
//! Handles WebSocket connection to backend for streaming price updates.

use crate::app::{AppEvent, AppState, WebSocketState};
use crate::app::PriceData;
use async_channel::Sender;
use std::sync::Arc;
use parking_lot::RwLock;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn, trace};

/// Price update message from WebSocket server
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PriceUpdateMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: PriceUpdateData,
}

/// Price update data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PriceUpdateData {
    pub symbol: String,
    pub mint: String,
    pub price: f64,
    pub source: String,
    pub timestamp: u64,
}

/// WebSocket URL for price streaming
fn price_stream_url() -> String {
    let base_url = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:3001".to_string());
    base_url
        .replace("http://", "ws://")
        .replace("https://", "wss://")
        + "/api/ws/prices"
}

/// Connect to price stream WebSocket and forward updates to event channel.
///
/// This function handles:
/// - WebSocket connection establishment
/// - Automatic reconnection on disconnect (with max retry limit)
/// - Message parsing and forwarding
/// - Error handling and logging
///
/// # Arguments
/// * `event_tx` - Channel sender for price update events
///
/// Global counter for total price update messages received
pub static MESSAGE_COUNTER: AtomicU64 = AtomicU64::new(0);
/// Counter for reconnection attempts
pub static RECONNECT_COUNTER: AtomicU64 = AtomicU64::new(0);
/// Maximum number of connection attempts before giving up
const MAX_CONNECTION_ATTEMPTS: u64 = 5;
/// Flag to track if WebSocket is disabled due to repeated failures
static WEBSOCKET_DISABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub async fn connect_price_stream(event_tx: Sender<AppEvent>, app_state: Option<Arc<RwLock<AppState>>>) {
    // Check if WebSocket is disabled
    if WEBSOCKET_DISABLED.load(Ordering::Relaxed) {
        warn!("WebSocket connection is disabled due to repeated failures. Price updates will not be available.");
        return;
    }

    let url = price_stream_url();
    info!(url = %url, "Connecting to price stream WebSocket");
    
    // Clone app_state for use in the loop (needed because it's moved into the connection handler)
    let app_state_for_loop = app_state.clone();
    
    // Update status to Connecting
    if let Some(state) = app_state_for_loop.as_ref() {
        let mut ws_status = state.write().websocket_status.clone();
        ws_status.state = WebSocketState::Connecting;
        ws_status.connection_attempts += 1;
        state.write().websocket_status = ws_status.clone();
        let _ = event_tx.send(AppEvent::WebSocketStatusUpdate(ws_status)).await;
    }
    
    let mut reconnect_delay = Duration::from_secs(1);
    const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(60);
    let mut total_attempts = 0u64;
    
    loop {
        total_attempts += 1;
        let attempt = RECONNECT_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
        
        // Update status to Reconnecting if not first attempt
        if total_attempts > 1 {
            if let Some(state) = app_state_for_loop.as_ref() {
                let mut ws_status = state.write().websocket_status.clone();
                ws_status.state = WebSocketState::Reconnecting;
                ws_status.connection_attempts = total_attempts;
                state.write().websocket_status = ws_status.clone();
                let _ = event_tx.send(AppEvent::WebSocketStatusUpdate(ws_status)).await;
            }
        }
        
        // Check if we've exceeded max attempts
        if total_attempts > MAX_CONNECTION_ATTEMPTS {
            error!(
                url = %url,
                total_attempts = total_attempts,
                "WebSocket connection failed after {} attempts. Disabling WebSocket to prevent UI blocking.",
                MAX_CONNECTION_ATTEMPTS
            );
            WEBSOCKET_DISABLED.store(true, Ordering::Relaxed);
            
            // Update status to Disabled
            if let Some(state) = app_state_for_loop.as_ref() {
                let mut ws_status = state.write().websocket_status.clone();
                ws_status.state = WebSocketState::Disabled;
                state.write().websocket_status = ws_status.clone();
                let _ = event_tx.send(AppEvent::WebSocketStatusUpdate(ws_status)).await;
            }
            
            // Send notification to UI
            let _ = event_tx.send(AppEvent::Loading(format!(
                "NOTIFY_WARNING:WebSocket connection unavailable after {} attempts. Price updates disabled.",
                MAX_CONNECTION_ATTEMPTS
            ))).await;
            return;
        }
        match connect_async(&url).await {
            Ok((ws_stream, response)) => {
                info!(
                    url = %url,
                    status = ?response.status(),
                    attempt = attempt,
                    "WebSocket connection established successfully"
                );
                reconnect_delay = Duration::from_secs(1); // Reset delay on successful connection
                RECONNECT_COUNTER.store(0, Ordering::Relaxed); // Reset counter on success
                
                // Update status to Connected
                if let Some(state) = app_state_for_loop.as_ref() {
                    let mut ws_status = state.write().websocket_status.clone();
                    ws_status.state = WebSocketState::Connected;
                    ws_status.last_connected = Some(std::time::Instant::now());
                    ws_status.last_error = None;
                    state.write().websocket_status = ws_status.clone();
                    state.write().websocket_connected = true;
                    let _ = event_tx.send(AppEvent::WebSocketStatusUpdate(ws_status)).await;
                }
                
                // Send connection success event to update UI state
                // Note: We can't update state directly here, but the connection is established
                // The UI should check websocket_connected flag which is set when connection starts
                
                let (mut write, mut read) = ws_stream.split();
                
                // Spawn task to handle incoming messages
                let event_tx_clone = event_tx.clone();
                let app_state_for_read = app_state_for_loop.clone();
                let read_task = tokio::spawn(async move {
                    let mut message_count = 0u64;
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                debug!(
                                    message_length = text.len(),
                                    message_preview = if text.len() > 200 { format!("{}...", &text[..200]) } else { text.clone() },
                                    "Received WebSocket text message"
                                );
                                match serde_json::from_str::<PriceUpdateMessage>(&text) {
                                    Ok(update) => {
                                        debug!(
                                            message_type = %update.message_type,
                                            symbol = %update.data.symbol,
                                            price = update.data.price,
                                            "Parsed WebSocket message successfully"
                                        );
                                        if update.message_type == "price_update" {
                                            message_count += 1;
                                            let total_messages = MESSAGE_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
                                            
                                            let price_data = PriceData {
                                                symbol: update.data.symbol.clone(),
                                                price: update.data.price,
                                                change_24h: 0.0, // 24h change not in stream
                                                previous_price: None,
                                                source: Some(update.data.source.clone()),
                                            };
                                            
                                            info!(
                                                symbol = %update.data.symbol,
                                                price = update.data.price,
                                                source = %update.data.source,
                                                message_count = message_count,
                                                total_messages = total_messages,
                                                "Price update received from WebSocket - preparing to send to event channel"
                                            );
                                            
                                            // Update message count in status
                                            if let Some(state) = app_state_for_read.as_ref() {
                                                let mut ws_status = state.write().websocket_status.clone();
                                                let old_count = ws_status.messages_received;
                                                ws_status.messages_received += 1;
                                                ws_status.last_message = Some(std::time::Instant::now());
                                                state.write().websocket_status = ws_status.clone();
                                                debug!(
                                                    old_count = old_count,
                                                    new_count = ws_status.messages_received,
                                                    "Updated WebSocket status - message count incremented"
                                                );
                                                match event_tx_clone.send(AppEvent::WebSocketStatusUpdate(ws_status)).await {
                                                    Ok(_) => {
                                                        debug!("WebSocket status update event sent successfully");
                                                    }
                                                    Err(e) => {
                                                        error!(error = %e, "Failed to send WebSocket status update event");
                                                    }
                                                }
                                            } else {
                                                warn!("App state not available for WebSocket status update");
                                            }
                                            
                                            // Send single price update - CRITICAL for real-time updates
                                            debug!(
                                                symbol = %price_data.symbol,
                                                price = price_data.price,
                                                timestamp = price_data.change_24h, // Using as placeholder for timestamp
                                                "Sending PriceUpdated event to event channel for immediate processing"
                                            );
                                            match event_tx_clone.send(AppEvent::PriceUpdated(price_data)).await {
                                                Ok(_) => {
                                                    // Log at debug level to avoid spam, but ensure we can track if needed
                                                    debug!(
                                                        symbol = %update.data.symbol,
                                                        price = update.data.price,
                                                        message_count = message_count,
                                                        "PriceUpdated event sent successfully - will trigger immediate UI repaint"
                                                    );
                                                }
                                                Err(e) => {
                                                    error!(
                                                        error = %e,
                                                        symbol = %update.data.symbol,
                                                        price = update.data.price,
                                                        message_count = message_count,
                                                        "CRITICAL: Failed to send PriceUpdated event to event channel - price update will be lost"
                                                    );
                                                }
                                            }
                                        } else {
                                            debug!(
                                                message_type = %update.message_type,
                                                "Received non-price-update message, ignoring"
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        warn!(
                                            error = %e,
                                            message_length = text.len(),
                                            message_preview = if text.len() > 100 { format!("{}...", &text[..100]) } else { text.clone() },
                                            "Failed to parse price update JSON - message may be malformed"
                                        );
                                    }
                                }
                            }
                            Ok(Message::Close(frame)) => {
                                let close_code = frame.as_ref().map(|f| f.code);
                                let close_reason = frame.as_ref().map(|f| f.reason.to_string());
                                info!(
                                    code = ?close_code,
                                    reason = ?close_reason,
                                    message_count = message_count,
                                    "WebSocket connection closed by server"
                                );
                                break;
                            }
                            Ok(Message::Ping(data)) => {
                                trace!(data_len = data.len(), "Received ping, sending pong");
                                // Send pong response
                                if let Err(e) = write.send(Message::Pong(data)).await {
                                    error!(
                                        error = %e,
                                        "Failed to send pong response"
                                    );
                                    break;
                                }
                            }
                            Ok(Message::Pong(_)) => {
                                trace!("Received pong, connection is alive");
                            }
                            Err(e) => {
                                error!(
                                    error = %e,
                                    message_count = message_count,
                                    "WebSocket read error"
                                );
                                break;
                            }
                            _ => {
                                trace!("Received other WebSocket message type");
                            }
                        }
                    }
                    info!(
                        message_count = message_count,
                        "WebSocket read task ended"
                    );
                });
                
                // Wait for read task to complete (connection closed)
                read_task.await.ok();
                warn!(
                    attempt = attempt,
                    "WebSocket connection lost, reconnecting..."
                );
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                let error_description = if error_msg.contains("500") {
                    "HTTP error: 500 Internal Server Error"
                } else {
                    &error_msg
                };
                
                error!(
                    url = %url,
                    error = %e,
                    attempt = attempt,
                    total_attempts = total_attempts,
                    max_attempts = MAX_CONNECTION_ATTEMPTS,
                    reconnect_delay_secs = reconnect_delay.as_secs(),
                    "Failed to connect to price stream WebSocket, url: {}, error: {}, attempt: {}, reconnect_delay_secs: {}",
                    url,
                    error_description,
                    attempt,
                    reconnect_delay.as_secs()
                );
                
                // Update status with error
                if let Some(state) = app_state_for_loop.as_ref() {
                    let mut ws_status = state.write().websocket_status.clone();
                    ws_status.last_error = Some(error_msg.clone());
                    ws_status.state = if total_attempts >= MAX_CONNECTION_ATTEMPTS {
                        WebSocketState::Disabled
                    } else {
                        WebSocketState::Reconnecting
                    };
                    state.write().websocket_status = ws_status.clone();
                    let _ = event_tx.send(AppEvent::WebSocketStatusUpdate(ws_status)).await;
                }
                
                // Check if we should give up
                if total_attempts >= MAX_CONNECTION_ATTEMPTS {
                    error!(
                        url = %url,
                        total_attempts = total_attempts,
                        "Maximum connection attempts reached. Disabling WebSocket."
                    );
                    WEBSOCKET_DISABLED.store(true, Ordering::Relaxed);
                    // Send notification to UI
                    let _ = event_tx.send(AppEvent::Loading(format!(
                        "NOTIFY_WARNING:WebSocket connection failed after {} attempts. Price updates disabled.",
                        MAX_CONNECTION_ATTEMPTS
                    ))).await;
                    return;
                }
            }
        }
        
        // Exponential backoff for reconnection (only if we haven't exceeded max attempts)
        if total_attempts < MAX_CONNECTION_ATTEMPTS {
            info!(
                attempt = attempt,
                total_attempts = total_attempts,
                max_attempts = MAX_CONNECTION_ATTEMPTS,
                delay_secs = reconnect_delay.as_secs(),
                "Reconnecting in {}s..., attempt: {}, delay_secs: {}",
                reconnect_delay.as_secs(),
                attempt,
                reconnect_delay.as_secs()
            );
            sleep(reconnect_delay).await;
            reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY);
        } else {
            break;
        }
    }
}

/// Reset WebSocket disabled flag (for testing or manual retry)
#[allow(dead_code)]
pub fn reset_websocket_disabled() {
    WEBSOCKET_DISABLED.store(false, Ordering::Relaxed);
    RECONNECT_COUNTER.store(0, Ordering::Relaxed);
}

