//! # Token Picker Widget
//!
//! Interactive modal popup for selecting tokens using egui widgets.

use egui;
use crate::app::{App, AppState, TokenInfo};
use crate::ui::theme::Theme;
use crate::ui::widgets::icons::{Icons, material, size};

/// Render token picker popup
pub fn render_token_picker(ctx: &egui::Context, _state: &AppState, app: &mut App) {
    let theme = Theme::default();
    
    egui::Window::new("Select Token")
        .collapsible(false)
        .resizable(true)
        .default_size([600.0, 400.0])
        .show(ctx, |ui| {
            // Read state inside closure to get latest values
            let state = app.state.read();
            let token_picker_for = state.terminal.swap.token_picker_for;
            let token_filter = state.terminal.swap.token_filter.clone();
            let selected_token_index = state.terminal.swap.selected_token_index;
            let token_list = state.terminal.swap.token_list.clone();
            drop(state); // Release lock

            // Header with icon
            ui.horizontal(|ui| {
                ui.label(Icons::icon_red(material::TOKEN, size::MEDIUM));
                ui.heading("Select Token");
            });
            ui.add_space(10.0);

            // Search input with icon
            ui.horizontal(|ui| {
                ui.label(Icons::icon_dim(material::SEARCH, size::SMALL));
                ui.label("Search tokens:");
            });
            let mut filter = token_filter.clone();
            if ui.text_edit_singleline(&mut filter).changed() {
                let mut state_write = app.state.write();
                state_write.terminal.swap.token_filter = filter.clone();
                state_write.terminal.swap.selected_token_index = 0; // Reset selection when filtering
            }

            ui.separator();
            ui.add_space(5.0);

            // Filter tokens based on search
            let filtered_tokens: Vec<&TokenInfo> = if filter.is_empty() {
                token_list.iter().collect()
            } else {
                let filter_lower = filter.to_lowercase();
                token_list
                    .iter()
                    .filter(|token| {
                        token.symbol.to_lowercase().contains(&filter_lower)
                            || token.name.to_lowercase().contains(&filter_lower)
                    })
                    .collect()
            };

            // Sort: favorites first, then by symbol
            let mut sorted_tokens = filtered_tokens;
            sorted_tokens.sort_by(|a, b| {
                match (a.is_favorite, b.is_favorite) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.symbol.cmp(&b.symbol),
                }
            });

            // Token table with enhanced styling
            egui::ScrollArea::vertical()
                .show(ui, |ui| {
                    egui::Grid::new("token_picker")
                        .num_columns(6)
                        .spacing([10.0, 5.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Header row with styling
                            ui.colored_label(theme.selected, "Symbol");
                            ui.colored_label(theme.selected, "Name");
                            ui.colored_label(theme.selected, "Price");
                            ui.colored_label(theme.selected, "24h %");
                            ui.colored_label(theme.selected, "Balance");
                            ui.colored_label(theme.selected, "");
                            ui.end_row();

                            // Rows
                            for (idx, token) in sorted_tokens.iter().enumerate() {
                                let is_selected = idx == selected_token_index;
                                let (change_text, change_color) = theme.format_price_change(token.change_24h);

                                let symbol_text = if token.is_favorite {
                                    format!("★ {}", token.symbol)
                                } else {
                                    token.symbol.clone()
                                };

                                // Selectable row - highlight selected with red accent
                                let response = if is_selected {
                                    ui.colored_label(theme.selected, &symbol_text)
                                } else {
                                    ui.label(&symbol_text)
                                };
                                
                                if response.clicked() {
                                    app.handle_token_select((*token).clone(), token_picker_for);
                                }
                                
                                // Update selection index on hover
                                if response.hovered() {
                                    let mut state_write = app.state.write();
                                    state_write.terminal.swap.selected_token_index = idx;
                                }
                                
                                ui.label(&token.name);
                                ui.monospace(format!("${:.4}", token.price));
                                ui.colored_label(change_color, change_text);
                                ui.monospace(format!("{:.4}", token.balance));
                                // Change indicator icon
                                let change_icon = if token.change_24h > 0.0 {
                                    material::ARROW_UP
                                } else if token.change_24h < 0.0 {
                                    material::ARROW_DOWN
                                } else {
                                    ""
                                };
                                if !change_icon.is_empty() {
                                    ui.label(Icons::icon_color(change_icon, size::SMALL, change_color));
                                } else {
                                    ui.label("");
                                }
                                ui.end_row();
                            }
                        });
                });

            ui.separator();
            ui.add_space(5.0);

            // Actions with icons
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(format!("{} Select", material::CHECK)).fill(theme.selected)).clicked() {
                    // Get selected token
                    let state = app.state.read();
                    if let Some(token) = sorted_tokens.get(selected_token_index) {
                        let token = (*token).clone();
                        let target = state.terminal.swap.token_picker_for;
                        drop(state);
                        app.handle_token_select(token, target);
                    }
                }

                if ui.button(format!("{} Cancel", material::CLOSE)).clicked() {
                    let mut state_write = app.state.write();
                    state_write.terminal.swap.show_token_picker = false;
                }
            });
        });
}
