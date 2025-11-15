//! # Terminal Screen (Trading View)
//!
//! Main trading interface with SOL candlestick chart and real-time token prices.
//! Redesigned with full-screen layout: Chart (60%) | Token List (40%) + collapsible swap panel.

use egui;
use crate::app::{AppState, AppLike};
use crate::ui::theme::Theme;
use crate::ui::widgets::icons::{Icons, material, size};

/// Render main trading terminal screen
pub fn render(ui: &mut egui::Ui, state: &AppState, app: &mut impl AppLike) {
    let theme = Theme::default();

    // Top toolbar with swap toggle button
    ui.horizontal(|ui| {
        // Swap panel toggle button
        let swap_label = if state.terminal.swap_panel_open { "Hide Swap" } else { "Show Swap" };
        if ui.button(format!("{} {}", material::SWAP, swap_label)).clicked() {
            let mut state_write = app.state().write();
            state_write.terminal.swap_panel_open = !state_write.terminal.swap_panel_open;
        }
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Current SOL price display (if available)
            if let Some(sol_price) = state.terminal.prices.iter().find(|p| p.symbol == "SOL") {
                ui.colored_label(theme.selected, format!("SOL: ${:.2}", sol_price.price));
                let (change_text, change_color) = theme.format_price_change(sol_price.change_24h);
                ui.colored_label(change_color, change_text);
            }
        });
    });
    
    ui.separator();
    ui.add_space(5.0);

    // Main 2-column layout: Chart (60%) | Token List (40%)
    // If swap panel is open, adjust layout
    if state.terminal.swap_panel_open {
        // 3-column: Swap Panel | Chart | Token List
        ui.columns(3, |columns| {
            // Swap panel (left, ~300px)
            columns[0].vertical(|ui| {
                ui.set_width(300.0);
                render_swap_panel(ui, state, app, &theme);
            });

            // Chart (middle, 60% of remaining)
            columns[1].vertical(|ui| {
                render_chart_panel(ui, state, app, &theme);
            });

            // Token list (right, 40% of remaining)
            columns[2].vertical(|ui| {
                render_price_list(ui, state, &theme);
            });
        });
    } else {
        // 2-column: Chart (60%) | Token List (40%)
        ui.columns(2, |columns| {
            // Chart (left, 60%)
            columns[0].vertical(|ui| {
                render_chart_panel(ui, state, app, &theme);
            });

            // Token list (right, 40%)
            columns[1].vertical(|ui| {
                render_price_list(ui, state, &theme);
            });
        });
    }
}

/// Render SOL candlestick chart panel (left side, 60%)
fn render_chart_panel(ui: &mut egui::Ui, state: &AppState, app: &mut impl AppLike, theme: &Theme) {
    use crate::ui::widgets::layouts;
    
    layouts::render_panel(ui, None, |ui| {
        // Chart header with timeframe selector
        ui.horizontal(|ui| {
            ui.label(Icons::icon_red(material::CHART, size::MEDIUM));
            ui.heading("SOL Price Chart");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Timeframe selector buttons
                let timeframes = [
                    shared::dto::market::Timeframe::OneMinute,
                    shared::dto::market::Timeframe::FiveMinutes,
                    shared::dto::market::Timeframe::FifteenMinutes,
                    shared::dto::market::Timeframe::OneHour,
                    shared::dto::market::Timeframe::FourHours,
                    shared::dto::market::Timeframe::OneDay,
                ];
                
                for tf in timeframes.iter().rev() {
                    let is_selected = *tf == state.terminal.chart_timeframe;
                    let button = if is_selected {
                        egui::Button::new(tf.label()).fill(theme.selected)
                    } else {
                        egui::Button::new(tf.label())
                    };
                    
                    if ui.add(button).clicked() && !is_selected {
                        // Change timeframe
                        let old_timeframe = state.terminal.chart_timeframe;
                        let new_timeframe = *tf;
                        let old_str = match old_timeframe {
                            shared::dto::market::Timeframe::OneMinute => "1m",
                            shared::dto::market::Timeframe::FiveMinutes => "5m",
                            shared::dto::market::Timeframe::FifteenMinutes => "15m",
                            shared::dto::market::Timeframe::OneHour => "1h",
                            shared::dto::market::Timeframe::FourHours => "4h",
                            shared::dto::market::Timeframe::OneDay => "1d",
                            shared::dto::market::Timeframe::OneWeek => "1w",
                        };
                        let new_str = tf.label();
                        
                        tracing::debug!(
                            old_timeframe = %old_str,
                            new_timeframe = %new_str,
                            symbol = "SOL",
                            "Timeframe changed by user"
                        );
                        
                        let mut state_write = app.state().write();
                        state_write.terminal.chart_timeframe = new_timeframe;
                        state_write.terminal.chart_loading = true;
                        drop(state_write);
                        
                        // Fetch new candles
                        app.fetch_candles("SOL", new_timeframe);
                    }
                }
            });
        });
        
        ui.add_space(5.0);
        
        // Chart content
        if state.terminal.chart_loading && state.terminal.sol_candles.is_empty() {
            ui.colored_label(theme.dim, "Loading chart data...");
        } else if state.terminal.sol_candles.is_empty() {
            ui.colored_label(theme.dim, "No chart data available");
            if state.websocket_connected {
                ui.label("Waiting for SOL price updates to generate candles...");
                if let Some(sol_price) = state.terminal.prices.iter().find(|p| p.symbol == "SOL") {
                    ui.colored_label(theme.success, format!("SOL price received: ${:.4}", sol_price.price));
                    ui.label("Chart should load automatically...");
                } else {
                    ui.colored_label(theme.warning, "SOL price not yet received from WebSocket");
                }
            } else {
                ui.label("Chart will display once WebSocket connection is established");
            }
        } else {
            // Render candlestick chart
            crate::ui::chart::render_candlestick_chart(ui, &state.terminal.sol_candles, theme);
            
            // Show current price info
            if let Some(last_candle) = state.terminal.sol_candles.last() {
                ui.add_space(10.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Current Price:");
                    ui.colored_label(theme.selected, format!("${:.4}", last_candle.close));
                    
                    if last_candle.is_bullish() {
                        ui.colored_label(theme.success, "▲");
                    } else if last_candle.is_bearish() {
                        ui.colored_label(theme.error, "▼");
                    }
                    
                    let change = last_candle.close - last_candle.open;
                    let change_pct = (change / last_candle.open) * 100.0;
                    let (change_text, change_color) = theme.format_price_change(change_pct);
                    ui.colored_label(change_color, change_text);
                });
            }
        }
    });
}

/// Render token price list (right side, 40%)
fn render_price_list(ui: &mut egui::Ui, state: &AppState, theme: &Theme) {
    use crate::ui::widgets::{layouts, tables};
    
    layouts::render_panel(ui, None, |ui| {
        ui.horizontal(|ui| {
            ui.label(Icons::icon_red(material::CHART, size::MEDIUM));
            ui.heading("Token Prices");
        });
        ui.add_space(5.0);

        // Check if prices are available
        // Debug logging for UI rendering
        let price_count = state.terminal.prices.len();
        let websocket_status = &state.websocket_status;
        tracing::debug!(
            price_count = price_count,
            websocket_state = ?websocket_status.state,
            messages_received = websocket_status.messages_received,
            last_message = ?websocket_status.last_message,
            "UI RENDER: Rendering price table"
        );
        
        if state.terminal.prices.is_empty() {
            tracing::warn!(
                price_count = 0,
                websocket_connected = state.websocket_connected,
                websocket_state = ?websocket_status.state,
                messages_received = websocket_status.messages_received,
                "UI RENDER: Price list is empty - no data to display"
            );
            ui.colored_label(theme.dim, "Waiting for price updates...");
            if state.websocket_connected {
                ui.colored_label(theme.warning, "WebSocket connected but no prices received yet");
                ui.label("This may indicate the backend is not sending price updates");
            } else {
                ui.label("Prices will appear here once WebSocket connection is established");
                ui.colored_label(theme.error, "WebSocket: Disconnected");
            }
            return;
        }

        tracing::info!(
            price_count = price_count,
            symbols = ?state.terminal.prices.iter().map(|p| p.symbol.clone()).collect::<Vec<_>>(),
            "UI RENDER: Rendering price table with data"
        );

        // Enhanced table using modular component
        let config = tables::TableConfig {
            num_columns: 4,
            spacing: [10.0, 2.0],
            striped: true,
            scrollable: true,
        };

        tables::render_table(
            ui,
            "prices",
            config,
            &["Symbol", "Price", "24h %", "Change"],
            theme,
            |ui| {
                // Sort prices by price (descending) - show ALL tokens, not just top 10
                let mut sorted_prices: Vec<_> = state.terminal.prices.iter().collect();
                sorted_prices.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));
                
                tracing::debug!(
                    sorted_count = sorted_prices.len(),
                    "UI RENDER: Rendering {} price rows in table",
                    sorted_prices.len()
                );
                
                // Data rows - show all tokens with real-time flash effects
                for price in sorted_prices {
                    let (change_text, change_color) = theme.format_price_change(price.change_24h);
                    let change_icon = if price.change_24h > 0.0 {
                        material::ARROW_UP
                    } else if price.change_24h < 0.0 {
                        material::ARROW_DOWN
                    } else {
                        ""
                    };

                    // Bloomberg-style flash effect: highlight price if it changed recently
                    let (price_color, is_flashing) = if let Some(prev_price) = price.previous_price {
                        let price_change = price.price - prev_price;
                        let time_since_update = state.last_price_update_time.elapsed();
                        
                        // Flash for 500ms after price change
                        if time_since_update.as_millis() < 500 && price_change.abs() > 0.0001 {
                            if price_change > 0.0 {
                                // Bright green for price increase
                                (egui::Color32::from_rgb(0, 255, 0), true)
                            } else {
                                // Bright red for price decrease
                                (egui::Color32::from_rgb(255, 0, 0), true)
                            }
                        } else {
                            // Normal color
                            (theme.selected, false)
                        }
                    } else {
                        // Normal color for new tokens
                        (theme.selected, false)
                    };

                    // Render symbol
                    ui.label(&price.symbol);
                    
                    // Render price with flash effect (bright color when changing)
                    if is_flashing {
                        ui.colored_label(price_color, format!("${:.4}", price.price));
                    } else {
                        ui.monospace(format!("${:.4}", price.price));
                    }
                    
                    // Render change percentage
                    ui.colored_label(change_color, change_text);
                    
                    // Render change icon
                    if !change_icon.is_empty() {
                        ui.label(Icons::icon_color(change_icon, size::SMALL, change_color));
                    } else {
                        ui.label("");
                    }
                    ui.end_row();
                }
            },
        );
    });
}

/// Render collapsible swap panel
fn render_swap_panel(ui: &mut egui::Ui, state: &AppState, app: &mut impl AppLike, theme: &Theme) {
    use crate::ui::widgets::layouts;
    layouts::render_panel(ui, None, |ui| {
        ui.horizontal(|ui| {
            ui.label(Icons::icon_red(material::SWAP, size::MEDIUM));
            ui.heading("Swap Tokens");
        });
        ui.add_space(10.0);

        // From token
        ui.horizontal(|ui| {
            ui.label("From:");
            if ui.button(&state.terminal.swap.input_token).clicked() {
                {
                    let mut state_write = app.state().write();
                    app.open_token_picker_internal(&mut *state_write, crate::app::TokenPickerTarget::Input);
                }
            }
        });
        ui.add_space(5.0);

        // To token
        ui.horizontal(|ui| {
            ui.label("To:");
            if ui.button(&state.terminal.swap.output_token).clicked() {
                {
                    let mut state_write = app.state().write();
                    app.open_token_picker_internal(&mut *state_write, crate::app::TokenPickerTarget::Output);
                }
            }
        });
        ui.add_space(5.0);

        // Amount input
        ui.label("Amount:");
        let mut amount = state.terminal.swap.amount.clone();
        let amount_response = ui.text_edit_singleline(&mut amount);
        if amount_response.changed() {
            {
                let mut state_write = app.state().write();
                state_write.terminal.swap.amount = amount.clone();
            }
            if !amount.is_empty() {
                app.trigger_quote_fetch();
            }
        }
        if ui.button("Max").clicked() {
            app.set_max_amount();
            app.trigger_quote_fetch();
        }
        ui.add_space(10.0);

        // Quote display
        if state.terminal.swap.quote_loading {
            ui.colored_label(theme.info, "Fetching quote...");
            ui.label("Please wait");
        } else if let Some(quote) = &state.terminal.swap.quote {
            ui.label("Estimated Output:");
            ui.colored_label(theme.success, format!("{:.6}", quote.output_amount));
            ui.label("Price Impact:");
            ui.colored_label(theme.warning, format!("{:.2}%", quote.price_impact));
            ui.label("Est. Fee:");
            ui.colored_label(theme.dim, format!("{:.6}", quote.estimated_fee));
        } else {
            ui.colored_label(theme.dim, "No quote available");
            ui.label("Enter amount to get quote");
        }
        ui.add_space(10.0);

        // Execute button
        if ui.add(egui::Button::new(format!("{} Execute Swap", material::SEND)).fill(theme.selected)).clicked() {
            app.handle_swap_execute_click();
        }
    });
}
