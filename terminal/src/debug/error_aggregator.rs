//! Error aggregation and pattern detection
//!
//! Tracks errors and warnings to detect error storms, patterns, and anomalies.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

/// Global error aggregator
static ERROR_AGGREGATOR: Lazy<Mutex<ErrorAggregator>> = Lazy::new(|| {
    Mutex::new(ErrorAggregator::new())
});

/// Error entry
#[derive(Debug, Clone)]
pub struct ErrorEntry {
    pub timestamp: Instant,
    pub level: ErrorLevel,
    pub message: String,
    pub location: Option<String>,
    pub trace_id: Option<String>,
}

/// Error level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorLevel {
    Error,
    Warning,
    Panic,
}

impl std::fmt::Display for ErrorLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorLevel::Error => write!(f, "ERROR"),
            ErrorLevel::Warning => write!(f, "WARN"),
            ErrorLevel::Panic => write!(f, "PANIC"),
        }
    }
}

/// Error aggregator
pub struct ErrorAggregator {
    /// Recent errors (ring buffer of last N errors)
    recent_errors: VecDeque<ErrorEntry>,
    /// Maximum errors to keep
    max_errors: usize,
    /// Error count by type (for statistics)
    error_counts: HashMap<ErrorLevel, u64>,
    /// Last error storm warning time
    last_storm_warning: Option<Instant>,
    /// Error storm threshold (errors per second)
    storm_threshold: usize,
}

impl ErrorAggregator {
    fn new() -> Self {
        Self {
            recent_errors: VecDeque::with_capacity(100),
            max_errors: 100,
            error_counts: HashMap::new(),
            last_storm_warning: None,
            storm_threshold: 10,
        }
    }

    /// Add an error entry
    fn add_error(&mut self, entry: ErrorEntry) {
        // Update counts
        *self.error_counts.entry(entry.level).or_insert(0) += 1;

        // Add to recent errors
        if self.recent_errors.len() >= self.max_errors {
            self.recent_errors.pop_front();
        }
        self.recent_errors.push_back(entry);

        // Check for error storm
        self.check_error_storm();
    }

    /// Check for error storm (many errors in short time)
    fn check_error_storm(&mut self) {
        let now = Instant::now();
        let one_second_ago = now - Duration::from_secs(1);

        // Count errors in last second
        let recent_count = self.recent_errors
            .iter()
            .rev()
            .take_while(|e| e.timestamp > one_second_ago)
            .count();

        if recent_count >= self.storm_threshold {
            // Rate limit warnings (only every 5 seconds)
            let should_warn = self.last_storm_warning
                .map(|t| now.duration_since(t) >= Duration::from_secs(5))
                .unwrap_or(true);

            if should_warn {
                tracing::error!(
                    errors_per_second = recent_count,
                    threshold = self.storm_threshold,
                    "!!!!! ERROR STORM DETECTED !!!!! - High error rate"
                );
                self.last_storm_warning = Some(now);
            }
        }
    }

    /// Get recent errors
    fn get_recent(&self, count: usize) -> Vec<ErrorEntry> {
        self.recent_errors
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }

    /// Get error statistics
    fn get_stats(&self) -> HashMap<ErrorLevel, u64> {
        self.error_counts.clone()
    }

    /// Get total error count
    fn total_errors(&self) -> u64 {
        self.error_counts.values().sum()
    }
}

/// Record an error
pub fn record_error(message: String, location: Option<String>) {
    let trace_id = super::trace_context::get_trace_id();

    let entry = ErrorEntry {
        timestamp: Instant::now(),
        level: ErrorLevel::Error,
        message: message.clone(),
        location: location.clone(),
        trace_id: trace_id.clone(),
    };

    if let Ok(mut aggregator) = ERROR_AGGREGATOR.lock() {
        aggregator.add_error(entry);
    }

    // Log the error
    if let Some(tid) = trace_id {
        if let Some(loc) = location {
            tracing::error!(
                trace_id = %tid,
                location = %loc,
                message = %message,
                "Error recorded"
            );
        } else {
            tracing::error!(
                trace_id = %tid,
                message = %message,
                "Error recorded"
            );
        }
    } else {
        if let Some(loc) = location {
            tracing::error!(
                location = %loc,
                message = %message,
                "Error recorded"
            );
        } else {
            tracing::error!(
                message = %message,
                "Error recorded"
            );
        }
    }
}

/// Record a warning
pub fn record_warning(message: String, location: Option<String>) {
    let trace_id = super::trace_context::get_trace_id();

    let entry = ErrorEntry {
        timestamp: Instant::now(),
        level: ErrorLevel::Warning,
        message: message.clone(),
        location: location.clone(),
        trace_id: trace_id.clone(),
    };

    if let Ok(mut aggregator) = ERROR_AGGREGATOR.lock() {
        aggregator.add_error(entry);
    }

    // Log the warning
    if let Some(tid) = trace_id {
        if let Some(loc) = location {
            tracing::warn!(
                trace_id = %tid,
                location = %loc,
                message = %message,
                "Warning recorded"
            );
        } else {
            tracing::warn!(
                trace_id = %tid,
                message = %message,
                "Warning recorded"
            );
        }
    } else {
        if let Some(loc) = location {
            tracing::warn!(
                location = %loc,
                message = %message,
                "Warning recorded"
            );
        } else {
            tracing::warn!(
                message = %message,
                "Warning recorded"
            );
        }
    }
}

/// Record a panic
pub fn record_panic(message: String, location: Option<String>) {
    let trace_id = super::trace_context::get_trace_id();

    let entry = ErrorEntry {
        timestamp: Instant::now(),
        level: ErrorLevel::Panic,
        message,
        location,
        trace_id,
    };

    if let Ok(mut aggregator) = ERROR_AGGREGATOR.lock() {
        aggregator.add_error(entry);
    }
}

/// Get recent errors
pub fn get_recent_errors(count: usize) -> Vec<ErrorEntry> {
    ERROR_AGGREGATOR
        .lock()
        .map(|a| a.get_recent(count))
        .unwrap_or_default()
}

/// Get error statistics
pub fn get_error_stats() -> HashMap<ErrorLevel, u64> {
    ERROR_AGGREGATOR
        .lock()
        .map(|a| a.get_stats())
        .unwrap_or_default()
}

/// Get total error count
pub fn total_error_count() -> u64 {
    ERROR_AGGREGATOR
        .lock()
        .map(|a| a.total_errors())
        .unwrap_or(0)
}

/// Log error statistics
pub fn log_error_stats() {
    let stats = get_error_stats();
    let total = total_error_count();

    tracing::info!(
        total_errors = total,
        errors = stats.get(&ErrorLevel::Error).unwrap_or(&0),
        warnings = stats.get(&ErrorLevel::Warning).unwrap_or(&0),
        panics = stats.get(&ErrorLevel::Panic).unwrap_or(&0),
        "Error aggregation statistics"
    );
}

/// Macro to wrap Result with automatic error recording
#[macro_export]
macro_rules! track_result {
    ($result:expr) => {{
        let result = $result;
        if let Err(ref e) = result {
            $crate::debug::error_aggregator::record_error(
                format!("{}", e),
                Some(format!("{}:{}", file!(), line!()))
            );
        }
        result
    }};
}
