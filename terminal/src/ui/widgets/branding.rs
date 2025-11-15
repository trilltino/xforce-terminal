//! # Branding Components
//!
//! Reusable branding elements used across screens (landing, auth, etc.)

use egui::{self, Color32};

/// Render the XFTerminal title with red XF accent
pub fn render_title(ui: &mut egui::Ui, size: f32) {
    let title_font = crate::ui::fonts::FontConfig::get_avenir_font(ui.ctx(), size);
    let red_color = Color32::from_rgb(204, 0, 0);
    
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("X").font(title_font.clone()).color(red_color).strong().italics());
        ui.label(egui::RichText::new("F").font(title_font.clone()).color(red_color).strong().italics());
        ui.label(egui::RichText::new("Terminal").font(title_font.clone()).color(Color32::WHITE).strong());
    });
}

/// Render the subtitle "Trade anywhere"
pub fn render_subtitle(ui: &mut egui::Ui, size: f32) {
    let subtitle_font = crate::ui::fonts::FontConfig::get_avenir_font(ui.ctx(), size);
    ui.label(egui::RichText::new("Trade anywhere").font(subtitle_font).color(Color32::WHITE).strong());
}

/// Render the prompt text (e.g., "<Enter> to begin")
pub fn render_prompt(ui: &mut egui::Ui, text: &str, size: f32) {
    let prompt_font = crate::ui::fonts::FontConfig::get_avenir_font(ui.ctx(), size);
    ui.label(egui::RichText::new(text).font(prompt_font).color(egui::Color32::LIGHT_GRAY));
}

/// Render the complete branding section (title, subtitle, prompt)
pub fn render_branding_section(ui: &mut egui::Ui, prompt_text: &str) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        render_title(ui, 72.0);
        ui.add_space(5.0);
        render_subtitle(ui, 56.0);
        ui.add_space(8.0);
        render_prompt(ui, prompt_text, 18.0);
    });
}

/// Render version info and legal disclaimer footer
pub fn render_footer(ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.add_space(10.0);
        
        // Disclaimer text in grey transparent text
        let disclaimer_text = "The XFORCETERMINAL service and data products are owned and distributed by XFSolutions LTD. XFSolutions provides global marketing and operational support for these products. XFSolutions believe the information herein came from reliable sources, but do not guarantee their accuracy. NO information here constitutes a solicitation of the Purchase of any Cryptocurrency or Asset.";
        
        ui.label(
            egui::RichText::new(disclaimer_text)
                .size(12.0)
                .color(egui::Color32::from_rgba_unmultiplied(128, 128, 128, 180))
        );
        
        // Version info above disclaimer
        ui.add_space(8.0);
        let footer_font = crate::ui::fonts::FontConfig::get_avenir_font(ui.ctx(), 12.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("X").font(footer_font.clone()).color(egui::Color32::from_rgba_unmultiplied(128, 128, 128, 180)).italics());
            ui.label(egui::RichText::new("F").font(footer_font.clone()).color(egui::Color32::from_rgba_unmultiplied(128, 128, 128, 180)).italics());
            ui.label(egui::RichText::new("Terminal.com | Version 12 Nov 25")
                .font(footer_font.clone())
                .color(egui::Color32::from_rgba_unmultiplied(128, 128, 128, 180))
            );
        });
    });
}

/// Render the rotating cube with proper spacing
pub fn render_cube_section(ui: &mut egui::Ui, cube: &mut crate::ui::cube::RotatingCube) {
    ui.add_space(30.0);
    
    ui.allocate_ui_with_layout(
        egui::Vec2::new(cube.size + 50.0, cube.size + 50.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            let center = ui.available_rect_before_wrap().center();
            cube.render(ui.painter(), center, cube.size);
        }
    );
}

