//! # App Trait
//!
//! Trait that both App and WindowApp implement, allowing screen renderers
//! to work with either type. This enables full rendering in secondary windows.

use std::sync::Arc;
use parking_lot::RwLock;
use crate::app::{
    AppState, Screen, SwapTab, TokenPickerTarget, TokenInfo,
    window_manager::WindowManager,
};

/// Trait for application-like types that screen renderers can use.
/// 
/// Both `App` and `WindowApp` implement this trait, allowing screen renderers
/// to work seamlessly with either the main app or secondary window apps.
pub trait AppLike {
    /// Get access to the application state.
    fn state(&self) -> &Arc<RwLock<AppState>>;
    
    /// Get access to the window manager.
    fn window_manager(&self) -> &Arc<RwLock<WindowManager>>;
    
    // Auth methods
    fn handle_login_click(&mut self, username: String, password: String);
    fn handle_signup_click(&mut self, username: String, email: String, password: String, confirm_password: String);
    fn handle_switch_to_login(&mut self);
    fn handle_switch_to_signup(&mut self);
    
    // Navigation methods
    fn handle_screen_change(&mut self, screen: Screen);
    fn handle_swap_tab_change(&mut self, tab: SwapTab);
    fn next_screen(&mut self);
    fn previous_screen(&mut self);
    
    // Swap methods
    fn handle_swap_execute_click(&mut self);
    fn handle_token_select(&mut self, token: TokenInfo, target: TokenPickerTarget);
    fn trigger_quote_fetch(&mut self);
    fn fetch_token_list(&mut self);
    fn set_max_amount(&mut self);
    fn open_token_picker_internal(&self, state: &mut AppState, target: TokenPickerTarget);
    fn fetch_candles(&mut self, symbol: &str, timeframe: shared::dto::market::Timeframe);
    
    // Wallet methods
    fn handle_wallet_connect_click(&mut self);
    fn handle_wallet_generate_click(&mut self);
    fn handle_wallet_disconnect_click(&mut self);
    
    // Settings methods
    fn handle_theme_color_change(&mut self, config: crate::ui::theme::ThemeConfig);
    fn handle_settings_save(&mut self);
    fn handle_settings_reset(&mut self);
    fn handle_settings_apply(&mut self);
}

