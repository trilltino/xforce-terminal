//! # Messaging Data Transfer Objects
//!
//! Defines request and response structures for messaging and friend management endpoints.

use serde::{Deserialize, Serialize};

/// Friend request to send to another user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequestRequest {
    pub receiver_id: i64,
}

/// Response after sending friend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequestResponse {
    pub id: i64,
    pub sender_id: i64,
    pub receiver_id: i64,
    pub status: String,
    pub created_at: String,
}

/// Friend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friend {
    pub id: i64,
    pub user_id: i64,
    pub username: String,
    pub friendship_id: i64,
    pub unread_count: i32,
    pub last_message_at: Option<String>,
    pub last_message_preview: Option<String>,
}

/// Friend request information (incoming or outgoing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequest {
    pub id: i64,
    pub sender_id: i64,
    pub receiver_id: i64,
    pub sender_username: String,
    pub receiver_username: String,
    pub status: String,
    pub created_at: String,
}

/// List of friends and pending requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendsListResponse {
    pub friends: Vec<Friend>,
    pub incoming_requests: Vec<FriendRequest>,
    pub outgoing_requests: Vec<FriendRequest>,
}

/// Search users response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchResult {
    pub id: i64,
    pub username: String,
    pub email: String,
}

/// List of search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchResponse {
    pub users: Vec<UserSearchResult>,
}

/// Message for direct messaging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub text: String,
    pub author: String,
    pub author_id: i64,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Message {
    pub fn new(text: String, author: String, author_id: i64) -> Self {
        Self {
            text,
            author,
            author_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: None,
        }
    }

    pub fn with_version(text: String, author: String, author_id: i64, version: String) -> Self {
        Self {
            text,
            author,
            author_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: Some(version),
        }
    }
}

/// Conversation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationInfo {
    pub conversation_id: String,
    pub user1_id: i64,
    pub user2_id: i64,
    pub last_version: Option<String>,
    pub last_message_at: Option<String>,
    pub unread_count: i32,
}

/// Typing indicator request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingRequest {
    pub is_typing: bool,
}

/// Typing indicator event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingEvent {
    pub user_id: i64,
    pub username: String,
    pub is_typing: bool,
}

