//! # Chat Module
//!
//! Provides Braid protocol implementation for direct messaging between users.
//!
//! This module handles real-time messaging using Braid HTTP protocol with SSE subscriptions
//! and PUT requests for sending messages.

pub mod state;
pub mod handlers;
pub mod db;
#[cfg(feature = "genai")]
pub mod ai_bot;

pub use state::{ChatState, ChatAppState};
pub use handlers::{handle_braid_subscription, handle_braid_put, handle_typing_event};
#[cfg(feature = "genai")]
pub use ai_bot::{start_ai_bot_for_conversation, BotConfig, AiProvider};

