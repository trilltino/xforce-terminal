//! # Pyth Network Price Feed Screen
//!
//! Displays real-time price feeds from Pyth Network oracle.

use egui;
use crate::app::{AppState, AppLike};
use crate::ui::theme::Theme;
use crate::ui::widgets::{icons::{Icons, material, size}, tables};

/// Render Pyth Network price feed screen
pub fn render(ui: &mut egui::Ui, state: &AppState, _app: &mut impl AppLike) {
    let theme = Theme::default();

    // Header with live indicators
    ui.horizontal(|ui| {
        ui.label(Icons::icon_red(material::NETWORK, size::MEDIUM));
        ui.heading("Pyth Network Price Feed");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Live indicator
            let recently_updated = state.last_price_update_time.elapsed().as_millis() < 500;
            if state.websocket_connected && recently_updated {
                let pulse = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() / 500) % 2;
                if pulse == 0 {
                    ui.colored_label(theme.success, "● LIVE");
                } else {
                    ui.colored_label(theme.dim, "○ LIVE");
                }
            }
            ui.colored_label(theme.dim, "On-chain Oracle Data");
        });
    });
    
    ui.separator();
    ui.add_space(10.0);

    // Filter prices by source
    let pyth_prices: Vec<_> = state.terminal.prices
        .iter()
        .filter(|p| p.source.as_ref().map(|s| s == "pyth").unwrap_or(false))
        .collect();

    if pyth_prices.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(Icons::icon_dim(material::NETWORK, size::LARGE));
            ui.add_space(10.0);
            ui.colored_label(theme.dim, "No Pyth prices available");
            ui.colored_label(theme.dim, "Waiting for WebSocket connection...");
        });
        return;
    }

    // Price table
    let config = tables::TableConfig {
        num_columns: 4,
        scrollable: true,
        ..Default::default()
    };

    // Check if data was recently updated for flash effect
    let recently_updated = state.last_price_update_time.elapsed().as_millis() < 500;
    
    tables::render_table(
        ui,
        "pyth_prices_table",
        config,
        &["Symbol", "Price (USD)", "24h Change", "Last Update"],
        &theme,
        |ui| {
            for price in &pyth_prices {
                // Symbol
                ui.label(&price.symbol);
                
                // Price with flash effect if recently updated
                let price_color = if recently_updated {
                    theme.selected
                } else {
                    theme.normal
                };
                ui.colored_label(price_color, format!("${:.4}", price.price));
                
                // 24h Change
                let (change_text, change_color) = theme.format_price_change(price.change_24h);
                ui.colored_label(change_color, change_text);
                
                // Last update time
                let last_update = state.last_price_update_time.elapsed();
                let update_text = if last_update.as_secs() < 60 {
                    format!("{}s ago", last_update.as_secs())
                } else {
                    format!("{}m ago", last_update.as_secs() / 60)
                };
                ui.colored_label(theme.dim, update_text);
                
                ui.end_row();
            }
        },
    );
}

