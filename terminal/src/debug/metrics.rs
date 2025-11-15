//! Performance metrics collection

use std::collections::VecDeque;
use std::sync::{OnceLock, Mutex};
use std::time::{Duration, Instant};
use sysinfo::System;

/// Frame timing metrics
#[derive(Debug, Clone)]
pub struct FrameMetrics {
    /// Last frame total time
    pub last_frame_time: Duration,
    /// Input processing time
    pub input_time: Duration,
    /// Tick processing time (async events)
    pub tick_time: Duration,
    /// Render time
    pub render_time: Duration,
    /// Rolling window of last 60 frame times
    pub frame_history: VecDeque<Duration>,
    /// Number of slow frames (>100ms)
    pub slow_frame_count: u32,
    /// Last update timestamp
    pub last_update: Instant,
}

impl Default for FrameMetrics {
    fn default() -> Self {
        Self {
            last_frame_time: Duration::ZERO,
            input_time: Duration::ZERO,
            tick_time: Duration::ZERO,
            render_time: Duration::ZERO,
            frame_history: VecDeque::with_capacity(60),
            slow_frame_count: 0,
            last_update: Instant::now(),
        }
    }
}

impl FrameMetrics {
    /// Record a new frame time
    pub fn record_frame(&mut self, input: Duration, tick: Duration, render: Duration) {
        let total = input + tick + render;

        self.last_frame_time = total;
        self.input_time = input;
        self.tick_time = tick;
        self.render_time = render;
        self.last_update = Instant::now();

        // Add to history
        self.frame_history.push_back(total);
        if self.frame_history.len() > 60 {
            self.frame_history.pop_front();
        }

        // Count slow frames
        if total > Duration::from_millis(100) {
            self.slow_frame_count += 1;

            tracing::warn!(
                total_ms = total.as_millis(),
                input_ms = input.as_millis(),
                tick_ms = tick.as_millis(),
                render_ms = render.as_millis(),
                "Slow frame detected"
            );
        }
    }

    /// Get average frame time from history
    pub fn avg_frame_time(&self) -> Duration {
        if self.frame_history.is_empty() {
            return Duration::ZERO;
        }

        let sum: Duration = self.frame_history.iter().sum();
        sum / self.frame_history.len() as u32
    }

    /// Get max frame time from history
    pub fn max_frame_time(&self) -> Duration {
        self.frame_history
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::ZERO)
    }

    /// Get frame rate (FPS)
    pub fn fps(&self) -> f64 {
        let avg = self.avg_frame_time();
        if avg.is_zero() {
            return 0.0;
        }
        1000.0 / avg.as_millis() as f64
    }
}

/// Memory metrics
#[derive(Debug, Clone, Copy)]
pub struct MemoryMetrics {
    /// Process memory usage in MB
    pub process_mb: f64,
    /// Last update timestamp
    pub last_update: Instant,
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            process_mb: 0.0,
            last_update: Instant::now(),
        }
    }
}

impl MemoryMetrics {
    /// Update memory metrics
    /// PERFORMANCE FIX: Only refresh memory for current process
    pub fn update(&mut self) {
        use sysinfo::{ProcessRefreshKind, ProcessesToUpdate};

        let mut system = System::new();

        // Only refresh process memory (fast), not CPU usage
        let refresh_kind = ProcessRefreshKind::nothing().with_memory();

        // Only update current process
        // Note: sysinfo 0.37 API - refresh_processes_specifics takes (ProcessesToUpdate, bool, ProcessRefreshKind)
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,  // second argument is a bool flag
            refresh_kind
        );

        let pid = sysinfo::get_current_pid().expect("Failed to get PID");
        if let Some(process) = system.process(pid) {
            self.process_mb = process.memory() as f64 / 1024.0 / 1024.0;
            self.last_update = Instant::now();

            tracing::debug!(
                memory_mb = self.process_mb,
                "Memory usage updated"
            );
        }
    }
}

/// Global metrics singleton (thread-safe using OnceLock and Mutex)
static FRAME_METRICS: OnceLock<Mutex<FrameMetrics>> = OnceLock::new();
static MEMORY_METRICS: OnceLock<Mutex<MemoryMetrics>> = OnceLock::new();

/// Initialize global metrics
pub fn init_metrics() {
    FRAME_METRICS.get_or_init(|| Mutex::new(FrameMetrics::default()));
    MEMORY_METRICS.get_or_init(|| Mutex::new(MemoryMetrics::default()));
}

/// Record frame time (thread-safe)
pub fn record_frame_time(input: Duration, tick: Duration, render: Duration) {
    if let Some(metrics) = FRAME_METRICS.get() {
        if let Ok(mut m) = metrics.lock() {
            m.record_frame(input, tick, render);
        }
    }
}

/// Get current frame metrics (thread-safe)
pub fn get_frame_metrics() -> Option<FrameMetrics> {
    FRAME_METRICS.get().and_then(|m| {
        m.lock().ok().map(|guard| guard.clone())
    })
}

/// Update memory metrics (thread-safe)
pub fn update_memory_metrics() {
    if let Some(metrics) = MEMORY_METRICS.get() {
        if let Ok(mut m) = metrics.lock() {
            m.update();
        }
    }
}

/// Get current memory metrics (thread-safe)
pub fn get_memory_metrics() -> Option<MemoryMetrics> {
    MEMORY_METRICS.get().and_then(|m| {
        m.lock().ok().map(|guard| *guard)
    })
}
