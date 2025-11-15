//! # Window Controls Widget
//!
//! UI controls for managing multiple windows (Bloomberg-style).
//! Provides "New Window" button and window management UI.

use egui;
use crate::app::{App, Screen};

/// Create a new window with the specified screen
pub fn create_new_window(app: &mut App, screen: Screen) {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let viewport_id = egui::ViewportId::from_hash_of(format!("window_{}", timestamp));
    
    let window_id = {
        let mut window_manager = app.window_manager.write();
        window_manager.create_window(viewport_id, screen, Some(format!("Terminal - {}", screen.title())))
    };
    
    tracing::info!("Created new window: {:?} with screen: {:?}", window_id, screen);
}

/// Render window list/management panel (optional)
pub fn render_window_list(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Open Windows");
    ui.separator();
    
    let windows: Vec<_> = {
        let window_manager = app.window_manager.read();
        window_manager.all_windows().into_iter().cloned().collect()
    };
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        for window in windows {
            ui.horizontal(|ui| {
                ui.label(format!("Window #{}: {}", window.id.0, window.title));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() && window.id.0 != 0 {
                        // Don't allow closing root window
                        let mut window_manager = app.window_manager.write();
                        window_manager.remove_window(window.viewport_id);
                    }
                });
            });
            ui.separator();
        }
    });
}

