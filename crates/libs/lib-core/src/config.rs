//! # Application Configuration
//!
//! This module manages application configuration loaded from environment variables.
//! All configuration is validated on startup to fail fast if misconfigured.
//!
//! ## Global Config Access
//!
//! Use [`core_config()`] to access the global configuration instance:
//!
//! ```rust,no_run
//! use lib_core::config::core_config;
//!
//! let config = core_config();
//! let db_url = &config.database_url;
//! ```
//!
//! The config must be initialized once at application startup using [`init_config()`].

use std::env;
use std::sync::OnceLock;

/// Application configuration loaded from environment variables.
#[derive(Clone, Debug)]
pub struct Config {
    /// SQLite database connection URL
    pub database_url: String,

    /// Secret key for JWT token signing and verification
    ///
    /// **Must be at least 32 characters long** for security.
    pub jwt_secret: String,

    /// JWT token validity period in hours
    ///
    /// After this period, users must re-authenticate.
    /// Valid range: 1-720 hours (1 hour to 30 days)
    pub jwt_expiration_hours: i64,
}

impl Config {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, String> {
        // Default to data/terminal.db for better organization, fallback to terminal.db
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:data/terminal.db".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET must be set in environment")?;

        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .map_err(|e| format!("JWT_EXPIRATION_HOURS must be a valid number: {}", e))?;

        Ok(Self {
            database_url,
            jwt_secret,
            jwt_expiration_hours,
        })
    }

    /// Validate configuration values against security and business rules.
    pub fn validate(&self) -> Result<(), String> {
        if self.jwt_secret.len() < 32 {
            return Err("JWT_SECRET must be at least 32 characters long".to_string());
        }

        if self.jwt_expiration_hours < 1 || self.jwt_expiration_hours > 720 {
            return Err("JWT_EXPIRATION_HOURS must be between 1 and 720 (30 days)".to_string());
        }

        Ok(())
    }
}

/// Global configuration instance (initialized once at startup).
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Initialize the global configuration.
///
/// This should be called once at application startup, before any handlers
/// or services that need configuration are used.
///
/// # Errors
///
/// Returns an error if:
/// - Environment variables are missing or invalid
/// - Configuration validation fails
/// - Config has already been initialized
///
/// # Example
///
/// ```rust,no_run
/// use lib_core::config::init_config;
///
/// fn main() -> Result<(), String> {
///     init_config()?;
///     // ... rest of application startup
///     Ok(())
/// }
/// ```
pub fn init_config() -> Result<(), String> {
    let config = Config::from_env()?;
    config.validate()?;
    
    CONFIG.set(config)
        .map_err(|_| "Config has already been initialized".to_string())
}

/// Get a reference to the global configuration.
///
/// # Panics
///
/// Panics if [`init_config()`] has not been called yet. This ensures
/// configuration is always available when accessed.
///
/// # Example
///
/// ```rust,no_run
/// use lib_core::config::core_config;
///
/// let config = core_config();
/// let jwt_secret = &config.jwt_secret;
/// ```
pub fn core_config() -> &'static Config {
    CONFIG.get().expect("Config must be initialized with init_config() before use")
}
