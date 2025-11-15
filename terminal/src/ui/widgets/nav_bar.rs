//! # Bloomberg-Style Navigation Bar
//!
//! Navigation bar component with token selector, navigation arrows, and exclusive access
//! to Messaging and Settings screens.

use egui;
use crate::app::{AppState, AppLike, Screen};
use crate::ui::theme::Theme;

/// Render Bloomberg-style navigation bar
/// Only visible when user is authenticated
pub fn render_nav_bar(ui: &mut egui::Ui, state: &AppState, app: &mut impl AppLike) {
    if !state.is_authenticated() {
        return; // Only show when logged in
    }

    let _theme = Theme::default();
    
    // Bloomberg-style dark background with white boxes
    ui.style_mut().visuals.panel_fill = egui::Color32::from_rgb(20, 20, 30); // Dark blue-grey
    ui.style_mut().visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 20, 30);
    
    ui.horizontal(|ui| {
        ui.set_height(35.0); // Fixed height for nav bar
        
        // Navigation arrows at far left
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);
            
            // Left arrow (<) - navigates to previous screen (excludes Messaging/Settings)
            if ui.button("<").clicked() {
                app.previous_screen();
            }
            
            // Right arrow (>) - navigates to next screen (excludes Messaging/Settings)
            if ui.button(">").clicked() {
                app.next_screen();
            }
        });
        
        ui.add_space(10.0);
        
        // Token selector box (white rectangle)
        ui.horizontal(|ui| {
            let selected_token = state.nav_bar_selected_token.as_deref().unwrap_or("SOL");
            let token_display = format!("{} TOKEN Crypto", selected_token);
            
            // White box background
            let response = ui.allocate_response(
                egui::vec2(200.0, 28.0),
                egui::Sense::click()
            );
            
            // Draw white box
            let rect = response.rect;
            ui.painter().rect_filled(
                rect,
                2.0,
                egui::Color32::WHITE
            );
            
            // Draw text on white box
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &token_display,
                egui::FontId::proportional(12.0),
                egui::Color32::BLACK
            );
            
            // Handle click to show dropdown
            if response.clicked() {
                let mut state_write = app.state().write();
                state_write.nav_bar_show_token_picker = !state_write.nav_bar_show_token_picker;
            }
        });
        
        ui.add_space(5.0);
        
        // "CRY" box (white rectangle)
        ui.horizontal(|ui| {
            let response = ui.allocate_response(
                egui::vec2(50.0, 28.0),
                egui::Sense::hover()
            );
            
            let rect = response.rect;
            ui.painter().rect_filled(
                rect,
                2.0,
                egui::Color32::WHITE
            );
            
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "CRY",
                egui::FontId::proportional(12.0),
                egui::Color32::BLACK
            );
        });
        
        ui.add_space(5.0);
        
        // "Related Functions Menu" box (white rectangle)
        ui.horizontal(|ui| {
            let response = ui.allocate_response(
                egui::vec2(180.0, 28.0),
                egui::Sense::hover()
            );
            
            let rect = response.rect;
            ui.painter().rect_filled(
                rect,
                2.0,
                egui::Color32::WHITE
            );
            
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Related Functions Menu",
                egui::FontId::proportional(12.0),
                egui::Color32::BLACK
            );
        });
        
        // Spacer to push right-side items to far right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(10.0);
            
            // Settings button
            if ui.button("⚙ Settings").clicked() {
                app.handle_screen_change(Screen::Settings);
            }
            
            ui.add_space(10.0);
            
            // Message link (exclusive access to Messaging)
            if ui.link("Message").clicked() {
                app.handle_screen_change(Screen::Messaging);
            }
        });
    });
    
    // Token picker dropdown (if open)
    if state.nav_bar_show_token_picker {
        // Get token list and selected token (clone to avoid borrow issues)
        let (token_list, selected_token) = {
            let state_read = app.state().read();
            let token_list = state_read.terminal.swap.token_list.clone();
            let selected_token = state_read.nav_bar_selected_token.as_deref().unwrap_or("SOL").to_string();
            (token_list, selected_token)
        };
        
        // Show dropdown menu
        egui::Window::new("Select Token")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                // Default SOL if no tokens loaded
                if token_list.is_empty() {
                    if ui.selectable_label(selected_token == "SOL", "SOL TOKEN Crypto").clicked() {
                        let mut state_write = app.state().write();
                        state_write.nav_bar_selected_token = Some("SOL".to_string());
                        state_write.nav_bar_show_token_picker = false;
                    }
                } else {
                    for token in token_list.iter() {
                        let token_display = format!("{} TOKEN Crypto", token.symbol);
                        let is_selected = selected_token == token.symbol;
                        
                        if ui.selectable_label(is_selected, &token_display).clicked() {
                            let mut state_write = app.state().write();
                            state_write.nav_bar_selected_token = Some(token.symbol.clone());
                            state_write.nav_bar_show_token_picker = false;
                        }
                    }
                }
            });
    }
}

