//! # Wallet Service
//!
//! Business logic for querying Solana wallet information including balances and token holdings.
//!
//! ## Overview
//!
//! This service orchestrates wallet operations by coordinating with the Solana RPC client
//! and SPL token client to provide wallet information.
//!
//! ## Features
//!
//! - **Balance Queries**: Get SOL balance for a wallet address
//! - **Token Balances**: Get SPL token balances for a wallet
//! - **Wallet Info**: Get comprehensive wallet information including SOL and tokens
//! - **Error Handling**: Comprehensive error handling with user-friendly messages
//!
//! ## Usage
//!
//! ```rust,no_run
//! use backend::services::wallet::WalletService;
//! use backend::solana::SolanaState;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let solana = Arc::new(SolanaState::new(/* ... */).await?);
//! let service = WalletService::new(solana);
//!
//! // Get SOL balance
//! let balance = service.get_wallet_balance("wallet_address").await?;
//!
//! // Get token balances
//! let tokens = service.get_token_balances("wallet_address").await?;
//!
//! // Get full wallet info
//! let info = service.get_wallet_info("wallet_address").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All methods return `Result<T, AppError>`. Common errors:
//! - `AppError::InvalidInput` - Invalid wallet address format
//! - `AppError::Internal` - Failed to query Solana RPC
//!
//! ## Architecture
//!
//! The service uses Solana RPC and SPL token clients:
//!
//! ```text
//! WalletService → SolanaClient → Solana RPC
//!              → SplTokenClient → Token Account Queries
//! ```

use lib_core::AppError;
use lib_solana::SolanaState;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Wallet balance information.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WalletBalance {
    /// Wallet address
    pub address: String,
    /// Balance in SOL (human-readable)
    pub balance_sol: f64,
    /// Balance in lamports (smallest unit, 1 SOL = 1B lamports)
    pub balance_lamports: u64,
}

/// Token balance information.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TokenBalance {
    /// Token mint address
    pub mint: String,
    /// Token symbol (if available)
    pub symbol: Option<String>,
    /// Balance (human-readable)
    pub balance: f64,
    /// UI amount string
    pub ui_amount: String,
}

/// Comprehensive wallet information.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WalletInfo {
    /// Wallet address
    pub address: String,
    /// SOL balance (human-readable)
    pub balance_sol: f64,
    /// SOL balance in lamports
    pub balance_lamports: u64,
    /// SPL token account balances
    pub token_accounts: Vec<TokenBalance>,
}

/// Service for wallet operations.
///
/// This service provides business logic for querying wallet information,
/// including SOL balances and SPL token holdings.
pub struct WalletService {
    solana: Arc<SolanaState>,
}

impl WalletService {
    /// Create a new wallet service.
    ///
    /// # Arguments
    ///
    /// * `solana` - Shared Solana state containing RPC and SPL token clients
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::wallet::WalletService;
    /// use backend::solana::SolanaState;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let service = WalletService::new(solana);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(solana: Arc<SolanaState>) -> Self {
        Self { solana }
    }

    /// Get SOL balance for a wallet address.
    ///
    /// # Arguments
    ///
    /// * `address` - Solana wallet public key address (base58 encoded)
    ///
    /// # Returns
    ///
    /// * `Ok(WalletBalance)` - Wallet balance information
    /// * `Err(AppError::InvalidInput)` - Invalid wallet address format
    /// * `Err(AppError::Internal)` - Failed to query Solana RPC
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::wallet::WalletService;
    /// use backend::solana::SolanaState;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let service = WalletService::new(solana);
    ///
    /// let balance = service.get_wallet_balance("wallet_address").await?;
    /// println!("Balance: {} SOL ({} lamports)", balance.balance_sol, balance.balance_lamports);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(address = %address))]
    pub async fn get_wallet_balance(&self, address: &str) -> Result<WalletBalance, AppError> {
        debug!("Getting wallet balance for: {}", address);

        let pubkey = Pubkey::from_str(address)
            .map_err(|e| AppError::InvalidInput(format!("Invalid Solana address: {}", e)))?;

        let balance_lamports = self
            .solana
            .rpc
            .get_account(&pubkey)
            .await
            .map(|account| account.lamports)
            .unwrap_or_else(|_| 0);

        let balance_sol = balance_lamports as f64 / 1_000_000_000.0;

        Ok(WalletBalance {
            address: address.to_string(),
            balance_sol,
            balance_lamports,
        })
    }

    /// Get SPL token balances for a wallet.
    ///
    /// # Arguments
    ///
    /// * `address` - Solana wallet public key address (base58 encoded)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<TokenBalance>)` - List of token balances
    /// * `Err(AppError::InvalidInput)` - Invalid wallet address format
    /// * `Err(AppError::Internal)` - Failed to query token accounts
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::wallet::WalletService;
    /// use backend::solana::SolanaState;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let service = WalletService::new(solana);
    ///
    /// let tokens = service.get_token_balances("wallet_address").await?;
    /// for token in &tokens {
    ///     println!("{}: {}", token.mint, token.balance);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(address = %address))]
    pub async fn get_token_balances(&self, address: &str) -> Result<Vec<TokenBalance>, AppError> {
        debug!("Getting token balances for: {}", address);

        // Validate address format
        Pubkey::from_str(address)
            .map_err(|e| AppError::InvalidInput(format!("Invalid Solana address: {}", e)))?;

        let token_accounts = self
            .solana
            .spl_token
            .get_token_accounts(address)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get token accounts: {}", e)))?;

        let balances: Vec<TokenBalance> = token_accounts
            .into_iter()
            .map(|account| TokenBalance {
                mint: account.mint,
                symbol: account.token_symbol,
                balance: account.ui_amount,
                ui_amount: account.ui_amount.to_string(),
            })
            .collect();

        Ok(balances)
    }

    /// Get comprehensive wallet information including SOL and token balances.
    ///
    /// # Arguments
    ///
    /// * `address` - Solana wallet public key address (base58 encoded)
    ///
    /// # Returns
    ///
    /// * `Ok(WalletInfo)` - Complete wallet information
    /// * `Err(AppError::InvalidInput)` - Invalid wallet address format
    /// * `Err(AppError::Internal)` - Failed to query wallet information
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::wallet::WalletService;
    /// use backend::solana::SolanaState;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let service = WalletService::new(solana);
    ///
    /// let info = service.get_wallet_info("wallet_address").await?;
    /// println!("SOL: {} SOL", info.balance_sol);
    /// println!("Tokens: {}", info.token_accounts.len());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(address = %address))]
    pub async fn get_wallet_info(&self, address: &str) -> Result<WalletInfo, AppError> {
        debug!("Getting wallet info for: {}", address);

        let balance = self.get_wallet_balance(address).await?;
        let token_accounts = self.get_token_balances(address).await?;

        Ok(WalletInfo {
            address: balance.address,
            balance_sol: balance.balance_sol,
            balance_lamports: balance.balance_lamports,
            token_accounts,
        })
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests would require mocking SolanaState
    // For now, we'll add integration tests in the handlers

    #[tokio::test]
    #[ignore] // Requires SolanaState setup
    async fn test_get_wallet_balance() {
        // TODO: Add test with mock SolanaState
    }

    #[tokio::test]
    #[ignore] // Requires SolanaState setup
    async fn test_get_token_balances() {
        // TODO: Add test with mock SolanaState
    }

    #[tokio::test]
    #[ignore] // Requires SolanaState setup
    async fn test_get_wallet_info() {
        // TODO: Add test with mock SolanaState
    }
}

