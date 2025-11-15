//! # Application State Types
//!
//! All state-related types for the application, including screens, authentication,
//! terminal state, wallet state, and transaction state.

use std::sync::Arc;

/// Application screens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Landing screen (splash/welcome)
    Landing,
    /// Authentication screen (login/signup)
    Auth,
    /// Main trading terminal with charts and prices
    Terminal,
    /// Pyth Network price feed screen
    PythFeed,
    /// Jupiter WebSocket price feed screen
    JupiterFeed,
    /// Wallet management screen
    Wallet,
    /// Transaction history screen
    Transactions,
    /// SPL Token management screen
    Tokens,
    /// Messaging screen with friends and direct messages
    Messaging,
    /// AI Chat screen for talking to the AI assistant
    AIChat,
    /// Settings screen for UI customization
    Settings,
    /// Live chart screen with real-time candlestick updates
    LiveChart,
    /// Live assets list screen with simple asset display
    LiveAssets,
    /// Live data table screen with comprehensive metrics
    LiveTable,
}

impl Screen {
    /// Get all screens in Tab navigation order
    pub fn all() -> &'static [Screen] {
        &[
            Screen::Landing,
            Screen::Auth,
            Screen::Terminal,
            Screen::PythFeed,
            Screen::JupiterFeed,
            Screen::Wallet,
            Screen::Transactions,
            Screen::Tokens,
            Screen::Messaging,
            Screen::AIChat,
            Screen::Settings,
            Screen::LiveChart,
            Screen::LiveAssets,
            Screen::LiveTable,
        ]
    }

    /// Get screen title for header display
    pub fn title(&self) -> &'static str {
        match self {
            Screen::Landing => "Welcome",
            Screen::Auth => "Authentication",
            Screen::Terminal => "DeFi Trading Terminal",
            Screen::PythFeed => "Pyth Network Feed",
            Screen::JupiterFeed => "Jupiter WebSocket Feed",
            Screen::Wallet => "Wallet Management",
            Screen::Transactions => "Transaction History",
            Screen::Tokens => "SPL Tokens",
            Screen::Messaging => "Messaging",
            Screen::AIChat => "AI Assistant",
            Screen::Settings => "Settings",
            Screen::LiveChart => "Live Chart",
            Screen::LiveAssets => "Live Assets",
            Screen::LiveTable => "Live Table",
        }
    }
}

/// Authentication sub-state
#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    /// Login form
    Login {
        username: String,
        password: String,
        error: Option<String>,
        active_field: LoginField,
    },
    /// Signup form
    Signup {
        username: String,
        email: String,
        password: String,
        confirm_password: String,
        error: Option<String>,
        active_field: SignupField,
    },
}

/// Active field in login form
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginField {
    Username,
    Password,
}

/// Active field in signup form
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignupField {
    Username,
    Email,
    Password,
    ConfirmPassword,
}

/// Active tab in terminal screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapTab {
    /// Simple swap interface (default)
    Simple,
    /// Advanced swap with limit orders
    Advanced,
    /// Swap transaction history
    History,
    /// Token explorer and search
    TokenExplorer,
}

impl SwapTab {
    /// Get all swap tabs in order
    pub fn all() -> &'static [SwapTab] {
        &[
            SwapTab::Simple,
            SwapTab::Advanced,
            SwapTab::History,
            SwapTab::TokenExplorer,
        ]
    }

    /// Get tab display name
    pub fn title(&self) -> &'static str {
        match self {
            SwapTab::Simple => "Simple Swap",
            SwapTab::Advanced => "Advanced",
            SwapTab::History => "History",
            SwapTab::TokenExplorer => "Token Explorer",
        }
    }
}

/// Active field in swap interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapField {
    /// Amount input field
    Amount,
    /// Slippage tolerance input
    Slippage,
    /// Token search field (in popup)
    TokenSearch,
}

/// Target for token picker popup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenPickerTarget {
    /// Picking input token
    Input,
    /// Picking output token
    Output,
}

/// Token information for selection
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub symbol: String,
    pub name: String,
    pub mint: String,
    pub price: f64,
    pub balance: f64,
    pub change_24h: f64,
    pub is_favorite: bool,
}

/// Swap history item
#[derive(Debug, Clone)]
pub struct SwapHistoryItem {
    pub signature: String,
    pub timestamp: i64,
    pub input_symbol: String,
    pub output_symbol: String,
    pub input_amount: f64,
    pub output_amount: f64,
    pub status: String,
}

/// Comprehensive swap state
#[derive(Debug, Clone)]
pub struct SwapState {
    /// Active swap tab
    pub active_tab: SwapTab,
    /// Active input field
    pub active_field: SwapField,
    /// Input token symbol
    pub input_token: String,
    /// Input token mint address
    pub input_mint: String,
    /// Output token symbol
    pub output_token: String,
    /// Output token mint address
    pub output_mint: String,
    /// Amount to swap (as string for input handling)
    pub amount: String,
    /// Slippage tolerance in basis points (default 50 = 0.5%)
    pub slippage_bps: u16,
    /// Current swap quote (if any)
    pub quote: Option<SwapQuote>,
    /// Quote is currently being fetched
    pub quote_loading: bool,
    /// Show token picker popup
    pub show_token_picker: bool,
    /// Token picker is for input or output
    pub token_picker_for: TokenPickerTarget,
    /// Available tokens for selection
    pub token_list: Vec<TokenInfo>,
    /// Filter text for token search
    pub token_filter: String,
    /// Selected index in token picker
    pub selected_token_index: usize,
    /// Swap transaction history
    pub swap_history: Vec<SwapHistoryItem>,
    /// Last quote fetch timestamp
    pub last_quote_fetch: std::time::Instant,
}

/// WebSocket connection status details
#[derive(Debug, Clone)]
pub struct WebSocketStatus {
    /// Connection state
    pub state: WebSocketState,
    /// Number of connection attempts
    pub connection_attempts: u64,
    /// Last error message (if any)
    pub last_error: Option<String>,
    /// Last successful connection time
    pub last_connected: Option<std::time::Instant>,
    /// Total messages received
    pub messages_received: u64,
    /// Last message time
    pub last_message: Option<std::time::Instant>,
}

/// WebSocket connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketState {
    /// Not connected, not attempting
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Successfully connected
    Connected,
    /// Connection failed, retrying
    Reconnecting,
    /// Permanently disabled (max retries reached)
    Disabled,
}

impl Default for WebSocketStatus {
    fn default() -> Self {
        Self {
            state: WebSocketState::Disconnected,
            connection_attempts: 0,
            last_error: None,
            last_connected: None,
            messages_received: 0,
            last_message: None,
        }
    }
}

impl Default for SwapState {
    fn default() -> Self {
        Self {
            active_tab: SwapTab::Simple,
            active_field: SwapField::Amount,
            input_token: "SOL".to_string(),
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_token: "USDC".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount: String::new(),
            slippage_bps: 50, // 0.5% default
            quote: None,
            quote_loading: false,
            show_token_picker: false,
            token_picker_for: TokenPickerTarget::Input,
            token_list: Vec::new(),
            token_filter: String::new(),
            selected_token_index: 0,
            swap_history: Vec::new(),
            last_quote_fetch: std::time::Instant::now(),
        }
    }
}

/// Terminal screen state (trading view)
#[derive(Debug, Clone)]
pub struct TerminalState {
    /// Comprehensive swap state
    pub swap: SwapState,
    /// Price data for all tokens
    pub prices: Vec<PriceData>,
    /// Chart data (OHLC candles) - real data from API
    pub chart_data: Vec<shared::dto::OHLC>,
    /// SOL candles for main chart
    pub sol_candles: Vec<shared::dto::OHLC>,
    /// Selected chart timeframe
    pub chart_timeframe: shared::dto::market::Timeframe,
    /// Chart loading state
    pub chart_loading: bool,
    /// Active chart for display (legacy, may be removed)
    pub active_chart: Option<crate::ui::chart::ChartData>,
    /// Last price update timestamp
    pub last_price_update: std::time::Instant,
    /// Flag to prevent concurrent price fetches (prevents task pileup)
    pub fetching_prices: bool,
    /// Swap panel visibility (collapsible)
    pub swap_panel_open: bool,
}

/// Swap quote information
#[derive(Debug, Clone)]
pub struct SwapQuote {
    pub input_amount: f64,
    pub output_amount: f64,
    pub price_impact: f64,
    pub estimated_fee: f64,
}

/// Price data for a single token
#[derive(Debug, Clone)]
pub struct PriceData {
    pub symbol: String,
    pub price: f64,
    pub change_24h: f64,
    /// Previous price for change detection (triggers flash effects)
    pub previous_price: Option<f64>,
    /// Price source (e.g., "pyth", "jupiter")
    pub source: Option<String>,
}

/// Global application state
pub struct AppState {
    /// Current active screen
    pub current_screen: Screen,
    /// Authentication state
    pub auth: AuthState,
    /// Terminal (trading) state
    pub terminal: TerminalState,
    /// Wallet balances and info
    pub wallet: Option<WalletState>,
    /// Transaction history
    pub transactions: Vec<TransactionItem>,
    /// JWT token (once logged in)
    pub auth_token: Option<String>,
    /// Current user info (from JWT)
    pub current_user: Option<CurrentUser>,
    /// API client
    pub api_client: Option<Arc<crate::services::api::ApiClient>>,
    /// Wallet service for signing transactions
    pub wallet_service: Option<crate::services::wallet::WalletService>,
    /// Credentials for polling wallet status (username, password)
    pub polling_credentials: Option<(String, String)>,
    /// Pending notifications to display (level, message)
    pub pending_notifications: Vec<(String, String)>,
    /// WebSocket connection status for price stream
    pub websocket_connected: bool,
    /// WebSocket connection status details
    pub websocket_status: WebSocketStatus,
    /// Messaging state
    pub messaging: MessagingState,
    /// AI Chat state
    pub ai_chat: AIChatState,
    /// Settings state (theme configuration, etc.)
    pub settings: SettingsState,
    /// Debug overlay visibility (toggled with Ctrl+D)
    pub debug_overlay_visible: bool,
    /// Flag to request immediate repaint (set when price updates arrive)
    pub needs_immediate_repaint: bool,
    /// Timestamp of last price update for flash effect tracking
    pub last_price_update_time: std::time::Instant,
    /// Navigation bar: Selected token symbol (defaults to SOL)
    pub nav_bar_selected_token: Option<String>,
    /// Navigation bar: Show token picker dropdown
    pub nav_bar_show_token_picker: bool,
}

impl AppState {
    /// Check if user is authenticated (has valid auth token)
    pub fn is_authenticated(&self) -> bool {
        self.auth_token.is_some()
    }

    /// Check if a screen requires authentication
    pub fn requires_auth(screen: Screen) -> bool {
        matches!(screen, Screen::Terminal | Screen::PythFeed | Screen::JupiterFeed | Screen::Wallet | Screen::Transactions | Screen::Tokens | Screen::Messaging | Screen::AIChat)
    }
}

// Manual Clone implementation to handle non-cloneable wallet_service
impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            current_screen: self.current_screen,
            auth: self.auth.clone(),
            terminal: self.terminal.clone(),
            wallet: self.wallet.clone(),
            transactions: self.transactions.clone(),
            auth_token: self.auth_token.clone(),
            current_user: self.current_user.clone(),
            api_client: self.api_client.clone(),
            // IMPORTANT: wallet_service is intentionally NOT cloned (contains Keypair secret)
            // Rendering doesn't need access to signing capabilities anyway
            wallet_service: None,
            polling_credentials: self.polling_credentials.clone(),
            pending_notifications: self.pending_notifications.clone(),
            websocket_connected: self.websocket_connected,
            websocket_status: self.websocket_status.clone(),
            messaging: self.messaging.clone(),
            ai_chat: self.ai_chat.clone(),
            settings: self.settings.clone(),
            debug_overlay_visible: self.debug_overlay_visible,
            needs_immediate_repaint: self.needs_immediate_repaint,
            last_price_update_time: self.last_price_update_time,
            nav_bar_selected_token: self.nav_bar_selected_token.clone(),
            nav_bar_show_token_picker: self.nav_bar_show_token_picker,
        }
    }
}

/// Wallet state
#[derive(Debug, Clone)]
pub struct WalletState {
    pub address: String,
    pub sol_balance: f64,
    pub token_balances: Vec<TokenBalance>,
}

/// Token balance in wallet
#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub symbol: String,
    pub amount: f64,
    pub usd_value: f64,
}

/// Transaction history item
#[derive(Debug, Clone)]
pub struct TransactionItem {
    pub signature: String,
    pub timestamp: i64,
    pub tx_type: String,
    pub status: String,
    pub amount: String,
}

/// Current user information
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: i64,
    pub username: String,
}

/// Messaging state
#[derive(Debug, Clone)]
pub struct MessagingState {
    /// List of friends
    pub friends: Vec<shared::dto::messaging::Friend>,
    /// Pending friend requests (incoming)
    pub incoming_requests: Vec<shared::dto::messaging::FriendRequest>,
    /// Pending friend requests (outgoing)
    pub outgoing_requests: Vec<shared::dto::messaging::FriendRequest>,
    /// Currently active conversation ID
    pub active_conversation_id: Option<String>,
    /// Messages by conversation ID
    pub messages: std::collections::HashMap<String, Vec<shared::dto::messaging::Message>>,
    /// Selected user ID for conversation
    pub selected_user_id: Option<i64>,
    /// Search query for finding users
    pub search_query: String,
    /// Search results
    pub search_results: Vec<shared::dto::messaging::UserSearchResult>,
    /// Typing indicators by conversation ID
    pub typing_indicators: std::collections::HashMap<String, (i64, String)>,
    /// Current message input text
    pub message_input: String,
}

impl Default for MessagingState {
    fn default() -> Self {
        Self {
            friends: vec![],
            incoming_requests: vec![],
            outgoing_requests: vec![],
            active_conversation_id: None,
            messages: std::collections::HashMap::new(),
            selected_user_id: None,
            search_query: String::new(),
            search_results: vec![],
            typing_indicators: std::collections::HashMap::new(),
            message_input: String::new(),
        }
    }
}

/// Settings state for theme and UI configuration
#[derive(Debug, Clone)]
pub struct SettingsState {
    /// Current theme configuration
    pub theme_config: crate::ui::theme::ThemeConfig,
    /// Path to config file
    pub config_path: String,
    /// Whether there are unsaved changes
    pub unsaved_changes: bool,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            theme_config: crate::ui::theme::ThemeConfig::default(),
            config_path: "./xterminal-config.json".to_string(),
            unsaved_changes: false,
        }
    }
}

/// AI Chat state
#[derive(Debug, Clone)]
pub struct AIChatState {
    /// Conversation ID for AI chat (uses special format: "ai:{user_id}")
    pub conversation_id: Option<String>,
    /// Messages in the AI conversation
    pub messages: Vec<shared::dto::messaging::Message>,
    /// Current message input text
    pub message_input: String,
    /// Whether the AI is currently typing/responding
    pub ai_typing: bool,
    /// Whether we're subscribed to conversation updates
    pub subscribed: bool,
}

impl Default for AIChatState {
    fn default() -> Self {
        Self {
            conversation_id: None,
            messages: vec![],
            message_input: String::new(),
            ai_typing: false,
            subscribed: false,
        }
    }
}

