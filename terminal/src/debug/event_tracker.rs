//! Event tracking system for monitoring application events
//!
//! Tracks all AppEvent dispatches and receipts, providing visibility into
//! the event flow through the application.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use std::collections::VecDeque;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Global event counter
static EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Event history for debugging
static EVENT_HISTORY: Lazy<Mutex<EventHistory>> = Lazy::new(|| {
    Mutex::new(EventHistory::new(100)) // Keep last 100 events
});

/// Event tracking information
#[derive(Debug, Clone)]
pub struct EventInfo {
    pub event_id: u64,
    pub event_type: String,
    pub trace_id: Option<String>,
    pub timestamp: Instant,
    pub received: bool,
}

/// Ring buffer of recent events
struct EventHistory {
    events: VecDeque<EventInfo>,
    max_size: usize,
}

impl EventHistory {
    fn new(max_size: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn push(&mut self, event: EventInfo) {
        if self.events.len() >= self.max_size {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    fn get_recent(&self, count: usize) -> Vec<EventInfo> {
        self.events
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }

    fn pending_count(&self) -> usize {
        self.events.iter().filter(|e| !e.received).count()
    }
}

/// Track an event dispatch
pub fn track_event_send(event_type: &str) -> u64 {
    let event_id = EVENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let trace_id = super::trace_context::get_trace_id();

    let event_info = EventInfo {
        event_id,
        event_type: event_type.to_string(),
        trace_id: trace_id.clone(),
        timestamp: Instant::now(),
        received: false,
    };

    // Add to history
    if let Ok(mut history) = EVENT_HISTORY.lock() {
        history.push(event_info);
    }

    // Log the event
    if let Some(tid) = trace_id {
        tracing::debug!(
            event_id = event_id,
            trace_id = %tid,
            event_type = event_type,
            "Event dispatched"
        );
    } else {
        tracing::debug!(
            event_id = event_id,
            event_type = event_type,
            "Event dispatched"
        );
    }

    event_id
}

/// Track an event receipt
pub fn track_event_receive(event_type: &str, event_id: Option<u64>) {
    let trace_id = super::trace_context::get_trace_id();

    if let Some(tid) = &trace_id {
        if let Some(eid) = event_id {
            tracing::debug!(
                event_id = eid,
                trace_id = %tid,
                event_type = event_type,
                "Event received"
            );
        } else {
            tracing::debug!(
                trace_id = %tid,
                event_type = event_type,
                "Event received"
            );
        }
    } else {
        if let Some(eid) = event_id {
            tracing::debug!(
                event_id = eid,
                event_type = event_type,
                "Event received"
            );
        } else {
            tracing::debug!(
                event_type = event_type,
                "Event received"
            );
        }
    }

    // Mark as received in history
    if let (Some(eid), Ok(mut history)) = (event_id, EVENT_HISTORY.lock()) {
        if let Some(event) = history.events.iter_mut().rev().find(|e| e.event_id == eid) {
            event.received = true;
        }
    }
}

/// Get recent event history
pub fn get_recent_events(count: usize) -> Vec<EventInfo> {
    EVENT_HISTORY
        .lock()
        .map(|h| h.get_recent(count))
        .unwrap_or_default()
}

/// Get count of pending (not yet received) events
pub fn pending_event_count() -> usize {
    EVENT_HISTORY
        .lock()
        .map(|h| h.pending_count())
        .unwrap_or(0)
}

/// Get total event count
pub fn total_event_count() -> u64 {
    EVENT_COUNTER.load(Ordering::Relaxed)
}

/// Log event queue statistics
pub fn log_event_stats() {
    let total = total_event_count();
    let pending = pending_event_count();

    tracing::info!(
        total_events = total,
        pending_events = pending,
        "Event queue statistics"
    );
}

/// Macro to wrap event sends with automatic tracking
#[macro_export]
macro_rules! tracked_send {
    ($sender:expr, $event:expr) => {{
        let event_type = format!("{:?}", $event);
        let event_id = $crate::debug::event_tracker::track_event_send(&event_type);
        let result = $sender.send($event).await;
        if result.is_err() {
            tracing::error!(
                event_id = event_id,
                event_type = %event_type,
                "Failed to send event - channel closed"
            );
        }
        result
    }};
}

/// Macro to wrap event receives with automatic tracking
#[macro_export]
macro_rules! tracked_receive {
    ($receiver:expr) => {{
        match $receiver.recv().await {
            Ok(event) => {
                let event_type = format!("{:?}", event);
                $crate::debug::event_tracker::track_event_receive(&event_type, None);
                Ok(event)
            }
            Err(e) => Err(e),
        }
    }};
}
