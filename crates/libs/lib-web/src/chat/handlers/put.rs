//! # Chat Message Creation Handler
//!
//! Handler for creating new messages via Braid PUT protocol.

use super::utils::{extract_user_id_from_token, parse_conversation_id, check_friendship, get_username};
use crate::chat::state::{ChatAppState, ChatState};
use crate::chat::db as chat_db;
use lib_core::dto::Message;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use std::sync::Arc;

/// Handle Braid PUT request
pub async fn handle_braid_put(
    Path(conversation_id): Path<String>,
    State(app_state): State<Arc<ChatAppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<Response<Body>, StatusCode> {
    // Authenticate user
    let user_id = extract_user_id_from_token(&headers, &app_state.config)?;
    
    // Parse conversation ID
    let (user1_id, user2_id) = parse_conversation_id(&conversation_id)?;
    
    // Verify user is part of this conversation
    if user_id != user1_id && user_id != user2_id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Check if this is a conversation with the AI bot
    // User ID 0 is reserved for the AI bot, or check for system@ai.bot user
    let is_ai_bot_conversation = {
        // First check if user_id 0 is involved (AI bot uses ID 0)
        if user1_id == 0 || user2_id == 0 {
            true
        } else {
            // Otherwise check if bot user exists in database
            let bot_user_id_result = sqlx::query_scalar::<_, i64>(
                r#"
                SELECT id
                FROM users
                WHERE email = 'system@ai.bot'
                LIMIT 1
                "#
            )
            .fetch_optional(&app_state.db)
            .await
            .unwrap_or(None);
            
            if let Some(bot_user_id) = bot_user_id_result {
                user_id == bot_user_id || user1_id == bot_user_id || user2_id == bot_user_id
            } else {
                false
            }
        }
    };
    
    // Check friendship status (skip for AI bot conversations)
    if !is_ai_bot_conversation {
        let friendship_status = check_friendship(&app_state.db, user1_id, user2_id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        if friendship_status != "accepted" {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    
    // Get user info for message author
    // For AI bot (user_id 0), use default name if not in database
    let username = if user_id == 0 {
        get_username(&app_state.db, user_id).await
            .unwrap_or_else(|_| "DeepSeek AI".to_string())
    } else {
        get_username(&app_state.db, user_id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    
    // Parse message from request body
    let mut message: Message = serde_json::from_slice(&body)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Validate message
    if message.text.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    const MAX_MESSAGE_LENGTH: usize = 10000;
    if message.text.len() > MAX_MESSAGE_LENGTH {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Set author info
    message.author = username;
    message.author_id = user_id;
    
    // Get Parents header
    let parents = headers.get("parents")
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .map(str::to_string)
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>()
        })
        .filter(|v| !v.is_empty());
    
    // Add message to state
    let (version_id, _) = {
        let mut states = app_state.chat_states.write().await;
        
        let state = states.entry(conversation_id.clone())
            .or_insert_with(|| ChatState::new());
        
        let v_id = state.add_message(message.clone(), parents.clone());
        let parents = state.version_history.get(&v_id)
            .cloned()
            .unwrap_or_default();
        (v_id, parents)
    };
    
    // Save message to database
    // Skip database save if receiver is AI bot (user_id 0) and doesn't exist in database
    let receiver_id = if user_id == user1_id { user2_id } else { user1_id };
    if receiver_id != 0 {
        // Only save to database if receiver is not the AI bot (user_id 0)
        // AI bot messages are stored in memory state only
        if let Err(e) = chat_db::save_message(
            &app_state.db,
            user_id,
            receiver_id,
            &conversation_id,
            &message,
            &version_id,
        ).await {
            tracing::error!("Failed to save message to database: {:?}", e);
        }
    } else {
        tracing::debug!("Skipping database save for AI bot conversation (user_id 0)");
    }
    
    // Update conversation state
    if let Err(e) = chat_db::update_conversation_state(
        &app_state.db,
        &conversation_id,
        &version_id,
        user1_id,
        user2_id,
        user_id,
    ).await {
        tracing::error!("Failed to update conversation state: {:?}", e);
    }
    
    // Get all current messages to broadcast
    let messages_to_broadcast = {
        let states = app_state.chat_states.read().await;
        if let Some(state) = states.get(&conversation_id) {
            state.messages.clone()
        } else {
            vec![message]
        }
    };
    
    // Broadcast to subscribers
    app_state.broadcast_message(&conversation_id, messages_to_broadcast, version_id.clone()).await;
    
    // Check if this is a conversation with AI bot and trigger AI response
    #[cfg(feature = "genai")]
    {
        // Determine bot_user_id: use 0 if user_id 0 is involved, otherwise check database
        let bot_user_id = if user1_id == 0 || user2_id == 0 {
            Some(0) // User ID 0 is reserved for AI bot
        } else {
            // Check if bot user exists in database
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT id
                FROM users
                WHERE email = 'system@ai.bot'
                LIMIT 1
                "#
            )
            .fetch_optional(&app_state.db)
            .await
            .ok()
            .flatten()
        };
        
        if let Some(bot_user_id) = bot_user_id {
            // Check if bot is part of this conversation
            let bot_in_conversation = if bot_user_id == 0 {
                user1_id == 0 || user2_id == 0
            } else {
                user1_id == bot_user_id || user2_id == bot_user_id
            };
            
            if bot_in_conversation {
                use crate::chat::ai_bot::{start_ai_bot_for_conversation, BotConfig, AiProvider};
                
                let provider = if std::env::var("DEEPSEEK_API_KEY").is_ok() {
                    AiProvider::DeepSeek
                } else if std::env::var("OPENAI_API_KEY").is_ok() {
                    AiProvider::OpenAI
                } else if std::env::var("ANTHROPIC_API_KEY").is_ok() {
                    AiProvider::Anthropic
                } else if std::env::var("GEMINI_API_KEY").is_ok() {
                    AiProvider::Gemini
                } else {
                    AiProvider::DeepSeek
                };
                
                let api_key = std::env::var(provider.api_key_env())
                    .unwrap_or_else(|_| String::new());
                
                if !api_key.is_empty() {
                    // Detect if this is an AI chat conversation (format: "0:user_id")
                    // For AI chat, the bot should respond to all messages, not just when mentioned
                    let is_ai_chat = conversation_id.starts_with("0:");
                    
                    let bot_config = BotConfig {
                        provider,
                        api_key,
                        bot_user_id,
                        respond_to_all: is_ai_chat, // Respond to all messages in AI chat
                        ..BotConfig::default()
                    };
                    
                    start_ai_bot_for_conversation(
                        conversation_id.clone(),
                        Arc::clone(&app_state),
                        bot_config,
                    );
                }
            }
        }
    }
    
    // Return success response with Version header
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Version", version_id)
        .body(Body::empty())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}

