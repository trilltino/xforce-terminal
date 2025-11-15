//! # Request Stamping Middleware
//!
//! Adds request metadata (ID, timestamp) to requests for tracing and debugging.
//!
//! This middleware generates a unique request ID and adds it to request extensions
//! and response headers, enabling request tracing across the application.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use axum::Router;
//! use lib_web::middleware::mw_req_stamp::stamp_req;
//!
//! let app = Router::new()
//!     .route("/api/endpoint", get(handler))
//!     .layer(stamp_req());
//! ```
//!
//! Request ID is available in handlers via `Extension<RequestStamp>`:
//!
//! ```rust,no_run
//! use axum::extract::Extension;
//! use lib_web::middleware::mw_req_stamp::RequestStamp;
//!
//! async fn handler(Extension(stamp): Extension<RequestStamp>) -> String {
//!     format!("Request ID: {}", stamp.id)
//! }
//! ```

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use std::time::SystemTime;
use uuid::Uuid;

/// Request metadata for tracing and debugging.
#[derive(Clone, Debug)]
pub struct RequestStamp {
    /// Unique request identifier
    pub id: String,
    /// Request timestamp
    pub timestamp: SystemTime,
}

impl RequestStamp {
    /// Create a new request stamp with generated ID.
    fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
        }
    }
}

/// Request stamping middleware.
///
/// Generates a unique request ID and adds it to:
/// - Request extensions (for handler access)
/// - Response headers (`X-Request-ID`)
///
/// # Example
///
/// ```rust,no_run
/// use axum::Router;
/// use lib_web::middleware::mw_req_stamp::stamp_req;
///
/// let app = Router::new()
///     .layer(stamp_req());
/// ```
pub async fn stamp_req(mut req: Request, next: Next) -> Response {
    // Generate request stamp
    let stamp = RequestStamp::new();

    // Add to request extensions
    req.extensions_mut().insert(stamp.clone());

    // Process request
    let mut res = next.run(req).await;

    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&stamp.id) {
        res.headers_mut().insert("X-Request-ID", header_value);
    }

    res
}

