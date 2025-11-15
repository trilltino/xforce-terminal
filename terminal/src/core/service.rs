//! # Service Traits
//!
//! Traits for dependency injection, enabling better testability and modularity.

use shared::AuthResponse;
use crate::services::api::{PriceResponse, SwapQuoteResponse, SwapExecuteResponse, SwapHistoryItem, TokenListItem, WalletBalance, TokenBalance, TransactionHistory};
use crate::services::wallet::{WalletError, WalletService as WalletServiceImpl};
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction;
use async_trait::async_trait;

/// Trait for API service operations
/// 
/// This trait allows for dependency injection and mocking in tests.
///
/// Note: This trait is exported for public API and testing purposes.
/// It may appear unused but enables dependency injection patterns.
#[async_trait]
#[allow(dead_code)] // Exported for dependency injection and testing
pub trait ApiService: Send + Sync {
    /// Login with username/email and password
    async fn login(&self, email_or_username: String, password: String) -> Result<AuthResponse, String>;
    
    /// Sign up a new user
    async fn signup(&self, username: String, email: String, password: String) -> Result<AuthResponse, String>;
    
    /// Get prices for multiple symbols
    async fn get_prices(&self, symbols: &[&str]) -> Result<PriceResponse, String>;
    
    /// Get wallet SOL balance
    async fn get_wallet_balance(&self, address: &str) -> Result<WalletBalance, String>;
    
    /// Get transaction history for an address
    async fn get_transaction_history(&self, address: &str, limit: usize) -> Result<TransactionHistory, String>;
    
    /// Get swap quote from Jupiter
    async fn get_swap_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<SwapQuoteResponse, String>;
    
    /// Execute swap and get unsigned transaction
    async fn execute_swap(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
        user_pubkey: &str,
        jwt_token: &str,
    ) -> Result<SwapExecuteResponse, String>;
    
    /// Submit signed transaction
    /// Submit a signed transaction to the backend
    ///
    /// Note: This function has many parameters which is acceptable for transaction
    /// submission as all fields are required for proper tracking.
    #[allow(clippy::too_many_arguments)] // All parameters are required for transaction tracking
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
    ) -> Result<crate::services::api::TransactionSubmitResponse, String>;
    
    /// Get SPL token balances for an address
    async fn get_token_balances(&self, address: &str) -> Result<Vec<TokenBalance>, String>;
    
    /// Get list of available tokens
    async fn get_token_list(&self) -> Result<Vec<TokenListItem>, String>;
    
    /// Get swap history for authenticated user
    async fn get_swap_history(&self, jwt_token: &str, limit: usize) -> Result<Vec<SwapHistoryItem>, String>;
    
    /// Get OHLC candlestick data for a token
    async fn get_candles(&self, symbol: &str, timeframe: &str, limit: usize) -> Result<Vec<shared::dto::market::OHLC>, String>;
}

/// Trait for wallet service operations
/// 
/// This trait allows for dependency injection and mocking in tests.
///
/// Note: This trait is exported for public API and testing purposes.
/// It may appear unused but enables dependency injection patterns.
#[async_trait]
#[allow(dead_code)] // Exported for dependency injection and testing
pub trait WalletService: Send + Sync {
    /// Get the public key of the connected wallet (as string)
    fn get_public_key(&self) -> Option<String>;
    
    /// Sign a transaction
    fn sign_transaction(&self, transaction: &mut Transaction) -> Result<Signature, WalletError>;
    
    /// Get wallet balance
    async fn get_balance(&self) -> Result<f64, WalletError>;
    
    /// Disconnect the wallet
    fn disconnect(&mut self);
}

// Implement the trait for the concrete WalletService
#[async_trait]
impl WalletService for WalletServiceImpl {
    fn get_public_key(&self) -> Option<String> {
        WalletServiceImpl::get_public_key(self)
    }
    
    fn sign_transaction(&self, transaction: &mut Transaction) -> Result<Signature, WalletError> {
        WalletServiceImpl::sign_transaction(self, transaction)
    }
    
    async fn get_balance(&self) -> Result<f64, WalletError> {
        WalletServiceImpl::get_balance(self).await
    }
    
    fn disconnect(&mut self) {
        WalletServiceImpl::disconnect(self)
    }
}

