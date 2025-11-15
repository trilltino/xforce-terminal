//! Debug configuration from environment variables

use std::path::PathBuf;

/// Debug system configuration
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Log file path
    pub log_file: PathBuf,
    /// Log level filter (e.g., "terminal=debug,info")
    pub log_level: String,
    /// Enable in-UI debug overlay
    pub show_debug_ui: bool,
    /// Log directory (for rotation)
    pub log_dir: PathBuf,
    /// Enable realtime debug log (separate from main log)
    pub enable_realtime_log: bool,
    /// Enable trace ID system
    pub enable_trace_ids: bool,
    /// Freeze detection threshold in milliseconds
    pub freeze_threshold_ms: u64,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            log_file: PathBuf::from("logs/terminal-debug.log"),
            log_level: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "terminal=info,warn".to_string()),
            show_debug_ui: std::env::var("TERMINAL_DEBUG_UI")
                .map(|v| v == "1")
                .unwrap_or(false),
            log_dir: PathBuf::from("logs"),
            enable_realtime_log: std::env::var("TERMINAL_DEBUG_REALTIME")
                .map(|v| v == "1")
                .unwrap_or(true), // Default ON for comprehensive debugging
            enable_trace_ids: std::env::var("TERMINAL_TRACE_ENABLED")
                .map(|v| v == "1")
                .unwrap_or(true), // Default ON
            freeze_threshold_ms: std::env::var("TERMINAL_FREEZE_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000), // Default 1 second
        }
    }
}

impl DebugConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let log_dir = std::env::var("TERMINAL_LOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("logs"));

        Self {
            log_file: log_dir.join("terminal-debug.log"),
            log_level: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "terminal=info,warn".to_string()),
            show_debug_ui: std::env::var("TERMINAL_DEBUG_UI")
                .map(|v| v == "1")
                .unwrap_or(cfg!(feature = "debug-mode")),
            log_dir,
            enable_realtime_log: std::env::var("TERMINAL_DEBUG_REALTIME")
                .map(|v| v == "1")
                .unwrap_or(true),
            enable_trace_ids: std::env::var("TERMINAL_TRACE_ENABLED")
                .map(|v| v == "1")
                .unwrap_or(true),
            freeze_threshold_ms: std::env::var("TERMINAL_FREEZE_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
        }
    }

    /// Check if debug logging is enabled
    pub fn is_debug_enabled(&self) -> bool {
        self.log_level.contains("debug")
    }

    /// Check if trace logging is enabled
    pub fn is_trace_enabled(&self) -> bool {
        self.log_level.contains("trace")
    }
}
