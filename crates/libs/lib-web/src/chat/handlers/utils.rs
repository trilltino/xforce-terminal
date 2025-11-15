//! # Chat Handler Utilities
//!
//! Shared helper functions for chat handlers.

use lib_core::{Config, DbPool};
use lib_auth::decode_jwt;
use axum::http::{HeaderMap, StatusCode, header::AUTHORIZATION};
use sqlx;

/// Helper to compute conversation ID from two user IDs
pub fn compute_conversation_id(user1_id: i64, user2_id: i64) -> String {
    if user1_id < user2_id {
        format!("{}:{}", user1_id, user2_id)
    } else {
        format!("{}:{}", user2_id, user1_id)
    }
}

/// Helper to extract user ID from JWT token
pub fn extract_user_id_from_token(headers: &HeaderMap, config: &Config) -> Result<i64, StatusCode> {
    let auth_header = headers.get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token = auth_header.strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let claims = decode_jwt(token, &config.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    claims.sub.parse::<i64>()
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

/// Parse conversation ID into user IDs
pub fn parse_conversation_id(conversation_id: &str) -> Result<(i64, i64), StatusCode> {
    let parts: Vec<&str> = conversation_id.split(':').collect();
    if parts.len() != 2 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let user1_id = parts[0].parse::<i64>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let user2_id = parts[1].parse::<i64>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok((user1_id, user2_id))
}

/// Check friendship status
pub async fn check_friendship(pool: &DbPool, user1_id: i64, user2_id: i64) -> Result<String, sqlx::Error> {
    let result = sqlx::query_scalar::<_, String>(
        r#"
        SELECT status
        FROM friendships
        WHERE ((sender_id = ? AND receiver_id = ?) OR (sender_id = ? AND receiver_id = ?))
          AND status = 'accepted'
        LIMIT 1
        "#
    )
    .bind(user1_id)
    .bind(user2_id)
    .bind(user2_id)
    .bind(user1_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(result.unwrap_or_else(|| "none".to_string()))
}

/// Get username by user ID
pub async fn get_username(pool: &DbPool, user_id: i64) -> Result<String, sqlx::Error> {
    let username = sqlx::query_scalar::<_, String>(
        r#"
        SELECT username
        FROM users
        WHERE id = ?
        "#
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    
    Ok(username)
}

