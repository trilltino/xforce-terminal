//! # Application Events
//!
//! Event types for async task communication between background tasks and the main thread.

use crate::app::state::{PriceData, SwapQuote, TokenInfo, SwapHistoryItem};

/// Async task results sent to main thread
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Login completed
    LoginResult(Result<shared::AuthResponse, String>),
    /// Signup completed
    SignupResult(Result<shared::AuthResponse, String>),
    /// Wallet connection status checked
    WalletStatusChecked(Result<shared::AuthResponse, String>),
    /// Prices updated (batch)
    PricesUpdated(Vec<PriceData>),
    /// Single price updated (from WebSocket stream)
    PriceUpdated(PriceData),
    /// Swap quote received
    SwapQuoteResult(Result<SwapQuote, String>),
    /// Token list received
    TokenListResult(Result<Vec<TokenInfo>, String>),
    /// Swap history received
    SwapHistoryResult(Result<Vec<SwapHistoryItem>, String>),
    /// Candles (OHLC data) received
    CandlesResult(Result<Vec<shared::dto::OHLC>, String>),
    /// Loading state
    Loading(String),
    /// WebSocket status update
    WebSocketStatusUpdate(crate::app::WebSocketStatus),
}

