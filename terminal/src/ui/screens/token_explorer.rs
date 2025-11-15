//! # Token Explorer Screen
//!
//! Displays detailed information about available tokens using egui widgets.

use egui;
use crate::app::{AppState, AppLike};
use crate::ui::theme::Theme;
use crate::ui::widgets::icons::material;

/// Render token explorer content
pub fn render(ui: &mut egui::Ui, state: &AppState, app: &mut impl AppLike, theme: &Theme) {
    // Check which tab is active
    use crate::app::SwapTab;
    if state.terminal.swap.active_tab != SwapTab::TokenExplorer {
        return;
    }

    ui.columns(2, |columns| {
        // Token list (left)
        columns[0].vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Token List");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(format!("{} Refresh", material::REFRESH))).clicked() {
                        app.fetch_token_list();
                    }
                });
            });
            ui.add_space(5.0);

            let tokens = &state.terminal.swap.token_list;

            if tokens.is_empty() {
                ui.label("No tokens loaded");
                ui.label("Click Refresh to fetch token list from API");
                return;
            }

            // Sort tokens: favorites first
            let mut sorted_tokens: Vec<&crate::app::TokenInfo> = tokens.iter().collect();
            sorted_tokens.sort_by(|a, b| match (a.is_favorite, b.is_favorite) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.symbol.cmp(&b.symbol),
            });

            use crate::ui::widgets::tables;
            let config = tables::TableConfig {
                num_columns: 7,
                spacing: [10.0, 5.0],
                striped: true,
                scrollable: true,
            };

            tables::render_table(
                ui,
                "token_list",
                config,
                &["Symbol", "Name", "Mint Address", "Price", "24h %", "Balance", "Fav"],
                theme,
                |ui| {
                    // Rows
                    for token in sorted_tokens {
                        let (change_text, change_color) = theme.format_price_change(token.change_24h);
                        let fav_indicator = if token.is_favorite { "★" } else { "☆" };
                        
                        // Format mint address (show first 8 and last 8 chars)
                        let mint_display = if token.mint.len() > 16 {
                            format!("{}...{}", &token.mint[..8], &token.mint[token.mint.len()-8..])
                        } else {
                            token.mint.clone()
                        };

                        ui.label(&token.symbol);
                        ui.label(&token.name);
                        
                        // Mint address with copy button
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&mint_display).family(egui::FontFamily::Monospace).size(11.0));
                            if ui.small_button("📋").clicked() {
                                ui.ctx().copy_text(token.mint.clone());
                            }
                        });
                        
                        ui.label(format!("${:.4}", token.price));
                        ui.colored_label(change_color, change_text);
                        ui.label(format!("{:.2}", token.balance));
                        ui.colored_label(theme.warning, fav_indicator);
                        ui.end_row();
                    }
                },
            );
        });

        // Token details (right)
        columns[1].vertical(|ui| {
            ui.heading("Token Details");
            ui.add_space(10.0);

            // Placeholder - in the future, show detailed info about selected token
            ui.label("Select a token from the list");
            ui.label("to view detailed information");
            ui.add_space(10.0);
            ui.colored_label(theme.dim, "(Coming soon: Price charts, recent trades)");
        });
    });
}
