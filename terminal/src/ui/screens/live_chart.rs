//! # Live Chart Screen
//!
//! Real-time candlestick chart that updates on price changes with live price overlay.

use egui;
use crate::app::{AppState, AppLike};
use crate::ui::theme::Theme;
use crate::ui::widgets::icons::{Icons, material, size};
use crate::ui::chart;
use shared::dto::market::Timeframe;

/// Render live chart screen with real-time updates
pub fn render(ui: &mut egui::Ui, state: &AppState, app: &mut impl AppLike) {
    let theme = Theme::default();

    // Header with symbol selector and timeframe controls
    ui.horizontal(|ui| {
        ui.label(Icons::icon_red(material::CHART, size::MEDIUM));
        ui.heading("Live Chart");
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Timeframe selector
            let timeframes = [
                Timeframe::OneMinute,
                Timeframe::FiveMinutes,
                Timeframe::FifteenMinutes,
                Timeframe::OneHour,
                Timeframe::FourHours,
                Timeframe::OneDay,
            ];
            
            for tf in timeframes.iter().rev() {
                let is_selected = *tf == state.terminal.chart_timeframe;
                let button = if is_selected {
                    egui::Button::new(tf.label()).fill(theme.selected)
                } else {
                    egui::Button::new(tf.label())
                };
                
                if ui.add(button).clicked() && !is_selected {
                    let mut state_write = app.state().write();
                    state_write.terminal.chart_timeframe = *tf;
                    state_write.terminal.chart_loading = true;
                    drop(state_write);
                    app.fetch_candles("SOL", *tf);
                }
            }
            
            ui.add_space(10.0);
            
            // Symbol selector (default to SOL for now, can be expanded)
            ui.label("Symbol:");
            ui.selectable_label(true, "SOL"); // TODO: Make this selectable
        });
    });
    
    ui.separator();
    ui.add_space(5.0);

    // Get current symbol's price for overlay
    let current_price = state.terminal.prices
        .iter()
        .find(|p| p.symbol == "SOL")
        .map(|p| p.price)
        .unwrap_or(0.0);
    
    // Check if data was recently updated for flash effect
    let recently_updated = state.last_price_update_time.elapsed().as_millis() < 500;
    
    // Price overlay at top
    ui.horizontal(|ui| {
        ui.label("Current Price:");
            if recently_updated {
                // Flash effect when recently updated - request immediate repaint
                ui.ctx().request_repaint();
            }
        let price_color = if recently_updated {
            theme.selected
        } else {
            theme.normal
        };
        ui.colored_label(price_color, format!("${:.4}", current_price));
        
        // Show change from previous price
        if let Some(sol_price) = state.terminal.prices.iter().find(|p| p.symbol == "SOL") {
            if let Some(prev_price) = sol_price.previous_price {
                let change = sol_price.price - prev_price;
                let change_percent = (change / prev_price) * 100.0;
                let change_color = if change >= 0.0 {
                    theme.success
                } else {
                    theme.error
                };
                let arrow = if change >= 0.0 { "↑" } else { "↓" };
                ui.colored_label(
                    change_color,
                    format!("{} {:+.2}% ({:+.4})", arrow, change_percent, change)
                );
            }
        }
        
        // WebSocket status indicator
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if state.websocket_connected && matches!(state.websocket_status.state, crate::app::WebSocketState::Connected) {
                let pulse = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() / 500) % 2;
                if pulse == 0 {
                    ui.colored_label(theme.success, "● LIVE");
                } else {
                    ui.colored_label(theme.dim, "○ LIVE");
                }
            } else {
                ui.colored_label(theme.dim, "○ OFFLINE");
            }
        });
    });
    
    ui.add_space(5.0);
    ui.separator();
    ui.add_space(5.0);

    // Chart area - use existing chart rendering
    if state.terminal.chart_loading && state.terminal.sol_candles.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.colored_label(theme.dim, "Loading chart data...");
        });
    } else if state.terminal.sol_candles.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.colored_label(theme.dim, "No chart data available");
            if state.websocket_connected {
                ui.label("Waiting for price updates to generate candles...");
            } else {
                ui.colored_label(theme.warning, "WebSocket not connected");
            }
        });
    } else {
        // Render candlestick chart - use existing chart rendering function
        chart::render_candlestick_chart(ui, &state.terminal.sol_candles, &theme);
        
        // Show current price info with live update indicator
        if let Some(last_candle) = state.terminal.sol_candles.last() {
            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Latest Candle:");
                ui.colored_label(theme.selected, format!("${:.4}", last_candle.close));
                
                let change = last_candle.close - last_candle.open;
                let change_pct = (change / last_candle.open) * 100.0;
                let (change_text, change_color) = theme.format_price_change(change_pct);
                ui.colored_label(change_color, change_text);
                
                if recently_updated {
                    ui.colored_label(theme.success, "● LIVE");
                }
            });
        }
    }
    
    ui.add_space(10.0);
    
    // Volume bars at bottom (if we have volume data)
    if !state.terminal.sol_candles.is_empty() {
        ui.label("Volume:");
        ui.horizontal(|ui| {
            let max_volume = state.terminal.sol_candles
                .iter()
                .map(|c| c.volume)
                .fold(0.0, f64::max);
            
            let available_width = ui.available_width();
            let bar_width = (available_width / state.terminal.sol_candles.len() as f32).max(2.0);
            
            for (i, candle) in state.terminal.sol_candles.iter().enumerate() {
                let bar_height = ((candle.volume / max_volume * 100.0) as f32).max(5.0);
                let rect = egui::Rect::from_min_size(
                    egui::pos2(i as f32 * bar_width, 0.0),
                    egui::vec2(bar_width - 1.0, bar_height),
                );
                ui.painter().rect_filled(rect, 0.0, theme.dim);
            }
        });
    }
}

