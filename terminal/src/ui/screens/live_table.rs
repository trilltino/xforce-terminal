//! # Live Data Table Screen
//!
//! Comprehensive data table with multiple columns showing live data points.

use egui;
use crate::app::{AppState, AppLike};
use crate::ui::theme::Theme;
use crate::ui::widgets::icons::{Icons, material, size};
use crate::ui::widgets::tables;

/// Sort column for the table
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SortColumn {
    Symbol,
    Price,
    Change24h,
    High24h,
    Low24h,
    Volume24h,
    Source,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortDirection {
    Ascending,
    Descending,
}


/// Table state for sorting and filtering
#[derive(Clone)]
struct TableState {
    sort_column: Option<SortColumn>,
    sort_direction: SortDirection,
    filter_symbol: String,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            sort_column: Some(SortColumn::Symbol),
            sort_direction: SortDirection::Ascending,
            filter_symbol: String::new(),
        }
    }
}

/// Render live data table screen
pub fn render(ui: &mut egui::Ui, state: &AppState, _app: &mut impl AppLike) {
    let theme = Theme::default();
    
    // Use egui memory to persist table state across frames
    let table_state_id = egui::Id::new("live_table_state");
    let mut table_state: TableState = ui.memory_mut(|m| {
        m.data.get_temp(table_state_id).unwrap_or_default()
    });

    // Header
    ui.horizontal(|ui| {
        ui.label(Icons::icon_red(material::CHART, size::MEDIUM));
        ui.heading("Live Data Table");
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Filter input
            ui.add(egui::TextEdit::singleline(&mut table_state.filter_symbol)
                .hint_text("Filter...")
                .desired_width(150.0));
            ui.label("Filter:");
            
            // Live indicator
            let recently_updated = state.last_price_update_time.elapsed().as_millis() < 500;
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
    
    ui.separator();
    ui.add_space(10.0);

    // Check if prices are available
    if state.terminal.prices.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.colored_label(theme.dim, "No data available");
            if state.websocket_connected {
                ui.colored_label(theme.warning, "Waiting for price updates...");
            } else {
                ui.colored_label(theme.error, "WebSocket disconnected");
            }
        });
        // Save state
        ui.memory_mut(|m| {
            m.data.insert_temp(table_state_id, table_state);
        });
        return;
    }

    // Filter and sort prices
    let mut filtered_prices: Vec<_> = state.terminal.prices
        .iter()
        .filter(|p| {
            table_state.filter_symbol.is_empty() 
                || p.symbol.to_lowercase().contains(&table_state.filter_symbol.to_lowercase())
        })
        .collect();
    
    // Sort prices
    if let Some(sort_col) = table_state.sort_column {
        filtered_prices.sort_by(|a, b| {
            let cmp = match sort_col {
                SortColumn::Symbol => a.symbol.cmp(&b.symbol),
                SortColumn::Price => a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Change24h => a.change_24h.partial_cmp(&b.change_24h).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::High24h => calculate_high_24h(a, &state.terminal.sol_candles)
                    .partial_cmp(&calculate_high_24h(b, &state.terminal.sol_candles))
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Low24h => calculate_low_24h(a, &state.terminal.sol_candles)
                    .partial_cmp(&calculate_low_24h(b, &state.terminal.sol_candles))
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Volume24h => calculate_volume_24h(a, &state.terminal.sol_candles)
                    .partial_cmp(&calculate_volume_24h(b, &state.terminal.sol_candles))
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Source => a.source.as_ref().unwrap_or(&String::new())
                    .cmp(b.source.as_ref().unwrap_or(&String::new())),
            };
            if matches!(table_state.sort_direction, SortDirection::Descending) {
                cmp.reverse()
            } else {
                cmp
            }
        });
    }

    // Check if data was recently updated
    let recently_updated = state.last_price_update_time.elapsed().as_millis() < 500;

    // Get count before move into closure
    let filtered_count = filtered_prices.len();

    // Render table
    let config = tables::TableConfig {
        num_columns: 7,
        spacing: [10.0, 5.0],
        striped: true,
        scrollable: true,
    };

    tables::render_table(
        ui,
        "live_data_table",
        config,
        &["Symbol", "Price", "Change 24h", "High 24h", "Low 24h", "Volume 24h", "Source"],
        &theme,
        |ui| {
            // Header row with sortable columns
            for (col_idx, header) in ["Symbol", "Price", "Change 24h", "High 24h", "Low 24h", "Volume 24h", "Source"].iter().enumerate() {
                let sort_col = match col_idx {
                    0 => SortColumn::Symbol,
                    1 => SortColumn::Price,
                    2 => SortColumn::Change24h,
                    3 => SortColumn::High24h,
                    4 => SortColumn::Low24h,
                    5 => SortColumn::Volume24h,
                    6 => SortColumn::Source,
                    _ => continue,
                };
                
                let is_sorted = table_state.sort_column == Some(sort_col);
                let sort_indicator = if is_sorted {
                    match table_state.sort_direction {
                        SortDirection::Ascending => " ▲",
                        SortDirection::Descending => " ▼",
                    }
                } else {
                    ""
                };
                
                let button = if is_sorted {
                    egui::Button::new(format!("{}{}", header, sort_indicator))
                        .fill(theme.selected)
                } else {
                    egui::Button::new(*header)
                };
                
                if ui.add(button).clicked() {
                    if is_sorted && matches!(table_state.sort_direction, SortDirection::Ascending) {
                        table_state.sort_direction = SortDirection::Descending;
                    } else {
                        table_state.sort_column = Some(sort_col);
                        table_state.sort_direction = SortDirection::Ascending;
                    }
                }
            }
            ui.end_row();

            // Data rows
            for price in filtered_prices {
                let high_24h = calculate_high_24h(price, &state.terminal.sol_candles);
                let low_24h = calculate_low_24h(price, &state.terminal.sol_candles);
                let volume_24h = calculate_volume_24h(price, &state.terminal.sol_candles);
                
                // Flash row if recently updated
                let row_color = if recently_updated {
                    Some(theme.selected.linear_multiply(0.1))
                } else {
                    None
                };
                
                if let Some(color) = row_color {
                    ui.painter().rect_filled(
                        ui.available_rect_before_wrap(),
                        0.0,
                        color,
                    );
                }
                
                // Symbol
                ui.label(&price.symbol);
                
                // Price with flash effect
                let price_color = if recently_updated {
                    theme.selected
                } else {
                    theme.normal
                };
                ui.colored_label(price_color, format!("${:.4}", price.price));
                
                // Change 24h
                let (change_text, change_color) = theme.format_price_change(price.change_24h);
                ui.colored_label(change_color, change_text);
                
                // High 24h
                if let Some(high) = high_24h {
                    ui.label(format!("${:.4}", high));
                } else {
                    ui.colored_label(theme.dim, "-");
                }
                
                // Low 24h
                if let Some(low) = low_24h {
                    ui.label(format!("${:.4}", low));
                } else {
                    ui.colored_label(theme.dim, "-");
                }
                
                // Volume 24h
                if let Some(vol) = volume_24h {
                    ui.label(format!("{:.2}", vol));
                } else {
                    ui.colored_label(theme.dim, "-");
                }
                
                // Source
                if let Some(source) = &price.source {
                    ui.label(source);
                } else {
                    ui.colored_label(theme.dim, "-");
                }
                
                ui.end_row();
            }
        },
    );
    
    // Save state
    ui.memory_mut(|m| {
        m.data.insert_temp(table_state_id, table_state);
    });
    
    ui.add_space(10.0);
    ui.separator();
    
    // Footer with stats
    ui.horizontal(|ui| {
        ui.label(format!("Showing {} assets", filtered_count));
    });
}

/// Calculate 24h high from candles (if available)
fn calculate_high_24h(_price: &crate::app::PriceData, candles: &[shared::dto::market::OHLC]) -> Option<f64> {
    // For now, return None if candles aren't available
    // TODO: Calculate from actual 24h candles when CandleAggregator data is available
    if candles.is_empty() {
        return None;
    }
    
    // Find highest high in last 24h
    let now = chrono::Utc::now().timestamp();
    let one_day_ago = now - 86400;
    
    Some(
        candles
            .iter()
            .filter(|c| c.timestamp >= one_day_ago)
            .map(|c| c.high)
            .fold(0.0f64, f64::max)
    )
}

/// Calculate 24h low from candles (if available)
fn calculate_low_24h(_price: &crate::app::PriceData, candles: &[shared::dto::market::OHLC]) -> Option<f64> {
    if candles.is_empty() {
        return None;
    }
    
    let now = chrono::Utc::now().timestamp();
    let one_day_ago = now - 86400;
    
    Some(
        candles
            .iter()
            .filter(|c| c.timestamp >= one_day_ago)
            .map(|c| c.low)
            .fold(f64::MAX, f64::min)
    )
}

/// Calculate 24h volume from candles (if available)
fn calculate_volume_24h(_price: &crate::app::PriceData, candles: &[shared::dto::market::OHLC]) -> Option<f64> {
    if candles.is_empty() {
        return None;
    }
    
    let now = chrono::Utc::now().timestamp();
    let one_day_ago = now - 86400;
    
    Some(
        candles
            .iter()
            .filter(|c| c.timestamp >= one_day_ago)
            .map(|c| c.volume)
            .sum()
    )
}

