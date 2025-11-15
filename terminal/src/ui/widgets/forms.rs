//! # Form Components
//!
//! Reusable form elements for consistent UI across screens

use egui;
use crate::ui::theme::Theme;

/// Render a styled text input field
pub fn render_text_input(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    hint: &str,
    password: bool,
    size: [f32; 2],
) -> egui::Response {
    let label_font = crate::ui::fonts::FontConfig::get_avenir_font(ui.ctx(), 14.0);
    ui.label(egui::RichText::new(label).font(label_font));
    let response = if password {
        ui.add_sized(
            size,
            egui::TextEdit::singleline(value)
                .password(true)
                .hint_text(hint)
                .font(crate::ui::fonts::FontConfig::get_avenir_font(ui.ctx(), 14.0))
        )
    } else {
        ui.add_sized(
            size,
            egui::TextEdit::singleline(value)
                .hint_text(hint)
                .font(crate::ui::fonts::FontConfig::get_avenir_font(ui.ctx(), 14.0))
        )
    };
    response
}

/// Render a styled button with optional icon
pub fn render_button(
    ui: &mut egui::Ui,
    text: &str,
    icon: Option<&str>,
    _theme: &Theme,
    fill_color: Option<egui::Color32>,
    min_size: Option<egui::Vec2>,
) -> egui::Response {
    let button_text = if let Some(icon) = icon {
        format!("{} {}", icon, text)
    } else {
        text.to_string()
    };
    
    let button_font = crate::ui::fonts::FontConfig::get_avenir_font(ui.ctx(), 16.0);
    let mut button = egui::Button::new(
        egui::RichText::new(button_text).font(button_font)
    );
    
    if let Some(color) = fill_color {
        button = button.fill(color);
    }
    
    if let Some(size) = min_size {
        button = button.min_size(size);
    }
    
    ui.add(button)
}

/// Render a form heading
pub fn render_form_heading(ui: &mut egui::Ui, text: &str, theme: &Theme) {
    let heading_font = crate::ui::fonts::FontConfig::get_avenir_font(ui.ctx(), 24.0);
    let heading = egui::RichText::new(text)
        .font(heading_font)
        .strong()
        .color(theme.selected);
    ui.label(heading);
    ui.add_space(20.0);
}

/// Render an error message
pub fn render_error(ui: &mut egui::Ui, error: &str, theme: &Theme) {
    let error_font = crate::ui::fonts::FontConfig::get_avenir_font(ui.ctx(), 14.0);
    ui.label(egui::RichText::new(error).font(error_font).color(theme.error));
    ui.add_space(10.0);
}

/// Render a help/hint text
pub fn render_hint(ui: &mut egui::Ui, hint: &str, theme: &Theme) {
    let hint_font = crate::ui::fonts::FontConfig::get_avenir_font(ui.ctx(), 14.0);
    ui.label(egui::RichText::new(hint).font(hint_font).color(theme.dim));
}

