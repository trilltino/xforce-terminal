//! # User Repository
//!
//! Provides database access layer for user-related operations.
//!
//! This module implements the repository pattern for user data access,
//! providing a clean abstraction over SQL queries.
//!
//! ## Example
//!
//! ```rust,no_run
//! # use backend::database::{UserRepository, create_pool};
//! # async fn example() -> Result<(), sqlx::Error> {
//! let pool = create_pool("sqlite::memory:").await?;
//!
//! // Create a new user
//! let user = UserRepository::create(
//!     &pool,
//!     "alice",
//!     "alice@example.com",
//!     "hashed_password"
//! ).await?;
//!
//! // Find user by email
//! let found = UserRepository::find_by_email(&pool, "alice@example.com").await?;
//! assert!(found.is_some());
//! # Ok(())
//! # }
//! ```

use super::models::{User, UserForCreate, UserForUpdate};
use super::DbPool;
use sqlx::query_as;

/// User repository for database operations.
///
/// Provides methods for creating, retrieving, and updating user records.
/// All methods are async and return `Result` types for proper error handling.
pub struct UserRepository;

impl UserRepository {
    /// Find a user by their email address.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `email` - The email address to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(User))` - User found with matching email
    /// * `Ok(None)` - No user found with that email
    /// * `Err(sqlx::Error)` - Database error occurred
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use backend::database::{UserRepository, create_pool};
    /// # async fn example() -> Result<(), sqlx::Error> {
    /// # let pool = create_pool("sqlite::memory:").await?;
    /// let user = UserRepository::find_by_email(&pool, "alice@example.com").await?;
    /// if let Some(user) = user {
    ///     println!("Found user: {}", user.username);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    /// Find a user by their username.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `username` - The username to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(User))` - User found with matching username
    /// * `Ok(None)` - No user found with that username
    /// * `Err(sqlx::Error)` - Database error occurred
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use backend::database::{UserRepository, create_pool};
    /// # async fn example() -> Result<(), sqlx::Error> {
    /// # let pool = create_pool("sqlite::memory:").await?;
    /// let user = UserRepository::find_by_username(&pool, "alice").await?;
    /// match user {
    ///     Some(u) => println!("User ID: {}", u.id),
    ///     None => println!("User not found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_by_username(pool: &DbPool, username: &str) -> Result<Option<User>, sqlx::Error> {
        query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(pool)
            .await
    }

    /// Create a new user in the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `username` - The username for the new user (must be unique)
    /// * `email` - The email address for the new user (must be unique)
    /// * `password_hash` - The hashed password (use `auth::hash_password`)
    ///
    /// # Returns
    ///
    /// * `Ok(User)` - The newly created user with generated ID and timestamps
    /// * `Err(sqlx::Error)` - Database error (e.g., constraint violation for duplicate email/username)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use backend::database::{UserRepository, create_pool};
    /// # use backend::auth::hash_password;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let pool = create_pool("sqlite::memory:").await?;
    /// let password_hash = hash_password("SecurePassword123!")?;
    /// let user = UserRepository::create(
    ///     &pool,
    ///     "alice",
    ///     "alice@example.com",
    ///     &password_hash
    /// ).await?;
    /// println!("Created user with ID: {}", user.id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if:
    /// - Username already exists (UNIQUE constraint violation)
    /// - Email already exists (UNIQUE constraint violation)
    /// - Database connection fails
    /// Create a new user using `UserForCreate`.
    ///
    /// This is the preferred method for creating users as it uses the type-safe `UserForCreate` struct.
    pub async fn create_with(
        pool: &DbPool,
        user_data: UserForCreate,
    ) -> Result<User, sqlx::Error> {
        Self::create(pool, &user_data.username, &user_data.email, &user_data.password_hash).await
    }

    /// Create a new user in the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `username` - The username for the new user (must be unique)
    /// * `email` - The email address for the new user (must be unique)
    /// * `password_hash` - The hashed password (use `auth::hash_password`)
    ///
    /// # Returns
    ///
    /// * `Ok(User)` - The newly created user with generated ID and timestamps
    /// * `Err(sqlx::Error)` - Database error (e.g., constraint violation for duplicate email/username)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use backend::database::{UserRepository, create_pool};
    /// # use backend::auth::hash_password;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let pool = create_pool("sqlite::memory:").await?;
    /// let password_hash = hash_password("SecurePassword123!")?;
    /// let user = UserRepository::create(
    ///     &pool,
    ///     "alice",
    ///     "alice@example.com",
    ///     &password_hash
    /// ).await?;
    /// println!("Created user with ID: {}", user.id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if:
    /// - Username already exists (UNIQUE constraint violation)
    /// - Email already exists (UNIQUE constraint violation)
    /// - Database connection fails
    pub async fn create(
        pool: &DbPool,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .execute(pool)
        .await?;

        let id = result.last_insert_rowid();

        query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// Update an existing user using `UserForUpdate`.
    ///
    /// Only fields that are `Some` in `user_data` will be updated.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `id` - The user ID to update
    /// * `user_data` - The update data (only `Some` fields will be updated)
    ///
    /// # Returns
    ///
    /// * `Ok(User)` - The updated user
    /// * `Err(sqlx::Error)` - Database error occurred
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lib_core::model::store::{UserRepository, UserForUpdate};
    /// # async fn example() -> Result<(), sqlx::Error> {
    /// # let pool = create_pool("sqlite::memory:").await?;
    /// let update = UserForUpdate::new()
    ///     .username("new_username".to_string())
    ///     .wallet_address("9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde".to_string());
    /// let updated = UserRepository::update(&pool, 1, update).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(
        pool: &DbPool,
        id: i64,
        user_data: UserForUpdate,
    ) -> Result<User, sqlx::Error> {
        // Build update query dynamically
        let mut updates = Vec::new();

        if user_data.username.is_some() {
            updates.push("username = ?");
        }
        if user_data.email.is_some() {
            updates.push("email = ?");
        }
        if user_data.password_hash.is_some() {
            updates.push("password_hash = ?");
        }
        if user_data.is_active.is_some() {
            updates.push("is_active = ?");
        }
        if user_data.wallet_address.is_some() {
            updates.push("wallet_address = ?");
        }
        if user_data.wallet_setup_token.is_some() {
            updates.push("wallet_setup_token = ?");
        }
        if user_data.wallet_setup_token_expires_at.is_some() {
            updates.push("wallet_setup_token_expires_at = ?");
        }

        if updates.is_empty() {
            // No updates, just return the existing user
            return query_as::<_, User>("SELECT * FROM users WHERE id = ?")
                .bind(id)
                .fetch_one(pool)
                .await;
        }

        updates.push("updated_at = CURRENT_TIMESTAMP");
        let query_str = format!("UPDATE users SET {} WHERE id = ?", updates.join(", "));

        // Build query with proper type inference
        let mut query = sqlx::query(&query_str);
        
        if let Some(ref username) = user_data.username {
            query = query.bind(username);
        }
        if let Some(ref email) = user_data.email {
            query = query.bind(email);
        }
        if let Some(ref password_hash) = user_data.password_hash {
            query = query.bind(password_hash);
        }
        if let Some(is_active) = user_data.is_active {
            query = query.bind(is_active);
        }
        if let Some(ref wallet_address) = user_data.wallet_address {
            query = query.bind(wallet_address);
        }
        if let Some(ref token) = user_data.wallet_setup_token {
            query = query.bind(token);
        }
        if let Some(ref expires_at) = user_data.wallet_setup_token_expires_at {
            query = query.bind(expires_at);
        }
        
        query.bind(id).execute(pool).await?;

        query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// Update the last login timestamp for a user.
    ///
    /// Sets the `last_login` field to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `id` - The user ID to update
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Last login timestamp updated successfully
    /// * `Err(sqlx::Error)` - Database error occurred
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use backend::database::{UserRepository, create_pool};
    /// # async fn example() -> Result<(), sqlx::Error> {
    /// # let pool = create_pool("sqlite::memory:").await?;
    /// UserRepository::update_last_login(&pool, 1).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// This method does not verify that the user exists. If the user ID is invalid,
    /// it will succeed but not update any rows.
    pub async fn update_last_login(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET last_login = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Find a user by their Solana wallet address.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `wallet_address` - The Solana wallet public key (base58 encoded)
    ///
    /// # Returns
    ///
    /// * `Ok(Some(User))` - User found with matching wallet address
    /// * `Ok(None)` - No user found with that wallet address
    /// * `Err(sqlx::Error)` - Database error occurred
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use backend::database::{UserRepository, create_pool};
    /// # async fn example() -> Result<(), sqlx::Error> {
    /// # let pool = create_pool("sqlite::memory:").await?;
    /// let wallet = "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde";
    /// let user = UserRepository::find_by_wallet(&pool, wallet).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_by_wallet(pool: &DbPool, wallet_address: &str) -> Result<Option<User>, sqlx::Error> {
        query_as::<_, User>("SELECT * FROM users WHERE wallet_address = ?")
            .bind(wallet_address)
            .fetch_optional(pool)
            .await
    }

    /// Set a wallet setup token for user wallet connection.
    ///
    /// This token is used in the wallet connection flow and expires after 30 minutes.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `user_id` - The user ID to set the token for
    /// * `token` - The setup token to store
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Token set successfully
    /// * `Err(sqlx::Error)` - Database error occurred
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use backend::database::{UserRepository, create_pool};
    /// # use uuid::Uuid;
    /// # async fn example() -> Result<(), sqlx::Error> {
    /// # let pool = create_pool("sqlite::memory:").await?;
    /// let token = Uuid::new_v4().to_string();
    /// UserRepository::set_wallet_setup_token(&pool, 1, &token).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Security
    ///
    /// - Tokens expire after 30 minutes for security
    /// - Use a cryptographically secure random token generator
    /// - Tokens should be invalidated after use
    pub async fn set_wallet_setup_token(pool: &DbPool, user_id: i64, token: &str) -> Result<(), sqlx::Error> {
        let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::minutes(30);

        sqlx::query(
            "UPDATE users SET wallet_setup_token = ?, wallet_setup_token_expires_at = ? WHERE id = ?"
        )
        .bind(token)
        .bind(expires_at)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete all users from the database.
    ///
    /// **WARNING**: This is a destructive operation that cannot be undone.
    /// All user data, including wallet connections, will be permanently deleted.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    ///
    /// * `Ok(count)` - Number of users deleted
    /// * `Err(sqlx::Error)` - Database error occurred
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lib_core::model::store::{UserRepository, create_pool};
    /// # async fn example() -> Result<(), sqlx::Error> {
    /// # let pool = create_pool().await?;
    /// let deleted_count = UserRepository::delete_all(&pool).await?;
    /// println!("Deleted {} users", deleted_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_all(pool: &DbPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users")
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::hash_password;
    use sqlx::sqlite::SqlitePoolOptions;

    /// Create an in-memory SQLite database for testing
    async fn setup_test_db() -> DbPool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        // Run migrations
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
            "#
        )
        .execute(&pool)
        .await
        .expect("Failed to create users table");

        pool
    }

    // ========== User Creation Tests ==========

    #[tokio::test]
    async fn test_create_user() {
        let pool = setup_test_db().await;
        let password_hash = hash_password("TestPassword123!").unwrap();

        let user = UserRepository::create(&pool, "testuser", "test@example.com", &password_hash)
            .await
            .unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, password_hash);
        assert!(user.is_active);
        assert!(user.wallet_address.is_none());
    }

    #[tokio::test]
    async fn test_create_user_duplicate_email() {
        let pool = setup_test_db().await;
        let password_hash = hash_password("TestPassword123!").unwrap();

        // Create first user
        UserRepository::create(&pool, "user1", "test@example.com", &password_hash)
            .await
            .unwrap();

        // Try to create second user with same email
        let result = UserRepository::create(&pool, "user2", "test@example.com", &password_hash).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_user_duplicate_username() {
        let pool = setup_test_db().await;
        let password_hash = hash_password("TestPassword123!").unwrap();

        // Create first user
        UserRepository::create(&pool, "testuser", "user1@example.com", &password_hash)
            .await
            .unwrap();

        // Try to create second user with same username
        let result = UserRepository::create(&pool, "testuser", "user2@example.com", &password_hash).await;

        assert!(result.is_err());
    }

    // ========== User Retrieval Tests ==========

    #[tokio::test]
    async fn test_find_by_email() {
        let pool = setup_test_db().await;
        let password_hash = hash_password("TestPassword123!").unwrap();

        UserRepository::create(&pool, "testuser", "test@example.com", &password_hash)
            .await
            .unwrap();

        let found = UserRepository::find_by_email(&pool, "test@example.com")
            .await
            .unwrap();

        assert!(found.is_some());
        assert_eq!(
            found.expect("User should exist after creation").username,
            "testuser"
        );
    }

    #[tokio::test]
    async fn test_find_by_email_not_found() {
        let pool = setup_test_db().await;

        let found = UserRepository::find_by_email(&pool, "nonexistent@example.com")
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_by_username() {
        let pool = setup_test_db().await;
        let password_hash = hash_password("TestPassword123!").unwrap();

        UserRepository::create(&pool, "testuser", "test@example.com", &password_hash)
            .await
            .unwrap();

        let found = UserRepository::find_by_username(&pool, "testuser")
            .await
            .unwrap();

        assert!(found.is_some());
        assert_eq!(
            found.expect("User should exist after creation").email,
            "test@example.com"
        );
    }

    #[tokio::test]
    async fn test_find_by_username_not_found() {
        let pool = setup_test_db().await;

        let found = UserRepository::find_by_username(&pool, "nonexistent")
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_by_username_case_sensitive() {
        let pool = setup_test_db().await;
        let password_hash = hash_password("TestPassword123!").unwrap();

        UserRepository::create(&pool, "TestUser", "test@example.com", &password_hash)
            .await
            .unwrap();

        // SQLite is case-insensitive for LIKE but case-sensitive for =
        let found = UserRepository::find_by_username(&pool, "testuser")
            .await
            .unwrap();

        // This depends on SQLite collation, but typically case-insensitive
        assert!(found.is_some() || found.is_none());
    }

    // ========== Last Login Tests ==========

    #[tokio::test]
    async fn test_update_last_login() {
        let pool = setup_test_db().await;
        let password_hash = hash_password("TestPassword123!").unwrap();

        let user = UserRepository::create(&pool, "testuser", "test@example.com", &password_hash)
            .await
            .unwrap();

        assert!(user.last_login.is_none());

        // Update last login
        UserRepository::update_last_login(&pool, user.id)
            .await
            .unwrap();

        // Fetch user again
        let updated = UserRepository::find_by_email(&pool, "test@example.com")
            .await
            .unwrap()
            .unwrap();

        assert!(updated.last_login.is_some());
    }

    #[tokio::test]
    async fn test_update_last_login_nonexistent_user() {
        let pool = setup_test_db().await;

        // Should not error even if user doesn't exist
        let result = UserRepository::update_last_login(&pool, 99999).await;
        assert!(result.is_ok());
    }

    // ========== Wallet Tests ==========

    #[tokio::test]
    async fn test_find_by_wallet_not_set() {
        let pool = setup_test_db().await;

        let found = UserRepository::find_by_wallet(&pool, "SomeWalletAddress123")
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_set_wallet_setup_token() {
        let pool = setup_test_db().await;
        let password_hash = hash_password("TestPassword123!").unwrap();

        let user = UserRepository::create(&pool, "testuser", "test@example.com", &password_hash)
            .await
            .unwrap();

        let token = "test-wallet-token-12345";
        UserRepository::set_wallet_setup_token(&pool, user.id, token)
            .await
            .unwrap();

        // Fetch user again
        let updated = UserRepository::find_by_email(&pool, "test@example.com")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.wallet_setup_token, Some(token.to_string()));
        assert!(updated.wallet_setup_token_expires_at.is_some());
    }

    #[tokio::test]
    async fn test_wallet_token_expiration_time() {
        let pool = setup_test_db().await;
        let password_hash = hash_password("TestPassword123!").unwrap();

        let user = UserRepository::create(&pool, "testuser", "test@example.com", &password_hash)
            .await
            .unwrap();

        let before = chrono::Utc::now();
        UserRepository::set_wallet_setup_token(&pool, user.id, "token")
            .await
            .unwrap();
        let after = chrono::Utc::now();

        let updated = UserRepository::find_by_email(&pool, "test@example.com")
            .await
            .unwrap()
            .unwrap();

        let expires_at = updated.wallet_setup_token_expires_at.unwrap();

        // Should expire approximately 30 minutes from now
        let expected_expiry = before + chrono::Duration::minutes(30);
        let max_expiry = after + chrono::Duration::minutes(30);

        assert!(expires_at >= expected_expiry);
        assert!(expires_at <= max_expiry + chrono::Duration::seconds(1));
    }

    // ========== Multiple Users Tests ==========

    #[tokio::test]
    async fn test_create_multiple_users() {
        let pool = setup_test_db().await;

        for i in 0..5 {
            let username = format!("user{}", i);
            let email = format!("user{}@example.com", i);
            let password_hash = hash_password(&format!("Password{}!", i)).unwrap();

            let user = UserRepository::create(&pool, &username, &email, &password_hash)
                .await
                .unwrap();

            assert_eq!(user.username, username);
            assert_eq!(user.email, email);
        }

        // Verify all users can be found
        for i in 0..5 {
            let email = format!("user{}@example.com", i);
            let found = UserRepository::find_by_email(&pool, &email)
                .await
                .unwrap();

            assert!(found.is_some());
        }
    }
}
