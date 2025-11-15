use super::{models::User, user_repository::UserRepository, DbPool};

/// Find user by wallet address
pub async fn find_by_wallet(pool: &DbPool, wallet_address: &str) -> Result<Option<User>, sqlx::Error> {
    UserRepository::find_by_wallet(pool, wallet_address).await
}

/// Set wallet setup token for user
pub async fn set_wallet_setup_token(pool: &DbPool, user_id: i64, token: &str) -> Result<(), sqlx::Error> {
    UserRepository::set_wallet_setup_token(pool, user_id, token).await
}
