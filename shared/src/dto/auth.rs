use serde::{Deserialize, Serialize};

/// Login request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginRequest {
    pub email_or_username: String,
    pub password: String,
}

/// Signup request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Authentication response (login/signup success)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthResponse {
    pub user: UserInfo,
    pub token: String,
    pub message: String,
}

/// User information (public, safe to send to client)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorResponse {
    pub error: String,
}

/// Price data for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub timestamp: i64,
    pub price: f64,
}

/// Market data response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataResponse {
    pub asset: String,
    pub prices: Vec<PriceData>,
}
