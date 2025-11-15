//! # Data Transfer Objects (DTOs)
//!
//! This module contains all data structures used for communication between
//! the frontend and backend via the REST API.

pub mod auth;
pub mod market;
pub mod messaging;

pub use auth::*;
pub use market::*;
pub use messaging::*;
