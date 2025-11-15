//! # Status Bar Widget
//!
//! Bottom status bar showing WebSocket status, update rates, and connection info.

use egui;
use crate::app::AppState;
use crate::ui::theme::Theme;
use crate::ui::widgets::icons::{Icons, material, size};
use crate::ui::widgets::live_indicator;

/// Render enhanced status bar at bottom
pub fn render_status_bar(ui: &mut egui::Ui, state: &AppState) {
    let theme = Theme::default();
    
    ui.horizontal(|ui| {
        // WebSocket connection status
        let is_connected = state.websocket_connected 
            && matches!(state.websocket_status.state, crate::app::WebSocketState::Connected);
        let recently_updated = state.last_price_update_time.elapsed().as_millis() < 1000;
        
        // Live indicator
        if is_connected && recently_updated {
            live_indicator::render_live_indicator(ui, true, &theme);
        } else if is_connected {
            live_indicator::render_connection_status(
                ui,
                true,
                state.websocket_status.messages_received,
                &theme,
            );
        } else {
            live_indicator::render_connection_status(ui, false, 0, &theme);
        }
        
        ui.separator();
        
        // Update rate calculation (messages per second)
        let update_rate = calculate_update_rate(state);
        if update_rate > 0.0 {
            live_indicator::render_update_rate(ui, update_rate, &theme);
        }
        
        ui.separator();
        
        // Last update timestamp
        let last_update = state.last_price_update_time.elapsed();
        let last_update_text = if last_update.as_secs() < 60 {
            format!("{}s ago", last_update.as_secs())
        } else if last_update.as_secs() < 3600 {
            format!("{}m ago", last_update.as_secs() / 60)
        } else {
            format!("{}h ago", last_update.as_secs() / 3600)
        };
        ui.label(format!("Last: {}", last_update_text));
        
        ui.separator();
        
        // Total messages received
        if state.websocket_status.messages_received > 0 {
            ui.label(format!("Total: {} msgs", state.websocket_status.messages_received));
        }
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Selected asset (if any)
            if let Some(selected_token) = &state.nav_bar_selected_token {
                ui.label(format!("Selected: {}", selected_token));
                ui.separator();
            }
            
            // Price count
            ui.label(format!("{} assets", state.terminal.prices.len()));
        });
    });
}

/// Calculate update rate (messages per second)
fn calculate_update_rate(state: &AppState) -> f64 {
    if let Some(last_message_time) = state.websocket_status.last_message {
        let elapsed = last_message_time.elapsed();
        if elapsed.as_secs() > 0 {
            state.websocket_status.messages_received as f64 / elapsed.as_secs() as f64
        } else {
            // If less than a second, estimate based on recent updates
            let recent_updates = if state.last_price_update_time.elapsed().as_millis() < 1000 {
                1.0
            } else {
                0.0
            };
            recent_updates / elapsed.as_secs_f64().max(1.0)
        }
    } else {
        0.0
    }
}

