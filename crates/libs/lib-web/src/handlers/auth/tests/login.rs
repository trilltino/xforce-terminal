//! # Login Tests
//!
//! Tests for user login functionality.

use super::*;
use lib_auth::hash_password;
use lib_core::model::store::user_repository::UserRepository;
use axum::body::Body;
use axum::http::{Request, StatusCode};

#[tokio::test]
async fn test_login_success_with_email() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    // Create user
    let password = "TestPassword123!";
    let password_hash = hash_password(password)
        .expect("Password hashing should succeed in test");
    UserRepository::create(&pool, "testuser", "test@example.com", &password_hash)
        .await
        .expect("User creation should succeed in test");

    let app = test_app(pool, config);

    let login_req = LoginRequest {
        email_or_username: "test@example.com".to_string(),
        password: password.to_string(),
    };

    // Act
    let response = app
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
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(auth_response.user.username, "testuser");
    assert_eq!(auth_response.user.email, "test@example.com");
    assert_eq!(auth_response.message, "Login successful");
    assert!(!auth_response.token.is_empty());
}

#[tokio::test]
async fn test_login_success_with_username() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    // Create user
    let password = "TestPassword123!";
    let password_hash = hash_password(password)
        .expect("Password hashing should succeed in test");
    UserRepository::create(&pool, "testuser", "test@example.com", &password_hash)
        .await
        .expect("User creation should succeed in test");

    let app = test_app(pool, config);

    let login_req = LoginRequest {
        email_or_username: "testuser".to_string(),
        password: password.to_string(),
    };

    // Act
    let response = app
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
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(auth_response.user.username, "testuser");
    assert_eq!(auth_response.message, "Login successful");
}

#[tokio::test]
async fn test_login_user_not_found() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();
    let app = test_app(pool, config);

    let login_req = LoginRequest {
        email_or_username: "nonexistent@example.com".to_string(),
        password: "TestPassword123!".to_string(),
    };

    // Act
    let response = app
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
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(error_response.error, "Invalid credentials");
}

#[tokio::test]
async fn test_login_wrong_password() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    // Create user
    let password_hash = hash_password("CorrectPassword123!")
        .expect("Password hashing should succeed in test");
    UserRepository::create(&pool, "testuser", "test@example.com", &password_hash)
        .await
        .expect("User creation should succeed in test");

    let app = test_app(pool, config);

    let login_req = LoginRequest {
        email_or_username: "test@example.com".to_string(),
        password: "WrongPassword123!".to_string(),
    };

    // Act
    let response = app
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
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(error_response.error, "Invalid credentials");
}

#[tokio::test]
async fn test_login_inactive_account() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    // Create user
    let password = "TestPassword123!";
    let password_hash = hash_password(password).unwrap();
    let user = UserRepository::create(&pool, "testuser", "test@example.com", &password_hash)
        .await
        .unwrap();

    // Deactivate user
    sqlx::query("UPDATE users SET is_active = 0 WHERE id = ?")
        .bind(user.id)
        .execute(&pool)
        .await
        .expect("User deactivation should succeed in test");

    let app = test_app(pool, config);

    let login_req = LoginRequest {
        email_or_username: "test@example.com".to_string(),
        password: password.to_string(),
    };

    // Act
    let response = app
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
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(error_response.error, "Account is deactivated");
}

#[tokio::test]
async fn test_login_updates_last_login() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    // Create user
    let password = "TestPassword123!";
    let password_hash = hash_password(password).unwrap();
    let user = UserRepository::create(&pool, "testuser", "test@example.com", &password_hash)
        .await
        .unwrap();

    // Verify initial last_login is None
    assert!(user.last_login.is_none());

    let app = test_app(pool.clone(), config);

    let login_req = LoginRequest {
        email_or_username: "test@example.com".to_string(),
        password: password.to_string(),
    };

    // Act
    let response = app
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
    assert_eq!(response.status(), StatusCode::OK);

    // Check last_login was updated
    let updated_user = UserRepository::find_by_email(&pool, "test@example.com")
        .await
        .expect("User lookup should succeed in test")
        .expect("User should exist after creation");

    assert!(updated_user.last_login.is_some());
}

