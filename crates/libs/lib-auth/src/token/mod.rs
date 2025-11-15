//! # JWT Token Management
//!
//! JWT token generation, validation, and management.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT Claims structure containing user authentication information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at time (Unix timestamp)
    pub iat: i64,
}

/// Encode a JWT token with user claims.
pub fn encode_jwt(
    user_id: i64,
    username: String,
    secret: &str,
    expiration_hours: i64,
) -> Result<String, String> {
    let now = Utc::now();
    let exp = now + Duration::hours(expiration_hours);

    let claims = Claims {
        sub: user_id.to_string(),
        username,
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("Failed to encode JWT: {}", e))
}

/// Decode and validate a JWT token.
pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, String> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| format!("Failed to decode JWT: {}", e))?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_encoding_decoding() {
        let secret = "test-secret-key-must-be-at-least-32-chars-long!";
        let user_id = 1;
        let username = "testuser".to_string();

        let token = encode_jwt(user_id, username.clone(), secret, 24)
            .expect("JWT encoding should succeed");
        let claims = decode_jwt(&token, secret)
            .expect("JWT decoding should succeed");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, username);
    }
}
