//! # Live Update Indicator
//!
//! Visual indicator showing when live updates are being received.

use egui;
use crate::ui::theme::Theme;

/// Render live update indicator
pub fn render_live_indicator(ui: &mut egui::Ui, is_live: bool, theme: &Theme) {
    if is_live {
        // Pulsing dot animation
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let pulse = (now / 500) % 2;
        
        if pulse == 0 {
            ui.colored_label(theme.success, "● LIVE");
        } else {
            ui.colored_label(theme.dim, "○ LIVE");
        }
    } else {
        ui.colored_label(theme.dim, "○ OFFLINE");
    }
}

/// Render connection status badge
pub fn render_connection_status(
    ui: &mut egui::Ui,
    connected: bool,
    messages_received: u64,
    theme: &Theme,
) {
    if connected {
        ui.horizontal(|ui| {
            ui.colored_label(theme.success, "●");
            ui.label("Connected");
            ui.separator();
            ui.label(format!("{} msgs", messages_received));
        });
    } else {
        ui.horizontal(|ui| {
            ui.colored_label(theme.error, "○");
            ui.label("Disconnected");
        });
    }
}

/// Render update rate display (messages/second)
pub fn render_update_rate(ui: &mut egui::Ui, messages_per_second: f64, theme: &Theme) {
    ui.label(format!("{:.1} msg/s", messages_per_second));
    
    if messages_per_second > 10.0 {
        ui.colored_label(theme.success, "High");
    } else if messages_per_second > 1.0 {
        ui.colored_label(theme.warning, "Medium");
    } else {
        ui.colored_label(theme.dim, "Low");
    }
}

