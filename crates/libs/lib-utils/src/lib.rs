//! # Utilities Library
//!
//! Shared utility functions for base64 encoding, environment variables, time, and validation.

pub mod b64;
pub mod envs;
pub mod time;
pub mod validation;

// Re-export commonly used functions
pub use b64::{b64u_encode, b64u_decode, b64u_decode_to_string};
pub use envs::{get_env, get_env_parse};
pub use time::{now_utc, format_time, parse_utc};
pub use validation::{validate_not_empty, validate_email, validate_min_length};

