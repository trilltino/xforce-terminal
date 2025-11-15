//! # Swap Repository
//!
//! Provides database access layer for swap transaction operations.
//!
//! This module implements the repository pattern for swap data access,
//! providing a clean abstraction over SQL queries for swap transactions.
//!
//! ## Example
//!
//! ```rust,no_run
//! use backend::database::{SwapRepository, create_pool};
//! use backend::database::models::{Swap, SwapStatus};
//!
//! # async fn example() -> Result<(), sqlx::Error> {
//! let pool = create_pool().await?;
//!
//! // Create a new swap record
//! let swap = SwapRepository::create(
//!     &pool,
//!     1, // user_id
//!     "signature123",
//!     "input_mint",
//!     "output_mint",
//!     1000000,
//!     2000000,
//!     Some(0.05),
//!     Some(50),
//! ).await?;
//!
//! // Find swap by signature
//! let found = SwapRepository::find_by_signature(&pool, "signature123").await?;
//! assert!(found.is_some());
//! # Ok(())
//! # }
//! ```

use super::models::{Swap, SwapStatus};
use super::DbPool;
use sqlx::query_as;
use chrono::Utc;

/// Swap repository for database operations.
///
/// Provides methods for creating, retrieving, and updating swap records.
/// All methods are async and return `Result` types for proper error handling.
pub struct SwapRepository;

impl SwapRepository {
    /// Create a new swap record in the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `user_id` - User ID who initiated the swap
    /// * `signature` - Transaction signature (unique identifier)
    /// * `input_mint` - Input token mint address
    /// * `output_mint` - Output token mint address
    /// * `input_amount` - Input amount in smallest unit
    /// * `output_amount` - Output amount in smallest unit
    /// * `price_impact` - Optional price impact percentage
    /// * `slippage_bps` - Optional slippage tolerance in basis points
    ///
    /// # Returns
    ///
    /// * `Ok(Swap)` - The newly created swap record
    /// * `Err(sqlx::Error)` - Database error (e.g., duplicate signature)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::database::SwapRepository;
    /// use backend::database::create_pool;
    ///
    /// # async fn example() -> Result<(), sqlx::Error> {
    /// let pool = create_pool().await?;
    /// let swap = SwapRepository::create(
    ///     &pool,
    ///     1,
    ///     "signature123",
    ///     "input_mint",
    ///     "output_mint",
    ///     1000000,
    ///     2000000,
    ///     Some(0.05),
    ///     Some(50),
    /// ).await?;
    /// println!("Created swap with ID: {}", swap.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(
        pool: &DbPool,
        user_id: i64,
        signature: &str,
        input_mint: &str,
        output_mint: &str,
        input_amount: i64,
        output_amount: i64,
        price_impact: Option<f64>,
        slippage_bps: Option<i32>,
    ) -> Result<Swap, sqlx::Error> {
        let status = SwapStatus::Pending;
        let created_at = Utc::now();

        // Insert swap record
        sqlx::query(
            r#"
            INSERT INTO swaps (user_id, signature, input_mint, output_mint, input_amount, output_amount, price_impact, slippage_bps, status, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#
        )
        .bind(user_id)
        .bind(signature)
        .bind(input_mint)
        .bind(output_mint)
        .bind(input_amount)
        .bind(output_amount)
        .bind(price_impact)
        .bind(slippage_bps)
        .bind(status.to_string())
        .bind(created_at)
        .execute(pool)
        .await?;

        // Fetch the created swap record (should always succeed after insert)
        Self::find_by_signature(pool, signature)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    /// Find a swap by transaction signature.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `signature` - Transaction signature to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Swap))` - Swap found with matching signature
    /// * `Ok(None)` - No swap found with that signature
    /// * `Err(sqlx::Error)` - Database error occurred
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::database::SwapRepository;
    /// use backend::database::create_pool;
    ///
    /// # async fn example() -> Result<(), sqlx::Error> {
    /// let pool = create_pool().await?;
    /// let swap = SwapRepository::find_by_signature(&pool, "signature123").await?;
    /// if let Some(swap) = swap {
    ///     println!("Found swap: {}", swap.signature);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_by_signature(pool: &DbPool, signature: &str) -> Result<Option<Swap>, sqlx::Error> {
        query_as::<_, Swap>("SELECT * FROM swaps WHERE signature = ?")
            .bind(signature)
            .fetch_optional(pool)
            .await
    }

    /// Find all swaps for a user.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `user_id` - User ID to search for
    /// * `limit` - Maximum number of swaps to return (optional)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Swap>)` - List of swaps for the user, ordered by created_at DESC
    /// * `Err(sqlx::Error)` - Database error occurred
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::database::SwapRepository;
    /// use backend::database::create_pool;
    ///
    /// # async fn example() -> Result<(), sqlx::Error> {
    /// let pool = create_pool().await?;
    /// let swaps = SwapRepository::find_by_user(&pool, 1, Some(10)).await?;
    /// println!("Found {} swaps", swaps.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_by_user(
        pool: &DbPool,
        user_id: i64,
        limit: Option<usize>,
    ) -> Result<Vec<Swap>, sqlx::Error> {
        if let Some(limit) = limit {
            query_as::<_, Swap>(
                "SELECT * FROM swaps WHERE user_id = ? ORDER BY created_at DESC LIMIT ?"
            )
            .bind(user_id)
            .bind(limit as i64)
            .fetch_all(pool)
            .await
        } else {
            query_as::<_, Swap>(
                "SELECT * FROM swaps WHERE user_id = ? ORDER BY created_at DESC"
            )
            .bind(user_id)
            .fetch_all(pool)
            .await
        }
    }

    /// Update swap status.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `signature` - Transaction signature
    /// * `status` - New status
    /// * `error_message` - Optional error message if status is Failed
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Status updated successfully
    /// * `Err(sqlx::Error)` - Database error occurred
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::database::{SwapRepository, models::SwapStatus};
    /// use backend::database::create_pool;
    ///
    /// # async fn example() -> Result<(), sqlx::Error> {
    /// let pool = create_pool().await?;
    /// SwapRepository::update_status(&pool, "signature123", SwapStatus::Confirmed, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_status(
        pool: &DbPool,
        signature: &str,
        status: SwapStatus,
        error_message: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let confirmed_at = if status == SwapStatus::Confirmed {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query(
            r#"
            UPDATE swaps 
            SET status = ?, error_message = ?, confirmed_at = ?
            WHERE signature = ?
            "#
        )
        .bind(status.to_string())
        .bind(error_message)
        .bind(confirmed_at)
        .bind(signature)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> DbPool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS swaps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                signature TEXT UNIQUE NOT NULL,
                input_mint TEXT NOT NULL,
                output_mint TEXT NOT NULL,
                input_amount INTEGER NOT NULL,
                output_amount INTEGER NOT NULL,
                price_impact REAL,
                slippage_bps INTEGER,
                status TEXT NOT NULL,
                error_message TEXT,
                created_at TIMESTAMP NOT NULL,
                confirmed_at TIMESTAMP
            )
            "#
        )
        .execute(&pool)
        .await
        .expect("Failed to create swaps table");

        pool
    }

    #[tokio::test]
    async fn test_create_swap() {
        let pool = setup_test_db().await;

        let swap = SwapRepository::create(
            &pool,
            1,
            "signature123",
            "input_mint",
            "output_mint",
            1000000,
            2000000,
            Some(0.05),
            Some(50),
        )
        .await
        .unwrap();

        assert_eq!(swap.signature, "signature123");
        assert_eq!(swap.user_id, 1);
        assert_eq!(swap.status, SwapStatus::Pending);
    }

    #[tokio::test]
    async fn test_find_by_signature() {
        let pool = setup_test_db().await;

        SwapRepository::create(
            &pool,
            1,
            "signature123",
            "input_mint",
            "output_mint",
            1000000,
            2000000,
            None,
            None,
        )
        .await
        .unwrap();

        let found = SwapRepository::find_by_signature(&pool, "signature123")
            .await
            .unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().signature, "signature123");
    }

    #[tokio::test]
    async fn test_update_status() {
        let pool = setup_test_db().await;

        SwapRepository::create(
            &pool,
            1,
            "signature123",
            "input_mint",
            "output_mint",
            1000000,
            2000000,
            None,
            None,
        )
        .await
        .unwrap();

        SwapRepository::update_status(&pool, "signature123", SwapStatus::Confirmed, None)
            .await
            .unwrap();

        let swap = SwapRepository::find_by_signature(&pool, "signature123")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(swap.status, SwapStatus::Confirmed);
        assert!(swap.confirmed_at.is_some());
    }
}

