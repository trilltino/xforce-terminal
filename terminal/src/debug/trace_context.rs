//! Trace ID system for correlating operations across async boundaries
//!
//! Provides a trace context that propagates through the application,
//! allowing correlation of logs, events, and API calls that are part
//! of the same user action or operation.

use std::cell::RefCell;
use uuid::Uuid;

thread_local! {
    static TRACE_ID: RefCell<Option<String>> = RefCell::new(None);
}

/// Generate a new trace ID and set it for the current thread
pub fn new_trace_id() -> String {
    let trace_id = Uuid::new_v4().to_string();
    set_trace_id(Some(trace_id.clone()));
    trace_id
}

/// Set the trace ID for the current thread
pub fn set_trace_id(id: Option<String>) {
    TRACE_ID.with(|cell| {
        *cell.borrow_mut() = id;
    });
}

/// Get the current trace ID, if one is set
pub fn get_trace_id() -> Option<String> {
    TRACE_ID.with(|cell| cell.borrow().clone())
}

/// Clear the trace ID for the current thread
pub fn clear_trace_id() {
    set_trace_id(None);
}

/// Execute a closure with a new trace ID
pub fn with_trace_id<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let trace_id = new_trace_id();
    let result = f();
    clear_trace_id();
    tracing::debug!(trace_id = %trace_id, "Trace context completed");
    result
}

/// Execute an async closure with a new trace ID
pub async fn with_trace_id_async<F, Fut, R>(f: F) -> R
where
    F: FnOnce(String) -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let trace_id = new_trace_id();
    let result = f(trace_id.clone()).await;
    clear_trace_id();
    tracing::debug!(trace_id = %trace_id, "Trace context completed");
    result
}

/// Macro to include trace ID in log messages
#[macro_export]
macro_rules! trace_info {
    ($($arg:tt)*) => {
        if let Some(trace_id) = $crate::debug::trace_context::get_trace_id() {
            tracing::info!(trace_id = %trace_id, $($arg)*);
        } else {
            tracing::info!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! trace_warn {
    ($($arg:tt)*) => {
        if let Some(trace_id) = $crate::debug::trace_context::get_trace_id() {
            tracing::warn!(trace_id = %trace_id, $($arg)*);
        } else {
            tracing::warn!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! trace_error {
    ($($arg:tt)*) => {
        if let Some(trace_id) = $crate::debug::trace_context::get_trace_id() {
            tracing::error!(trace_id = %trace_id, $($arg)*);
        } else {
            tracing::error!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! trace_debug {
    ($($arg:tt)*) => {
        if let Some(trace_id) = $crate::debug::trace_context::get_trace_id() {
            tracing::debug!(trace_id = %trace_id, $($arg)*);
        } else {
            tracing::debug!($($arg)*);
        }
    };
}

/// Struct to automatically manage trace context scope
pub struct TraceGuard {
    trace_id: String,
}

impl TraceGuard {
    /// Create a new trace guard with a fresh trace ID
    pub fn new() -> Self {
        let trace_id = new_trace_id();
        Self { trace_id }
    }

    /// Create a trace guard with a specific trace ID
    pub fn with_id(trace_id: String) -> Self {
        set_trace_id(Some(trace_id.clone()));
        Self { trace_id }
    }

    /// Get the trace ID
    pub fn id(&self) -> &str {
        &self.trace_id
    }
}

impl Drop for TraceGuard {
    fn drop(&mut self) {
        clear_trace_id();
    }
}

impl Default for TraceGuard {
    fn default() -> Self {
        Self::new()
    }
}
