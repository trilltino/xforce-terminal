use super::*;
use crate::{
    auth::hash_password,
    database::repository::UserRepository,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

/// Setup test database with schema
async fn setup_test_db() -> DbPool {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_login TIMESTAMP,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            wallet_address TEXT UNIQUE,
            wallet_connected_at TIMESTAMP,
            wallet_setup_token TEXT,
            wallet_setup_token_expires_at TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    pool
}

/// Create test config
fn test_config() -> Config {
    Config {
        database_url: "sqlite::memory:".to_string(),
        jwt_secret: "test-secret-key-must-be-at-least-32-characters-long!".to_string(),
        jwt_expiration_hours: 24,
    }
}

/// Application state for testing
#[derive(Clone)]
struct AppState {
    pool: DbPool,
    config: Config,
}

/// Create test app with routes
fn test_app(pool: DbPool, config: Config) -> Router {
    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    Router::new()
        .route("/signup", axum::routing::post(|
            axum::extract::State(AppState { pool, config }): axum::extract::State<AppState>,
            Json(req): Json<SignupRequest>,
        | async move {
            signup(
                axum::extract::State(pool),
                axum::extract::State(config),
                Json(req),
            ).await
        }))
        .route("/login", axum::routing::post(|
            axum::extract::State(AppState { pool, config }): axum::extract::State<AppState>,
            Json(req): Json<LoginRequest>,
        | async move {
            login(
                axum::extract::State(pool),
                axum::extract::State(config),
                Json(req),
            ).await
        }))
        .with_state(state)
}

// ========== Signup Tests ==========

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

// ========== Login Tests ==========

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

// ========== Edge Cases and Integration Tests ==========

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
    let claims = crate::auth::decode_jwt(&auth_response.token, &config.jwt_secret)
        .expect("JWT decoding should succeed for valid token");

    // Assert
    assert_eq!(claims.username, "testuser");
    assert!(!claims.sub.is_empty());
}

