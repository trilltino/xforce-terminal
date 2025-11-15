//! # Auth Handler Tests
//!
//! Test suite for authentication handlers (signup and login).

mod signup;
mod login;
mod integration;

use super::*;
use lib_auth::hash_password;
use lib_core::model::store::user_repository::UserRepository;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

/// Setup test database with schema
pub async fn setup_test_db() -> DbPool {
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
pub fn test_config() -> Config {
    Config {
        database_url: "sqlite::memory:".to_string(),
        jwt_secret: "test-secret-key-must-be-at-least-32-characters-long!".to_string(),
        jwt_expiration_hours: 24,
    }
}

/// Application state for testing
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub config: Config,
}

/// Create test app with routes
pub fn test_app(pool: DbPool, config: Config) -> Router {
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

