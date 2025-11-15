//! # Settings Handlers
//!
//! Handlers for settings-related actions including theme customization and persistence.

use crate::ui::theme::ThemeConfig;
use parking_lot::RwLock;
use std::sync::Arc;
use crate::app::AppState;

/// Get default config file path
pub fn get_config_path() -> std::path::PathBuf {
    std::path::PathBuf::from("./xterminal-config.json")
}

/// Load settings from file
pub fn load_settings() -> ThemeConfig {
    let path = get_config_path();
    match ThemeConfig::load_from_file(&path) {
        Ok(config) => {
            tracing::info!("Loaded theme configuration from {:?}", path);
            config
        }
        Err(e) => {
            tracing::warn!("Failed to load theme config from {:?}: {}. Using defaults.", path, e);
            ThemeConfig::default()
        }
    }
}

/// Save settings to file
pub fn save_settings(config: &ThemeConfig) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_config_path();
    config.save_to_file(&path)?;
    tracing::info!("Saved theme configuration to {:?}", path);
    Ok(())
}

/// Handle theme color change
pub fn handle_theme_color_change(state: Arc<RwLock<AppState>>, config: ThemeConfig) {
    let mut app_state = state.write();
    app_state.settings.theme_config = config;
    app_state.settings.unsaved_changes = true;
}

/// Handle settings save
pub fn handle_settings_save(state: Arc<RwLock<AppState>>) {
    let app_state = state.write();
    let config = app_state.settings.theme_config.clone();
    drop(app_state);

    match save_settings(&config) {
        Ok(_) => {
            let mut app_state = state.write();
            app_state.settings.unsaved_changes = false;
            tracing::info!("Settings saved successfully");
        }
        Err(e) => {
            tracing::error!("Failed to save settings: {}", e);
        }
    }
}

/// Handle settings reset to defaults
pub fn handle_settings_reset(state: Arc<RwLock<AppState>>) {
    let default_config = ThemeConfig::default();
    
    let mut app_state = state.write();
    app_state.settings.theme_config = default_config;
    app_state.settings.unsaved_changes = true;
}

/// Handle settings apply (apply without saving)
pub fn handle_settings_apply(_state: Arc<RwLock<AppState>>) {
    // This is a no-op at the handler level - theme is applied in UI
    // The state already has the config, UI will read it and apply
}

