//! # Live Assets Screen
//!
//! Simple vertical list of assets with prices that updates in real-time.

use egui;
use crate::app::{AppState, AppLike};
use crate::ui::theme::Theme;
use crate::ui::widgets::icons::{Icons, material, size};

/// Render live assets list screen
pub fn render(ui: &mut egui::Ui, state: &AppState, _app: &mut impl AppLike) {
    let theme = Theme::default();

    // Header with filter
    ui.horizontal(|ui| {
        ui.label(Icons::icon_red(material::TOKEN, size::MEDIUM));
        ui.heading("Live Assets");
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Search/filter box
            ui.add(egui::TextEdit::singleline(&mut String::new())
                .hint_text("Filter by symbol...")
                .desired_width(150.0));
        });
    });
    
    ui.separator();
    ui.add_space(10.0);

    // Check if prices are available
    if state.terminal.prices.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.colored_label(theme.dim, "No assets available");
            if state.websocket_connected {
                ui.colored_label(theme.warning, "Waiting for price updates...");
            } else {
                ui.colored_label(theme.error, "WebSocket disconnected");
            }
        });
        return;
    }

    // Check if data was recently updated
    let recently_updated = state.last_price_update_time.elapsed().as_millis() < 500;
    
    // Sort assets by symbol
    let mut sorted_prices = state.terminal.prices.clone();
    sorted_prices.sort_by(|a, b| a.symbol.cmp(&b.symbol));

    // Render asset list with live updates
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for price in &sorted_prices {
                render_asset_row(ui, price, &theme, recently_updated);
                ui.add_space(2.0);
            }
        });
    
    ui.add_space(10.0);
    ui.separator();
    
    // Footer with stats
    ui.horizontal(|ui| {
        ui.label(format!("Total Assets: {}", state.terminal.prices.len()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if recently_updated {
                let pulse = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() / 500) % 2;
                if pulse == 0 {
                    ui.colored_label(theme.success, "● LIVE");
                } else {
                    ui.colored_label(theme.dim, "○ LIVE");
                }
            } else if state.websocket_connected {
                ui.colored_label(theme.dim, "○ Connected");
            } else {
                ui.colored_label(theme.error, "○ Offline");
            }
        });
    });
}

/// Render a single asset row
fn render_asset_row(ui: &mut egui::Ui, price: &crate::app::PriceData, theme: &Theme, recently_updated: bool) {
    ui.horizontal(|ui| {
        // Symbol
        ui.label(format!("{}", price.symbol));
        
        // Price with flash effect if recently updated
        let price_color = if recently_updated {
            theme.selected
        } else {
            theme.normal
        };
        ui.colored_label(price_color, format!("${:.4}", price.price));
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Change 24h
            let (change_text, change_color) = theme.format_price_change(price.change_24h);
            ui.colored_label(change_color, change_text);
            
            // Source indicator
            if let Some(source) = &price.source {
                ui.colored_label(theme.dim, format!("[{}]", source));
            }
        });
    });
}

