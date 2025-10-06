pub mod models;
pub mod repository;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::env;

pub type DbPool = SqlitePool;

pub async fn create_pool() -> anyhow::Result<DbPool> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:terminal.db".to_string());

    let options = database_url
        .parse::<SqliteConnectOptions>()?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    Ok(pool)
}
