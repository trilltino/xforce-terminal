//! # AI Chat Bot
//!
//! AI bot that integrates with the Braid messaging protocol using rust-genai.
//! Supports multiple AI providers (DeepSeek, OpenAI, Anthropic, Gemini, etc.)
//! and responds to messages via Braid PUT protocol.

use crate::{chat::state::ChatState, chat::db as chat_db};
use lib_core::dto::Message;
use std::sync::Arc;
use tokio::sync::broadcast;

/// AI Provider type
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AiProvider {
    /// DeepSeek (default)
    DeepSeek,
    /// OpenAI
    OpenAI,
    /// Anthropic
    Anthropic,
    /// Google Gemini
    Gemini,
}

impl Default for AiProvider {
    fn default() -> Self {
        AiProvider::DeepSeek
    }
}

impl AiProvider {
    /// Get the default model name for this provider
    pub fn default_model(&self) -> &'static str {
        match self {
            AiProvider::DeepSeek => "deepseek-chat",
            AiProvider::OpenAI => "gpt-4o-mini",
            AiProvider::Anthropic => "claude-3-haiku-20240307",
            AiProvider::Gemini => "gemini-2.0-flash",
        }
    }

    /// Get the environment variable name for the API key
    pub fn api_key_env(&self) -> &'static str {
        match self {
            AiProvider::DeepSeek => "DEEPSEEK_API_KEY",
            AiProvider::OpenAI => "OPENAI_API_KEY",
            AiProvider::Anthropic => "ANTHROPIC_API_KEY",
            AiProvider::Gemini => "GEMINI_API_KEY",
        }
    }
}

/// Bot configuration
#[derive(Clone, Debug)]
pub struct BotConfig {
    /// Bot's display name
    pub name: String,
    /// AI provider
    pub provider: AiProvider,
    /// API key for the AI provider
    pub api_key: String,
    /// Model name (e.g., "deepseek-chat", "gpt-4o-mini")
    pub model: String,
    /// Whether the bot should respond to all messages or only when mentioned
    pub respond_to_all: bool,
    /// Maximum number of recent messages to include in context
    pub context_window: usize,
    /// Maximum response length in tokens
    pub max_tokens: u32,
    /// Temperature for response generation
    pub temperature: f32,
    /// Custom system prompt for the AI
    pub system_prompt: Option<String>,
    /// Bot user ID (system user)
    pub bot_user_id: i64,
}

impl Default for BotConfig {
    fn default() -> Self {
        let provider = AiProvider::DeepSeek;
        let api_key_env = provider.api_key_env();
        
        // Enhanced default system prompt for trading terminal
        let default_system_prompt = "You are a helpful AI assistant in a Solana trading terminal chat. \
            - Provide accurate, helpful information about Solana, DeFi, and trading when you can \
            - Be concise but thorough (2-4 sentences when needed) \
            - Maintain context from the conversation and reference previous messages when relevant \
            - If you're unsure about something, say so rather than guessing \
            - Use a friendly, conversational tone that fits a chat room environment \
            - Keep responses appropriate for a casual group chat setting \
            - Be engaging and personable while remaining professional \
            - You can help with market analysis, token information, and trading strategies".to_string();
        
        // Get system prompt from environment variable or use default
        let system_prompt = std::env::var("AI_SYSTEM_PROMPT")
            .ok()
            .or(Some(default_system_prompt));
        
        // Get configuration from environment variables with defaults
        let max_tokens = std::env::var("AI_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(500);
        
        let temperature = std::env::var("AI_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.8);
        
        let context_window = std::env::var("AI_CONTEXT_WINDOW")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(20);
        
        Self {
            name: "DeepSeek AI".to_string(),
            provider: provider.clone(),
            api_key: std::env::var(api_key_env)
                .unwrap_or_else(|_| String::new()),
            model: provider.default_model().to_string(),
            respond_to_all: false, // Only respond when mentioned
            context_window,
            max_tokens,
            temperature,
            system_prompt,
            bot_user_id: 0, // Will be set when bot is started
        }
    }
}

/// Start the AI bot for a conversation
/// 
/// This function spawns a background task that:
/// 1. Subscribes to message broadcasts for the conversation
/// 2. Detects when to respond (when mentioned or all messages)
/// 3. Generates AI responses using rust-genai
/// 4. Posts responses back via the Braid protocol
pub fn start_ai_bot_for_conversation(
    conversation_id: String,
    chat_state: Arc<ChatAppState>,
    config: BotConfig,
) {
    tracing::info!("🤖 Starting AI bot for conversation {}: {} (provider: {:?}, model: {})", 
        conversation_id, config.name, config.provider, config.model);
    
    // Spawn the bot task
    tokio::spawn(async move {
        let mut broadcast_rx = chat_state.get_broadcast_sender(&conversation_id).await.subscribe();
        let mut last_processed_version: Option<String> = None;
        
        tracing::info!("🤖 AI bot initialized for conversation {} and listening for messages", conversation_id);
        
        loop {
            match broadcast_rx.recv().await {
                Ok((messages, version)) => {
                    // Skip if we've already processed this version
                    if last_processed_version.as_ref() == Some(&version) {
                        continue;
                    }
                    
                    // Get the most recent message
                    if let Some(last_message) = messages.last() {
                        // Skip if this is the bot's own message
                        if last_message.author == config.name {
                            last_processed_version = Some(version);
                            continue;
                        }
                        
                        // Check if we should respond
                        let should_respond = if config.respond_to_all {
                            true
                        } else {
                            // Only respond if mentioned
                            last_message.text.to_lowercase().contains(&config.name.to_lowercase())
                                || last_message.text.to_lowercase().contains("@bot")
                                || last_message.text.to_lowercase().contains("@ai")
                                || last_message.text.to_lowercase().contains("@deepseek")
                        };
                        
                        if should_respond {
                            tracing::info!("🤖 Bot detected message from {}: {}", last_message.author, last_message.text);
                            
                            // Add a small delay to make the bot feel more natural (1-2 seconds)
                            tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                            
                            // Get context (recent messages)
                            let context_messages: Vec<Message> = messages
                                .iter()
                                .rev()
                                .take(config.context_window)
                                .rev()
                                .cloned()
                                .collect();
                            
                            // Generate response using rust-genai
                            let response_text = match generate_response(&config, &context_messages).await {
                                Ok(text) => text,
                                Err(e) => {
                                    tracing::error!("🤖 Failed to generate AI response: {:?}", e);
                                    last_processed_version = Some(version);
                                    continue;
                                }
                            };
                            
                            // Post response via Braid PUT
                            if let Err(e) = post_bot_response(&chat_state, &config, &conversation_id, &response_text, &version).await {
                                tracing::error!("🤖 Failed to post bot response: {:?}", e);
                            } else {
                                tracing::info!("🤖 Bot posted response: {}", response_text);
                            }
                        }
                    }
                    
                    last_processed_version = Some(version);
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("🤖 Bot lagged, skipped {} messages. Reconnecting...", skipped);
                    // Continue - we'll catch up on next message
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::error!("🤖 Bot broadcast channel closed. Bot shutting down for conversation {}.", conversation_id);
                    break;
                }
            }
        }
    });
}

/// Generate AI response using rust-genai
#[cfg(feature = "genai")]
async fn generate_response(
    config: &BotConfig,
    context_messages: &[Message],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use genai::chat::{ChatMessage, ChatOptions, ChatRequest};
    use genai::resolver::{AuthData, AuthResolver};
    use genai::Client;
    
    // Build auth resolver for custom API key
    let api_key = config.api_key.clone();
    let auth_resolver = AuthResolver::from_resolver_fn(
        move |_model_iden| -> Result<Option<AuthData>, genai::resolver::Error> {
            Ok(Some(AuthData::from_single(api_key.clone())))
        },
    );
    
    // Build client with auth resolver
    let client = Client::builder()
        .with_auth_resolver(auth_resolver)
        .build();
    
    // Build chat request with system message
    let system_prompt = config.system_prompt.clone().unwrap_or_else(|| {
        "You are a helpful AI assistant in a Solana trading terminal chat.".to_string()
    });
    
    let mut chat_req = ChatRequest::default().with_system(&system_prompt);
    
    // Add recent messages to context
    for msg in context_messages {
        if msg.author == config.name {
            chat_req = chat_req.append_message(ChatMessage::assistant(&msg.text));
        } else {
            chat_req = chat_req.append_message(ChatMessage::user(&msg.text));
        }
    }
    
    // Build chat options
    let chat_options = ChatOptions::default()
        .with_temperature(config.temperature as f64)
        .with_max_tokens(config.max_tokens);
    
    // Call AI provider
    tracing::debug!("🤖 Calling AI API with model: {}", config.model);
    let chat_res = client.exec_chat(&config.model, chat_req, Some(&chat_options)).await
        .map_err(|e| format!("AI API error: {:?}", e))?;
    
    // Extract response text
    let mut response_text = chat_res.first_text()
        .ok_or_else(|| "No response from AI".to_string())?
        .trim()
        .to_string();
    
    if response_text.is_empty() {
        return Err("Empty response from AI".into());
    }
    
    // Post-process response: clean up common AI artifacts
    response_text = response_text
        .replace("As an AI assistant, ", "")
        .replace("As an AI, ", "")
        .replace("I'm an AI assistant, ", "")
        .replace("I'm an AI, ", "")
        .replace("As a language model, ", "")
        .trim()
        .to_string();
    
    // Validate response is not empty after post-processing
    if response_text.trim().is_empty() {
        return Err("Empty response from AI after post-processing".into());
    }
    
    // Limit response length
    const MAX_RESPONSE_LENGTH: usize = 1000;
    let response_text = if response_text.len() > MAX_RESPONSE_LENGTH {
        // Try to cut at a sentence boundary
        if let Some(cut_point) = response_text[..MAX_RESPONSE_LENGTH].rfind('.') {
            format!("{}...", &response_text[..=cut_point])
        } else if let Some(cut_point) = response_text[..MAX_RESPONSE_LENGTH].rfind(' ') {
            format!("{}...", &response_text[..cut_point])
        } else {
            format!("{}...", &response_text[..MAX_RESPONSE_LENGTH])
        }
    } else {
        response_text
    };
    
    Ok(response_text)
}

/// Fallback when genai feature is not enabled
#[cfg(not(feature = "genai"))]
async fn generate_response(
    _config: &BotConfig,
    _context_messages: &[Message],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Err("AI chat is not enabled. Please enable the 'genai' feature.".into())
}

/// Post bot response via Braid PUT protocol
async fn post_bot_response(
    chat_state: &ChatAppState,
    config: &BotConfig,
    conversation_id: &str,
    response_text: &str,
    parent_version: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Parse conversation ID to get user IDs
    let (user1_id, user2_id) = parse_conversation_id(conversation_id)
        .map_err(|_| "Invalid conversation ID format".to_string())?;
    
    // Create bot message - use author_id 0 for bot messages
    let bot_message = Message::new(response_text.to_string(), config.name.clone(), 0);
    
    // Add message to state
    let version_id = {
        let mut states = chat_state.chat_states.write().await;
        
        let state = states.entry(conversation_id.to_string())
            .or_insert_with(|| ChatState::new());
        
        state.add_message(bot_message.clone(), Some(vec![parent_version.to_string()]))
    };
    
    // Save message to database (skip if bot_user_id is 0, as it doesn't exist in DB)
    // Determine receiver ID (the other user in the conversation)
    let receiver_id = if config.bot_user_id == user1_id { user2_id } else { user1_id };
    
    // Skip database save for AI bot messages when bot_user_id is 0
    // Messages are already stored in memory via state.add_message() and accessible via SSE subscription
    if config.bot_user_id != 0 {
        if let Err(e) = chat_db::save_message(
            &chat_state.db,
            config.bot_user_id,
            receiver_id,
            conversation_id,
            &bot_message,
            &version_id,
        ).await {
            tracing::error!("🤖 Failed to save bot message to database: {:?}", e);
        } else {
            tracing::debug!("🤖 Bot message saved to database successfully");
        }
    } else {
        tracing::debug!(
            conversation_id = %conversation_id,
            "Skipping database save for AI bot message (user_id 0) - stored in memory only, accessible via SSE"
        );
    }
    
    // Update conversation state (skip if bot_user_id is 0)
    if config.bot_user_id != 0 {
        if let Err(e) = chat_db::update_conversation_state(
            &chat_state.db,
            conversation_id,
            &version_id,
            user1_id,
            user2_id,
            config.bot_user_id,
        ).await {
            tracing::error!("🤖 Failed to update conversation state: {:?}", e);
        }
    } else {
        tracing::debug!("Skipping conversation state update for AI bot (user_id 0)");
    }
    
    // Get all current messages to broadcast
    let messages_to_broadcast = {
        let states = chat_state.chat_states.read().await;
        if let Some(state) = states.get(conversation_id) {
            state.messages.clone()
        } else {
            vec![bot_message]
        }
    };
    
    // Broadcast to subscribers
    chat_state.broadcast_message(conversation_id, messages_to_broadcast, version_id.clone()).await;
    
    Ok(version_id)
}

/// Parse conversation ID into user IDs
fn parse_conversation_id(conversation_id: &str) -> Result<(i64, i64), String> {
    let parts: Vec<&str> = conversation_id.split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid conversation ID format: expected 'user1_id:user2_id'".to_string());
    }
    
    let user1_id = parts[0].parse::<i64>()
        .map_err(|e| format!("Failed to parse user1_id: {}", e))?;
    let user2_id = parts[1].parse::<i64>()
        .map_err(|e| format!("Failed to parse user2_id: {}", e))?;
    
    Ok((user1_id, user2_id))
}

/// Application state for chat module (needed by AI bot)
pub use super::state::ChatAppState;

