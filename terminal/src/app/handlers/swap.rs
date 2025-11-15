//! # Swap Handlers
//!
//! Handlers for swap-related actions including token selection and swap execution.

use crate::app::state::{AppState, TokenInfo, TokenPickerTarget};
use crate::app::events::AppEvent;
use async_channel::Sender;
use parking_lot::RwLock;
use std::sync::Arc;

/// Open token picker popup
///
/// Internal handler function - use [`crate::app::App::open_token_picker_internal`] instead.
pub(crate) fn open_token_picker(state: Arc<RwLock<AppState>>, target: TokenPickerTarget) {
    let mut state = state.write();
    state.terminal.swap.show_token_picker = true;
    state.terminal.swap.token_picker_for = target;
    state.terminal.swap.token_filter.clear();
    state.terminal.swap.selected_token_index = 0;

    // Populate token list from prices (TODO: fetch from API)
    state.terminal.swap.token_list = state
        .terminal
        .prices
        .iter()
        .map(|price| TokenInfo {
            symbol: price.symbol.clone(),
            name: price.symbol.clone(), // TODO: Get full name from API
            mint: "placeholder_mint".to_string(), // TODO: Get from API
            price: price.price,
            balance: 0.0, // TODO: Get from wallet
            change_24h: price.change_24h,
            is_favorite: false,
        })
        .collect();
}

/// Handle token selection from picker
///
/// Internal handler function - use [`crate::app::App::handle_token_select`] instead.
pub(crate) fn handle_token_select(
    state: Arc<RwLock<AppState>>,
    _event_tx: Sender<AppEvent>,
    token: TokenInfo,
    target: TokenPickerTarget,
) {
    {
        let mut state = state.write();
        match target {
            TokenPickerTarget::Input => {
                state.terminal.swap.input_token = token.symbol.clone();
                state.terminal.swap.input_mint = token.mint.clone();
            }
            TokenPickerTarget::Output => {
                state.terminal.swap.output_token = token.symbol.clone();
                state.terminal.swap.output_mint = token.mint.clone();
            }
        }
        state.terminal.swap.show_token_picker = false;
    }
    // Note: Quote fetch will be triggered by the caller or via on_tick
}

/// Set max amount from wallet balance
///
/// Internal handler function - use [`crate::app::App::set_max_amount`] instead.
pub(crate) fn set_max_amount(state: Arc<RwLock<AppState>>) {
    let mut state = state.write();
    // TODO: Get actual balance from wallet
    // For now, use a placeholder
    state.terminal.swap.amount = "100.0".to_string();
}

