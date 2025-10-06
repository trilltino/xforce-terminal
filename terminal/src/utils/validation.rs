/// Validation utilities for user input

pub struct ValidationResult {
    pub is_valid: bool,
    pub error: Option<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            is_valid: true,
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            error: Some(message.into()),
        }
    }
}

/// Validate email format
pub fn validate_email(email: &str) -> ValidationResult {
    if email.is_empty() {
        return ValidationResult::err("Email is required");
    }

    if !email.contains('@') {
        return ValidationResult::err("Invalid email format");
    }

    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return ValidationResult::err("Invalid email format");
    }

    if parts[0].is_empty() {
        return ValidationResult::err("Email username cannot be empty");
    }

    if parts[1].is_empty() || !parts[1].contains('.') {
        return ValidationResult::err("Invalid email domain");
    }

    ValidationResult::ok()
}

/// Validate username
pub fn validate_username(username: &str) -> ValidationResult {
    if username.is_empty() {
        return ValidationResult::err("Username is required");
    }

    if username.len() < 3 {
        return ValidationResult::err("Username must be at least 3 characters");
    }

    if username.len() > 20 {
        return ValidationResult::err("Username must be less than 20 characters");
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return ValidationResult::err("Username can only contain letters, numbers, _ and -");
    }

    ValidationResult::ok()
}

/// Validate password strength
pub fn validate_password(password: &str) -> ValidationResult {
    if password.is_empty() {
        return ValidationResult::err("Password is required");
    }

    if password.len() < 8 {
        return ValidationResult::err("Password must be at least 8 characters");
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());

    if !has_uppercase {
        return ValidationResult::err("Password must contain at least one uppercase letter");
    }

    if !has_lowercase {
        return ValidationResult::err("Password must contain at least one lowercase letter");
    }

    if !has_digit {
        return ValidationResult::err("Password must contain at least one number");
    }

    ValidationResult::ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_valid);
        assert!(validate_email("user@domain.co.uk").is_valid);
        assert!(!validate_email("").is_valid);
        assert!(!validate_email("invalid").is_valid);
        assert!(!validate_email("@example.com").is_valid);
        assert!(!validate_email("test@").is_valid);
    }

    #[test]
    fn test_username_validation() {
        assert!(validate_username("alice").is_valid);
        assert!(validate_username("user_123").is_valid);
        assert!(!validate_username("ab").is_valid); // too short
        assert!(!validate_username("").is_valid);
        assert!(!validate_username("user@invalid").is_valid);
    }

    #[test]
    fn test_password_validation() {
        assert!(validate_password("SecurePass123").is_valid);
        assert!(!validate_password("short").is_valid);
        assert!(!validate_password("nouppercase123").is_valid);
        assert!(!validate_password("NOLOWERCASE123").is_valid);
        assert!(!validate_password("NoDigits").is_valid);
    }
}
