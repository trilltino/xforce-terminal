//! # Navigation Handlers
//!
//! Handlers for screen navigation and tab changes.

use crate::app::state::{AppState, Screen, SwapTab};
use parking_lot::RwLock;
use std::sync::Arc;

/// Handle screen change with authentication guard
///
/// Internal handler function - use [`crate::app::App::handle_screen_change`] instead.
pub(crate) fn handle_screen_change(state: Arc<RwLock<AppState>>, screen: Screen) {
    let mut state = state.write();
    
    // Check if screen requires authentication
    if AppState::requires_auth(screen) && !state.is_authenticated() {
        // Redirect to Auth screen if not authenticated
        tracing::info!("Access denied: {} requires authentication, redirecting to Auth", screen.title());
        state.current_screen = Screen::Auth;
    } else {
        state.current_screen = screen;
    }
}

/// Handle swap tab change
///
/// Internal handler function - use [`crate::app::App::handle_swap_tab_change`] instead.
pub(crate) fn handle_swap_tab_change(state: Arc<RwLock<AppState>>, tab: SwapTab) {
    let mut state = state.write();
    state.terminal.swap.active_tab = tab;
}

/// Navigate to next screen in Tab order (skips protected screens if not authenticated)
///
/// Internal handler function - use [`crate::app::App::next_screen`] instead.
pub(crate) fn next_screen(state: Arc<RwLock<AppState>>) {
    let mut state = match state.try_write() {
        Some(guard) => guard,
        None => {
            tracing::warn!("Skipped screen navigation - state locked");
            return;
        }
    };

    // Get screens excluding Messaging and Settings (only accessible via nav bar)
    let all_screens = Screen::all();
    let screens: Vec<Screen> = all_screens
        .iter()
        .copied()
        .filter(|&s| s != Screen::Messaging && s != Screen::Settings)
        .collect();
    
    let current_idx = screens
        .iter()
        .position(|&s| s == state.current_screen)
        .unwrap_or(0);
    
    let is_authenticated = state.is_authenticated();
    
    // Find next screen, skipping protected screens if not authenticated
    let mut next_idx = (current_idx + 1) % screens.len();
    let mut attempts = 0;
    while attempts < screens.len() {
        let screen = screens[next_idx];
        // Allow if it's a public screen or user is authenticated
        if !AppState::requires_auth(screen) || is_authenticated {
            state.current_screen = screen;
            return;
        }
        next_idx = (next_idx + 1) % screens.len();
        attempts += 1;
    }
    
    // Fallback: if all screens are protected and not authenticated, go to Auth
    if !is_authenticated {
        state.current_screen = Screen::Auth;
    }
}

/// Navigate to previous screen in Tab order (skips protected screens if not authenticated)
///
/// Internal handler function - use [`crate::app::App::previous_screen`] instead.
pub(crate) fn previous_screen(state: Arc<RwLock<AppState>>) {
    let mut state = match state.try_write() {
        Some(guard) => guard,
        None => {
            tracing::warn!("Skipped screen navigation - state locked");
            return;
        }
    };

    // Get screens excluding Messaging and Settings (only accessible via nav bar)
    let all_screens = Screen::all();
    let screens: Vec<Screen> = all_screens
        .iter()
        .copied()
        .filter(|&s| s != Screen::Messaging && s != Screen::Settings)
        .collect();
    
    let current_idx = screens
        .iter()
        .position(|&s| s == state.current_screen)
        .unwrap_or(0);
    
    let is_authenticated = state.is_authenticated();
    
    // Find previous screen, skipping protected screens if not authenticated
    let mut prev_idx = if current_idx == 0 {
        screens.len() - 1
    } else {
        current_idx - 1
    };
    let mut attempts = 0;
    while attempts < screens.len() {
        let screen = screens[prev_idx];
        // Allow if it's a public screen or user is authenticated
        if !AppState::requires_auth(screen) || is_authenticated {
            state.current_screen = screen;
            return;
        }
        prev_idx = if prev_idx == 0 {
            screens.len() - 1
        } else {
            prev_idx - 1
        };
        attempts += 1;
    }
    
    // Fallback: if all screens are protected and not authenticated, go to Auth
    if !is_authenticated {
        state.current_screen = Screen::Auth;
    }
}

