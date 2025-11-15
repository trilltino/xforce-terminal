//! Async task lifecycle tracking for debugging hung tasks and performance

use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::task::JoinHandle;

/// Global task counter
static TASK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Get current number of active tasks
pub fn active_task_count() -> u64 {
    TASK_COUNTER.load(Ordering::Relaxed)
}

/// Spawn an instrumented async task with lifecycle tracking
///
/// # Arguments
///
/// * `name` - Task name for logging (e.g., "price_fetch", "swap_execution")
/// * `future` - The async task to execute
///
/// # Returns
///
/// JoinHandle that can be awaited or detached
///
/// # Example
///
/// ```rust
/// spawn_tracked("api_call", async move {
///     api_client.get_prices(&symbols).await
/// });
/// ```
pub fn spawn_tracked<F>(name: &'static str, future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let task_id = TASK_COUNTER.fetch_add(1, Ordering::Relaxed);
    let start = Instant::now();

    tracing::info!(
        task = %name,
        task_id = task_id,
        "Task spawned"
    );

    tokio::spawn(async move {
        let result = future.await;
        let duration = start.elapsed();

        // Log completion
        tracing::info!(
            task = %name,
            task_id = task_id,
            duration_ms = duration.as_millis(),
            "Task completed"
        );

        // Warn about long-running tasks
        if duration.as_secs() > 30 {
            tracing::warn!(
                task = %name,
                task_id = task_id,
                duration_ms = duration.as_millis(),
                "Task took very long (potential hang)"
            );
        }

        TASK_COUNTER.fetch_sub(1, Ordering::Relaxed);
        result
    })
}

/// Spawn a task with timeout detection
///
/// Logs a warning if task doesn't complete within the specified duration.
/// The task continues running; this just provides visibility.
pub fn spawn_with_timeout<F>(
    name: &'static str,
    timeout_secs: u64,
    future: F,
) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let task_id = TASK_COUNTER.fetch_add(1, Ordering::Relaxed);
    let start = Instant::now();

    tracing::info!(
        task = %name,
        task_id = task_id,
        timeout_secs = timeout_secs,
        "Task spawned with timeout"
    );

    // Spawn timeout monitor
    let timeout_name = name;
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(timeout_secs)).await;

        // Check if task is still running
        if TASK_COUNTER.load(Ordering::Relaxed) > 0 {
            tracing::warn!(
                task = %timeout_name,
                task_id = task_id,
                timeout_secs = timeout_secs,
                "Task timeout exceeded (still running)"
            );
        }
    });

    // Spawn actual task
    tokio::spawn(async move {
        let result = future.await;
        let duration = start.elapsed();

        tracing::info!(
            task = %name,
            task_id = task_id,
            duration_ms = duration.as_millis(),
            "Task completed"
        );

        TASK_COUNTER.fetch_sub(1, Ordering::Relaxed);
        result
    })
}

/// Track a sync operation (blocking call on async runtime)
///
/// Use this for operations that block the async runtime thread pool.
pub fn track_blocking<F, R>(name: &'static str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();

    tracing::debug!(
        operation = %name,
        "Blocking operation started"
    );

    // Check every 100ms if we're hanging
    let _check_handle = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let elapsed = start.elapsed();
        if elapsed.as_millis() > 500 {
            eprintln!("\n!!!!! DEADLOCK DETECTED !!!!!");
            eprintln!("Operation '{}' has been blocked for {}ms", name, elapsed.as_millis());
            eprintln!("This indicates a lock is being held by another thread!");
            eprintln!("!!!!! DEADLOCK DETECTED !!!!!\n");
        }
    });

    let result = f();
    let duration = start.elapsed();

    if duration.as_millis() > 100 {
        tracing::warn!(
            operation = %name,
            duration_ms = duration.as_millis(),
            "Blocking operation took too long (starving async runtime)"
        );
    } else {
        tracing::debug!(
            operation = %name,
            duration_ms = duration.as_millis(),
            "Blocking operation completed"
        );
    }

    result
}
