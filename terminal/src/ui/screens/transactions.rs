//! # Transactions Screen
//!
//! Display transaction history using egui widgets.

use egui;
use crate::app::AppState;
use crate::ui::theme::Theme;
use crate::ui::widgets::tables;

/// Render transactions screen
pub fn render(ui: &mut egui::Ui, state: &AppState) {
    let theme = Theme::default();

    if state.transactions.is_empty() {
        tables::render_empty_state(
            ui,
            "No Transactions Yet",
            Some("Execute swaps to see transaction history"),
            &theme,
        );
    } else {
        render_transactions_table(ui, state, &theme);
    }
}

/// Render transactions table
fn render_transactions_table(ui: &mut egui::Ui, state: &AppState, theme: &Theme) {
    ui.heading("Transaction History");
    ui.add_space(10.0);

    let config = tables::TableConfig {
        num_columns: 5,
        spacing: [10.0, 5.0],
        striped: true,
        scrollable: false,
    };

    tables::render_table(
        ui,
        "transactions",
        config,
        &["Time", "Type", "Amount", "Status", "Signature"],
        theme,
        |ui| {
            // Rows
            for tx in &state.transactions {
                // Format timestamp
                let time = chrono::DateTime::from_timestamp(tx.timestamp, 0)
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                // Status color
                let status_color = match tx.status.as_str() {
                    "confirmed" => theme.success,
                    "pending" => theme.warning,
                    "failed" => theme.error,
                    _ => theme.dim,
                };

                ui.label(time);
                ui.label(&tx.tx_type);
                ui.label(&tx.amount);
                ui.colored_label(status_color, &tx.status);
                ui.label(&tx.signature[..8.min(tx.signature.len())]); // First 8 chars
                ui.end_row();
            }
        },
    );
}
