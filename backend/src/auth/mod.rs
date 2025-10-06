use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,       // subject (user id)
    pub username: String,  // username
    pub exp: i64,          // expiration time
    pub iat: i64,          // issued at
}

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String, String> {
    // Validate password strength
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash password: {}", e))?
        .to_string();

    Ok(password_hash)
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| format!("Failed to parse hash: {}", e))?;

    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Encode a JWT token
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

/// Decode and validate a JWT token
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
    fn test_password_hashing() {
        let password = "TestPassword123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_jwt_encoding_decoding() {
        let secret = "test-secret-key-must-be-at-least-32-chars-long!";
        let user_id = 1;
        let username = "testuser".to_string();

        let token = encode_jwt(user_id, username.clone(), secret, 24).unwrap();
        let claims = decode_jwt(&token, secret).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, username);
    }
}
