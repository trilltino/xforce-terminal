//! # Validation Utilities
//!
//! Input validation helpers.

/// Validate that a string is not empty.
pub fn validate_not_empty(value: &str, field_name: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{} cannot be empty", field_name))
    } else {
        Ok(())
    }
}

/// Validate email format (basic check).
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.contains('@') && email.contains('.') {
        Ok(())
    } else {
        Err("Invalid email format".to_string())
    }
}

/// Validate minimum length.
pub fn validate_min_length(value: &str, min: usize, field_name: &str) -> Result<(), String> {
    if value.len() < min {
        Err(format!("{} must be at least {} characters", field_name, min))
    } else {
        Ok(())
    }
}

