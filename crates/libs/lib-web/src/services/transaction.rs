//! # Transaction Service
//!
//! Business logic for submitting Solana transactions and querying transaction history.
//!
//! ## Overview
//!
//! This service orchestrates transaction operations by coordinating with the Solana RPC client
//! and database to submit transactions and track transaction history.
//!
//! ## Features
//!
//! - **Transaction Submission**: Submit signed transactions to Solana
//! - **Transaction History**: Query transaction history for a wallet
//! - **Database Recording**: Record swap transactions in the database
//! - **Error Handling**: Comprehensive error handling with user-friendly messages
//!
//! ## Usage
//!
//! ```rust,no_run
//! use backend::services::transaction::TransactionService;
//! use backend::solana::SolanaState;
//! use backend::database::DbPool;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let solana = Arc::new(SolanaState::new(/* ... */).await?);
//! let pool = /* ... */;
//! let service = TransactionService::new(solana, pool);
//!
//! // Submit transaction
//! let result = service.submit_transaction(/* ... */).await?;
//!
//! // Get transaction history
//! let history = service.get_transaction_history("wallet_address", 10).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All methods return `Result<T, AppError>`. Common errors:
//! - `AppError::InvalidInput` - Invalid transaction format
//! - `AppError::Internal` - Failed to submit transaction or query history
//!
//! ## Architecture
//!
//! The service uses Solana RPC and database:
//!
//! ```text
//! TransactionService → SolanaClient → Solana RPC
//!                   → Database → Transaction Records
//! ```

use lib_core::AppError;
use lib_solana::SolanaState;
use lib_core::DbPool;
use solana_sdk::{
    pubkey::Pubkey,
    transaction::Transaction,
};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Transaction submission result.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransactionSubmitResult {
    /// Transaction signature (unique identifier on Solana)
    pub signature: String,
    /// Transaction status ("pending" initially)
    pub status: String,
}

/// Transaction summary information.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransactionSummary {
    /// Transaction signature
    pub signature: String,
    /// Blockchain slot number where transaction was processed
    pub slot: u64,
    /// Unix timestamp of when transaction was confirmed (optional)
    pub block_time: Option<i64>,
    /// Transaction status ("Success" or "Failed")
    pub status: String,
}

/// Transaction history response.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransactionHistory {
    /// Wallet address
    pub address: String,
    /// List of transaction summaries
    pub transactions: Vec<TransactionSummary>,
}

/// Request to submit a swap transaction.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SwapTransactionSubmitRequest {
    /// Base64-encoded signed transaction
    #[serde(rename = "signedTransaction")]
    pub signed_transaction: String,
    /// Input token mint address
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    /// Output token mint address
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    /// Amount swapped in (smallest unit)
    #[serde(rename = "inputAmount")]
    pub input_amount: i64,
    /// Amount received out (smallest unit)
    #[serde(rename = "outputAmount")]
    pub output_amount: i64,
    /// Price impact percentage (optional)
    #[serde(rename = "priceImpact")]
    pub price_impact: Option<f64>,
    /// Slippage tolerance used (optional)
    #[serde(rename = "slippageBps")]
    pub slippage_bps: Option<i32>,
}

/// Service for transaction operations.
///
/// This service provides business logic for submitting transactions and
/// querying transaction history.
pub struct TransactionService {
    solana: Arc<SolanaState>,
    db: DbPool,
}

impl TransactionService {
    /// Create a new transaction service.
    ///
    /// # Arguments
    ///
    /// * `solana` - Shared Solana state containing RPC client
    /// * `db` - Database connection pool
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::transaction::TransactionService;
    /// use backend::solana::SolanaState;
    /// use backend::database::DbPool;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let pool = /* ... */;
    /// let service = TransactionService::new(solana, pool);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(solana: Arc<SolanaState>, db: DbPool) -> Self {
        Self { solana, db }
    }

    /// Submit a signed swap transaction to Solana and record it in the database.
    ///
    /// # Arguments
    ///
    /// * `request` - Swap transaction submission request
    /// * `user_id` - User ID from JWT claims
    ///
    /// # Returns
    ///
    /// * `Ok(TransactionSubmitResult)` - Submission result with signature
    /// * `Err(AppError::InvalidInput)` - Invalid transaction format
    /// * `Err(AppError::Internal)` - Failed to submit transaction or record in database
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::transaction::TransactionService;
    /// use backend::solana::SolanaState;
    /// use backend::database::DbPool;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let pool = /* ... */;
    /// let service = TransactionService::new(solana, pool);
    ///
    /// let request = SwapTransactionSubmitRequest { /* ... */ };
    /// let result = service.submit_swap_transaction(request, "user_id").await?;
    /// println!("Signature: {}", result.signature);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Notes
    ///
    /// - Transaction is submitted to Solana RPC with `send_transaction`
    /// - Signature is returned immediately (transaction is pending)
    /// - Swap is recorded in database with "pending" status
    /// - Transaction may still fail during processing (check status on-chain)
    #[instrument(skip(self), fields(user_id = %user_id, input_mint = %request.input_mint, output_mint = %request.output_mint))]
    pub async fn submit_swap_transaction(
        &self,
        request: SwapTransactionSubmitRequest,
        user_id: &str,
    ) -> Result<TransactionSubmitResult, AppError> {
        use base64::{Engine as _, engine::general_purpose};

        debug!("Submitting swap transaction for user: {}", user_id);

        // Decode the signed transaction
        let tx_bytes = general_purpose::STANDARD
            .decode(&request.signed_transaction)
            .map_err(|e| AppError::InvalidInput(format!("Invalid base64 transaction: {}", e)))?;

        let transaction: Transaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| AppError::InvalidInput(format!("Invalid transaction format: {}", e)))?;

        // Submit transaction to Solana
        let signature = self
            .solana
            .rpc
            .send_transaction(&transaction)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to submit transaction: {}", e)))?;

        // Record swap in database using repository
        use lib_core::model::store::swap_repository::SwapRepository;
        let user_id_int = user_id.parse::<i64>()
            .map_err(|_| AppError::InvalidInput("Invalid user ID".to_string()))?;
        
        SwapRepository::create(
            &self.db,
            user_id_int,
            &signature.to_string(),
            &request.input_mint,
            &request.output_mint,
            request.input_amount,
            request.output_amount,
            request.price_impact,
            request.slippage_bps,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to record swap: {}", e)))?;

        debug!("Swap transaction submitted: {} for user {}", signature, user_id);

        Ok(TransactionSubmitResult {
            signature: signature.to_string(),
            status: "pending".to_string(),
        })
    }

    /// Get recent transaction history for a Solana wallet.
    ///
    /// # Arguments
    ///
    /// * `address` - Solana wallet public key address (base58 encoded)
    /// * `limit` - Maximum number of transactions to return (default: 10)
    ///
    /// # Returns
    ///
    /// * `Ok(TransactionHistory)` - Transaction history
    /// * `Err(AppError::InvalidInput)` - Invalid wallet address format
    /// * `Err(AppError::Internal)` - Failed to fetch transaction signatures
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::transaction::TransactionService;
    /// use backend::solana::SolanaState;
    /// use backend::database::DbPool;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let pool = /* ... */;
    /// let service = TransactionService::new(solana, pool);
    ///
    /// let history = service.get_transaction_history("wallet_address", 10).await?;
    /// println!("Found {} transactions", history.transactions.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Notes
    ///
    /// - Transactions are returned in reverse chronological order (newest first)
    /// - Only transaction signatures and basic info are returned, not full transaction details
    /// - Failed transactions are included with status "Failed"
    /// - Block time may be null for very recent transactions
    #[instrument(skip(self), fields(address = %address, limit))]
    pub async fn get_transaction_history(
        &self,
        address: &str,
        limit: usize,
    ) -> Result<TransactionHistory, AppError> {
        debug!("Getting transaction history for: {} (limit: {})", address, limit);

        let pubkey = Pubkey::from_str(address)
            .map_err(|e| AppError::InvalidInput(format!("Invalid Solana address: {}", e)))?;

        // Get transaction signatures
        let signatures = self
            .solana
            .rpc
            .get_signatures_for_address(&pubkey)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch transactions: {}", e)))?;

        let mut transactions = Vec::new();

        // Take only requested number of transactions
        for sig_info in signatures.iter().take(limit) {
            let tx_summary = TransactionSummary {
                signature: sig_info.signature.clone(),
                slot: sig_info.slot,
                block_time: sig_info.block_time,
                status: if sig_info.err.is_none() {
                    "Success".to_string()
                } else {
                    "Failed".to_string()
                },
            };
            transactions.push(tx_summary);
        }

        Ok(TransactionHistory {
            address: address.to_string(),
            transactions,
        })
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests would require mocking SolanaState and database
    // For now, we'll add integration tests in the handlers

    #[tokio::test]
    #[ignore] // Requires SolanaState and database setup
    async fn test_submit_swap_transaction() {
        // TODO: Add test with mock SolanaState and database
    }

    #[tokio::test]
    #[ignore] // Requires SolanaState setup
    async fn test_get_transaction_history() {
        // TODO: Add test with mock SolanaState
    }
}

