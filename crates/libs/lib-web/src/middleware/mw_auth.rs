//! # Authentication Middleware
//!
//! Axum middleware for JWT token validation and user authentication.
//!
//! This middleware extracts and validates JWT tokens from the `Authorization` header,
//! then injects the authenticated user's claims into the request extensions.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use axum::{Router, routing::get};
//! use lib_web::middleware::mw_auth::require_auth;
//!
//! let app = Router::new()
//!     .route("/protected", get(protected_handler))
//!     .layer(require_auth());
//! ```
//!
//! Handlers can then extract claims using `Extension<Claims>`:
//!
//! ```rust,no_run
//! use axum::extract::Extension;
//! use lib_auth::Claims;
//!
//! async fn protected_handler(Extension(claims): Extension<Claims>) -> String {
//!     format!("Hello, user {}!", claims.username)
//! }
//! ```

use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use lib_auth::decode_jwt;
use lib_core::config::core_config;
use tracing::{debug, warn};

/// Authentication middleware that validates JWT tokens.
///
/// Extracts the `Authorization: Bearer <token>` header, validates the JWT token,
/// and injects the `Claims` into request extensions for use by handlers.
///
/// # Behavior
///
/// - **Valid token**: Continues to next middleware/handler with `Claims` in extensions
/// - **Missing/invalid token**: Returns `401 Unauthorized` with error message
///
/// # Example
///
/// ```rust,no_run
/// use axum::Router;
/// use lib_web::middleware::mw_auth::require_auth;
///
/// let app = Router::new()
///     .route("/api/protected", get(handler))
///     .layer(require_auth());
/// ```
pub async fn require_auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            warn!("[AUTH] Missing Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    // Extract Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            warn!("[AUTH] Invalid Authorization header format");
            StatusCode::UNAUTHORIZED
        })?;

    // Decode and validate JWT
    let config = core_config();
    let claims = decode_jwt(token, &config.jwt_secret)
        .map_err(|e| {
            warn!("[AUTH] JWT validation failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    debug!("[AUTH] Authenticated user: {} (id: {})", claims.username, claims.sub);

    // Inject claims into request extensions
    req.extensions_mut().insert(claims);

    // Continue to next middleware/handler
    Ok(next.run(req).await)
}

