//! # WebSocket Handlers
//!
//! HTTP endpoints for WebSocket connections, primarily for real-time price streaming.
//!
//! ## Endpoints
//!
//! - `GET /api/ws/prices` - WebSocket connection for real-time price updates

use lib_solana::price_stream::{PriceStreamServer, PriceUpdateMessage};
use axum::extract::{ws::WebSocketUpgrade, State, ConnectInfo};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use std::net::SocketAddr;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket handler for real-time price streaming.
///
/// **Route**: `GET /api/ws/prices`
///
/// # Returns
///
/// WebSocket upgrade response that streams price updates in JSON format:
///
/// ```json
/// {
///   "type": "price_update",
///   "data": {
///     "symbol": "SOL",
///     "mint": "So11111111111111111111111111111111111111112",
///     "price": 145.50,
///     "source": "jupiter",
///     "timestamp": 1234567890
///   }
/// }
/// ```
///
/// # Example
///
/// ```javascript
/// const ws = new WebSocket('ws://localhost:3001/api/ws/prices');
/// ws.onmessage = (event) => {
///   const update = JSON.parse(event.data);
///   console.log(`${update.data.symbol}: $${update.data.price}`);
/// };
/// ```
pub async fn price_stream_websocket(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(price_stream): State<Arc<PriceStreamServer>>,
) -> Response {
    // Extract connection metadata from request
    let client_id = Uuid::new_v4().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    let client_ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| Some(addr.ip().to_string()));
    
    // Log connection attempt with full details
    info!(
        client_id = %client_id,
        client_ip = ?client_ip,
        user_agent = ?user_agent,
        path = "/api/ws/prices",
        price_stream_ref_count = Arc::strong_count(&price_stream),
        "[WS] CONNECT_ATTEMPT client_id={} ip={:?} user_agent={:?} path=/api/ws/prices ref_count={}",
        client_id,
        client_ip,
        user_agent,
        Arc::strong_count(&price_stream)
    );
    
    // Verify PriceStreamServer is valid (should always be valid, but check for safety)
    // The State extractor would have failed earlier if price_stream wasn't in AppState
    // This is just a defensive check
    let price_stream_ref_count = Arc::strong_count(&price_stream);
    if price_stream_ref_count == 0 {
        error!(
            client_id = %client_id,
            "[WS] INVALID_STATE client_id={} - PriceStreamServer has zero reference count",
            client_id
        );
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Price stream server is not available"
        ).into_response();
    }
    
    debug!(
        client_id = %client_id,
        price_stream_ref_count = price_stream_ref_count,
        "[WS] PriceStreamServer verified - ref_count={}",
        price_stream_ref_count
    );
    
    // Subscribe to price stream - this should never fail, but handle gracefully
    debug!(
        client_id = %client_id,
        "[WS] Subscribing to price stream..."
    );
    let price_rx = price_stream.subscribe();
    
    // Verify price stream receiver is valid
    let receiver_is_empty = price_rx.is_empty();
    if receiver_is_empty {
        warn!(
            client_id = %client_id,
            "[WS] SUBSCRIBE_EMPTY client_id={} - Price stream receiver is empty (no active stream), but allowing connection",
            client_id
        );
        // Note: We still allow the connection even if the stream is empty
        // The client will just not receive updates until the stream starts
    } else {
        debug!(
            client_id = %client_id,
            "[WS] Successfully subscribed to price stream (receiver has messages)"
        );
    }
    
    info!(
        client_id = %client_id,
        receiver_empty = receiver_is_empty,
        "[WS] UPGRADE_START client_id={} receiver_empty={} - Starting WebSocket upgrade",
        client_id,
        receiver_is_empty
    );
    
    // Perform WebSocket upgrade
    // If this fails, Axum will automatically return a 500 error
    // We log the upgrade attempt for debugging
    debug!(
        client_id = %client_id,
        "[WS] Calling ws.on_upgrade() to initiate WebSocket handshake"
    );
    ws.on_upgrade(move |socket| async move {
        let client_id_clone = client_id.clone();
        let client_id_log = client_id_clone.clone();
        let client_ip_clone = client_ip.clone();
        let user_agent_clone = user_agent.clone();
        
        info!(
            client_id = %client_id_clone,
            "[WS] UPGRADE_COMPLETE client_id={} - WebSocket upgrade completed, starting handler",
            client_id_clone
        );
        
        // Spawn handler task with panic handling
        let handle = tokio::task::spawn(async move {
            handle_price_websocket(socket, price_rx, client_id_clone, client_ip_clone, user_agent_clone).await;
        });
        
        // Wait for handler and log any panics
        match handle.await {
            Ok(_) => {
                debug!(
                    client_id = %client_id_log,
                    "[WS] HANDLER_COMPLETE client_id={} - Handler completed normally",
                    client_id_log
                );
            }
            Err(e) => {
                error!(
                    client_id = %client_id_log,
                    error = ?e,
                    "[WS] HANDLER_PANIC client_id={} error={:?} - WebSocket handler panicked",
                    client_id_log,
                    e
                );
            }
        }
    })
    .into_response()
}

/// Handle an individual WebSocket connection for price streaming.
///
/// # Arguments
/// * `socket` - WebSocket stream
/// * `price_rx` - Receiver for price updates from the stream server
/// * `client_id` - Unique identifier for this connection
/// * `client_ip` - Client IP address if available
/// * `user_agent` - User agent string if available
async fn handle_price_websocket(
    socket: axum::extract::ws::WebSocket,
    mut price_rx: tokio::sync::broadcast::Receiver<PriceUpdateMessage>,
    client_id: String,
    client_ip: Option<String>,
    _user_agent: Option<String>,
) {
    let (mut sender, mut receiver) = socket.split();
    let connection_start = Instant::now();
    let messages_sent = Arc::new(AtomicU64::new(0));
    let messages_received = Arc::new(AtomicU64::new(0));
    
    info!(
        client_id = %client_id,
        client_ip = ?client_ip,
        "[WS] CONNECTED client_id={} ip={:?} - WebSocket connection established",
        client_id,
        client_ip
    );
    
    // Spawn task to send price updates to client
    let client_id_send = client_id.clone();
    let messages_sent_send = Arc::clone(&messages_sent);
    let mut send_task = tokio::spawn(async move {
        while let Ok(update) = price_rx.recv().await {
            let json = match serde_json::to_string(&update) {
                Ok(json) => json,
                Err(e) => {
                    error!(
                        client_id = %client_id_send,
                        error = %e,
                        "[WS] SERIALIZE_ERROR client_id={} error={}",
                        client_id_send,
                        e
                    );
                    continue;
                }
            };
            
            let message_size = json.len();
            match sender.send(axum::extract::ws::Message::Text(json.into())).await {
                Ok(_) => {
                    let count = messages_sent_send.fetch_add(1, Ordering::Relaxed) + 1;
                    debug!(
                        client_id = %client_id_send,
                        message_type = "price_update",
                        message_size,
                        total_sent = count,
                        "[WS] MESSAGE_SENT client_id={} type=price_update size={} total={}",
                        client_id_send,
                        message_size,
                        count
                    );
                }
                Err(e) => {
                    let count = messages_sent_send.load(Ordering::Relaxed);
                    warn!(
                        client_id = %client_id_send,
                        error = %e,
                        messages_sent = count,
                        "[WS] SEND_ERROR client_id={} error={} messages_sent={}",
                        client_id_send,
                        e,
                        count
                    );
                    break;
                }
            }
        }
    });
    
    // Handle incoming messages from client (ping/pong, close, etc.)
    let client_id_recv = client_id.clone();
    let messages_received_recv = Arc::clone(&messages_received);
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(axum::extract::ws::Message::Close(frame)) => {
                    let close_reason = frame
                        .as_ref()
                        .and_then(|f| f.code.into())
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    info!(
                        client_id = %client_id_recv,
                        reason = %close_reason,
                        "[WS] CLOSE_RECEIVED client_id={} reason={}",
                        client_id_recv,
                        close_reason
                    );
                    break;
                }
                Ok(axum::extract::ws::Message::Ping(data)) => {
                    messages_received_recv.fetch_add(1, Ordering::Relaxed);
                    debug!(
                        client_id = %client_id_recv,
                        data_size = data.len(),
                        "[WS] PING_RECEIVED client_id={} size={}",
                        client_id_recv,
                        data.len()
                    );
                }
                Ok(axum::extract::ws::Message::Pong(data)) => {
                    messages_received_recv.fetch_add(1, Ordering::Relaxed);
                    debug!(
                        client_id = %client_id_recv,
                        data_size = data.len(),
                        "[WS] PONG_RECEIVED client_id={} size={}",
                        client_id_recv,
                        data.len()
                    );
                }
                Ok(axum::extract::ws::Message::Text(text)) => {
                    messages_received_recv.fetch_add(1, Ordering::Relaxed);
                    info!(
                        client_id = %client_id_recv,
                        message_size = text.len(),
                        message = %text,
                        "[WS] MESSAGE_RECEIVED client_id={} size={} message={}",
                        client_id_recv,
                        text.len(),
                        text
                    );
                    // Could implement subscription filtering here if needed
                }
                Ok(axum::extract::ws::Message::Binary(data)) => {
                    messages_received_recv.fetch_add(1, Ordering::Relaxed);
                    info!(
                        client_id = %client_id_recv,
                        message_size = data.len(),
                        "[WS] BINARY_RECEIVED client_id={} size={}",
                        client_id_recv,
                        data.len()
                    );
                }
                Err(e) => {
                    error!(
                        client_id = %client_id_recv,
                        error = %e,
                        "[WS] RECV_ERROR client_id={} error={}",
                        client_id_recv,
                        e
                    );
                    break;
                }
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        result = &mut send_task => {
            recv_task.abort();
            if let Err(e) = result {
                error!(
                    client_id = %client_id,
                    error = ?e,
                    "[WS] SEND_TASK_ERROR client_id={} error={:?}",
                    client_id,
                    e
                );
            }
        }
        result = &mut recv_task => {
            send_task.abort();
            if let Err(e) = result {
                error!(
                    client_id = %client_id,
                    error = ?e,
                    "[WS] RECV_TASK_ERROR client_id={} error={:?}",
                    client_id,
                    e
                );
            }
        }
    }
    
    let duration = connection_start.elapsed();
    let sent_count = messages_sent.load(Ordering::Relaxed);
    let received_count = messages_received.load(Ordering::Relaxed);
    info!(
        client_id = %client_id,
        client_ip = ?client_ip,
        duration_secs = duration.as_secs_f64(),
        duration_ms = duration.as_millis(),
        messages_sent = sent_count,
        messages_received = received_count,
        "[WS] DISCONNECTED client_id={} ip={:?} duration={:.2}s ({}ms) messages_sent={} messages_received={}",
        client_id,
        client_ip,
        duration.as_secs_f64(),
        duration.as_millis(),
        sent_count,
        received_count
    );
}
