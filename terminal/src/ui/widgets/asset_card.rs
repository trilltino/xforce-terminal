//! # Asset Card Widget
//!
//! Reusable card component for displaying a single asset with live updates.

use egui;
use crate::app::PriceData;
use crate::ui::theme::Theme;
use crate::ui::widgets::icons::{Icons, material, size};
use crate::ui::widgets::price_display;

/// Render asset card component
pub fn render_asset_card(
    ui: &mut egui::Ui,
    price: &PriceData,
    recently_updated: bool,
    theme: &Theme,
) {
    // Card frame
    ui.group(|ui| {
        ui.horizontal(|ui| {
            // Symbol with icon
            ui.label(Icons::icon_red(material::TOKEN, size::SMALL));
            ui.label(format!("{}", price.symbol));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Source badge
                if let Some(source) = &price.source {
                    ui.colored_label(theme.dim, format!("[{}]", source));
                }
                
                // Change 24h
                let (change_text, change_color) = theme.format_price_change(price.change_24h);
                ui.colored_label(change_color, change_text);
                
                // Price with flash effect
                price_display::render_price_with_flash(
                    ui,
                    price.price,
                    price.previous_price,
                    recently_updated,
                    theme,
                );
            });
        });
    });
}

/// Render clickable asset card (for navigation)
pub fn render_clickable_asset_card(
    ui: &mut egui::Ui,
    price: &PriceData,
    recently_updated: bool,
    theme: &Theme,
) -> egui::Response {
    // Make entire card clickable
    let inner_response = ui.horizontal(|ui| {
        // Symbol with icon
        ui.label(Icons::icon_red(material::TOKEN, size::SMALL));
        ui.label(format!("{}", price.symbol));
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Source badge
            if let Some(source) = &price.source {
                ui.colored_label(theme.dim, format!("[{}]", source));
            }
            
            // Change 24h
            let (change_text, change_color) = theme.format_price_change(price.change_24h);
            ui.colored_label(change_color, change_text);
            
            // Price with flash effect
            price_display::render_price_with_flash(
                ui,
                price.price,
                price.previous_price,
                recently_updated,
                theme,
            );
        });
    });
    
    let response = inner_response.response;
    
    // Make hoverable
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    
    response
}

