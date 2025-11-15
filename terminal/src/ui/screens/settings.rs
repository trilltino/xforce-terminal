//! # Settings Screen
//!
//! UI customization screen with color pickers for theme configuration.

use egui;
use crate::app::AppState;
use crate::ui::theme::ThemeConfig;
use crate::ui::widgets::icons::{Icons, material, size};

/// Render settings screen
pub fn render(ui: &mut egui::Ui, state: &AppState, app: &mut impl crate::app::AppLike) {
    // Apply theme immediately when settings screen is shown
    let ctx = ui.ctx();
    crate::ui::theme::Theme::apply_custom_theme(ctx, &state.settings.theme_config);
    use crate::ui::theme::Theme;
    let theme = Theme::default();

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(Icons::icon_red(material::SETTINGS, size::MEDIUM));
            ui.heading("Settings");
        });
        ui.add_space(10.0);

        // Theme Colors Section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(Icons::icon_dim(material::PALETTE, size::SMALL));
                ui.heading("Theme Colors");
            });
            ui.add_space(10.0);

            render_color_pickers(ui, &state.settings.theme_config, app, &theme);
        });

        ui.add_space(20.0);

        // Actions Section
        render_actions(ui, state, app, &theme);
    });
}

/// Render color pickers for all theme colors
fn render_color_pickers(
    ui: &mut egui::Ui,
    config: &ThemeConfig,
    app: &mut impl crate::app::AppLike,
    _theme: &crate::ui::theme::Theme,
) {
    // Primary Colors
    ui.collapsing("Primary Colors", |ui| {
        ui.horizontal(|ui| {
            ui.label("Background:");
            let mut color = egui::Color32::from_rgb(config.background[0], config.background[1], config.background[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.background = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config.clone());
                // Apply immediately for preview
                crate::ui::theme::Theme::apply_custom_theme(ui.ctx(), &new_config);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Text:");
            let mut color = egui::Color32::from_rgb(config.text[0], config.text[1], config.text[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.text = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Primary Red:");
            let mut color = egui::Color32::from_rgb(config.red_primary[0], config.red_primary[1], config.red_primary[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.red_primary = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Dark Red:");
            let mut color = egui::Color32::from_rgb(config.red_dark[0], config.red_dark[1], config.red_dark[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.red_dark = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Red Highlight:");
            let mut color = egui::Color32::from_rgb(config.red_highlight[0], config.red_highlight[1], config.red_highlight[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.red_highlight = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });
    });

    // Border Colors
    ui.collapsing("Border Colors", |ui| {
        ui.horizontal(|ui| {
            ui.label("Border Dark:");
            let mut color = egui::Color32::from_rgb(config.border_dark[0], config.border_dark[1], config.border_dark[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.border_dark = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Border Red Tint:");
            let mut color = egui::Color32::from_rgb(config.border_red_tint[0], config.border_red_tint[1], config.border_red_tint[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.border_red_tint = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });
    });

    // Status Colors
    ui.collapsing("Status Colors", |ui| {
        ui.horizontal(|ui| {
            ui.label("Success Green:");
            let mut color = egui::Color32::from_rgb(config.green_success[0], config.green_success[1], config.green_success[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.green_success = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Error Red:");
            let mut color = egui::Color32::from_rgb(config.red_error[0], config.red_error[1], config.red_error[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.red_error = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Warning Yellow:");
            let mut color = egui::Color32::from_rgb(config.yellow_warning[0], config.yellow_warning[1], config.yellow_warning[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.yellow_warning = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Info Blue:");
            let mut color = egui::Color32::from_rgb(config.blue_info[0], config.blue_info[1], config.blue_info[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.blue_info = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });
    });

    // Gray Colors
    ui.collapsing("Gray Colors", |ui| {
        ui.horizontal(|ui| {
            ui.label("Gray Inactive:");
            let mut color = egui::Color32::from_rgb(config.gray_inactive[0], config.gray_inactive[1], config.gray_inactive[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.gray_inactive = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Gray Secondary:");
            let mut color = egui::Color32::from_rgb(config.gray_secondary[0], config.gray_secondary[1], config.gray_secondary[2]);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_config = config.clone();
                new_config.gray_secondary = [color.r(), color.g(), color.b()];
                app.handle_theme_color_change(new_config);
            }
        });
    });
}

/// Render actions section (Save, Reset, Apply)
fn render_actions(
    ui: &mut egui::Ui,
    state: &AppState,
    app: &mut impl crate::app::AppLike,
    theme: &crate::ui::theme::Theme,
) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(Icons::icon_dim(material::SETTINGS, size::SMALL));
            ui.heading("Actions");
        });
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button(format!("{} Save Settings", material::SAVE)).clicked() {
                app.handle_settings_save();
                // Read updated state and apply theme
                let updated_state = app.state().read();
                crate::ui::theme::Theme::apply_custom_theme(ui.ctx(), &updated_state.settings.theme_config);
            }

            if ui.button(format!("{} Reset to Defaults", material::REFRESH)).clicked() {
                app.handle_settings_reset();
                // Read updated state and apply theme
                let updated_state = app.state().read();
                crate::ui::theme::Theme::apply_custom_theme(ui.ctx(), &updated_state.settings.theme_config);
            }

            if ui.button(format!("{} Apply Changes", material::CHECK)).clicked() {
                app.handle_settings_apply();
                // Apply current theme config
                crate::ui::theme::Theme::apply_custom_theme(ui.ctx(), &state.settings.theme_config);
            }
        });

        ui.add_space(10.0);

        // Status indicator
        if state.settings.unsaved_changes {
            ui.horizontal(|ui| {
                ui.label(Icons::icon_warning(material::WARNING, size::SMALL));
                ui.colored_label(theme.warning, "You have unsaved changes");
            });
        } else {
            ui.horizontal(|ui| {
                ui.label(Icons::icon_success(material::CHECK, size::SMALL));
                ui.colored_label(theme.success, "All changes saved");
            });
        }
    });
}

