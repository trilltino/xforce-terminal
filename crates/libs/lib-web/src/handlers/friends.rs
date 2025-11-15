//! # Friend Management Handlers
//!
//! HTTP endpoints for friend requests and friend management.
//!
//! ## Endpoints
//!
//! - `POST /api/friends/request` - Send friend request
//! - `POST /api/friends/accept/{request_id}` - Accept friend request
//! - `POST /api/friends/reject/{request_id}` - Reject friend request
//! - `POST /api/friends/block/{user_id}` - Block user
//! - `GET /api/friends` - List friends and pending requests
//! - `GET /api/friends/search?query={username}` - Search users to add as friends

use axum::{extract::{Query, Path, State}, http::{StatusCode, HeaderMap, header::AUTHORIZATION}, Json};
use serde::Deserialize;
use lib_core::dto::messaging::*;
use lib_auth::decode_jwt;
use lib_core::{Config, DbPool};
use tracing::instrument;

/// Helper to extract user ID from JWT token
fn extract_user_id(headers: &HeaderMap, config: &Config) -> Result<i64, (StatusCode, String)> {
    let auth_header = headers.get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()))?;
    
    let token = auth_header.strip_prefix("Bearer ")
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid authorization format".to_string()))?;
    
    let claims = decode_jwt(token, &config.jwt_secret)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;
    
    claims.sub.parse::<i64>()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))
}

/// Send a friend request
#[instrument(skip(db, config))]
pub async fn send_friend_request(
    State(db): State<DbPool>,
    State(config): State<Config>,
    headers: HeaderMap,
    Json(payload): Json<FriendRequestRequest>,
) -> Result<(StatusCode, Json<FriendRequestResponse>), (StatusCode, String)> {
    let sender_id = extract_user_id(&headers, &config)?;
    
    let receiver_id = payload.receiver_id;
    
    // Cannot send request to yourself
    if sender_id == receiver_id {
        return Err((StatusCode::BAD_REQUEST, "Cannot send friend request to yourself".to_string()));
    }
    
    // Check if receiver exists
    let receiver_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(SELECT 1 FROM users WHERE id = ? AND is_active = 1)
        "#
    )
    .bind(receiver_id)
    .fetch_one(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    if !receiver_exists {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }
    
    // Check if friendship already exists
    #[derive(sqlx::FromRow)]
    struct FriendshipCheck {
        id: i64,
        status: String,
    }
    
    let existing = sqlx::query_as::<_, FriendshipCheck>(
        r#"
        SELECT id, status
        FROM friendships
        WHERE (sender_id = ? AND receiver_id = ?) OR (sender_id = ? AND receiver_id = ?)
        LIMIT 1
        "#
    )
    .bind(sender_id)
    .bind(receiver_id)
    .bind(receiver_id)
    .bind(sender_id)
    .fetch_optional(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    if let Some(friendship) = existing {
        match friendship.status.as_str() {
            "accepted" => return Err((StatusCode::BAD_REQUEST, "Already friends".to_string())),
            "pending" => return Err((StatusCode::BAD_REQUEST, "Friend request already pending".to_string())),
            "blocked" => return Err((StatusCode::FORBIDDEN, "Cannot send request to blocked user".to_string())),
            _ => {}
        }
    }
    
    // Create or update friendship
    let friendship_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO friendships (sender_id, receiver_id, status, created_at, updated_at)
        VALUES (?, ?, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT(sender_id, receiver_id) DO UPDATE SET
            status = 'pending',
            updated_at = CURRENT_TIMESTAMP
        RETURNING id
        "#
    )
    .bind(sender_id)
    .bind(receiver_id)
    .fetch_one(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    Ok((StatusCode::CREATED, Json(FriendRequestResponse {
        id: friendship_id,
        sender_id,
        receiver_id,
        status: "pending".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    })))
}

/// Accept a friend request
#[instrument(skip(db, config), fields(request_id = %request_id))]
pub async fn accept_friend_request(
    Path(request_id): Path<i64>,
    State(db): State<DbPool>,
    State(config): State<Config>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let user_id = extract_user_id(&headers, &config)?;
    
    // Get friendship and verify receiver
    #[derive(sqlx::FromRow)]
    struct FriendshipRow {
        sender_id: i64,
        receiver_id: i64,
        status: String,
    }
    
    let friendship = sqlx::query_as::<_, FriendshipRow>(
        r#"
        SELECT sender_id, receiver_id, status
        FROM friendships
        WHERE id = ? AND receiver_id = ? AND status = 'pending'
        "#
    )
    .bind(request_id)
    .bind(user_id)
    .fetch_optional(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    let _friendship = friendship.ok_or_else(|| (StatusCode::NOT_FOUND, "Friend request not found".to_string()))?;
    
    // Update status to accepted
    sqlx::query(
        r#"
        UPDATE friendships
        SET status = 'accepted', updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#
    )
    .bind(request_id)
    .execute(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    Ok((StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": "Friend request accepted"
    }))))
}

/// Reject a friend request
#[instrument(skip(db, config), fields(request_id = %request_id))]
pub async fn reject_friend_request(
    Path(request_id): Path<i64>,
    State(db): State<DbPool>,
    State(config): State<Config>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let user_id = extract_user_id(&headers, &config)?;
    
    // Get friendship and verify receiver
    let friendship_id: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT id
        FROM friendships
        WHERE id = ? AND receiver_id = ? AND status = 'pending'
        "#
    )
    .bind(request_id)
    .bind(user_id)
    .fetch_optional(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    friendship_id.ok_or_else(|| (StatusCode::NOT_FOUND, "Friend request not found".to_string()))?;
    
    // Update status to rejected
    sqlx::query(
        r#"
        UPDATE friendships
        SET status = 'rejected', updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#
    )
    .bind(request_id)
    .execute(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    Ok((StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": "Friend request rejected"
    }))))
}

/// Block a user
#[instrument(skip(db, config), fields(blocked_user_id = %blocked_user_id))]
pub async fn block_user(
    Path(blocked_user_id): Path<i64>,
    State(db): State<DbPool>,
    State(config): State<Config>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let user_id = extract_user_id(&headers, &config)?;
    
    if user_id == blocked_user_id {
        return Err((StatusCode::BAD_REQUEST, "Cannot block yourself".to_string()));
    }
    
    // Create or update friendship to blocked
    sqlx::query(
        r#"
        INSERT INTO friendships (sender_id, receiver_id, status, created_at, updated_at)
        VALUES (?, ?, 'blocked', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT(sender_id, receiver_id) DO UPDATE SET
            status = 'blocked',
            updated_at = CURRENT_TIMESTAMP
        "#
    )
    .bind(user_id)
    .bind(blocked_user_id)
    .execute(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    Ok((StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": "User blocked"
    }))))
}

/// List friends and pending requests
#[instrument(skip(db, config))]
pub async fn get_friends(
    State(db): State<DbPool>,
    State(config): State<Config>,
    headers: HeaderMap,
) -> Result<Json<FriendsListResponse>, (StatusCode, String)> {
    let user_id = extract_user_id(&headers, &config)?;
    
    // Get accepted friends
    #[derive(sqlx::FromRow)]
    struct FriendRow {
        friendship_id: i64,
        friend_id: i64,
        username: String,
        last_message_at: Option<String>,
        unread_count: i32,
    }
    
    let friends = sqlx::query_as::<_, FriendRow>(
        r#"
        SELECT 
            f.id as friendship_id,
            CASE 
                WHEN f.sender_id = ? THEN f.receiver_id
                ELSE f.sender_id
            END as friend_id,
            u.username,
            cs.last_message_at,
            CASE
                WHEN f.sender_id = ? THEN cs.user2_unread_count
                ELSE cs.user1_unread_count
            END as unread_count
        FROM friendships f
        JOIN users u ON u.id = CASE 
            WHEN f.sender_id = ? THEN f.receiver_id
            ELSE f.sender_id
        END
        LEFT JOIN conversation_state cs ON (
            (cs.user1_id = f.sender_id AND cs.user2_id = f.receiver_id) OR
            (cs.user1_id = f.receiver_id AND cs.user2_id = f.sender_id)
        )
        WHERE (f.sender_id = ? OR f.receiver_id = ?)
          AND f.status = 'accepted'
        ORDER BY cs.last_message_at DESC NULLS LAST, u.username ASC
        "#
    )
    .bind(user_id)
    .bind(user_id)
    .bind(user_id)
    .bind(user_id)
    .bind(user_id)
    .fetch_all(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    let friends: Vec<Friend> = friends.into_iter().map(|row| {
        Friend {
            id: row.friendship_id,
            user_id: row.friend_id,
            username: row.username,
            friendship_id: row.friendship_id,
            unread_count: row.unread_count,
            last_message_at: row.last_message_at,
            last_message_preview: None, // TODO: Add last message preview
        }
    }).collect();
    
    // Get incoming requests
    #[derive(sqlx::FromRow)]
    struct RequestRow {
        id: i64,
        sender_id: i64,
        receiver_id: i64,
        sender_username: String,
        receiver_username: String,
        status: String,
        created_at: String,
    }
    
    let incoming = sqlx::query_as::<_, RequestRow>(
        r#"
        SELECT 
            f.id,
            f.sender_id,
            f.receiver_id,
            u_sender.username as sender_username,
            u_receiver.username as receiver_username,
            f.status,
            f.created_at
        FROM friendships f
        JOIN users u_sender ON f.sender_id = u_sender.id
        JOIN users u_receiver ON f.receiver_id = u_receiver.id
        WHERE f.receiver_id = ? AND f.status = 'pending'
        ORDER BY f.created_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    let incoming_requests: Vec<FriendRequest> = incoming.into_iter().map(|row| {
        FriendRequest {
            id: row.id,
            sender_id: row.sender_id,
            receiver_id: row.receiver_id,
            sender_username: row.sender_username,
            receiver_username: row.receiver_username,
            status: row.status,
            created_at: row.created_at,
        }
    }).collect();
    
    // Get outgoing requests
    let outgoing = sqlx::query_as::<_, RequestRow>(
        r#"
        SELECT 
            f.id,
            f.sender_id,
            f.receiver_id,
            u_sender.username as sender_username,
            u_receiver.username as receiver_username,
            f.status,
            f.created_at
        FROM friendships f
        JOIN users u_sender ON f.sender_id = u_sender.id
        JOIN users u_receiver ON f.receiver_id = u_receiver.id
        WHERE f.sender_id = ? AND f.status = 'pending'
        ORDER BY f.created_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    let outgoing_requests: Vec<FriendRequest> = outgoing.into_iter().map(|row| {
        FriendRequest {
            id: row.id,
            sender_id: row.sender_id,
            receiver_id: row.receiver_id,
            sender_username: row.sender_username,
            receiver_username: row.receiver_username,
            status: row.status,
            created_at: row.created_at,
        }
    }).collect();
    
    Ok(Json(FriendsListResponse {
        friends,
        incoming_requests,
        outgoing_requests,
    }))
}

/// Search users by username
#[derive(Deserialize, Debug)]
pub struct UserSearchQuery {
    pub query: String,
}

#[instrument(skip(db, config), fields(query = %params.query))]
pub async fn search_users(
    Query(params): Query<UserSearchQuery>,
    State(db): State<DbPool>,
    State(config): State<Config>,
    headers: HeaderMap,
) -> Result<Json<UserSearchResponse>, (StatusCode, String)> {
    let user_id = extract_user_id(&headers, &config)?;
    
    if params.query.trim().is_empty() {
        return Ok(Json(UserSearchResponse { users: vec![] }));
    }
    
    #[derive(sqlx::FromRow)]
    struct UserRow {
        id: i64,
        username: String,
        email: String,
    }
    
    let search_pattern = format!("%{}%", params.query.trim());
    
    let users = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, username, email
        FROM users
        WHERE username LIKE ? 
          AND id != ?
          AND is_active = 1
        LIMIT 20
        "#
    )
    .bind(&search_pattern)
    .bind(user_id)
    .fetch_all(&db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    let results: Vec<UserSearchResult> = users.into_iter().map(|row| {
        UserSearchResult {
            id: row.id,
            username: row.username,
            email: row.email,
        }
    }).collect();
    
    Ok(Json(UserSearchResponse { users: results }))
}

