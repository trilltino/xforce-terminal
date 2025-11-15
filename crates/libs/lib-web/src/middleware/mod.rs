//! # Middleware
//!
//! Axum middleware for authentication, request stamping, and response mapping.
//!
//! ## Modules
//!
//! - **[`mw_auth`]**: JWT authentication middleware
//! - **[`mw_req_stamp`]**: Request ID and timestamp stamping
//! - **[`mw_res_map`]**: Response mapping and standardization

// region: --- Modules
pub mod mw_auth;
pub mod mw_req_stamp;
pub mod mw_res_map;
pub mod mw_logging;
// endregion: --- Modules

// region: --- Re-exports
pub use mw_auth::require_auth;
pub use mw_req_stamp::{stamp_req, RequestStamp};
pub use mw_res_map::map_res;
pub use mw_logging::log_requests;
// endregion: --- Re-exports

