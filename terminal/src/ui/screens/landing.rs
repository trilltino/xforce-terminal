//! # Landing Screen
//!
//! Welcome/splash screen with branding. User presses Enter to continue.

use egui;
use crate::app::{AppState, AppLike};
use crate::ui::widgets::branding;

/// Render landing screen (welcome/splash)
pub fn render(ui: &mut egui::Ui, _state: &AppState, app: &mut impl AppLike, cube: &mut crate::ui::cube::RotatingCube) {
    // Split screen: Branding left, Empty right
    ui.columns(2, |columns| {
        // Left column - Branding
        columns[0].vertical_centered(|ui| {
            ui.add_space(100.0);
            branding::render_branding_section(ui, "<Enter> to begin");
            branding::render_cube_section(ui, cube);
        });

        // Right column - Empty
        columns[1].vertical_centered(|_ui| {
            // Right side is intentionally empty - user just presses Enter to continue
        });
    });

    // Version info and legal disclaimer footer
    branding::render_footer(ui);

    // Check for Enter key press to continue
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        app.handle_screen_change(crate::app::Screen::Auth);
    }
}

