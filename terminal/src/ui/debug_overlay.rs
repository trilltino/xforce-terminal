//! In-UI debug overlay (toggle with Ctrl+D)

use egui;

use crate::debug::metrics::{get_frame_metrics, get_memory_metrics};
use crate::debug::task_tracker::active_task_count;
use crate::debug::{get_trace_id, get_recent_errors, get_error_stats, total_error_count, pending_event_count};

/// Render debug overlay as an egui window
pub fn render_debug_overlay(ctx: &egui::Context, state: &crate::app::AppState) {
    egui::Window::new("Debug Monitor")
        .collapsible(true)
        .resizable(true)
        .default_size([400.0, 600.0])
        .max_size([600.0, 800.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Current Trace ID
                ui.heading("Trace Context");
                if let Some(trace_id) = get_trace_id() {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 255, 0),
                        format!("Trace ID: {}", &trace_id[..8]) // Show first 8 chars
                    );
                } else {
                    ui.label("No active trace");
                }

                ui.separator();

                // Performance Metrics
                ui.heading("Performance");
                let frame_metrics = get_frame_metrics();
                let memory_metrics = get_memory_metrics();
                let task_count = active_task_count();

                // Frame timing
                if let Some(metrics) = frame_metrics {
                    let total_ms = metrics.last_frame_time.as_secs_f64() * 1000.0;
                    let input_ms = metrics.input_time.as_secs_f64() * 1000.0;
                    let tick_ms = metrics.tick_time.as_secs_f64() * 1000.0;
                    let render_ms = metrics.render_time.as_secs_f64() * 1000.0;
                    let avg_ms = metrics.avg_frame_time().as_secs_f64() * 1000.0;
                    let fps = metrics.fps();

                    ui.label(format!("Frame: {:.1}ms", total_ms));
                    ui.label(format!("  Input:  {:.1}ms", input_ms));
                    ui.label(format!("  Tick:   {:.1}ms", tick_ms));
                    ui.label(format!("  Render: {:.1}ms", render_ms));
                    ui.label(format!("Avg:   {:.1}ms", avg_ms));
                    ui.label(format!("FPS:   {:.1}", fps));

                    // Slow frame warning
                    if metrics.slow_frame_count > 0 {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 0, 0),
                            format!("Slow frames: {}", metrics.slow_frame_count)
                        );
                    }
                }

                // Memory usage
                if let Some(memory) = memory_metrics {
                    ui.label(format!("Memory: {:.1} MB", memory.process_mb));
                }

                ui.separator();

                // Task and Event Queue
                ui.heading("Tasks & Events");
                ui.label(format!("Active Tasks: {}", task_count));
                let pending_events = pending_event_count();
                if pending_events > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 165, 0),
                        format!("Pending Events: {}", pending_events)
                    );
                } else {
                    ui.label(format!("Pending Events: {}", pending_events));
                }

                ui.separator();

                // Error Statistics
                ui.heading("Error Statistics");
                let error_stats = get_error_stats();
                let total_errors = total_error_count();

                ui.label(format!("Total: {}", total_errors));

                use crate::debug::ErrorLevel;
                if let Some(panic_count) = error_stats.get(&ErrorLevel::Panic) {
                    if *panic_count > 0 {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 0, 255),
                            format!("Panics: {}", panic_count)
                        );
                    }
                }

                if let Some(error_count) = error_stats.get(&ErrorLevel::Error) {
                    if *error_count > 0 {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 0, 0),
                            format!("Errors: {}", error_count)
                        );
                    } else {
                        ui.label(format!("Errors: {}", error_count));
                    }
                }

                if let Some(warning_count) = error_stats.get(&ErrorLevel::Warning) {
                    if *warning_count > 0 {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 255, 0),
                            format!("Warnings: {}", warning_count)
                        );
                    } else {
                        ui.label(format!("Warnings: {}", warning_count));
                    }
                }

                ui.separator();

                // Recent Errors
                ui.heading("Recent Errors (Last 10)");
                let recent_errors = get_recent_errors(10);

                if recent_errors.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(0, 255, 0),
                        "No recent errors"
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for error in recent_errors {
                                let color = match error.level {
                                    crate::debug::ErrorLevel::Panic => egui::Color32::from_rgb(255, 0, 255),
                                    crate::debug::ErrorLevel::Error => egui::Color32::from_rgb(255, 100, 100),
                                    crate::debug::ErrorLevel::Warning => egui::Color32::from_rgb(255, 200, 0),
                                };

                                ui.colored_label(color, format!("[{}]", error.level));

                                // Truncate long messages
                                let message = if error.message.len() > 50 {
                                    format!("{}...", &error.message[..50])
                                } else {
                                    error.message.clone()
                                };

                                ui.label(message);

                                if let Some(location) = error.location {
                                    ui.label(format!("  @ {}", location));
                                }

                                ui.add_space(4.0);
                            }
                        });
                }

                ui.separator();

                // WebSocket & Charts
                ui.heading("WebSocket & Charts");
                
                // WebSocket connection status
                let ws_url = std::env::var("API_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:3001".to_string())
                    .replace("http://", "ws://")
                    .replace("https://", "wss://")
                    + "/api/ws/prices";
                ui.label(format!("URL: {}", ws_url));
                
                if state.websocket_connected {
                    ui.colored_label(
                        egui::Color32::from_rgb(0, 255, 0),
                        "Status: Connected"
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 0, 0),
                        "Status: Disconnected"
                    );
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 165, 0),
                        "Note: Check backend server is running on port 3001"
                    );
                }
                
                // Price updates count (from static counter)
                use crate::services::api::websocket::{MESSAGE_COUNTER, RECONNECT_COUNTER};
                use std::sync::atomic::Ordering;
                let total_messages = MESSAGE_COUNTER.load(Ordering::Relaxed);
                let reconnect_attempts = RECONNECT_COUNTER.load(Ordering::Relaxed);
                ui.label(format!("Price Updates: {total_messages}"));
                if reconnect_attempts > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 165, 0),
                        format!("Reconnect Attempts: {}", reconnect_attempts)
                    );
                }
                
                // Chart information
                let timeframe_str = match state.terminal.chart_timeframe {
                    shared::dto::market::Timeframe::OneMinute => "1m",
                    shared::dto::market::Timeframe::FiveMinutes => "5m",
                    shared::dto::market::Timeframe::FifteenMinutes => "15m",
                    shared::dto::market::Timeframe::OneHour => "1h",
                    shared::dto::market::Timeframe::FourHours => "4h",
                    shared::dto::market::Timeframe::OneDay => "1d",
                    shared::dto::market::Timeframe::OneWeek => "1w",
                };
                ui.label(format!("Chart Symbol: SOL"));
                ui.label(format!("Timeframe: {}", timeframe_str));
                ui.label(format!("Candles Loaded: {}", state.terminal.sol_candles.len()));
                
                if state.terminal.chart_loading {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 255, 0),
                        "Chart: Loading..."
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(0, 255, 0),
                        "Chart: Ready"
                    );
                }
                
                // Last price update time
                let time_since_update = state.terminal.last_price_update.elapsed();
                if time_since_update.as_secs() < 5 {
                    ui.colored_label(
                        egui::Color32::from_rgb(0, 255, 0),
                        format!("Last Update: {:.1}s ago", time_since_update.as_secs_f64())
                    );
                } else if time_since_update.as_secs() < 30 {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 255, 0),
                        format!("Last Update: {:.1}s ago", time_since_update.as_secs_f64())
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 0, 0),
                        format!("Last Update: {:.1}s ago", time_since_update.as_secs_f64())
                    );
                }
                
                // Token prices count
                ui.label(format!("Tracked Tokens: {}", state.terminal.prices.len()));

                ui.separator();

                ui.label("Press Ctrl+D to toggle this overlay");
            });
        });
}

/// Check if debug overlay should be shown
///
/// Controlled by:
/// 1. Feature flag: `cfg!(feature = "debug-mode")`
/// 2. Environment variable: `TERMINAL_DEBUG_UI=1`
/// 3. Runtime toggle via state (Ctrl+D)
pub fn should_show_overlay(state: &crate::app::AppState) -> bool {
    // Check runtime toggle first (from state)
    if state.debug_overlay_visible {
        return true;
    }
    
    // Fall back to feature flag or env var
    cfg!(feature = "debug-mode")
        || std::env::var("TERMINAL_DEBUG_UI")
            .map(|v| v == "1")
            .unwrap_or(false)
}
