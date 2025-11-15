use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// User entity representing a complete user record from the database.
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub wallet_address: Option<String>,
    pub wallet_connected_at: Option<DateTime<Utc>>,
    pub wallet_setup_token: Option<String>,
    pub wallet_setup_token_expires_at: Option<DateTime<Utc>>,
}

/// Data structure for creating a new user.
///
/// Contains only the fields required for user creation.
/// Password should be hashed before creating.
#[derive(Debug, Clone)]
pub struct UserForCreate {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

impl UserForCreate {
    /// Create a new `UserForCreate` instance.
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        Self {
            username,
            email,
            password_hash,
        }
    }
}

/// Data structure for updating an existing user.
///
/// All fields are optional - only provided fields will be updated.
#[derive(Debug, Clone, Default)]
pub struct UserForUpdate {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub is_active: Option<bool>,
    pub wallet_address: Option<String>,
    pub wallet_setup_token: Option<String>,
    pub wallet_setup_token_expires_at: Option<DateTime<Utc>>,
}

impl UserForUpdate {
    /// Create a new empty `UserForUpdate` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the username.
    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    /// Set the email.
    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    /// Set the password hash.
    pub fn password_hash(mut self, password_hash: String) -> Self {
        self.password_hash = Some(password_hash);
        self
    }

    /// Set the active status.
    pub fn is_active(mut self, is_active: bool) -> Self {
        self.is_active = Some(is_active);
        self
    }

    /// Set the wallet address.
    pub fn wallet_address(mut self, wallet_address: String) -> Self {
        self.wallet_address = Some(wallet_address);
        self
    }

    /// Set the wallet setup token.
    pub fn wallet_setup_token(mut self, token: String) -> Self {
        self.wallet_setup_token = Some(token);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SwapStatus {
    Pending,
    Confirmed,
    Failed,
}

impl std::fmt::Display for SwapStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapStatus::Pending => write!(f, "pending"),
            SwapStatus::Confirmed => write!(f, "confirmed"),
            SwapStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for SwapStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(SwapStatus::Pending),
            "confirmed" => Ok(SwapStatus::Confirmed),
            "failed" => Ok(SwapStatus::Failed),
            _ => Err(format!("Invalid swap status: {}", s)),
        }
    }
}

impl From<String> for SwapStatus {
    fn from(s: String) -> Self {
        use std::str::FromStr;
        // Fall back to Failed if parsing fails (defensive approach for database data)
        SwapStatus::from_str(&s).unwrap_or_else(|_| SwapStatus::Failed)
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Swap {
    pub id: i64,
    pub user_id: i64,
    pub signature: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: i64,
    pub output_amount: i64,
    pub price_impact: Option<f64>,
    pub slippage_bps: Option<i32>,
    #[sqlx(try_from = "String")]
    pub status: SwapStatus,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}
