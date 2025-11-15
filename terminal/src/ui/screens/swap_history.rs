//! # Swap History Screen
//!
//! Displays past swap transactions using egui widgets.

use egui;
use crate::app::AppState;
use crate::ui::theme::Theme;
use crate::ui::widgets::tables;

/// Render swap history content
pub fn render(ui: &mut egui::Ui, state: &AppState, theme: &Theme) {
    // Check which tab is active
    use crate::app::SwapTab;
    if state.terminal.swap.active_tab != SwapTab::History {
        return;
    }

    // Draw stats
    let swap_count = state.terminal.swap.swap_history.len();
    let successful = state
        .terminal
        .swap
        .swap_history
        .iter()
        .filter(|s| s.status.to_lowercase() == "success" || s.status.to_lowercase() == "confirmed")
        .count();
    let pending = state
        .terminal
        .swap
        .swap_history
        .iter()
        .filter(|s| s.status.to_lowercase() == "pending")
        .count();
    let failed = swap_count - successful - pending;

    tables::render_stats_summary(ui, &[
        ("Total", swap_count),
        ("Success", successful),
        ("Pending", pending),
        ("Failed", failed),
    ]);

    ui.separator();
    ui.add_space(5.0);

    // Draw transaction table
    if state.terminal.swap.swap_history.is_empty() {
        tables::render_empty_state(
            ui,
            "No swap history available",
            Some("Execute your first swap to see it here!"),
            theme,
        );
        return;
    }

    let config = tables::TableConfig {
        num_columns: 7,
        spacing: [10.0, 5.0],
        striped: true,
        scrollable: false,
    };

    tables::render_table(
        ui,
        "swap_history",
        config,
        &["Time", "From", "To", "Input Amt", "Output Amt", "Status", "Signature"],
        theme,
        |ui| {
            // Rows
            for swap in &state.terminal.swap.swap_history {
                let time_ago = format!("{}s ago", swap.timestamp % 3600); // Simplified
                let status_color = match swap.status.to_lowercase().as_str() {
                    "success" | "confirmed" => theme.success,
                    "pending" => theme.warning,
                    "failed" | "error" => theme.error,
                    _ => theme.normal,
                };

                let sig_short = if swap.signature.len() > 12 {
                    format!("{}...{}", &swap.signature[..6], &swap.signature[swap.signature.len()-6..])
                } else {
                    swap.signature.clone()
                };

                ui.label(time_ago);
                ui.label(&swap.input_symbol);
                ui.label(&swap.output_symbol);
                ui.label(format!("{:.4}", swap.input_amount));
                ui.colored_label(theme.success, format!("{:.4}", swap.output_amount));
                ui.colored_label(status_color, &swap.status);
                ui.label(sig_short);
                ui.end_row();
            }
        },
    );
}
