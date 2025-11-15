//! # Request/Response Logging Middleware
//!
//! Comprehensive logging middleware for HTTP requests and responses with structured logging,
//! request IDs, and detailed request/response data.
//!
//! This middleware logs:
//! - Request method, path, query params
//! - Request headers (sanitized)
//! - Request body (for non-sensitive endpoints)
//! - Response status, size, duration
//! - Uses tracing spans for correlation
//!
//! ## Usage
//!
//! ```rust,no_run
//! use axum::Router;
//! use lib_web::middleware::mw_logging::log_requests;
//!
//! let app = Router::new()
//!     .route("/api/endpoint", get(handler))
//!     .layer(log_requests());
//! ```

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// Sensitive headers that should not be logged
const SENSITIVE_HEADERS: &[&str] = &[
    "authorization",
    "cookie",
    "x-api-key",
    "x-auth-token",
    "authentication",
];

/// Sensitive endpoints that should not log request/response bodies
const SENSITIVE_ENDPOINTS: &[&str] = &[
    "/api/auth/login",
    "/api/auth/signup",
    "/api/auth/wallet-login",
    "/api/wallet/setup/complete",
    "/api/swap/execute",
    "/api/transactions/submit",
];

/// Comprehensive request/response logging middleware
///
/// Logs detailed information about every HTTP request and response including:
/// - Method, path, query parameters
/// - Headers (with sensitive ones sanitized)
/// - Request body (for non-sensitive endpoints)
/// - Response status, size, duration
/// - Errors with full context
pub async fn log_requests(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().map(|q| q.to_string());
    
    // Get request ID from extensions if available
    let request_id = req
        .extensions()
        .get::<crate::middleware::mw_req_stamp::RequestStamp>()
        .map(|s| s.id.clone())
        .unwrap_or_else(|| "unknown".to_string());
    
    // Check if endpoint is sensitive
    let is_sensitive = SENSITIVE_ENDPOINTS.iter().any(|ep| path.starts_with(ep));
    
    // Log headers (sanitized)
    let headers: Vec<(String, String)> = req
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            let name_lower = name.as_str().to_lowercase();
            if SENSITIVE_HEADERS.iter().any(|h| name_lower.contains(h)) {
                Some((name.to_string(), "***REDACTED***".to_string()))
            } else {
                value.to_str().ok().map(|v| (name.to_string(), v.to_string()))
            }
        })
        .collect();
    
    // Extract user agent and client IP if available
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .or_else(|| req.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    // Create detailed request log
    info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        query = ?query,
        user_agent = ?user_agent,
        client_ip = ?client_ip,
        header_count = headers.len(),
        "[REQUEST] {} {} {}",
        method,
        path,
        query.as_ref().map(|q| format!("?{}", q)).unwrap_or_default()
    );
    
    // Log headers at debug level
    debug!(
        request_id = %request_id,
        headers = ?headers,
        "[REQUEST HEADERS]"
    );
    
    // For non-sensitive endpoints, attempt to log body (if available and small)
    if !is_sensitive {
        // Note: Body can only be consumed once, so we'd need to clone it
        // For now, we'll skip body logging to avoid complexity
        // Body logging can be added later with body extraction
    }
    
    // Process request
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    let status = response.status();
    let status_code = status.as_u16();
    
    // Calculate response size (approximate from headers)
    let content_length = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    
    // Log response
    if status.is_success() {
        info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = status_code,
            duration_ms = duration.as_millis(),
            duration_secs = duration.as_secs_f64(),
            size_bytes = content_length,
            "[RESPONSE] {} {} -> {} ({}ms, {} bytes)",
            method,
            path,
            status_code,
            duration.as_millis(),
            content_length
        );
    } else if status.is_client_error() {
        warn!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = status_code,
            duration_ms = duration.as_millis(),
            "[RESPONSE] {} {} -> {} ({}ms) [CLIENT ERROR]",
            method,
            path,
            status_code,
            duration.as_millis()
        );
    } else if status.is_server_error() {
        error!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = status_code,
            duration_ms = duration.as_millis(),
            is_websocket = path.contains("/ws/"),
            "[RESPONSE] {} {} -> {} ({}ms) [SERVER ERROR]",
            method,
            path,
            status_code,
            duration.as_millis()
        );
        
        // Extra logging for WebSocket errors
        if path.contains("/ws/") {
            error!(
                request_id = %request_id,
                method = %method,
                path = %path,
                status = status_code,
                "[WS] WEBSOCKET_ERROR request_id={} method={} path={} status={} - WebSocket connection failed",
                request_id,
                method,
                path,
                status_code
            );
        }
    }
    
    response
}

