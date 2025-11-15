//! # Application Orchestrator
//!
//! The main [`App`] struct orchestrates the entire application, coordinating between
//! the UI rendering layer, async task handlers, and application state management.
//!
//! ## Architecture
//!
//! The application follows an event-driven architecture pattern:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Main Thread (egui)                       │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │  App (orchestrator)                                  │   │
//! │  │  - on_tick() - called every frame                    │   │
//! │  │  - handle_event() - processes async results          │   │
//! │  │  - handle_*_click() - user action handlers          │   │
//! │  └────────────┬─────────────────────────────────────────┘   │
//! │               │                                              │
//! │  ┌────────────▼─────────────────────────────────────────┐   │
//! │  │  State: Arc<RwLock<AppState>>                       │   │
//! │  │  - Thread-safe shared state                         │   │
//! │  │  - Lock held briefly for minimal duration           │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! └───────────────────────┬─────────────────────────────────────┘
//!                         │ async_channel
//!                         │ (unbounded)
//! ┌───────────────────────▼─────────────────────────────────────┐
//! │              Async Task Threads (Tokio)                    │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │  Tasks Module                                         │   │
//! │  │  - fetch_prices() - market data                      │   │
//! │  │  - execute_swap() - swap execution                   │   │
//! │  │  - fetch_token_list() - token metadata               │   │
//! │  └────────────┬─────────────────────────────────────────┘   │
//! │               │                                              │
//! │  ┌────────────▼─────────────────────────────────────────┐   │
//! │  │  Handlers Module                                      │   │
//! │  │  - handle_login_click() - auth logic                 │   │
//! │  │  - handle_token_select() - swap logic                │   │
//! │  │  - handle_screen_change() - navigation               │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Components
//!
//! - **[`App`]**: Main application orchestrator with event handling
//! - **[`AppState`]**: Thread-safe shared application state (see [`state`] module)
//! - **[`AppEvent`]**: Event enum for async task results (see [`events`] module)
//! - **[`handlers`]**: User action handlers (auth, navigation, swap, wallet)
//! - **[`tasks`]**: Async background tasks (market data, swaps)
//!
//! ## State Management Pattern
//!
//! The application uses `Arc<RwLock<AppState>>` for thread-safe state:
//!
//! ```rust,ignore
//! // Main thread: Read state for rendering
//! let state = app.state.read(); // Shared read lock
//! // Render UI based on state
//! drop(state); // Lock released immediately
//!
//! // Async task: Write state updates
//! let mut state = app.state.write(); // Exclusive write lock
//! state.terminal.prices = new_prices;
//! drop(state); // Lock released immediately
//! ```
//!
//! **Critical**: Locks are held for minimal duration to prevent UI freezing.
//!
//! ## Event-Driven Communication
//!
//! Async tasks communicate results back to main thread via `AppEvent`:
//!
//! ```rust,ignore
//! // Async task sends event
//! event_tx.send(AppEvent::PricesUpdated(prices)).await?;
//!
//! // Main thread receives event in on_tick()
//! while let Ok(event) = app.event_rx.try_recv() {
//!     app.handle_event(event); // Updates state
//! }
//! ```
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use terminal::app::App;
//!
//! fn main() {
//!     // Create app with initial state
//!     let mut app = App::new();
//!
//!     // In egui update loop:
//!     loop {
//!         // Process async events (non-blocking)
//!         app.on_tick();
//!
//!         // Handle user actions
//!         if user_clicked_login {
//!             app.handle_login_click(username, password);
//!         }
//!
//!         // Render UI (reads state)
//!         let state = app.state.read();
//!         render_ui(&state);
//!         drop(state);
//!     }
//! }
//! ```
//!
//! ## Thread Safety
//!
//! - **Main Thread**: Single-threaded (egui requirement), handles all UI rendering
//! - **Async Tasks**: Multi-threaded (Tokio runtime), handles network I/O
//! - **Communication**: Via `async_channel::unbounded` (lock-free, async)
//! - **State Access**: `Arc<RwLock<AppState>>` ensures thread-safe access
//!
//! ## Related Modules
//!
//! - [`state`]: Application state types and definitions
//! - [`events`]: Event enum for async communication
//! - [`handlers`]: User action handlers
//! - [`tasks`]: Async background tasks

mod state;
mod events;
mod handlers;
mod tasks;
mod event_handler;
mod window_manager;
mod window_app;
mod viewport;
mod app_trait;

pub use state::*;
pub use events::AppEvent;
pub use window_manager::WindowManager;
pub use window_app::WindowApp;
pub use viewport::show_deferred_viewport;
pub use app_trait::AppLike;

use std::sync::Arc;
use parking_lot::RwLock;
use async_channel::{Sender, Receiver, unbounded};
use crate::core::service::ApiService;

/// Main application orchestrator that coordinates UI rendering, async tasks, and state management.
///
/// The [`App`] struct serves as the central coordinator between:
/// - **UI Layer**: egui rendering (main thread)
/// - **Async Tasks**: Network requests and blockchain operations (Tokio tasks)
/// - **State Management**: Thread-safe shared state (`Arc<RwLock<AppState>>`)
///
/// # Architecture
///
/// The application follows an event-driven pattern where async tasks send results
/// back to the main thread via `AppEvent` messages through an unbounded channel.
///
/// # Thread Safety
///
/// - **Main Thread**: All UI operations must run on the main thread (egui requirement)
/// - **Async Tasks**: Network I/O runs on Tokio runtime (multi-threaded)
/// - **State Access**: Thread-safe via `Arc<RwLock<AppState>>` (multiple readers, exclusive writers)
///
/// # Example
///
/// ```rust,no_run
/// use terminal::app::App;
///
/// let mut app = App::new();
///
/// // In egui update loop (main thread):
/// loop {
///     // Process async events (non-blocking)
///     app.on_tick();
///
///     // Handle user actions
///     app.handle_login_click(username.clone(), password.clone());
///
///     // Render UI (locks state briefly)
///     let state = app.state.read();
///     render_ui(&state);
///     drop(state); // Release lock immediately
/// }
/// ```
pub struct App {
    /// Thread-safe shared application state.
    ///
    /// Wrapped in `Arc<RwLock<AppState>>` for efficient sharing across threads.
    /// - Use `read()` for reading (shared lock, multiple readers)
    /// - Use `write()` for writing (exclusive lock, single writer)
    /// - **Critical**: Hold locks for minimal duration to prevent UI freezing
    pub state: Arc<RwLock<AppState>>,
    
    /// Channel receiver for async task results.
    ///
    /// Receives `AppEvent` messages from async tasks (network requests, swaps, etc.).
    /// Polled in `on_tick()` using `try_recv()` (non-blocking).
    pub event_rx: Receiver<AppEvent>,
    
    /// Channel sender for async task results (internal use).
    ///
    /// Cloned and passed to async tasks for sending results back to main thread.
    event_tx: Sender<AppEvent>,
    
    /// Time elapsed since last frame (for animations and effects).
    ///
    /// Used for frame-rate independent animations (e.g., rotating 3D icosahedron).
    pub last_tick: std::time::Duration,
    
    /// Window manager for multiple viewports
    pub window_manager: Arc<RwLock<WindowManager>>,
}

impl App {
    /// Create a new application instance with initial state.
    ///
    /// Initializes the application with:
    /// - Default state (landing screen, empty auth, demo prices)
    /// - API client for backend communication
    /// - Event channel for async task communication
    /// - Initial token list fetch task
    ///
    /// # Returns
    ///
    /// A new [`App`] instance ready for use in the egui update loop.
    ///
    /// # State Initialization
    ///
    /// Initial state includes:
    /// - Current screen: [`Screen::Landing`]
    /// - Auth state: Login form with empty fields
    /// - Terminal state: Default swap configuration (SOL → USDC)
    /// - Demo price data for UI preview
    ///
    /// # Async Tasks
    ///
    /// Spawns initial background task to fetch token list from backend.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use terminal::app::App;
    ///
    /// let app = App::new();
    /// let state = app.state.read();
    /// assert_eq!(state.current_screen, Screen::Landing);
    /// ```
    pub fn new() -> Self {
        // Create API client
        let api_client = Arc::new(crate::services::api::ApiClient::new());

        // Load settings from file
        let theme_config = handlers::settings::load_settings();
        let settings = crate::app::state::SettingsState {
            theme_config,
            config_path: handlers::settings::get_config_path().to_string_lossy().to_string(),
            unsaved_changes: false,
        };

        let state = AppState {
            current_screen: Screen::Landing,
            auth: AuthState::Login {
                username: String::new(),
                password: String::new(),
                error: None,
                active_field: LoginField::Username,
            },
            terminal: TerminalState {
                swap: SwapState::default(),
                prices: Vec::new(), // Start empty, will be populated from websocket
                chart_data: Vec::new(),
                sol_candles: Vec::new(), // Will be populated from API
                chart_timeframe: shared::dto::market::Timeframe::OneHour,
                chart_loading: false,
                active_chart: None, // Will use real OHLC data instead
                last_price_update: std::time::Instant::now(),
                fetching_prices: false,
                swap_panel_open: false,
            },
            wallet: None,
            transactions: Vec::new(),
            auth_token: None,
            current_user: None,
            api_client: Some(api_client),
            wallet_service: None, // Will be initialized when user connects wallet
            polling_credentials: None,
            pending_notifications: Vec::new(),
            websocket_connected: false,
            websocket_status: crate::app::state::WebSocketStatus::default(),
            messaging: crate::app::state::MessagingState::default(),
            ai_chat: crate::app::state::AIChatState::default(),
            settings,
            debug_overlay_visible: std::env::var("TERMINAL_DEBUG_UI")
                .map(|v| v == "1")
                .unwrap_or(false),
            needs_immediate_repaint: false,
            last_price_update_time: std::time::Instant::now(),
            nav_bar_selected_token: Some("SOL".to_string()), // Default to SOL
            nav_bar_show_token_picker: false,
        };

        // Create event channel
        let (event_tx, event_rx) = unbounded();

        // Create window manager
        let window_manager = Arc::new(RwLock::new(WindowManager::new()));
        
        let app = App {
            state: Arc::new(RwLock::new(state)),
            event_rx,
            event_tx,
            last_tick: std::time::Duration::from_millis(250),
            window_manager,
        };
        
        // Fetch initial token list
        tasks::market::fetch_token_list(app.state.clone(), app.event_tx.clone());
        
        tracing::info!("App state initialized - Event channel created, token list fetch started");
        tracing::debug!("WebSocket connection will be started after successful login");
        
        // WebSocket connection will be started after successful login
        // (see LoginResult event handler)
        
        app
    }

    /// Generate demo price data (TODO: fetch from API)
    fn generate_demo_prices() -> Vec<PriceData> {
        vec![
            PriceData {
                symbol: "SOL".to_string(),
                price: 145.32,
                change_24h: 5.2,
                previous_price: None,
                source: Some("jupiter".to_string()),
            },
            PriceData {
                symbol: "USDC".to_string(),
                price: 1.0,
                change_24h: 0.0,
                previous_price: None,
                source: Some("jupiter".to_string()),
            },
            PriceData {
                symbol: "BTC".to_string(),
                price: 64250.0,
                change_24h: 3.1,
                previous_price: None,
                source: Some("pyth".to_string()),
            },
            PriceData {
                symbol: "ETH".to_string(),
                price: 3100.5,
                change_24h: -1.5,
                previous_price: None,
                source: Some("pyth".to_string()),
            },
        ]
    }

    /// Navigate to next screen in Tab order
    pub fn next_screen(&mut self) {
        handlers::navigation::next_screen(self.state.clone());
    }

    /// Navigate to previous screen in Tab order
    pub fn previous_screen(&mut self) {
        handlers::navigation::previous_screen(self.state.clone());
    }

    /// Called every frame to process async events and update state.
    ///
    /// This method should be called from the egui update loop (main thread)
    /// on every frame to ensure async task results are processed promptly.
    ///
    /// # Operations Performed
    ///
    /// 1. **Updates watchdog heartbeat** - Detects UI freezes for debugging
    /// 2. **Processes async events** - Non-blocking poll of event channel
    /// 3. **Triggers price updates** - Fetches new prices every 5 seconds when on terminal screen
    ///
    /// # Event Processing
    ///
    /// Processes all pending events from `event_rx` using `try_recv()`:
    /// - Non-blocking (returns immediately if no events)
    /// - Processes multiple events per tick if available
    /// - Each event updates state via `handle_event()`
    ///
    /// # Price Update Logic
    ///
    /// Automatically fetches prices when:
    /// - User is authenticated (`auth_token` is set)
    /// - Current screen is `Screen::Terminal`
    /// - Last update was 5+ seconds ago
    ///
    /// # Performance
    ///
    /// - Fast: O(n) where n = number of pending events
    /// - Non-blocking: Never waits for async operations
    /// - Lock-free: Uses try_recv() for event polling
    ///
    /// # Usage
    ///
    /// ```rust,no_run
    /// use terminal::app::App;
    ///
    /// let mut app = App::new();
    ///
    /// // In egui update loop:
    /// loop {
    ///     // Process async events (non-blocking)
    ///     app.on_tick();
    ///
    ///     // Render UI
    ///     // ...
    /// }
    /// ```
    pub fn on_tick(&mut self) {
        // Update watchdog heartbeat to detect freezes
        crate::debug::update_heartbeat();

        // Process any pending async events - CRITICAL for instant Bloomberg-style updates
        // Process all events immediately without delays to ensure <10ms latency
        let event_processing_start = std::time::Instant::now();
        let mut events_processed = 0u32;
        let mut price_updated_events = 0u32;
        
        // Process all available events in the channel (non-blocking)
        // CRITICAL: Prioritize PriceUpdated events for instant processing
        // This ensures WebSocket price updates are handled immediately with <10ms latency
        let mut price_events = Vec::new();
        let mut other_events = Vec::new();
        
        // Separate events by priority
        while let Ok(event) = self.event_rx.try_recv() {
            events_processed += 1;
            if matches!(event, AppEvent::PriceUpdated(_)) {
                price_updated_events += 1;
                price_events.push(event);
            } else {
                other_events.push(event);
            }
        }
        
        // Process price update events FIRST for minimal latency
        for event in price_events {
            self.handle_event(event);
        }
        
        // Then process other events
        for event in other_events {
            self.handle_event(event);
        }
        
        // Log event processing statistics with latency tracking
        if events_processed > 0 {
            let processing_time = event_processing_start.elapsed();
            let processing_time_us = processing_time.as_micros();
            
            tracing::debug!(
                events_processed = events_processed,
                price_updated_events = price_updated_events,
                processing_time_us = processing_time_us,
                "on_tick: Processed events from event channel (latency: {}μs)",
                processing_time_us
            );
            
            // Warn if event processing takes too long (could delay price updates)
            if processing_time.as_millis() > 5 {
                tracing::warn!(
                    events_processed = events_processed,
                    processing_time_ms = processing_time.as_millis(),
                    "Event processing took longer than 5ms - may delay price updates"
                );
            }
        }

        // Fallback to REST API if WebSocket is disabled or disconnected for too long
        // Also fetch initial prices if we have none yet
        let should_fallback = {
            let state = self.state.read();
            let auth_ok = state.auth_token.is_some();
            let on_terminal_screen = state.current_screen == Screen::Terminal;
            let has_no_prices = state.terminal.prices.is_empty();
            let ws_disabled = matches!(state.websocket_status.state, crate::app::WebSocketState::Disabled);
            let ws_disconnected_too_long = !state.websocket_connected 
                && state.terminal.last_price_update.elapsed().as_secs() >= 5; // Reduced from 10s to 5s
            let no_recent_updates = state.terminal.last_price_update.elapsed().as_secs() >= 15; // Reduced from 30s to 15s
            let ws_failing = matches!(state.websocket_status.state, crate::app::WebSocketState::Reconnecting)
                && state.websocket_status.connection_attempts >= 3; // After 3 failed attempts, use REST API
            
            auth_ok && on_terminal_screen && (has_no_prices || ws_disabled || ws_disconnected_too_long || no_recent_updates || ws_failing)
        };
        
        if should_fallback {
            let state = self.state.read();
            let ws_state = state.websocket_status.state.clone();
            let last_update_secs = state.terminal.last_price_update.elapsed().as_secs();
            let price_count = state.terminal.prices.len();
            let connection_attempts = state.websocket_status.connection_attempts;
            drop(state);
            
            tracing::warn!(
                websocket_state = ?ws_state,
                last_update_secs = last_update_secs,
                price_count = price_count,
                connection_attempts = connection_attempts,
                "WebSocket unavailable or stale - falling back to REST API for price updates"
            );
            tasks::market::fetch_prices(self.state.clone(), self.event_tx.clone());
        }
    }

    /// Handle async event results
    /// 
    /// Delegates to the event_handler module for processing.
    /// CRITICAL: Acquires write lock per-event for minimal duration to prevent UI freezing
    fn handle_event(&mut self, event: AppEvent) {
        use event_handler::AppEventHandler;
        self.handle_event_impl(event);
    }

    /// Start polling for wallet connection status
    /// 
    /// Polls the backend every 3 seconds to check if the user's wallet has been connected.
    /// Stops polling when wallet is connected or credentials are cleared.
    fn start_wallet_connection_polling(&self) {
        let event_tx = self.event_tx.clone();
        let state_arc = Arc::clone(&self.state);

        tokio::spawn(async move {
            let mut poll_count = 0;
            loop {
                // Wait 3 seconds between polls
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                // Check if we should stop polling
                let (should_continue, credentials) = {
                    let state = state_arc.read();
                    let creds = state.polling_credentials.clone();
                    let should = creds.is_some() && state.current_screen == Screen::Auth;
                    (should, creds)
                };

                if !should_continue {
                    tracing::info!("Stopping wallet connection polling");
                    break;
                }

                if let Some((username, password)) = credentials {
                    poll_count += 1;
                    tracing::debug!("Polling wallet status (attempt {})...", poll_count);

                    // Get API client
                    let api_client = {
                        let state = state_arc.read();
                        state.api_client.clone()
                    };

                    if let Some(api_client) = api_client {
                        // Re-login to check wallet status
                        match api_client.login(username.clone(), password.clone()).await {
                            Ok(auth_response) => {
                                // Send result to event handler
                                let _ = event_tx.send(AppEvent::WalletStatusChecked(Ok(auth_response))).await;
                                
                                // Check if wallet is connected - if so, stop polling
                                let state = state_arc.read();
                                if state.polling_credentials.is_none() || state.current_screen != Screen::Auth {
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::debug!("Wallet status check failed: {}", e);
                                let _ = event_tx.send(AppEvent::WalletStatusChecked(Err(e))).await;
                            }
                        }
                    }
                } else {
                    break;
                }
            }
        });
    }

    // ========== GUI Action Methods - Delegating to Handlers ==========

    /// Handle login button click
    pub fn handle_login_click(&mut self, username: String, password: String) {
        handlers::auth::handle_login_click(self.state.clone(), self.event_tx.clone(), username, password);
    }

    /// Handle signup button click
    pub fn handle_signup_click(&mut self, username: String, email: String, password: String, confirm_password: String) {
        handlers::auth::handle_signup_click(self.state.clone(), self.event_tx.clone(), username, email, password, confirm_password);
    }

    /// Switch to login form
    pub fn handle_switch_to_login(&mut self) {
        handlers::auth::handle_switch_to_login(self.state.clone());
    }

    /// Switch to signup form
    pub fn handle_switch_to_signup(&mut self) {
        handlers::auth::handle_switch_to_signup(self.state.clone());
    }

    /// Handle screen change
    pub fn handle_screen_change(&mut self, screen: Screen) {
        handlers::navigation::handle_screen_change(self.state.clone(), screen);
    }

    /// Handle swap tab change
    pub fn handle_swap_tab_change(&mut self, tab: SwapTab) {
        handlers::navigation::handle_swap_tab_change(self.state.clone(), tab);
    }

    /// Handle swap execute button click
    pub fn handle_swap_execute_click(&mut self) {
        tasks::swap::execute_swap(self.state.clone(), self.event_tx.clone());
    }

    /// Handle token selection from picker
    pub fn handle_token_select(&mut self, token: TokenInfo, target: TokenPickerTarget) {
        handlers::swap::handle_token_select(self.state.clone(), self.event_tx.clone(), token, target);
        // Trigger quote fetch after token selection
        tasks::swap::trigger_quote_fetch(self.state.clone(), self.event_tx.clone());
    }

    /// Handle wallet connect button click
    pub fn handle_wallet_connect_click(&mut self) {
        handlers::wallet::handle_wallet_connect_click(self.state.clone(), self.event_tx.clone());
    }

    /// Handle wallet generate button click
    pub fn handle_wallet_generate_click(&mut self) {
        handlers::wallet::handle_wallet_generate_click(self.state.clone(), self.event_tx.clone());
    }

    /// Handle wallet disconnect button click
    pub fn handle_wallet_disconnect_click(&mut self) {
        handlers::wallet::handle_wallet_disconnect_click(self.state.clone());
    }

    /// Trigger async swap quote fetch with debouncing
    pub fn trigger_quote_fetch(&mut self) {
        tasks::swap::trigger_quote_fetch(self.state.clone(), self.event_tx.clone());
    }

    /// Fetch token list from backend API
    pub fn fetch_token_list(&mut self) {
        tasks::market::fetch_token_list(self.state.clone(), self.event_tx.clone());
    }

    /// Handle theme color change
    pub fn handle_theme_color_change(&mut self, config: crate::ui::theme::ThemeConfig) {
        handlers::settings::handle_theme_color_change(self.state.clone(), config);
    }

    /// Handle settings save
    pub fn handle_settings_save(&mut self) {
        handlers::settings::handle_settings_save(self.state.clone());
    }

    /// Handle settings reset to defaults
    pub fn handle_settings_reset(&mut self) {
        handlers::settings::handle_settings_reset(self.state.clone());
    }

    /// Handle settings apply (apply without saving)
    pub fn handle_settings_apply(&mut self) {
        handlers::settings::handle_settings_apply(self.state.clone());
    }

    /// Fetch candles for a symbol and timeframe
    pub fn fetch_candles(&mut self, symbol: &str, timeframe: shared::dto::market::Timeframe) {
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
            symbol = %symbol,
            timeframe = %timeframe_str,
            "Triggering candle fetch"
        );
        tasks::market::fetch_candles(self.state.clone(), self.event_tx.clone(), symbol.to_string(), timeframe);
    }

    /// Open token picker popup
    pub fn open_token_picker_internal(&self, _state: &mut AppState, target: TokenPickerTarget) {
        handlers::swap::open_token_picker(self.state.clone(), target);
    }

    /// Set max amount from wallet balance
    pub fn set_max_amount(&mut self) {
        handlers::swap::set_max_amount(self.state.clone());
    }

    /// Connect wallet from keypair file
    pub async fn connect_wallet_from_file(&self, path: &str) -> Result<String, String> {
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

        let mut wallet_service = crate::services::wallet::WalletService::new(&rpc_url);

        wallet_service
            .load_keypair_from_file(path)
            .map_err(|e| e.to_string())?;

        let pubkey = wallet_service.get_public_key()
            .ok_or_else(|| "Failed to get public key".to_string())?;

        // Get balance
        let balance = wallet_service.get_balance().await
            .map_err(|e| e.to_string())?;

        // Update state
        let mut state = self.state.write();
        state.wallet_service = Some(wallet_service);
        state.wallet = Some(WalletState {
            address: pubkey.clone(),
            sol_balance: balance,
            token_balances: Vec::new(),
        });

        Ok(pubkey)
    }

    /// Generate a new wallet
    pub async fn generate_wallet(&self) -> Result<String, String> {
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

        let mut wallet_service = crate::services::wallet::WalletService::new(&rpc_url);
        let pubkey = wallet_service.generate_new_keypair();

        // Get balance (will be 0 for new wallet)
        let balance = wallet_service.get_balance().await
            .map_err(|e| e.to_string())?;

        // Update state
        let mut state = self.state.write();
        state.wallet_service = Some(wallet_service);
        state.wallet = Some(WalletState {
            address: pubkey.clone(),
            sol_balance: balance,
            token_balances: Vec::new(),
        });

        Ok(pubkey)
    }

    /// Disconnect wallet
    pub async fn disconnect_wallet(&self) {
        let mut state = self.state.write();
        if let Some(ref mut wallet_service) = state.wallet_service {
            wallet_service.disconnect();
        }
        state.wallet_service = None;
        state.wallet = None;
    }

    /// Get the event sender for creating WindowApp instances.
    pub fn event_tx(&self) -> Sender<AppEvent> {
        self.event_tx.clone()
    }
}

impl AppLike for App {
    fn state(&self) -> &Arc<RwLock<AppState>> {
        &self.state
    }
    
    fn window_manager(&self) -> &Arc<RwLock<WindowManager>> {
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
        self.next_screen();
    }
    
    fn previous_screen(&mut self) {
        self.previous_screen();
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

// Include tests from the old app.rs
#[cfg(test)]
mod tests {
    use super::*;

    // ========== Screen Tests ==========

    #[test]
    fn test_screen_all_returns_correct_order() {
        let screens = Screen::all();

        assert_eq!(screens.len(), 4);
        assert_eq!(screens[0], Screen::Auth);
        assert_eq!(screens[1], Screen::Terminal);
        assert_eq!(screens[2], Screen::Wallet);
        assert_eq!(screens[3], Screen::Transactions);
    }

    #[test]
    fn test_screen_title() {
        assert_eq!(Screen::Auth.title(), "Authentication");
        assert_eq!(Screen::Terminal.title(), "DeFi Trading Terminal");
        assert_eq!(Screen::Wallet.title(), "Wallet Management");
        assert_eq!(Screen::Transactions.title(), "Transaction History");
    }

    // ========== Screen Navigation Tests ==========

    #[test]
    fn test_next_screen_cycles_forward() {
        let mut app = App::new();
        let state = app.state.read();
        assert_eq!(state.current_screen, Screen::Auth);
        drop(state);

        app.next_screen();
        let state = app.state.read();
        assert_eq!(state.current_screen, Screen::Terminal);
        drop(state);

        app.next_screen();
        let state = app.state.read();
        assert_eq!(state.current_screen, Screen::Wallet);
        drop(state);

        app.next_screen();
        let state = app.state.read();
        assert_eq!(state.current_screen, Screen::Transactions);
        drop(state);

        // Should wrap around
        app.next_screen();
        let state = app.state.read();
        assert_eq!(state.current_screen, Screen::Auth);
    }

    #[test]
    fn test_previous_screen_cycles_backward() {
        let mut app = App::new();

        // Start at Auth, go back should wrap to Transactions
        app.previous_screen();
        let state = app.state.read();
        assert_eq!(state.current_screen, Screen::Transactions);
        drop(state);

        app.previous_screen();
        let state = app.state.read();
        assert_eq!(state.current_screen, Screen::Wallet);
        drop(state);

        app.previous_screen();
        let state = app.state.read();
        assert_eq!(state.current_screen, Screen::Terminal);
        drop(state);

        app.previous_screen();
        let state = app.state.read();
        assert_eq!(state.current_screen, Screen::Auth);
    }

    #[test]
    fn test_next_then_previous_screen_returns_to_original() {
        let mut app = App::new();

        let original_state = app.state.read();
        let original_screen = original_state.current_screen;
        drop(original_state);

        app.next_screen();
        app.previous_screen();

        let state = app.state.read();
        assert_eq!(state.current_screen, original_screen);
    }

    // ========== Auth State Tests ==========

    #[test]
    fn test_initial_auth_state_is_login() {
        let app = App::new();
        let state = app.state.read();

        match &state.auth {
            AuthState::Login {
                username,
                password,
                error,
                active_field,
            } => {
                assert_eq!(username, "");
                assert_eq!(password, "");
                assert_eq!(error, &None);
                assert_eq!(active_field, &LoginField::Username);
            }
            _ => panic!("Expected Login state in test"),
        }
    }

    #[test]
    fn test_login_field_enum() {
        assert_eq!(LoginField::Username, LoginField::Username);
        assert_eq!(LoginField::Password, LoginField::Password);
        assert_ne!(LoginField::Username, LoginField::Password);
    }

    #[test]
    fn test_signup_field_enum() {
        assert_eq!(SignupField::Username, SignupField::Username);
        assert_eq!(SignupField::Email, SignupField::Email);
        assert_eq!(SignupField::Password, SignupField::Password);
        assert_eq!(SignupField::ConfirmPassword, SignupField::ConfirmPassword);

        assert_ne!(SignupField::Username, SignupField::Email);
        assert_ne!(SignupField::Password, SignupField::ConfirmPassword);
    }

    // ========== Terminal State Tests ==========

    #[test]
    fn test_initial_terminal_state() {
        let app = App::new();
        let state = app.state.read();

        assert_eq!(state.terminal.swap.input_token, "SOL");
        assert_eq!(state.terminal.swap.output_token, "USDC");
        assert_eq!(state.terminal.swap.amount, "");
        assert!(state.terminal.swap.quote.is_none());
        assert!(!state.terminal.prices.is_empty());
        assert_eq!(state.terminal.chart_data.len(), 0);
    }

    #[test]
    fn test_demo_prices_generation() {
        let app = App::new();
        let state = app.state.read();

        // Should have at least SOL, USDC, BTC, ETH
        assert!(state.terminal.prices.len() >= 4);

        // Find SOL price
        let sol_price = state.terminal.prices.iter().find(|p| p.symbol == "SOL");
        assert!(sol_price.is_some(), "SOL price should be in initial prices");

        let sol = sol_price.expect("SOL price should exist in test");
        assert!(sol.price > 0.0);
        assert!(sol.previous_price.is_none()); // Initial state

        // Find USDC price (should be stable ~1.0)
        let usdc_price = state.terminal.prices.iter().find(|p| p.symbol == "USDC");
        assert!(usdc_price.is_some(), "USDC price should be in initial prices");

        let usdc = usdc_price.expect("USDC price should exist in test");
        assert!((usdc.price - 1.0).abs() < 0.01); // Should be close to 1.0
        assert_eq!(usdc.change_24h, 0.0); // Stablecoin should have 0 change
    }

    // ========== Swap Quote Tests ==========

    #[test]
    fn test_swap_quote_creation() {
        let quote = SwapQuote {
            input_amount: 10.0,
            output_amount: 1450.0,
            price_impact: 0.5,
            estimated_fee: 0.25,
        };

        assert_eq!(quote.input_amount, 10.0);
        assert_eq!(quote.output_amount, 1450.0);
        assert_eq!(quote.price_impact, 0.5);
        assert_eq!(quote.estimated_fee, 0.25);
    }

    // ========== PriceData Tests ==========

    #[test]
    fn test_price_data_with_previous_price() {
        let price_data = PriceData {
            symbol: "SOL".to_string(),
            price: 150.0,
            change_24h: 5.0,
            previous_price: Some(145.0),
            source: Some("jupiter".to_string()),
        };

        assert_eq!(price_data.symbol, "SOL");
        assert_eq!(price_data.price, 150.0);
        assert_eq!(price_data.change_24h, 5.0);
        assert_eq!(price_data.previous_price, Some(145.0));
    }

    #[test]
    fn test_price_data_without_previous_price() {
        let price_data = PriceData {
            symbol: "BTC".to_string(),
            price: 64000.0,
            change_24h: -2.5,
            previous_price: None,
            source: Some("pyth".to_string()),
        };

        assert_eq!(price_data.symbol, "BTC");
        assert_eq!(price_data.price, 64000.0);
        assert_eq!(price_data.change_24h, -2.5);
        assert!(price_data.previous_price.is_none());
    }

    // ========== Wallet State Tests ==========

    #[test]
    fn test_initial_wallet_state_is_none() {
        let app = App::new();
        let state = app.state.read();

        assert!(state.wallet.is_none());
    }

    #[test]
    fn test_wallet_state_creation() {
        let wallet = WalletState {
            address: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
            sol_balance: 10.5,
            token_balances: vec![
                TokenBalance {
                    symbol: "USDC".to_string(),
                    amount: 1000.0,
                    usd_value: 1000.0,
                },
            ],
        };

        assert_eq!(wallet.address, "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU");
        assert_eq!(wallet.sol_balance, 10.5);
        assert_eq!(wallet.token_balances.len(), 1);
        assert_eq!(wallet.token_balances[0].symbol, "USDC");
        assert_eq!(wallet.token_balances[0].amount, 1000.0);
    }

    // ========== Transaction State Tests ==========

    #[test]
    fn test_initial_transaction_state_is_empty() {
        let app = App::new();
        let state = app.state.read();

        assert_eq!(state.transactions.len(), 0);
    }

    #[test]
    fn test_transaction_item_creation() {
        let tx = TransactionItem {
            signature: "5J7B...".to_string(),
            timestamp: 1640000000,
            tx_type: "Swap".to_string(),
            status: "Confirmed".to_string(),
            amount: "10.5 SOL".to_string(),
        };

        assert_eq!(tx.signature, "5J7B...");
        assert_eq!(tx.timestamp, 1640000000);
        assert_eq!(tx.tx_type, "Swap");
        assert_eq!(tx.status, "Confirmed");
        assert_eq!(tx.amount, "10.5 SOL");
    }

    // ========== Auth Token Tests ==========

    #[test]
    fn test_initial_auth_token_is_none() {
        let app = App::new();
        let state = app.state.read();

        assert!(state.auth_token.is_none());
    }

    // ========== API Client Tests ==========

    #[test]
    fn test_initial_api_client_is_some() {
        let app = App::new();
        let state = app.state.read();

        assert!(state.api_client.is_some());
    }

    // ========== AppEvent Tests ==========

    #[tokio::test]
    async fn test_app_event_prices_updated_stores_previous_prices() {
        let mut app = App::new();

        // Set initial prices
        {
            let mut state = app.state.write();
            state.terminal.prices = vec![
                PriceData {
                    symbol: "SOL".to_string(),
                    price: 145.0,
                    change_24h: 5.0,
                    previous_price: None,
                    source: Some("jupiter".to_string()),
                },
            ];
        }

        // Simulate price update
        let new_prices = vec![
            PriceData {
                symbol: "SOL".to_string(),
                price: 150.0,
                change_24h: 5.0,
                previous_price: None, // Will be set by the update logic
                source: Some("jupiter".to_string()),
            },
        ];

        app.handle_event(AppEvent::PricesUpdated(new_prices));

        // Check that previous price was stored
        let state = app.state.read();
        assert_eq!(state.terminal.prices[0].price, 150.0);
        assert_eq!(state.terminal.prices[0].previous_price, Some(145.0));
    }

    #[tokio::test]
    async fn test_app_event_login_result_success() {
        let mut app = App::new();

        let auth_response = shared::AuthResponse {
            user: shared::UserInfo {
                id: "1".to_string(),
                username: "testuser".to_string(),
                email: "test@example.com".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                wallet_address: None,
            },
            token: "jwt-token-here".to_string(),
            message: "Login successful".to_string(),
            wallet_setup_required: None,
            wallet_setup_token: None,
        };

        app.handle_event(AppEvent::LoginResult(Ok(auth_response.clone())));

        let state = app.state.read();
        assert_eq!(state.auth_token, Some("jwt-token-here".to_string()));
        assert_eq!(state.current_screen, Screen::Terminal);
    }

    #[tokio::test]
    async fn test_app_event_login_result_error() {
        let mut app = App::new();

        app.handle_event(AppEvent::LoginResult(Err("Invalid credentials".to_string())));

        let state = app.state.read();
        assert!(state.auth_token.is_none());
        assert_eq!(state.current_screen, Screen::Auth);

        match &state.auth {
            AuthState::Login { error, .. } => {
                assert_eq!(error, &Some("Invalid credentials".to_string()));
            }
            _ => panic!("Expected Login state in test"),
        }
    }

    #[tokio::test]
    async fn test_app_event_loading_updates_error_field() {
        let mut app = App::new();

        app.handle_event(AppEvent::Loading("Connecting...".to_string()));

        let state = app.state.read();
        match &state.auth {
            AuthState::Login { error, .. } => {
                assert_eq!(error, &Some("Connecting...".to_string()));
            }
            _ => panic!("Expected Login state in test"),
        }
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_app_creation_and_initial_state() {
        let app = App::new();
        let state = app.state.read();

        // Verify all initial state
        assert_eq!(state.current_screen, Screen::Auth);
        assert!(matches!(state.auth, AuthState::Login { .. }));
        assert!(state.auth_token.is_none());
        assert!(state.wallet.is_none());
        assert_eq!(state.transactions.len(), 0);
        assert!(state.api_client.is_some());
        assert!(!state.terminal.prices.is_empty());
    }

    #[test]
    fn test_screen_navigation_full_cycle() {
        let mut app = App::new();

        // Cycle through all screens
        let screens = [Screen::Auth, Screen::Terminal, Screen::Wallet, Screen::Transactions];

        for (i, expected_screen) in screens.iter().enumerate() {
            let state = app.state.read();
            assert_eq!(state.current_screen, *expected_screen);
            drop(state);

            if i < screens.len() - 1 {
                app.next_screen();
            }
        }

        // Verify wrap around
        app.next_screen();
        let state = app.state.read();
        assert_eq!(state.current_screen, Screen::Auth);
    }
}

