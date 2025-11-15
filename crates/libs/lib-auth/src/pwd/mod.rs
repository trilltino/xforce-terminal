//! # Password Hashing
//!
//! Password hashing and verification using Argon2.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hash a password using the Argon2 algorithm.
pub fn hash_password(password: &str) -> Result<String, String> {
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

/// Verify a plaintext password against an Argon2 hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| format!("Failed to parse hash: {}", e))?;

    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "TestPassword123!";
        let hash = hash_password(password)
            .expect("Password hashing should succeed for valid password");

        assert!(verify_password(password, &hash)
            .expect("Password verification should succeed for correct password"));
        assert!(!verify_password("WrongPassword", &hash)
            .expect("Password verification should fail for incorrect password"));
    }

    #[test]
    fn test_password_too_short() {
        let short_password = "short";
        let result = hash_password(short_password);

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Hash should fail for short password"),
            "Password must be at least 8 characters long"
        );
    }
}
