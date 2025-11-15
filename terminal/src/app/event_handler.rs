//! # Event Handler
//!
//! Handles async event results from background tasks, updating application state accordingly.
//!
//! This module processes `AppEvent` messages received from async tasks (network requests,
//! blockchain operations, etc.) and updates the application state in a thread-safe manner.

use crate::app::{App, AppEvent, Screen};
use crate::app::state::{AuthState, PriceData};

/// Trait for event handling implementation
pub(crate) trait AppEventHandler {
    fn handle_event_impl(&mut self, event: AppEvent);
}

impl AppEventHandler for App {
    /// Handle async event results
    /// 
    /// CRITICAL: Acquires write lock per-event for minimal duration to prevent UI freezing
    /// 
    /// # Arguments
    /// 
    /// * `event` - The event to process
    fn handle_event_impl(&mut self, event: AppEvent) {
        // Track event receipt
        let event_type = format!("{:?}", event);
        crate::debug::track_event_receive(&event_type, None);

        match event {
            AppEvent::LoginResult(result) => {
                self.handle_login_result(result);
            }
            AppEvent::SignupResult(result) => {
                self.handle_signup_result(result);
            }
            AppEvent::WalletStatusChecked(result) => {
                self.handle_wallet_status_checked(result);
            }
            AppEvent::PricesUpdated(mut new_prices) => {
                self.handle_prices_updated(&mut new_prices);
            }
            AppEvent::PriceUpdated(new_price) => {
                self.handle_price_updated(new_price);
            }
            AppEvent::SwapQuoteResult(result) => {
                self.handle_swap_quote_result(result);
            }
            AppEvent::TokenListResult(result) => {
                self.handle_token_list_result(result);
            }
            AppEvent::SwapHistoryResult(result) => {
                self.handle_swap_history_result(result);
            }
            AppEvent::CandlesResult(result) => {
                self.handle_candles_result(result);
            }
            AppEvent::Loading(msg) => {
                self.handle_loading(msg);
            }
            AppEvent::WebSocketStatusUpdate(status) => {
                self.handle_websocket_status_update(status);
            }
        }
    }
}

impl App {
    fn handle_websocket_status_update(&mut self, status: crate::app::WebSocketStatus) {
        let mut state = self.state.write();
        let old_state = state.websocket_status.state.clone();
        let old_connected = state.websocket_connected;
        let old_message_count = state.websocket_status.messages_received;
        state.websocket_status = status.clone();
        state.websocket_connected = matches!(status.state, crate::app::WebSocketState::Connected);
        
        // Set repaint flag if status changed significantly (for dynamic UI updates)
        let status_changed = old_state != status.state;
        let message_count_changed = old_message_count != status.messages_received;
        if status_changed || message_count_changed {
            state.needs_immediate_repaint = true;
        }
        
        tracing::info!(
            old_state = ?old_state,
            new_state = ?status.state,
            old_connected = old_connected,
            new_connected = state.websocket_connected,
            messages_received = status.messages_received,
            connection_attempts = status.connection_attempts,
            last_error = ?status.last_error,
            status_changed = status_changed,
            message_count_changed = message_count_changed,
            "WebSocket status updated - UI will repaint"
        );
        
        // Log state transitions
        if old_state != status.state {
            tracing::info!(
                old_state = ?old_state,
                new_state = ?status.state,
                "WebSocket state transition - UI should update"
            );
        }
    }

    fn handle_login_result(&mut self, result: Result<shared::AuthResponse, String>) {
        tracing::info!(event = "LoginResult", success = result.is_ok(), "Processing login result");

        let mut state = self.state.write();
        match result {
            Ok(auth_response) => {
                // Check if user has wallet connected
                let has_wallet = auth_response.user.wallet_address.is_some();
                let token = auth_response.token.clone();
                
                // Update auth state and clear error
                if let AuthState::Login { error, .. } = &mut state.auth {
                    *error = None;
                    if !has_wallet {
                        *error = Some("Wallet not connected. Please connect your wallet first.".to_string());
                    }
                }
                
                // Update state fields (outside the auth borrow)
                state.auth_token = Some(token);
                // Extract and store current user info
                if let Ok(user_id) = auth_response.user.id.parse::<i64>() {
                    state.current_user = Some(crate::app::state::CurrentUser {
                        id: user_id,
                        username: auth_response.user.username.clone(),
                    });
                }
                
                // Start WebSocket connection for real-time price updates (only once)
                // Delay connection slightly to allow UI to initialize first
                if !state.websocket_connected {
                    state.websocket_connected = true;
                    let event_tx_clone = self.event_tx.clone();
                    let app_state_clone = self.state.clone();
                    tokio::spawn(async move {
                        // Small delay to ensure UI is initialized before attempting connection
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        crate::services::api::websocket::connect_price_stream(event_tx_clone, Some(app_state_clone)).await;
                    });
                    tracing::info!("Scheduled WebSocket price stream connection (delayed for UI initialization)");
                }
                
                // Only switch to Terminal screen if wallet is connected
                if has_wallet {
                    state.current_screen = Screen::Terminal;
                    // Fetch initial candles for SOL chart
                    let timeframe = state.terminal.chart_timeframe;
                    let needs_initial_prices = state.terminal.prices.is_empty();
                    drop(state);
                    
                    // Fetch initial prices immediately if we have none (don't wait for WebSocket)
                    if needs_initial_prices {
                        tracing::info!("Fetching initial prices via REST API (WebSocket may not be ready yet)");
                        crate::app::tasks::market::fetch_prices(self.state.clone(), self.event_tx.clone());
                    }
                    
                    self.fetch_candles("SOL", timeframe);
                } else {
                    // No wallet - stay on Auth screen
                    state.current_screen = Screen::Auth;
                }
            }
            Err(err) => {
                if let AuthState::Login { error, .. } = &mut state.auth {
                    *error = Some(err);
                }
            }
        }
    }

    fn handle_signup_result(&mut self, result: Result<shared::AuthResponse, String>) {
        tracing::info!(event = "SignupResult", success = result.is_ok(), "Processing signup result");
        let mut state = self.state.write();

        match result {
            Ok(auth_response) => {
                // Extract all needed values first
                let token = auth_response.token.clone();
                let should_open_wallet = auth_response.wallet_setup_required == Some(true);
                let has_wallet = auth_response.user.wallet_address.is_some();
                let wallet_url = if should_open_wallet {
                    auth_response.wallet_setup_token.as_ref().map(|token| {
                        format!("http://localhost:8080/?token={}", token)
                    })
                } else {
                    None
                };
                
                // Get credentials for polling if needed (before mutable borrow)
                let polling_creds = if should_open_wallet && !has_wallet {
                    if let AuthState::Signup { username, password, .. } = &state.auth {
                        Some((username.clone(), password.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Update auth state and error message
                if let AuthState::Signup { error, .. } = &mut state.auth {
                    *error = None;
                    if should_open_wallet && !has_wallet {
                        *error = Some("Please connect your wallet in the browser window. The terminal will open automatically once your wallet is connected.".to_string());
                    }
                }

                // Update state fields (outside the auth borrow)
                state.auth_token = Some(token);
                // Extract and store current user info
                if let Ok(user_id) = auth_response.user.id.parse::<i64>() {
                    state.current_user = Some(crate::app::state::CurrentUser {
                        id: user_id,
                        username: auth_response.user.username.clone(),
                    });
                }
                if let Some(creds) = polling_creds {
                    state.polling_credentials = Some(creds);
                }

                // Start WebSocket connection for real-time price updates (only once)
                // Delay connection slightly to allow UI to initialize first
                if !state.websocket_connected {
                    state.websocket_connected = true;
                    let event_tx_clone = self.event_tx.clone();
                    let app_state_clone = self.state.clone();
                    tokio::spawn(async move {
                        // Small delay to ensure UI is initialized before attempting connection
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        crate::services::api::websocket::connect_price_stream(event_tx_clone, Some(app_state_clone)).await;
                    });
                    tracing::info!("Scheduled WebSocket price stream connection (delayed for UI initialization)");
                }

                // Only switch to Terminal screen if wallet is connected
                if has_wallet {
                    state.current_screen = Screen::Terminal;
                    state.polling_credentials = None; // Clear polling credentials
                } else if should_open_wallet {
                    // Wallet setup required - stay on Auth screen
                    state.current_screen = Screen::Auth;
                    
                    // Open wallet-web
                    let url_to_open = wallet_url.clone();
                    drop(state); // Release lock before opening browser
                    
                    if let Some(url) = url_to_open {
                        if let Err(e) = open::that(&url) {
                            tracing::error!("Failed to open wallet-web: {}", e);
                            let mut state = self.state.write();
                            if let AuthState::Signup { error, .. } = &mut state.auth {
                                *error = Some("Failed to open wallet connection page. Please visit http://localhost:8080 manually.".to_string());
                            }
                            drop(state);
                        } else {
                            tracing::info!("Opened wallet-web at {}", url);
                        }
                    }
                    
                    // Start polling for wallet connection
                    self.start_wallet_connection_polling();
                } else {
                    // No wallet setup required and no wallet - go to Terminal anyway
                    state.current_screen = Screen::Terminal;
                }
            }
            Err(err) => {
                if let AuthState::Signup { error, .. } = &mut state.auth {
                    *error = Some(err);
                }
            }
        }
    }

    fn handle_wallet_status_checked(&mut self, result: Result<shared::AuthResponse, String>) {
        tracing::debug!(event = "WalletStatusChecked", success = result.is_ok(), "Processing wallet status check");
        let mut state = self.state.write();
        match result {
            Ok(auth_response) => {
                // Check if wallet is now connected
                if auth_response.user.wallet_address.is_some() {
                    // Wallet connected! Switch to Terminal screen
                    state.auth_token = Some(auth_response.token.clone());
                    // Extract and store current user info
                    if let Ok(user_id) = auth_response.user.id.parse::<i64>() {
                        state.current_user = Some(crate::app::state::CurrentUser {
                            id: user_id,
                            username: auth_response.user.username.clone(),
                        });
                    }
                    state.current_screen = Screen::Terminal;
                    state.polling_credentials = None; // Stop polling
                    tracing::info!("Wallet connected! Switching to Terminal screen.");
                    
                    // Clear error message
                    if let AuthState::Signup { error, .. } = &mut state.auth {
                        *error = None;
                    }
                }
                // If wallet not connected yet, polling will continue
            }
            Err(_err) => {
                // Polling error - will retry on next poll
                tracing::debug!("Wallet status check failed (will retry): {}", _err);
            }
        }
    }

    fn handle_prices_updated(&mut self, new_prices: &mut Vec<PriceData>) {
        tracing::debug!(event = "PricesUpdated", count = new_prices.len(), "Processing price update");
        let mut state = self.state.write();
        let mut any_changes = false;
        
        // Store previous prices before updating
        for new_price in new_prices.iter_mut() {
            // Find matching existing price by symbol
            if let Some(existing) = state.terminal.prices.iter().find(|p| p.symbol == new_price.symbol) {
                // Store current price as previous price
                new_price.previous_price = Some(existing.price);
                // Check if price actually changed
                if (existing.price - new_price.price).abs() > 0.0001 {
                    any_changes = true;
                }
            } else {
                // New token
                any_changes = true;
            }
        }
        
        state.terminal.prices = new_prices.clone();
        state.terminal.last_price_update = std::time::Instant::now();
        
        // CRITICAL: Set immediate repaint flag for real-time updates
        if any_changes {
            state.needs_immediate_repaint = true;
            state.last_price_update_time = std::time::Instant::now();
        }
    }

    fn handle_price_updated(&mut self, new_price: PriceData) {
        tracing::info!(
            event = "PriceUpdated",
            symbol = %new_price.symbol,
            price = new_price.price,
            source = ?new_price.source,
            "EVENT HANDLER: Processing PriceUpdated event from WebSocket"
        );
        let is_sol = new_price.symbol == "SOL";
        let mut state = self.state.write();
        let price_count_before = state.terminal.prices.len();
        let is_new_token = !state.terminal.prices.iter().any(|p| p.symbol == new_price.symbol);
        
        tracing::debug!(
            symbol = %new_price.symbol,
            price_count_before = price_count_before,
            is_new_token = is_new_token,
            "Checking if token exists in price list"
        );
        
        // Find existing price and update it
        let _price_changed = if let Some(existing) = state.terminal.prices.iter_mut().find(|p| p.symbol == new_price.symbol) {
            // Store current price as previous price
            let old_price = existing.price;
            let price_changed = (existing.price - new_price.price).abs() > 0.0001; // Significant change
            existing.previous_price = Some(existing.price);
            existing.price = new_price.price;
            existing.change_24h = new_price.change_24h;
            tracing::info!(
                symbol = %new_price.symbol,
                old_price = old_price,
                new_price = new_price.price,
                price_change = new_price.price - old_price,
                "Updated existing token price in state - UI should reflect this change"
            );
            price_changed
        } else {
            // New token, add it to the list
            state.terminal.prices.push(new_price.clone());
            tracing::info!(
                symbol = %new_price.symbol,
                price = new_price.price,
                price_count_after = state.terminal.prices.len(),
                "Added new token to price list - UI should display this new token"
            );
            true // New token counts as a change
        };
        
        // CRITICAL: Always set immediate repaint flag for instant Bloomberg-style updates
        // Every WebSocket price update should trigger immediate UI refresh (<10ms latency)
        // Set these flags BEFORE releasing the lock to ensure they're visible immediately
        state.needs_immediate_repaint = true;
        state.last_price_update_time = std::time::Instant::now();
        state.terminal.last_price_update = std::time::Instant::now();
        
        // Log repaint trigger for debugging
        tracing::debug!(
            symbol = %new_price.symbol,
            needs_immediate_repaint = state.needs_immediate_repaint,
            "Repaint flag set - UI will update immediately on next frame"
        );
        
        let price_count_after = state.terminal.prices.len();
        tracing::info!(
            symbol = %new_price.symbol,
            price_count_before = price_count_before,
            price_count_after = price_count_after,
            total_tokens = price_count_after,
            needs_immediate_repaint = state.needs_immediate_repaint,
            "Price list state updated - ready for UI rendering"
        );
        
        // If this is SOL and we don't have candles yet, fetch them
        let should_fetch = is_sol && state.terminal.sol_candles.is_empty() && !state.terminal.chart_loading;
        let timeframe = if should_fetch {
            Some(state.terminal.chart_timeframe)
        } else {
            None
        };
        drop(state);
        
        if let Some(tf) = timeframe {
            tracing::debug!(symbol = "SOL", timeframe = ?tf, "Triggering candle fetch for SOL");
            self.fetch_candles("SOL", tf);
        }
    }

    fn handle_swap_quote_result(&mut self, result: Result<crate::app::state::SwapQuote, String>) {
        tracing::info!(event = "SwapQuoteResult", success = result.is_ok(), "Processing swap quote result");
        let mut state = self.state.write();
        match result {
            Ok(quote) => {
                state.terminal.swap.quote = Some(quote);
                state.terminal.swap.quote_loading = false;
            }
            Err(_err) => {
                // Failed to fetch quote - clear it and stop loading
                state.terminal.swap.quote = None;
                state.terminal.swap.quote_loading = false;
            }
        }
    }

    fn handle_token_list_result(&mut self, result: Result<Vec<crate::app::state::TokenInfo>, String>) {
        let count = result.as_ref().map(|t| t.len()).unwrap_or(0);
        tracing::info!(event = "TokenListResult", success = result.is_ok(), count = count, "Processing token list result");
        let mut state = self.state.write();
        match result {
            Ok(tokens) => {
                state.terminal.swap.token_list = tokens;
            }
            Err(_err) => {
                // Failed to fetch token list - keep existing
            }
        }
    }

    fn handle_swap_history_result(&mut self, result: Result<Vec<crate::app::state::SwapHistoryItem>, String>) {
        let count = result.as_ref().map(|h| h.len()).unwrap_or(0);
        tracing::info!(event = "SwapHistoryResult", success = result.is_ok(), count = count, "Processing swap history result");
        let mut state = self.state.write();
        match result {
            Ok(history) => {
                state.terminal.swap.swap_history = history;
            }
            Err(_err) => {
                // Failed to fetch history - keep existing
            }
        }
    }

    fn handle_candles_result(&mut self, result: Result<Vec<shared::dto::OHLC>, String>) {
        let count = result.as_ref().map(|c| c.len()).unwrap_or(0);
        let mut state = self.state.write();
        let timeframe = state.terminal.chart_timeframe;
        let timeframe_str = match timeframe {
            shared::dto::market::Timeframe::OneMinute => "1m",
            shared::dto::market::Timeframe::FiveMinutes => "5m",
            shared::dto::market::Timeframe::FifteenMinutes => "15m",
            shared::dto::market::Timeframe::OneHour => "1h",
            shared::dto::market::Timeframe::FourHours => "4h",
            shared::dto::market::Timeframe::OneDay => "1d",
            shared::dto::market::Timeframe::OneWeek => "1w",
        };
        
        tracing::info!(
            event = "CandlesResult",
            success = result.is_ok(),
            count = count,
            symbol = "SOL",
            timeframe = %timeframe_str,
            "Processing candles result"
        );
        
        match result {
            Ok(candles) => {
                if candles.is_empty() {
                    tracing::warn!(
                        symbol = "SOL",
                        timeframe = %timeframe_str,
                        "Received empty candle list"
                    );
                } else {
                    tracing::debug!(
                        symbol = "SOL",
                        timeframe = %timeframe_str,
                        count = candles.len(),
                        "Candles loaded successfully"
                    );
                }
                state.terminal.sol_candles = candles;
                state.terminal.chart_loading = false;
            }
            Err(err) => {
                tracing::warn!(
                    symbol = "SOL",
                    timeframe = %timeframe_str,
                    error = %err,
                    "Failed to fetch candles"
                );
                state.terminal.chart_loading = false;
                // Failed to fetch candles - keep existing
            }
        }
    }

    fn handle_loading(&mut self, msg: String) {
        tracing::debug!(event = "Loading", message = %msg, "Processing loading status");
        let mut state = self.state.write();
        
        // Check if this is a notification message
        if msg.starts_with("NOTIFY_SUCCESS:") {
            let message = msg.strip_prefix("NOTIFY_SUCCESS:").unwrap_or(&msg).to_string();
            state.pending_notifications.push(("success".to_string(), message));
        } else if msg.starts_with("NOTIFY_ERROR:") {
            let message = msg.strip_prefix("NOTIFY_ERROR:").unwrap_or(&msg).to_string();
            state.pending_notifications.push(("error".to_string(), message));
        } else if msg.starts_with("NOTIFY_WARNING:") {
            let message = msg.strip_prefix("NOTIFY_WARNING:").unwrap_or(&msg).to_string();
            state.pending_notifications.push(("warning".to_string(), message));
        } else if msg.starts_with("NOTIFY_INFO:") {
            let message = msg.strip_prefix("NOTIFY_INFO:").unwrap_or(&msg).to_string();
            state.pending_notifications.push(("info".to_string(), message));
        } else {
            // Regular loading message - also check for trade-related messages
            if msg.contains("Swap successful") || msg.contains("Trade confirmed") {
                state.pending_notifications.push(("success".to_string(), msg.clone()));
            } else if (msg.contains("failed") || msg.contains("Failed") || msg.contains("error"))
                && (msg.contains("Swap") || msg.contains("Trade"))
            {
                state.pending_notifications.push(("error".to_string(), msg.clone()));
            }
            
            // Update loading message in auth state
            match &mut state.auth {
                AuthState::Login { error, .. } => {
                    *error = Some(msg);
                }
                AuthState::Signup { error, .. } => {
                    *error = Some(msg);
                }
            }
        }
    }
}
