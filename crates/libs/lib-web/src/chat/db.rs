//! # Database Operations for Chat Messages
//!
//! Provides database operations for persisting chat messages and conversation state.

use lib_core::DbPool;
use lib_core::dto::Message;
use sqlx::FromRow;
use chrono::Utc;

/// Save a message to the database
pub async fn save_message(
    pool: &DbPool,
    sender_id: i64,
    receiver_id: i64,
    conversation_id: &str,
    message: &Message,
    version_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO direct_messages (sender_id, receiver_id, conversation_id, text, version, timestamp, created_at)
        VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#
    )
    .bind(sender_id)
    .bind(receiver_id)
    .bind(conversation_id)
    .bind(&message.text)
    .bind(version_id)
    .bind(&message.timestamp)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Load messages for a conversation
pub async fn load_messages_for_conversation(
    pool: &DbPool,
    conversation_id: &str,
) -> Result<Vec<Message>, sqlx::Error> {
    #[derive(FromRow)]
    struct MessageRow {
        text: String,
        author_id: i64,
        timestamp: String,
        version: Option<String>,
    }
    
    // We need to join with users to get username as author
    // For now, we'll use a simplified query and get username separately
    let rows = sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT 
            dm.text,
            dm.sender_id as author_id,
            dm.timestamp,
            dm.version
        FROM direct_messages dm
        WHERE dm.conversation_id = ?
        ORDER BY dm.created_at ASC
        "#
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await?;
    
    let messages = rows
        .into_iter()
        .map(|row| {
            // We'll need to fetch username separately or join
            // For now, use a placeholder
            Message {
                text: row.text,
                author: format!("User{}", row.author_id), // Temporary, will fix with join
                author_id: row.author_id,
                timestamp: row.timestamp,
                version: row.version,
            }
        })
        .collect();
    
    Ok(messages)
}

/// Get or create conversation state
pub async fn get_or_create_conversation_state(
    pool: &DbPool,
    conversation_id: &str,
    user1_id: i64,
    user2_id: i64,
) -> Result<(String, i32), sqlx::Error> {
    // Try to get existing conversation
    #[derive(sqlx::FromRow)]
    struct ConversationStateRow {
        last_version: Option<String>,
        user1_unread_count: i32,
        user2_unread_count: i32,
    }
    
    let result = sqlx::query_as::<_, ConversationStateRow>(
        r#"
        SELECT last_version, user1_unread_count, user2_unread_count
        FROM conversation_state
        WHERE conversation_id = ?
        "#
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;
    
    if let Some(row) = result {
        // Determine unread count for the requesting user
        let unread_count = if user1_id < user2_id {
            row.user1_unread_count
        } else {
            row.user2_unread_count
        };
        
        Ok((row.last_version.unwrap_or_default(), unread_count))
    } else {
        // Create new conversation state
        sqlx::query(
            r#"
            INSERT INTO conversation_state (conversation_id, user1_id, user2_id, last_version, last_message_at)
            VALUES (?, ?, ?, NULL, NULL)
            "#
        )
        .bind(conversation_id)
        .bind(user1_id)
        .bind(user2_id)
        .execute(pool)
        .await?;
        
        Ok((String::new(), 0))
    }
}

/// Update conversation state after message
pub async fn update_conversation_state(
    pool: &DbPool,
    conversation_id: &str,
    version_id: &str,
    user1_id: i64,
    _user2_id: i64,
    sender_id: i64,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    
    // Determine which user's unread count to increment
    if sender_id == user1_id {
        // Increment user2's unread count
        sqlx::query(
            r#"
            UPDATE conversation_state
            SET last_version = ?,
                last_message_at = ?,
                user2_unread_count = user2_unread_count + 1,
                updated_at = CURRENT_TIMESTAMP
            WHERE conversation_id = ?
            "#
        )
        .bind(version_id)
        .bind(now.to_rfc3339())
        .bind(conversation_id)
        .execute(pool)
        .await?;
    } else {
        // Increment user1's unread count
        sqlx::query(
            r#"
            UPDATE conversation_state
            SET last_version = ?,
                last_message_at = ?,
                user1_unread_count = user1_unread_count + 1,
                updated_at = CURRENT_TIMESTAMP
            WHERE conversation_id = ?
            "#
        )
        .bind(version_id)
        .bind(now.to_rfc3339())
        .bind(conversation_id)
        .execute(pool)
        .await?;
    }
    
    Ok(())
}

/// Mark conversation as read for a user
pub async fn mark_conversation_read(
    pool: &DbPool,
    conversation_id: &str,
    user_id: i64,
    user1_id: i64,
    _user2_id: i64,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    
    if user_id == user1_id {
        sqlx::query(
            r#"
            UPDATE conversation_state
            SET user1_unread_count = 0,
                user1_last_read_at = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE conversation_id = ?
            "#
        )
        .bind(now.to_rfc3339())
        .bind(conversation_id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE conversation_state
            SET user2_unread_count = 0,
                user2_last_read_at = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE conversation_id = ?
            "#
        )
        .bind(now.to_rfc3339())
        .bind(conversation_id)
        .execute(pool)
        .await?;
    }
    
    Ok(())
}

/// Load messages with usernames
pub async fn load_messages_with_usernames(
    pool: &DbPool,
    conversation_id: &str,
) -> Result<Vec<Message>, sqlx::Error> {
    #[derive(FromRow)]
    struct MessageRow {
        text: String,
        sender_id: i64,
        username: String,
        timestamp: String,
        version: Option<String>,
    }
    
    let rows = sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT 
            dm.text,
            dm.sender_id,
            u.username,
            dm.timestamp,
            dm.version
        FROM direct_messages dm
        JOIN users u ON dm.sender_id = u.id
        WHERE dm.conversation_id = ?
        ORDER BY dm.created_at ASC
        "#
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await?;
    
    let messages = rows
        .into_iter()
        .map(|row| Message {
            text: row.text,
            author: row.username,
            author_id: row.sender_id,
            timestamp: row.timestamp,
            version: row.version,
        })
        .collect();
    
    Ok(messages)
}

