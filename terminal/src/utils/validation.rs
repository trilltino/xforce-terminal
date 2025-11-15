//! # Input Validation Utilities
//!
//! User input validation functions for form fields and user-provided data.
//!
//! This module provides validation functions that check user input before it's sent
//! to the backend API. These are **client-side validations** - the backend should
//! also validate all input for security.
//!
//! ## Validation Functions
//!
//! - **[`validate_email`]**: Email address format validation
//! - **[`validate_username`]**: Username format and length validation
//! - **[`validate_password`]**: Password strength validation
//!
//! ## ValidationResult
//!
//! All validation functions return [`ValidationResult`], which indicates:
//! - `is_valid: true` - Input passes validation
//! - `is_valid: false` - Input fails validation (with error message)
//!
//! ## Usage Example
//!
//! ```rust
//! use terminal::utils::validation::{validate_email, validate_username, validate_password};
//!
//! // Validate email
//! let email_result = validate_email("user@example.com");
//! if !email_result.is_valid {
//!     println!("Email error: {}", email_result.error.unwrap());
//! }
//!
//! // Validate username
//! let username_result = validate_username("alice123");
//! if !username_result.is_valid {
//!     println!("Username error: {}", username_result.error.unwrap());
//! }
//!
//! // Validate password
//! let password_result = validate_password("SecurePass123!");
//! if !password_result.is_valid {
//!     println!("Password error: {}", password_result.error.unwrap());
//! }
//! ```
//!
//! ## Validation Rules
//!
//! ### Email Validation
//! - Must contain '@' symbol
//! - Must have non-empty username part (before @)
//! - Must have non-empty domain part (after @)
//! - Domain must contain at least one '.' (e.g., "example.com")
//!
//! ### Username Validation
//! - Length: 3-20 characters
//! - Characters: Alphanumeric, underscore (_), or dash (-)
//! - Must not be empty
//!
//! ### Password Validation
//! - Minimum length: 8 characters
//! - Must contain at least one uppercase letter
//! - Must contain at least one lowercase letter
//! - Must contain at least one digit
//!
//! ## Security Note
//!
//! These are **client-side validations only**. Always validate input on the backend
//! as well, as client-side validation can be bypassed.

/// Result of input validation.
///
/// Indicates whether input passed validation and includes an error message if not.
///
/// # Fields
///
/// * `is_valid` - `true` if input is valid, `false` otherwise
/// * `error` - Error message describing why validation failed (only set when `is_valid = false`)
///
/// # Example
///
/// ```rust
/// use terminal::utils::validation::{ValidationResult, validate_email};
///
/// let result = validate_email("invalid-email");
/// if !result.is_valid {
///     if let Some(error) = result.error {
///         println!("Validation failed: {}", error);
///     }
/// }
/// ```
pub struct ValidationResult {
    /// Whether the input passed validation.
    pub is_valid: bool,
    
    /// Error message describing why validation failed (only present when `is_valid = false`).
    pub error: Option<String>,
}

impl ValidationResult {
    /// Create a successful validation result.
    ///
    /// # Returns
    ///
    /// A `ValidationResult` with `is_valid = true` and no error message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use terminal::utils::validation::ValidationResult;
    ///
    /// let result = ValidationResult::ok();
    /// assert!(result.is_valid);
    /// assert!(result.error.is_none());
    /// ```
    pub fn ok() -> Self {
        Self {
            is_valid: true,
            error: None,
        }
    }

    /// Create a failed validation result with an error message.
    ///
    /// # Arguments
    ///
    /// * `message` - Error message describing why validation failed
    ///
    /// # Returns
    ///
    /// A `ValidationResult` with `is_valid = false` and the provided error message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use terminal::utils::validation::ValidationResult;
    ///
    /// let result = ValidationResult::err("Input cannot be empty");
    /// assert!(!result.is_valid);
    /// assert_eq!(result.error, Some("Input cannot be empty".to_string()));
    /// ```
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            error: Some(message.into()),
        }
    }
}

/// Validate email address format.
///
/// Checks if the email address matches the expected format:
/// - Contains '@' symbol
/// - Non-empty username part (before @)
/// - Non-empty domain part (after @) with at least one '.'
///
/// # Arguments
///
/// * `email` - The email address to validate
///
/// # Returns
///
/// * `ValidationResult` with `is_valid = true` if email format is valid
/// * `ValidationResult` with `is_valid = false` and error message if invalid
///
/// # Validation Rules
///
/// - Email must contain exactly one '@' symbol
/// - Username part (before @) must not be empty
/// - Domain part (after @) must not be empty
/// - Domain must contain at least one '.' character (e.g., "example.com")
///
/// # Examples
///
/// ```rust
/// use terminal::utils::validation::validate_email;
///
/// // Valid emails
/// assert!(validate_email("user@example.com").is_valid);
/// assert!(validate_email("test.user@domain.co.uk").is_valid);
///
/// // Invalid emails
/// assert!(!validate_email("").is_valid);
/// assert!(!validate_email("invalid").is_valid);
/// assert!(!validate_email("@example.com").is_valid);
/// assert!(!validate_email("user@").is_valid);
/// assert!(!validate_email("user@nodot").is_valid);
/// ```
///
/// # Note
///
/// This is a basic format check, not RFC-compliant email validation.
/// For production use, consider using a dedicated email validation library.
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

/// Validate username format and length.
///
/// Checks if the username meets the requirements:
/// - Length: 3-20 characters
/// - Characters: Alphanumeric, underscore (_), or dash (-)
/// - Not empty
///
/// # Arguments
///
/// * `username` - The username to validate
///
/// # Returns
///
/// * `ValidationResult` with `is_valid = true` if username is valid
/// * `ValidationResult` with `is_valid = false` and error message if invalid
///
/// # Validation Rules
///
/// - Must be between 3 and 20 characters (inclusive)
/// - Can only contain: letters (a-z, A-Z), numbers (0-9), underscore (_), dash (-)
/// - Must not be empty
///
/// # Examples
///
/// ```rust
/// use terminal::utils::validation::validate_username;
///
/// // Valid usernames
/// assert!(validate_username("alice").is_valid);
/// assert!(validate_username("user_123").is_valid);
/// assert!(validate_username("user-name").is_valid);
///
/// // Invalid usernames
/// assert!(!validate_username("ab").is_valid); // too short
/// assert!(!validate_username("").is_valid); // empty
/// assert!(!validate_username("user@invalid").is_valid); // contains @
/// assert!(!validate_username("user with spaces").is_valid); // contains spaces
/// ```
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

/// Validate password strength requirements.
///
/// Checks if the password meets minimum security requirements:
/// - Minimum length: 8 characters
/// - At least one uppercase letter (A-Z)
/// - At least one lowercase letter (a-z)
/// - At least one digit (0-9)
///
/// # Arguments
///
/// * `password` - The password to validate
///
/// # Returns
///
/// * `ValidationResult` with `is_valid = true` if password meets requirements
/// * `ValidationResult` with `is_valid = false` and error message if invalid
///
/// # Validation Rules
///
/// - **Minimum length**: 8 characters
/// - **Uppercase letter**: At least one (A-Z)
/// - **Lowercase letter**: At least one (a-z)
/// - **Digit**: At least one (0-9)
/// - Special characters are allowed but not required
///
/// # Examples
///
/// ```rust
/// use terminal::utils::validation::validate_password;
///
/// // Valid passwords
/// assert!(validate_password("SecurePass123").is_valid);
/// assert!(validate_password("P@ssw0rd!").is_valid);
///
/// // Invalid passwords
/// assert!(!validate_password("short").is_valid); // too short
/// assert!(!validate_password("nouppercase123").is_valid); // no uppercase
/// assert!(!validate_password("NOLOWERCASE123").is_valid); // no lowercase
/// assert!(!validate_password("NoDigits").is_valid); // no digits
/// ```
///
/// # Security Note
///
/// This is a basic strength check. For production, consider:
/// - Checking against common password lists
/// - Requiring special characters
/// - Enforcing longer minimum length for sensitive accounts
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
