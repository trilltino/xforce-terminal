//! # Chat State Management
//!
//! Manages server-side chat state for direct message conversations.
//! Implements Braid protocol version tracking using a DAG structure.

use lib_core::{Config, DbPool, dto::Message};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

/// Chat state for a single conversation
///
/// This structure stores all messages for a conversation and their version history.
/// It implements the Braid protocol's version tracking using a DAG structure.
#[derive(Debug, Clone)]
pub struct ChatState {
    /// List of all chat messages, ordered by creation time
    pub messages: Vec<Message>,
    
    /// Version history: maps version ID to its parent version IDs
    /// This forms a Directed Acyclic Graph (DAG) as specified by the Braid protocol.
    pub version_history: HashMap<String, Vec<String>>,
    
    /// Current version ID (the frontier of the version DAG)
    pub current_version: Option<String>,
}

impl ChatState {
    /// Create a new empty chat state
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            version_history: HashMap::new(),
            current_version: None,
        }
    }

    /// Add a new message to the chat state
    ///
    /// Generates a new version ID and updates the version history.
    /// Returns the version ID assigned to the new message.
    pub fn add_message(&mut self, mut message: Message, parent_versions: Option<Vec<String>>) -> String {
        // Generate a new version ID using UUID v4
        let version_id = Uuid::new_v4().to_string();
        message.version = Some(version_id.clone());
        
        // Add message to list
        self.messages.push(message);
        
        // Update version history with parent relationships
        if let Some(parents) = parent_versions {
            // Use provided parent versions
            self.version_history.insert(version_id.clone(), parents);
        } else if let Some(current) = &self.current_version {
            // No parents specified, use current version as parent
            self.version_history.insert(version_id.clone(), vec![current.clone()]);
        } else {
            // First message, no parents (root of DAG)
            self.version_history.insert(version_id.clone(), Vec::new());
        }
        
        // Update current version to the new message's version
        self.current_version = Some(version_id.clone());
        
        version_id
    }
    
    /// Get messages since a specific version
    ///
    /// Used for Braid protocol reconnection and catch-up.
    /// Returns all messages that were added after the specified version.
    pub fn get_messages_since(&self, since_version: Option<&String>) -> Vec<Message> {
        if let Some(since) = since_version {
            // Find the index of the message with the specified version
            let start_idx = self.messages.iter()
                .position(|m| m.version.as_ref() == Some(since))
                .map(|idx| idx + 1)
                .unwrap_or(0);
            
            // Return all messages after the specified version
            self.messages[start_idx..].to_vec()
        } else {
            // No version specified, return all messages
            self.messages.clone()
        }
    }
}

impl Default for ChatState {
    fn default() -> Self {
        Self::new()
    }
}

/// Application state for chat module
pub struct ChatAppState {
    pub db: DbPool,
    pub config: Config,
    pub chat_states: Arc<RwLock<HashMap<String, ChatState>>>,
    pub broadcast_senders: Arc<RwLock<HashMap<String, broadcast::Sender<(Vec<Message>, String)>>>>,
    pub typing_broadcast_senders: Arc<RwLock<HashMap<String, broadcast::Sender<(i64, String, bool)>>>>,
}

impl ChatAppState {
    pub fn new(db: DbPool, config: Config) -> Self {
        Self {
            db,
            config,
            chat_states: Arc::new(RwLock::new(HashMap::new())),
            broadcast_senders: Arc::new(RwLock::new(HashMap::new())),
            typing_broadcast_senders: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn get_broadcast_sender(&self, conversation_id: &str) -> broadcast::Sender<(Vec<Message>, String)> {
        let mut senders = self.broadcast_senders.write().await;
        
        if let Some(sender) = senders.get(conversation_id) {
            sender.clone()
        } else {
            let (tx, _) = broadcast::channel(100);
            senders.insert(conversation_id.to_string(), tx.clone());
            tx
        }
    }
    
    pub async fn broadcast_message(&self, conversation_id: &str, messages: Vec<Message>, version: String) {
        let sender = self.get_broadcast_sender(conversation_id).await;
        let _ = sender.send((messages, version));
    }
    
    async fn get_typing_broadcast_sender(&self, conversation_id: &str) -> broadcast::Sender<(i64, String, bool)> {
        let mut senders = self.typing_broadcast_senders.write().await;
        
        if let Some(sender) = senders.get(conversation_id) {
            sender.clone()
        } else {
            let (tx, _) = broadcast::channel(100);
            senders.insert(conversation_id.to_string(), tx.clone());
            tx
        }
    }
    
    pub async fn broadcast_typing(&self, conversation_id: &str, user_id: i64, username: String, is_typing: bool) {
        let sender = self.get_typing_broadcast_sender(conversation_id).await;
        let _ = sender.send((user_id, username, is_typing));
    }
}

impl axum::extract::FromRef<ChatAppState> for DbPool {
    fn from_ref(state: &ChatAppState) -> Self {
        state.db.clone()
    }
}

impl axum::extract::FromRef<ChatAppState> for Config {
    fn from_ref(state: &ChatAppState) -> Self {
        state.config.clone()
    }
}

