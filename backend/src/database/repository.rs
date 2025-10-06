use super::models::User;
use super::DbPool;
use sqlx::query_as;

pub struct UserRepository;

impl UserRepository {
    /// Find user by email
    pub async fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    /// Find user by username
    pub async fn find_by_username(pool: &DbPool, username: &str) -> Result<Option<User>, sqlx::Error> {
        query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(pool)
            .await
    }

    /// Create a new user
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

    /// Update last login timestamp
    pub async fn update_last_login(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET last_login = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
