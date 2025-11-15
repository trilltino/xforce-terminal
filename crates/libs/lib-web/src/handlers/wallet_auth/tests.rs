use super::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;
use solana_sdk::signer::{keypair::Keypair, Signer};

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

/// Create test user with wallet setup token
async fn create_test_user_with_token(pool: &DbPool) -> (i64, String, String) {
    let setup_token = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::minutes(30);

    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash, wallet_setup_token, wallet_setup_token_expires_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind("testuser")
    .bind("test@example.com")
    .bind("hash")
    .bind(&setup_token)
    .bind(expires_at)
    .execute(pool)
    .await
    .unwrap();

    (result.last_insert_rowid(), "testuser".to_string(), setup_token)
}

/// Create test user with linked wallet
async fn create_test_user_with_wallet(pool: &DbPool, wallet_address: &str) -> i64 {
    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash, wallet_address, wallet_connected_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind("walletuser")
    .bind("wallet@example.com")
    .bind("hash")
    .bind(wallet_address)
    .bind(chrono::Utc::now().naive_utc())
    .execute(pool)
    .await
    .unwrap();

    result.last_insert_rowid()
}

// ========== Validate Wallet Setup Tests ==========

#[tokio::test]
async fn test_validate_wallet_setup_success() {
    // Arrange
    let pool = setup_test_db().await;
    let (_user_id, _username, setup_token) = create_test_user_with_token(&pool).await;

    let app = Router::new()
        .route("/validate", axum::routing::get(validate_wallet_setup))
        .with_state(pool);

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/validate?token={}", setup_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let validate_response: WalletSetupValidateResponse = serde_json::from_slice(&body).unwrap();

    assert!(validate_response.valid);
    assert_eq!(validate_response.username, "testuser");
    assert!(!validate_response.challenge.is_empty());
}

#[tokio::test]
async fn test_validate_wallet_setup_invalid_token() {
    // Arrange
    let pool = setup_test_db().await;
    let app = Router::new()
        .route("/validate", axum::routing::get(validate_wallet_setup))
        .with_state(pool);

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/validate?token=invalid-token")
                .body(Body::empty())
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

    assert_eq!(error_response.error, "Invalid or expired setup token");
}

#[tokio::test]
async fn test_validate_wallet_setup_expired_token() {
    // Arrange
    let pool = setup_test_db().await;
    let setup_token = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now().naive_utc() - chrono::Duration::minutes(5); // Expired 5 minutes ago

    sqlx::query(
        "INSERT INTO users (username, email, password_hash, wallet_setup_token, wallet_setup_token_expires_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind("testuser")
    .bind("test@example.com")
    .bind("hash")
    .bind(&setup_token)
    .bind(expires_at)
    .execute(&pool)
    .await
    .unwrap();

    let app = Router::new()
        .route("/validate", axum::routing::get(validate_wallet_setup))
        .with_state(pool);

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/validate?token={}", setup_token))
                .body(Body::empty())
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

    assert_eq!(error_response.error, "Setup token expired. Please signup again.");
}

// ========== Complete Wallet Setup Tests ==========

#[tokio::test]
async fn test_complete_wallet_setup_success() {
    // Arrange
    let pool = setup_test_db().await;
    let (_user_id, _username, setup_token) = create_test_user_with_token(&pool).await;

    // Generate a real Solana keypair for testing
    let keypair = Keypair::new();
    let wallet_address = keypair.pubkey().to_string();
    let challenge = Uuid::new_v4().to_string();

    // Create the message and sign it
    let message = format!("Connect wallet to XForce Terminal\n\nChallenge: {}", challenge);
    let signature = keypair.sign_message(message.as_bytes());

    let app = Router::new()
        .route("/complete", axum::routing::post(complete_wallet_setup))
        .with_state(pool.clone());

    let request = WalletSetupCompleteRequest {
        setup_token: setup_token.clone(),
        wallet_address: wallet_address.clone(),
        signature: signature.to_string(),
        challenge: challenge.clone(),
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/complete")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let complete_response: WalletSetupCompleteResponse = serde_json::from_slice(&body).unwrap();

    assert!(complete_response.success);
    assert!(complete_response.message.contains("testuser"));

    // Verify wallet was linked in database
    let user: Option<(String,)> = sqlx::query_as("SELECT wallet_address FROM users WHERE username = ?")
        .bind("testuser")
        .fetch_optional(&pool)
        .await
        .unwrap();

    assert_eq!(user.unwrap().0, wallet_address);
}

#[tokio::test]
async fn test_complete_wallet_setup_invalid_address() {
    // Arrange
    let pool = setup_test_db().await;
    let (_user_id, _username, setup_token) = create_test_user_with_token(&pool).await;

    let app = Router::new()
        .route("/complete", axum::routing::post(complete_wallet_setup))
        .with_state(pool);

    let request = WalletSetupCompleteRequest {
        setup_token,
        wallet_address: "invalid-address".to_string(),
        signature: "invalid-signature".to_string(),
        challenge: "challenge".to_string(),
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/complete")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
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

    assert_eq!(error_response.error, "Invalid Solana wallet address");
}

#[tokio::test]
async fn test_complete_wallet_setup_invalid_signature() {
    // Arrange
    let pool = setup_test_db().await;
    let (_user_id, _username, setup_token) = create_test_user_with_token(&pool).await;

    let keypair = Keypair::new();
    let wallet_address = keypair.pubkey().to_string();

    // Sign a different message (signature won't match)
    let wrong_message = "wrong message";
    let signature = keypair.sign_message(wrong_message.as_bytes());

    let app = Router::new()
        .route("/complete", axum::routing::post(complete_wallet_setup))
        .with_state(pool);

    let request = WalletSetupCompleteRequest {
        setup_token,
        wallet_address,
        signature: signature.to_string(),
        challenge: "correct-challenge".to_string(),
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/complete")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
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

    assert_eq!(error_response.error, "Signature verification failed");
}

#[tokio::test]
async fn test_complete_wallet_setup_invalid_token() {
    // Arrange
    let pool = setup_test_db().await;

    let keypair = Keypair::new();
    let wallet_address = keypair.pubkey().to_string();
    let challenge = Uuid::new_v4().to_string();
    let message = format!("Connect wallet to XForce Terminal\n\nChallenge: {}", challenge);
    let signature = keypair.sign_message(message.as_bytes());

    let app = Router::new()
        .route("/complete", axum::routing::post(complete_wallet_setup))
        .with_state(pool);

    let request = WalletSetupCompleteRequest {
        setup_token: "invalid-token".to_string(),
        wallet_address,
        signature: signature.to_string(),
        challenge,
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/complete")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
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

    assert_eq!(error_response.error, "Invalid setup token");
}

#[tokio::test]
async fn test_complete_wallet_setup_duplicate_wallet() {
    // Arrange
    let pool = setup_test_db().await;

    // Create first user with wallet
    let keypair = Keypair::new();
    let wallet_address = keypair.pubkey().to_string();
    create_test_user_with_wallet(&pool, &wallet_address).await;

    // Create second user trying to link same wallet
    let (_user_id, _username, setup_token) = create_test_user_with_token(&pool).await;

    let challenge = Uuid::new_v4().to_string();
    let message = format!("Connect wallet to XForce Terminal\n\nChallenge: {}", challenge);
    let signature = keypair.sign_message(message.as_bytes());

    let app = Router::new()
        .route("/complete", axum::routing::post(complete_wallet_setup))
        .with_state(pool);

    let request = WalletSetupCompleteRequest {
        setup_token,
        wallet_address,
        signature: signature.to_string(),
        challenge,
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/complete")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
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

    assert!(error_response.error.contains("already connected"));
    assert!(error_response.error.contains("walletuser"));
}

// ========== Wallet Login Tests ==========

#[tokio::test]
async fn test_wallet_login_success() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    let keypair = Keypair::new();
    let wallet_address = keypair.pubkey().to_string();
    create_test_user_with_wallet(&pool, &wallet_address).await;

    #[derive(Clone)]
    struct AppState {
        pool: DbPool,
        config: Config,
    }

    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    let app = Router::new()
        .route("/wallet-login", axum::routing::post(|
            axum::extract::State(AppState { pool, config }): axum::extract::State<AppState>,
            Json(req): Json<WalletLoginRequest>,
        | async move {
            wallet_login(
                axum::extract::State(pool),
                axum::extract::State(config),
                Json(req),
            ).await
        }))
        .with_state(state);

    let challenge = Uuid::new_v4().to_string();
    let message = format!("Login to XForce Terminal\n\nChallenge: {}", challenge);
    let signature = keypair.sign_message(message.as_bytes());

    let request = WalletLoginRequest {
        wallet_address: wallet_address.clone(),
        signature: signature.to_string(),
        challenge,
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet-login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_response: lib_core::dto::AuthResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(auth_response.user.username, "walletuser");
    assert_eq!(auth_response.user.wallet_address, Some(wallet_address));
    assert_eq!(auth_response.message, "Successfully logged in with wallet");
    assert!(!auth_response.token.is_empty());
}

#[tokio::test]
async fn test_wallet_login_invalid_signature() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    let keypair = Keypair::new();
    let wallet_address = keypair.pubkey().to_string();
    create_test_user_with_wallet(&pool, &wallet_address).await;

    #[derive(Clone)]
    struct AppState {
        pool: DbPool,
        config: Config,
    }

    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    let app = Router::new()
        .route("/wallet-login", axum::routing::post(|
            axum::extract::State(AppState { pool, config }): axum::extract::State<AppState>,
            Json(req): Json<WalletLoginRequest>,
        | async move {
            wallet_login(
                axum::extract::State(pool),
                axum::extract::State(config),
                Json(req),
            ).await
        }))
        .with_state(state);

    // Sign wrong message
    let wrong_message = "wrong message";
    let signature = keypair.sign_message(wrong_message.as_bytes());

    let request = WalletLoginRequest {
        wallet_address,
        signature: signature.to_string(),
        challenge: "correct-challenge".to_string(),
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet-login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
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

    assert_eq!(error_response.error, "Invalid signature");
}

#[tokio::test]
async fn test_wallet_login_wallet_not_linked() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    let keypair = Keypair::new();
    let wallet_address = keypair.pubkey().to_string();
    // Note: NOT creating user with this wallet

    #[derive(Clone)]
    struct AppState {
        pool: DbPool,
        config: Config,
    }

    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    let app = Router::new()
        .route("/wallet-login", axum::routing::post(|
            axum::extract::State(AppState { pool, config }): axum::extract::State<AppState>,
            Json(req): Json<WalletLoginRequest>,
        | async move {
            wallet_login(
                axum::extract::State(pool),
                axum::extract::State(config),
                Json(req),
            ).await
        }))
        .with_state(state);

    let challenge = Uuid::new_v4().to_string();
    let message = format!("Login to XForce Terminal\n\nChallenge: {}", challenge);
    let signature = keypair.sign_message(message.as_bytes());

    let request = WalletLoginRequest {
        wallet_address,
        signature: signature.to_string(),
        challenge,
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet-login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
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

    assert_eq!(error_response.error, "No account found with this wallet. Please connect your wallet to an account first.");
}

#[tokio::test]
async fn test_wallet_login_invalid_wallet_address() {
    // Arrange
    let pool = setup_test_db().await;
    let config = test_config();

    #[derive(Clone)]
    struct AppState {
        pool: DbPool,
        config: Config,
    }

    let state = AppState { pool, config };

    let app = Router::new()
        .route("/wallet-login", axum::routing::post(|
            axum::extract::State(AppState { pool, config }): axum::extract::State<AppState>,
            Json(req): Json<WalletLoginRequest>,
        | async move {
            wallet_login(
                axum::extract::State(pool),
                axum::extract::State(config),
                Json(req),
            ).await
        }))
        .with_state(state);

    let request = WalletLoginRequest {
        wallet_address: "invalid-address".to_string(),
        signature: "invalid-signature".to_string(),
        challenge: "challenge".to_string(),
    };

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet-login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
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

    assert_eq!(error_response.error, "Invalid wallet address");
}

