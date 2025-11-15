//! # Integration Tests
//!
//! Edge cases and integration tests for authentication flow.

use super::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};

#[tokio::test]
async fn test_signup_then_login() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    // Signup
    let signup_app = test_app(pool.clone(), config.clone());
    let signup_req = SignupRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "TestPassword123!".to_string(),
    };

    let signup_response = signup_app
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

    assert_eq!(signup_response.status(), StatusCode::CREATED);

    // Login with the same credentials
    let login_app = test_app(pool, config);
    let login_req = LoginRequest {
        email_or_username: "test@example.com".to_string(),
        password: "TestPassword123!".to_string(),
    };

    // Act
    let login_response = login_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&login_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(login_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(auth_response.user.username, "testuser");
    assert_eq!(auth_response.message, "Login successful");
}

#[tokio::test]
async fn test_signup_with_special_characters() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();
    let app = test_app(pool, config);

    let signup_req = SignupRequest {
        username: "user_test-123".to_string(),
        email: "user+tag@example.com".to_string(),
        password: "P@ssw0rd!#$%".to_string(),
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

    assert_eq!(auth_response.user.username, "user_test-123");
    assert_eq!(auth_response.user.email, "user+tag@example.com");
}

#[tokio::test]
async fn test_jwt_token_is_valid() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();
    let app = test_app(pool, config.clone());

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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

    // Decode the JWT token and verify it's valid
    let claims = lib_auth::decode_jwt(&auth_response.token, &config.jwt_secret)
        .expect("JWT decoding should succeed for valid token");

    // Assert
    assert_eq!(claims.username, "testuser");
    assert!(!claims.sub.is_empty());
}

