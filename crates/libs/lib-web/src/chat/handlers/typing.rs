//! # Typing Indicator Handler
//!
//! Handler for typing indicator events.

use super::utils::{extract_user_id_from_token, get_username};
use crate::chat::state::ChatAppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use std::sync::Arc;

/// Handle typing indicator event
pub async fn handle_typing_event(
    Path(conversation_id): Path<String>,
    State(app_state): State<Arc<ChatAppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<Response<Body>, StatusCode> {
    // Authenticate user
    let user_id = extract_user_id_from_token(&headers, &app_state.config)?;
    
    // Get username
    let username = get_username(&app_state.db, user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Parse request body
    #[derive(serde::Deserialize)]
    struct TypingRequest {
        is_typing: bool,
    }
    
    let typing_request: TypingRequest = serde_json::from_slice(&body)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Broadcast typing event
    app_state.broadcast_typing(&conversation_id, user_id, username, typing_request.is_typing).await;
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}

