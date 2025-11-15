//! # Slippage Tolerance Selector Widget
//!
//! Interactive widget for selecting slippage tolerance percentage using egui widgets.

use egui;
use crate::app::AppState;
use crate::ui::theme::Theme;

/// Predefined slippage options in basis points
pub const SLIPPAGE_OPTIONS: &[u16] = &[
    25,   // 0.25%
    50,   // 0.5%
    100,  // 1.0%
    200,  // 2.0%
    500,  // 5.0%
];

/// Render slippage selector widget
pub fn render(ui: &mut egui::Ui, state: &AppState, app: &mut crate::app::App, theme: &Theme) {
    ui.group(|ui| {
        ui.label("Slippage Tolerance:");
        ui.horizontal(|ui| {
            let current_slippage = state.terminal.swap.slippage_bps;

            for &bps in SLIPPAGE_OPTIONS {
                let percent = (bps as f64) / 100.0;
                let text = format!("{:.2}%", percent);
                let is_selected = bps == current_slippage;

                if ui.selectable_label(is_selected, text).clicked() {
                    let mut state_write = app.state.write();
                    state_write.terminal.swap.slippage_bps = bps;
                }
            }

            // Show custom if current slippage is not in predefined options
            if !SLIPPAGE_OPTIONS.contains(&current_slippage) {
                let percent = (current_slippage as f64) / 100.0;
                ui.colored_label(theme.warning, format!("{:.2}% (custom)", percent));
            }
        });
    });
}

/// Convert slippage basis points to percentage string
pub fn bps_to_percent_str(bps: u16) -> String {
    let percent = (bps as f64) / 100.0;
    format!("{:.2}%", percent)
}
