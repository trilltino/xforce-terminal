//! # Window App Wrapper
//!
//! Provides an App-like interface for secondary windows that allows screen renderers
//! to work within deferred viewports. This wrapper delegates to the same handlers
//! as the main App, but updates window-specific screen state.

use std::sync::Arc;
use parking_lot::RwLock;
use async_channel::Sender;
use crate::app::{
    AppState, Screen, SwapTab, TokenPickerTarget, TokenInfo,
    events::AppEvent,
    window_manager::{WindowManager, WindowId},
};

/// WindowApp wrapper for secondary windows that provides App-like interface.
///
/// This allows screen renderers to work in deferred viewports by providing
/// the same API as the main App struct, but with window-specific screen management.
pub struct WindowApp {
    pub state: Arc<RwLock<AppState>>,
    pub window_manager: Arc<RwLock<WindowManager>>,
    event_tx: Sender<AppEvent>,
    window_id: WindowId,
}

impl WindowApp {
    /// Create a new WindowApp for a secondary window.
    pub fn new(
        state: Arc<RwLock<AppState>>,
        window_manager: Arc<RwLock<WindowManager>>,
        event_tx: Sender<AppEvent>,
        window_id: WindowId,
    ) -> Self {
        Self {
            state,
            window_manager,
            event_tx,
            window_id,
        }
    }

    /// Handle screen change - updates the window's screen instead of global state.
    pub fn handle_screen_change(&mut self, screen: Screen) {
        use crate::app::handlers::navigation;
        
        // Update window's screen
        let mut window_manager = self.window_manager.write();
        if let Some(window) = window_manager.get_window_mut(self.window_id) {
            window.screen = screen;
            window.title = format!("Terminal - {}", screen.title());
        }
        
        // Also update global state for consistency (some handlers might check it)
        navigation::handle_screen_change(self.state.clone(), screen);
    }
}

// Implement App methods for WindowApp by delegating to handlers
impl WindowApp {
    pub fn handle_login_click(&mut self, username: String, password: String) {
        use crate::app::handlers::auth;
        auth::handle_login_click(self.state.clone(), self.event_tx.clone(), username, password);
    }

    pub fn handle_signup_click(&mut self, username: String, email: String, password: String, confirm_password: String) {
        use crate::app::handlers::auth;
        auth::handle_signup_click(self.state.clone(), self.event_tx.clone(), username, email, password, confirm_password);
    }

    pub fn handle_switch_to_login(&mut self) {
        use crate::app::handlers::auth;
        auth::handle_switch_to_login(self.state.clone());
    }

    pub fn handle_switch_to_signup(&mut self) {
        use crate::app::handlers::auth;
        auth::handle_switch_to_signup(self.state.clone());
    }

    pub fn handle_swap_tab_change(&mut self, tab: SwapTab) {
        use crate::app::handlers::navigation;
        navigation::handle_swap_tab_change(self.state.clone(), tab);
    }

    pub fn handle_swap_execute_click(&mut self) {
        use crate::app::tasks::swap;
        swap::execute_swap(self.state.clone(), self.event_tx.clone());
    }

    pub fn handle_token_select(&mut self, token: TokenInfo, target: TokenPickerTarget) {
        use crate::app::handlers::swap;
        swap::handle_token_select(self.state.clone(), self.event_tx.clone(), token, target);
    }

    pub fn handle_wallet_connect_click(&mut self) {
        use crate::app::handlers::wallet;
        wallet::handle_wallet_connect_click(self.state.clone(), self.event_tx.clone());
    }

    pub fn handle_wallet_generate_click(&mut self) {
        use crate::app::handlers::wallet;
        wallet::handle_wallet_generate_click(self.state.clone(), self.event_tx.clone());
    }

    pub fn handle_wallet_disconnect_click(&mut self) {
        use crate::app::handlers::wallet;
        wallet::handle_wallet_disconnect_click(self.state.clone());
    }

    pub fn trigger_quote_fetch(&mut self) {
        use crate::app::tasks::swap;
        swap::trigger_quote_fetch(self.state.clone(), self.event_tx.clone());
    }

    pub fn fetch_token_list(&mut self) {
        use crate::app::tasks::market;
        market::fetch_token_list(self.state.clone(), self.event_tx.clone());
    }

    pub fn handle_theme_color_change(&mut self, config: crate::ui::theme::ThemeConfig) {
        use crate::app::handlers::settings;
        settings::handle_theme_color_change(self.state.clone(), config);
    }

    pub fn handle_settings_save(&mut self) {
        use crate::app::handlers::settings;
        settings::handle_settings_save(self.state.clone());
    }

    pub fn handle_settings_reset(&mut self) {
        use crate::app::handlers::settings;
        settings::handle_settings_reset(self.state.clone());
    }

    pub fn handle_settings_apply(&mut self) {
        use crate::app::handlers::settings;
        settings::handle_settings_apply(self.state.clone());
    }

    pub fn fetch_candles(&mut self, symbol: &str, timeframe: shared::dto::market::Timeframe) {
        use crate::app::tasks;
        tasks::market::fetch_candles(self.state.clone(), self.event_tx.clone(), symbol.to_string(), timeframe);
    }

    pub fn open_token_picker_internal(&self, _state: &mut AppState, target: TokenPickerTarget) {
        use crate::app::handlers::swap;
        swap::open_token_picker(self.state.clone(), target);
    }

    pub fn set_max_amount(&mut self) {
        use crate::app::handlers::swap;
        swap::set_max_amount(self.state.clone());
    }
}

impl crate::app::app_trait::AppLike for WindowApp {
    fn state(&self) -> &Arc<parking_lot::RwLock<AppState>> {
        &self.state
    }
    
    fn window_manager(&self) -> &Arc<parking_lot::RwLock<WindowManager>> {
        &self.window_manager
    }
    
    fn handle_login_click(&mut self, username: String, password: String) {
        self.handle_login_click(username, password);
    }
    
    fn handle_signup_click(&mut self, username: String, email: String, password: String, confirm_password: String) {
        self.handle_signup_click(username, email, password, confirm_password);
    }
    
    fn handle_switch_to_login(&mut self) {
        self.handle_switch_to_login();
    }
    
    fn handle_switch_to_signup(&mut self) {
        self.handle_switch_to_signup();
    }
    
    fn handle_screen_change(&mut self, screen: Screen) {
        self.handle_screen_change(screen);
    }
    
    fn handle_swap_tab_change(&mut self, tab: SwapTab) {
        self.handle_swap_tab_change(tab);
    }
    
    fn next_screen(&mut self) {
        use crate::app::handlers::navigation;
        navigation::next_screen(self.state.clone());
    }
    
    fn previous_screen(&mut self) {
        use crate::app::handlers::navigation;
        navigation::previous_screen(self.state.clone());
    }
    
    fn handle_swap_execute_click(&mut self) {
        self.handle_swap_execute_click();
    }
    
    fn handle_token_select(&mut self, token: TokenInfo, target: TokenPickerTarget) {
        self.handle_token_select(token, target);
    }
    
    fn handle_wallet_connect_click(&mut self) {
        self.handle_wallet_connect_click();
    }
    
    fn handle_wallet_generate_click(&mut self) {
        self.handle_wallet_generate_click();
    }
    
    fn handle_wallet_disconnect_click(&mut self) {
        self.handle_wallet_disconnect_click();
    }
    
    fn trigger_quote_fetch(&mut self) {
        self.trigger_quote_fetch();
    }
    
    fn fetch_token_list(&mut self) {
        self.fetch_token_list();
    }
    
    fn handle_theme_color_change(&mut self, config: crate::ui::theme::ThemeConfig) {
        self.handle_theme_color_change(config);
    }
    
    fn handle_settings_save(&mut self) {
        self.handle_settings_save();
    }
    
    fn handle_settings_reset(&mut self) {
        self.handle_settings_reset();
    }
    
    fn handle_settings_apply(&mut self) {
        self.handle_settings_apply();
    }
    
    fn fetch_candles(&mut self, symbol: &str, timeframe: shared::dto::market::Timeframe) {
        self.fetch_candles(symbol, timeframe);
    }
    
    fn open_token_picker_internal(&self, state: &mut AppState, target: TokenPickerTarget) {
        self.open_token_picker_internal(state, target);
    }
    
    fn set_max_amount(&mut self) {
        self.set_max_amount();
    }
}

