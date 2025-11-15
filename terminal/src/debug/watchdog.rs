//! Watchdog system for detecting UI freezes and hung operations
//!
//! Monitors the main thread's heartbeat and detects when the UI becomes
//! unresponsive for longer than the configured threshold.

use super::config::DebugConfig;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Global heartbeat timestamp (milliseconds since epoch)
static LAST_HEARTBEAT: AtomicU64 = AtomicU64::new(0);

/// Global watchdog enabled flag
static WATCHDOG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Watchdog state
pub struct Watchdog {
    threshold_ms: u64,
    check_interval_ms: u64,
    enabled: Arc<AtomicBool>,
}

impl Watchdog {
    /// Create a new watchdog with the given freeze threshold
    pub fn new(threshold_ms: u64) -> Self {
        let check_interval_ms = threshold_ms / 2; // Check twice per threshold
        Self {
            threshold_ms,
            check_interval_ms,
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Start the watchdog monitoring in a background task
    pub fn start(self) {
        WATCHDOG_ENABLED.store(true, Ordering::Relaxed);

        // Initialize heartbeat
        update_heartbeat();

        tracing::info!(
            threshold_ms = self.threshold_ms,
            check_interval_ms = self.check_interval_ms,
            "Watchdog started - monitoring for UI freezes"
        );

        // Spawn monitoring task
        tokio::spawn(async move {
            self.monitor_loop().await;
        });
    }

    /// Main monitoring loop
    async fn monitor_loop(&self) {
        let mut last_warning_time = Instant::now();
        let mut consecutive_warnings = 0;

        loop {
            sleep(Duration::from_millis(self.check_interval_ms)).await;

            if !self.enabled.load(Ordering::Relaxed) {
                break;
            }

            let last_heartbeat = LAST_HEARTBEAT.load(Ordering::Relaxed);
            if last_heartbeat == 0 {
                continue; // Not initialized yet
            }

            let now = current_time_ms();
            let elapsed = now.saturating_sub(last_heartbeat);

            if elapsed > self.threshold_ms {
                consecutive_warnings += 1;

                // Rate limit warnings (only every 5 seconds after first warning)
                let since_last_warning = last_warning_time.elapsed();
                if consecutive_warnings == 1 || since_last_warning >= Duration::from_secs(5) {
                    let elapsed_secs = elapsed as f64 / 1000.0;
                    let threshold_secs = self.threshold_ms as f64 / 1000.0;
                    
                    tracing::error!(
                        elapsed_ms = elapsed,
                        threshold_ms = self.threshold_ms,
                        consecutive_warnings = consecutive_warnings,
                        "UI FREEZE DETECTED - Main thread unresponsive for {:.1}s (threshold: {:.1}s), consecutive_warnings: {}",
                        elapsed_secs,
                        threshold_secs,
                        consecutive_warnings
                    );
                    last_warning_time = Instant::now();

                    // Log stack trace attempt (may not work on all platforms)
                    if consecutive_warnings == 1 {
                        tracing::warn!(
                            "Application freeze detected. The UI thread has not responded in {:.1} seconds. Consider enabling RUST_BACKTRACE=1 for detailed stack traces.",
                            elapsed_secs
                        );
                    }
                }
            } else {
                // Reset warning counter if heartbeat is healthy
                if consecutive_warnings > 0 {
                    tracing::info!(
                        after_warnings = consecutive_warnings,
                        "UI freeze resolved - Application is responding normally again after {} warning(s)",
                        consecutive_warnings
                    );
                    consecutive_warnings = 0;
                }
            }
        }

        tracing::info!("Watchdog stopped");
    }

    /// Stop the watchdog
    pub fn stop(&self) {
        self.enabled.store(false, Ordering::Relaxed);
        WATCHDOG_ENABLED.store(false, Ordering::Relaxed);
    }
}

/// Update the heartbeat timestamp (call this from the main UI thread)
pub fn update_heartbeat() {
    if !WATCHDOG_ENABLED.load(Ordering::Relaxed) {
        return;
    }

    let now = current_time_ms();
    LAST_HEARTBEAT.store(now, Ordering::Relaxed);
}

/// Check if watchdog is enabled
pub fn is_enabled() -> bool {
    WATCHDOG_ENABLED.load(Ordering::Relaxed)
}

/// Initialize and start the watchdog from config
pub fn init_from_config() {
    let config = DebugConfig::from_env();

    if config.freeze_threshold_ms > 0 {
        let watchdog = Watchdog::new(config.freeze_threshold_ms);
        watchdog.start();
    } else {
        tracing::info!("Watchdog disabled (freeze_threshold_ms = 0)");
    }
}

/// Get current time in milliseconds
fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_update() {
        update_heartbeat();
        let heartbeat = LAST_HEARTBEAT.load(Ordering::Relaxed);
        assert!(heartbeat > 0);
    }

    #[test]
    fn test_current_time() {
        let now = current_time_ms();
        assert!(now > 0);

        // Should be a reasonable Unix timestamp (after 2020)
        assert!(now > 1_577_836_800_000); // Jan 1, 2020 in ms
    }
}
