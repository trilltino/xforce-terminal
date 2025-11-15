//! # Signup Tests
//!
//! Tests for user signup functionality.

use super::*;
use lib_auth::hash_password;
use lib_core::model::store::user_repository::UserRepository;
use axum::body::Body;
use axum::http::{Request, StatusCode};

#[tokio::test]
async fn test_signup_success() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();
    let app = test_app(pool, config);

    let signup_req = SignupRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "TestPassword123!".to_string(),
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&signup_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(auth_response.user.username, "testuser");
    assert_eq!(auth_response.user.email, "test@example.com");
    assert_eq!(auth_response.message, "Signup successful");
    assert!(!auth_response.token.is_empty());
    assert_eq!(auth_response.wallet_setup_required, Some(true));
    assert!(auth_response.wallet_setup_token.is_some());
}

#[tokio::test]
async fn test_signup_username_too_short() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();
    let app = test_app(pool, config);

    let signup_req = SignupRequest {
        username: "ab".to_string(), // Only 2 characters
        email: "test@example.com".to_string(),
        password: "TestPassword123!".to_string(),
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&signup_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(error_response.error, "Username must be at least 3 characters");
}

#[tokio::test]
async fn test_signup_invalid_email() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();
    let app = test_app(pool, config);

    let signup_req = SignupRequest {
        username: "testuser".to_string(),
        email: "invalid-email".to_string(), // No @ symbol
        password: "TestPassword123!".to_string(),
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&signup_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(error_response.error, "Invalid email format");
}

#[tokio::test]
async fn test_signup_duplicate_email() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    // Create first user directly in database
    let password_hash = hash_password("Password123!").unwrap();
    UserRepository::create(&pool, "user1", "test@example.com", &password_hash)
        .await
        .unwrap();

    let app = test_app(pool, config);

    let signup_req = SignupRequest {
        username: "user2".to_string(),
        email: "test@example.com".to_string(), // Duplicate email
        password: "TestPassword123!".to_string(),
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&signup_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(error_response.error, "Email already registered");
}

#[tokio::test]
async fn test_signup_duplicate_username() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    // Create first user directly in database
    let password_hash = hash_password("Password123!")
        .expect("Password hashing should succeed in test");
    UserRepository::create(&pool, "testuser", "user1@example.com", &password_hash)
        .await
        .expect("User creation should succeed in test");

    let app = test_app(pool, config);

    let signup_req = SignupRequest {
        username: "testuser".to_string(), // Duplicate username
        email: "user2@example.com".to_string(),
        password: "TestPassword123!".to_string(),
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&signup_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(error_response.error, "Username already taken");
}

#[tokio::test]
async fn test_signup_password_too_short() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();
    let app = test_app(pool, config);

    let signup_req = SignupRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "short".to_string(), // Too short (less than 8 chars)
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&signup_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(error_response.error, "Password must be at least 8 characters long");
}

