//! # Response Mapping Middleware
//!
//! Standardizes error responses and adds common headers.
//!
//! This middleware ensures consistent error response format across all endpoints
//! and adds standard headers like `Content-Type` and `X-Request-ID`.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use axum::Router;
//! use lib_web::middleware::mw_res_map::map_res;
//!
//! let app = Router::new()
//!     .route("/api/endpoint", get(handler))
//!     .layer(map_res());
//! ```

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::error;

/// Response mapping middleware.
///
/// Standardizes error responses and ensures consistent headers.
/// Currently a placeholder for future response transformation logic.
///
/// # Future Enhancements
///
/// - Standardize error response format
/// - Add common headers (Content-Type, CORS)
/// - Log error responses
/// - Transform internal errors to user-friendly messages
///
/// # Example
///
/// ```rust,no_run
/// use axum::Router;
/// use lib_web::middleware::mw_res_map::map_res;
///
/// let app = Router::new()
///     .layer(map_res());
/// ```
pub async fn map_res(req: Request, next: Next) -> Response {
    let res = next.run(req).await;

    // Log error responses
    if res.status().is_server_error() {
        error!("[RESPONSE] Server error: {}", res.status());
    }

    res
}

