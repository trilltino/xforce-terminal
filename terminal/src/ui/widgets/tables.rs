//! # Table Components
//!
//! Reusable table/grid components for displaying data consistently

use egui;
use crate::ui::theme::Theme;

/// Configuration for table styling
pub struct TableConfig {
    pub num_columns: usize,
    pub spacing: [f32; 2],
    pub striped: bool,
    pub scrollable: bool,
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            num_columns: 4,
            spacing: [10.0, 5.0],
            striped: true,
            scrollable: false,
        }
    }
}

/// Render a data table with headers and rows
pub fn render_table<F>(
    ui: &mut egui::Ui,
    id: &str,
    config: TableConfig,
    headers: &[&str],
    theme: &Theme,
    render_rows: F,
) where
    F: FnOnce(&mut egui::Ui),
{
    let table_render = |ui: &mut egui::Ui| {
        egui::Grid::new(id)
            .num_columns(config.num_columns)
            .spacing(config.spacing)
            .striped(config.striped)
            .show(ui, |ui| {
                // Header row
                for header in headers {
                    ui.colored_label(theme.selected, *header);
                }
                ui.end_row();
                
                // Rows (rendered by callback)
                render_rows(ui);
            });
    };
    
    if config.scrollable {
        egui::ScrollArea::vertical().show(ui, table_render);
    } else {
        table_render(ui);
    }
}

/// Render an empty state message
pub fn render_empty_state(
    ui: &mut egui::Ui,
    primary_text: &str,
    secondary_text: Option<&str>,
    theme: &Theme,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.colored_label(theme.dim, primary_text);
        if let Some(secondary) = secondary_text {
            ui.add_space(10.0);
            ui.colored_label(theme.dim, secondary);
        }
    });
}

/// Render stats summary (e.g., "Total: X | Success: Y | Pending: Z")
pub fn render_stats_summary(ui: &mut egui::Ui, stats: &[(&str, usize)]) {
    ui.horizontal(|ui| {
        let mut parts = Vec::new();
        for (label, count) in stats {
            parts.push(format!("{}: {}", label, count));
        }
        ui.label(parts.join("  |  "));
    });
}

