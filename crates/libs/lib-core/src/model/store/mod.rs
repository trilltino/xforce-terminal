//! # Database Store
//!
//! Database connection pool and repository implementations.

// region: --- Modules
pub mod models;
pub mod user_repository;
pub mod swap_repository;
pub mod users;
// endregion: --- Modules

// region: --- Re-exports
pub use user_repository::UserRepository;
// endregion: --- Re-exports

// region: --- Types and Functions
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::env;

/// Type alias for SQLite connection pool.
pub type DbPool = SqlitePool;

/// Create a new SQLite connection pool.
pub async fn create_pool() -> anyhow::Result<DbPool> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:terminal.db".to_string());

    let options = database_url
        .parse::<SqliteConnectOptions>()?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    Ok(pool)
}
// endregion: --- Types and Functions
