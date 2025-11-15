//! # Chat Subscription Handler
//!
//! WebSocket subscription handler for Braid protocol.

use super::utils::{extract_user_id_from_token, parse_conversation_id};
use crate::chat::state::{ChatAppState, ChatState};
use crate::chat::db as chat_db;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::sse::{Event, Sse, KeepAlive},
};
use std::sync::Arc;
use futures_util::stream;

/// Handle Braid subscription request
pub async fn handle_braid_subscription(
    Path(conversation_id): Path<String>,
    State(app_state): State<Arc<ChatAppState>>,
    headers: HeaderMap,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, axum::Error>>>, StatusCode> {
    // Check if Subscribe header is present
    if !headers.contains_key("subscribe") {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Authenticate user
    let user_id = extract_user_id_from_token(&headers, &app_state.config)?;
    
    // Parse conversation ID to get user IDs
    let (user1_id, user2_id) = parse_conversation_id(&conversation_id)?;
    
    // Verify user is part of this conversation
    if user_id != user1_id && user_id != user2_id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Get Parents header for reconnection catch-up
    let parents_header = headers.get("parents")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    // Get or load conversation state
    let chat_state = {
        let mut states = app_state.chat_states.write().await;
        
        if let Some(state) = states.get(&conversation_id) {
            state.clone()
        } else {
            // Load from database
            let messages = chat_db::load_messages_with_usernames(&app_state.db, &conversation_id).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let mut state = ChatState::new();
            for msg in messages {
                state.messages.push(msg);
            }
            
            // Set current version from last message
            if let Some(last_msg) = state.messages.last() {
                state.current_version = last_msg.version.clone();
            }
            
            states.insert(conversation_id.clone(), state.clone());
            state
        }
    };
    
    // Get initial messages
    let initial_messages = chat_state.get_messages_since(parents_header.as_ref());
    let initial_version = chat_state.current_version.clone();
    
    // Mark conversation as read for this user
    let _ = chat_db::mark_conversation_read(&app_state.db, &conversation_id, user_id, user1_id, user2_id).await;
    
    // Subscribe to broadcast channel for real-time updates
    let broadcast_rx = app_state.get_broadcast_sender(conversation_id.as_str()).await.subscribe();
    
    // Prepare initial snapshot data
    let initial_event_data_str = {
        let event_data = serde_json::json!({
            "version": initial_version,
            "messages": initial_messages
        });
        
        serde_json::to_string(&event_data)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    
    let last_version_str = initial_version.as_ref().map(|s| s.clone()).unwrap_or_else(|| String::new());
    
    // Create stream that sends initial snapshot, then listens to broadcast channel
    let stream = stream::unfold(
        (broadcast_rx, last_version_str, false, initial_event_data_str),
        move |(mut rx, mut last_version, sent_initial, initial_data)| async move {
            // Send initial snapshot first
            if !sent_initial {
                let event = Event::default().data(initial_data);
                return Some((
                    Ok(event),
                    (rx, last_version, true, String::new()),
                ));
            }
            
            // After initial snapshot, listen to broadcast channel for new messages
            loop {
                match rx.recv().await {
                    Ok((new_messages, new_version)) => {
                        if new_version != last_version && !new_messages.is_empty() {
                            let event_data = serde_json::json!({
                                "version": new_version,
                                "messages": new_messages
                            });
                            
                            let event_data_str = match serde_json::to_string(&event_data) {
                                Ok(s) => s,
                                Err(_) => continue,
                            };
                            
                            let event = Event::default().data(event_data_str);
                            last_version = new_version.clone();
                            
                            return Some((
                                Ok(event),
                                (rx, last_version, true, String::new()),
                            ));
                        } else {
                            last_version = new_version;
                            continue;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        return None;
                    }
                }
            }
        },
    );
    
    // Create SSE response with keep-alive
    let sse = Sse::new(stream)
        .keep_alive(KeepAlive::default());
    
    Ok(sse)
}

