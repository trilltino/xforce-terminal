//! # Debugging and Tracing Infrastructure
//!
//! Comprehensive debugging system for the Solana DeFi Trading Terminal GUI.
//! Provides file-based logging and in-UI diagnostics for performance monitoring.
//!
//! ## Features
//!
//! - **File-based logging**: Structured logs to `logs/terminal-debug.log` (daily rotation)
//! - **Lock instrumentation**: Track RwLock acquisition times and contention
//! - **Async task tracking**: Monitor task lifecycle and detect hung tasks
//! - **Frame metrics**: Measure render performance and detect slow frames
//! - **Event monitoring**: Track async event queue depth and processing time
//! - **In-UI debug overlay**: Real-time diagnostics (toggle with Ctrl+D)
//!
//! ## Usage
//!
//! ```rust
//! // Initialize at app startup
//! debug::init();
//!
//! // Instrumented lock access
//! let state = app.state.read_traced("ui::draw").await;
//!
//! // Track async tasks
//! spawn_tracked("price_fetch", async move {
//!     api_client.get_prices(&symbols).await
//! });
//!
//! // Log with structured fields
//! info!(
//!     endpoint = "/api/prices",
//!     duration_ms = 234,
//!     "API call completed"
//! );
//! ```
//!
//! ## Configuration
//!
//! Environment variables:
//! - `RUST_LOG`: Log level filter (e.g., `terminal=debug,info`)
//! - `TERMINAL_LOG_FILE`: Custom log file path (default: `logs/terminal-debug.log`)
//! - `TERMINAL_DEBUG_UI`: Enable in-UI debug overlay (1=on, 0=off)

pub mod config;
pub mod lock_tracer;
pub mod logger;
pub mod metrics;
pub mod task_tracker;
pub mod trace_context;
pub mod watchdog;
pub mod event_tracker;
pub mod error_aggregator;

pub use config::DebugConfig;
pub use lock_tracer::{TracedRwLock, block_on_read, block_on_write};
pub use logger::init as init_logger;
pub use metrics::{FrameMetrics, record_frame_time, init_metrics, update_memory_metrics};
pub use task_tracker::{spawn_tracked, active_task_count, track_blocking};
pub use trace_context::{TraceGuard, new_trace_id, set_trace_id, get_trace_id, clear_trace_id, with_trace_id, with_trace_id_async};
pub use watchdog::{update_heartbeat, init_from_config as init_watchdog};
pub use event_tracker::{track_event_send, track_event_receive, get_recent_events, pending_event_count, log_event_stats};
pub use error_aggregator::{record_error, record_warning, record_panic, get_recent_errors, get_error_stats, total_error_count, log_error_stats, ErrorEntry, ErrorLevel};

/// Initialize the debugging system
///
/// Sets up file-based logging with daily rotation and structured output.
/// Call this at application startup, before any other operations.
///
/// # Example
///
/// ```rust
/// fn main() -> Result<(), Box<dyn Error>> {
///     debug::init();
///
///     // ... rest of application
/// }
/// ```
pub fn init() {
    init_logger();
    init_watchdog();
}

/// Check if debug mode is enabled via feature flag
pub fn is_debug_mode() -> bool {
    cfg!(feature = "debug-mode")
}

/// Check if profiling is enabled via feature flag
pub fn is_profile_mode() -> bool {
    cfg!(feature = "profile")
}
