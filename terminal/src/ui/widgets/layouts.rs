//! # Layout Components
//!
//! Reusable layout patterns for consistent screen organization

use egui;

/// Render a two-column split layout
pub fn render_split_layout<F1, F2>(ui: &mut egui::Ui, left_content: F1, right_content: F2)
where
    F1: FnOnce(&mut egui::Ui),
    F2: FnOnce(&mut egui::Ui),
{
    ui.columns(2, |columns| {
        left_content(&mut columns[0]);
        right_content(&mut columns[1]);
    });
}

/// Render a three-column split layout
pub fn render_three_column_layout<F1, F2, F3>(ui: &mut egui::Ui, left: F1, center: F2, right: F3)
where
    F1: FnOnce(&mut egui::Ui),
    F2: FnOnce(&mut egui::Ui),
    F3: FnOnce(&mut egui::Ui),
{
    ui.columns(3, |columns| {
        left(&mut columns[0]);
        center(&mut columns[1]);
        right(&mut columns[2]);
    });
}

/// Render vertically centered content
pub fn render_centered<F>(ui: &mut egui::Ui, content: F)
where
    F: FnOnce(&mut egui::Ui),
{
    ui.vertical_centered(|ui| {
        ui.add_space(80.0);
        content(ui);
    });
}

/// Render a grouped panel with optional heading
pub fn render_panel<F>(ui: &mut egui::Ui, heading: Option<&str>, content: F)
where
    F: FnOnce(&mut egui::Ui),
{
    ui.group(|ui| {
        if let Some(heading_text) = heading {
            ui.heading(heading_text);
            ui.add_space(10.0);
        }
        content(ui);
    });
}

