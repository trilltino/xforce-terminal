//! # Core Library
//!
//! Core models, database, configuration, and context for the application.

pub mod config;
pub mod error;
pub mod model;
pub mod dto;

// Re-export commonly used types
pub use config::Config;
pub use error::{AppError, Result};
pub use model::store::{DbPool, create_pool};

