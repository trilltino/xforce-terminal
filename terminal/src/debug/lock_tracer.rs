//! Lock timing instrumentation for detecting contention and deadlocks

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ops::{Deref, DerefMut};

/// Instrumented read guard that logs when lock is held too long
pub struct TracedReadGuard<'a, T> {
    guard: RwLockReadGuard<'a, T>,
    lock_name: &'static str,
    acquired_at: Instant,
}

impl<'a, T> Deref for TracedReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> Drop for TracedReadGuard<'a, T> {
    fn drop(&mut self) {
        let held_duration = self.acquired_at.elapsed();

        // Warn if lock held for more than 50ms (blocking UI)
        if held_duration.as_millis() > 50 {
            tracing::warn!(
                lock = %self.lock_name,
                held_ms = held_duration.as_millis(),
                "Read lock held too long (potential UI freeze)"
            );
        } else if held_duration.as_millis() > 10 {
            tracing::debug!(
                lock = %self.lock_name,
                held_ms = held_duration.as_millis(),
                "Read lock held"
            );
        }
    }
}

/// Instrumented write guard that logs when lock is held too long
pub struct TracedWriteGuard<'a, T> {
    guard: RwLockWriteGuard<'a, T>,
    lock_name: &'static str,
    acquired_at: Instant,
}

impl<'a, T> Deref for TracedWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> DerefMut for TracedWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<'a, T> Drop for TracedWriteGuard<'a, T> {
    fn drop(&mut self) {
        let held_duration = self.acquired_at.elapsed();

        // Warn if write lock held for more than 10ms (blocking readers)
        if held_duration.as_millis() > 10 {
            tracing::warn!(
                lock = %self.lock_name,
                held_ms = held_duration.as_millis(),
                "Write lock held too long (blocking reads)"
            );
        } else if held_duration.as_millis() > 1 {
            tracing::debug!(
                lock = %self.lock_name,
                held_ms = held_duration.as_millis(),
                "Write lock held"
            );
        }
    }
}

/// Instrumented RwLock wrapper that tracks acquisition and hold times
pub struct TracedRwLock<T> {
    inner: Arc<RwLock<T>>,
    name: &'static str,
}

impl<T> TracedRwLock<T> {
    /// Create a new instrumented RwLock
    pub fn new(value: T, name: &'static str) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
            name,
        }
    }

    /// Acquire read lock with timing instrumentation
    pub async fn read(&self) -> TracedReadGuard<'_, T> {
        self.read_traced("unknown").await
    }

    /// Acquire read lock with caller context for better debugging
    pub async fn read_traced(&self, caller: &'static str) -> TracedReadGuard<'_, T> {
        let start = Instant::now();
        let guard = self.inner.read().await;
        let wait_time = start.elapsed();

        // Log slow lock acquisitions (waited >10ms)
        if wait_time.as_millis() > 10 {
            tracing::warn!(
                lock = %self.name,
                caller = %caller,
                wait_ms = wait_time.as_millis(),
                "Slow read lock acquisition (contention detected)"
            );
        } else if wait_time.as_millis() > 1 {
            tracing::debug!(
                lock = %self.name,
                caller = %caller,
                wait_ms = wait_time.as_millis(),
                "Read lock acquired"
            );
        }

        TracedReadGuard {
            guard,
            lock_name: self.name,
            acquired_at: Instant::now(),
        }
    }

    /// Acquire write lock with timing instrumentation
    pub async fn write(&self) -> TracedWriteGuard<'_, T> {
        self.write_traced("unknown").await
    }

    /// Acquire write lock with caller context for better debugging
    pub async fn write_traced(&self, caller: &'static str) -> TracedWriteGuard<'_, T> {
        let start = Instant::now();
        let guard = self.inner.write().await;
        let wait_time = start.elapsed();

        // Log slow lock acquisitions (waited >5ms indicates contention)
        if wait_time.as_millis() > 5 {
            tracing::warn!(
                lock = %self.name,
                caller = %caller,
                wait_ms = wait_time.as_millis(),
                "Slow write lock acquisition (high contention)"
            );
        } else if wait_time.as_millis() > 1 {
            tracing::debug!(
                lock = %self.name,
                caller = %caller,
                wait_ms = wait_time.as_millis(),
                "Write lock acquired"
            );
        }

        TracedWriteGuard {
            guard,
            lock_name: self.name,
            acquired_at: Instant::now(),
        }
    }

    /// Get the inner Arc<RwLock<T>> for passing to async tasks
    pub fn inner(&self) -> Arc<RwLock<T>> {
        Arc::clone(&self.inner)
    }
}

impl<T> Clone for TracedRwLock<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            name: self.name,
        }
    }
}

/// Helper for blocking read (uses futures::executor::block_on)
pub fn block_on_read<'a, T>(lock: &'a Arc<RwLock<T>>, caller: &'static str) -> RwLockReadGuard<'a, T> {
    let start = Instant::now();
    let guard = futures::executor::block_on(lock.read());
    let wait_time = start.elapsed();

    if wait_time.as_millis() > 10 {
        tracing::warn!(
            caller = %caller,
            wait_ms = wait_time.as_millis(),
            "Slow blocking read (UI thread blocked)"
        );
    }

    guard
}

/// Helper for blocking write (uses futures::executor::block_on)
pub fn block_on_write<'a, T>(lock: &'a Arc<RwLock<T>>, caller: &'static str) -> RwLockWriteGuard<'a, T> {
    let start = Instant::now();
    let guard = futures::executor::block_on(lock.write());
    let wait_time = start.elapsed();

    if wait_time.as_millis() > 5 {
        tracing::warn!(
            caller = %caller,
            wait_ms = wait_time.as_millis(),
            "Slow blocking write (UI thread blocked)"
        );
    }

    guard
}
