//! # Authentication Library
//!
//! Authentication, password hashing, and JWT token management.

pub mod pwd;
pub mod token;

// Re-export commonly used types
pub use pwd::{hash_password, verify_password};
pub use token::{Claims, encode_jwt, decode_jwt};

