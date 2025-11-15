//! # API Client
//!
//! Main HTTP client for backend API communication.

use reqwest::Client;
use crate::core::service::ApiService;

/// Base URL for backend API server
const API_BASE_URL: &str = "http://127.0.0.1:3001";

/// HTTP client for communicating with the backend API server.
///
/// This client handles all REST API calls and maintains a connection pool
/// for efficient HTTP/2 multiplexing.
pub struct ApiClient {
    pub(crate) client: Client,
}

impl ApiClient {
    /// Create a new API client with default configuration.
    ///
    /// The client is configured with a 10 second timeout to prevent freezing.
    pub fn new() -> Self {
        // Create client with 10 second timeout to prevent freezing
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    /// Get the base URL for API requests.
    pub(crate) fn base_url() -> &'static str {
        API_BASE_URL
    }
}

// Implement ApiService trait for ApiClient
#[async_trait::async_trait]
impl ApiService for ApiClient {
    async fn login(&self, email_or_username: String, password: String) -> Result<shared::AuthResponse, String> {
        crate::services::api::auth::login(self, email_or_username, password).await
    }
    
    async fn signup(&self, username: String, email: String, password: String) -> Result<shared::AuthResponse, String> {
        crate::services::api::auth::signup(self, username, email, password).await
    }
    
    async fn get_prices(&self, symbols: &[&str]) -> Result<crate::services::api::market::PriceResponse, String> {
        crate::services::api::market::get_prices(self, symbols).await
    }
    
    async fn get_wallet_balance(&self, address: &str) -> Result<crate::services::api::wallet::WalletBalance, String> {
        crate::services::api::wallet::get_wallet_balance(self, address).await
    }
    
    async fn get_transaction_history(&self, address: &str, limit: usize) -> Result<crate::services::api::wallet::TransactionHistory, String> {
        crate::services::api::wallet::get_transaction_history(self, address, limit).await
    }
    
    async fn get_swap_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<crate::services::api::swap::SwapQuoteResponse, String> {
        crate::services::api::swap::get_swap_quote(self, input_mint, output_mint, amount, slippage_bps).await
    }
    
    async fn execute_swap(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
        user_pubkey: &str,
        jwt_token: &str,
    ) -> Result<crate::services::api::swap::SwapExecuteResponse, String> {
        crate::services::api::swap::execute_swap(self, input_mint, output_mint, amount, slippage_bps, user_pubkey, jwt_token).await
    }
    
    async fn submit_transaction(
        &self,
        signed_transaction: String,
        input_mint: String,
        output_mint: String,
        input_amount: i64,
        output_amount: i64,
        price_impact: Option<f64>,
        slippage_bps: Option<i32>,
        jwt_token: &str,
    ) -> Result<crate::services::api::swap::TransactionSubmitResponse, String> {
        crate::services::api::swap::submit_transaction(self, signed_transaction, input_mint, output_mint, input_amount, output_amount, price_impact, slippage_bps, jwt_token).await
    }
    
    async fn get_token_balances(&self, address: &str) -> Result<Vec<crate::services::api::wallet::TokenBalance>, String> {
        crate::services::api::wallet::get_token_balances(self, address).await
    }
    
    async fn get_token_list(&self) -> Result<Vec<crate::services::api::market::TokenListItem>, String> {
        crate::services::api::market::get_token_list(self).await
    }
    
    async fn get_swap_history(&self, jwt_token: &str, limit: usize) -> Result<Vec<crate::services::api::swap::SwapHistoryItem>, String> {
        crate::services::api::swap::get_swap_history(self, jwt_token, limit).await
    }
    
    async fn get_candles(&self, symbol: &str, timeframe: &str, limit: usize) -> Result<Vec<shared::dto::market::OHLC>, String> {
        crate::services::api::market::get_candles(self, symbol, timeframe, limit).await
    }
}

