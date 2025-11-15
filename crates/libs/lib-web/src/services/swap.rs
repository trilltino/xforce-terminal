//! # Swap Service
//!
//! Business logic for token swaps using Jupiter Aggregator on Solana.
//!
//! ## Overview
//!
//! This service orchestrates swap operations by coordinating with Jupiter Aggregator
//! to get swap quotes and build unsigned transactions for client-side signing.
//!
//! ## Features
//!
//! - **Quote Fetching**: Get optimal swap quotes from Jupiter Aggregator
//! - **Transaction Building**: Build unsigned swap transactions for client signing
//! - **Route Information**: Extract routing information from quotes
//! - **Error Handling**: Comprehensive error handling with user-friendly messages
//!
//! ## Usage
//!
//! ```rust,no_run
//! use backend::services::swap::SwapService;
//! use backend::solana::SolanaState;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let solana = Arc::new(SolanaState::new(/* ... */).await?);
//! let service = SwapService::new(solana);
//!
//! // Get swap quote
//! let quote = service.get_swap_quote(
//!     "So11111111111111111111111111111111111111112", // SOL
//!     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
//!     1000000000, // 1 SOL in lamports
//!     50, // 0.5% slippage
//! ).await?;
//!
//! // Build unsigned transaction
//! let tx = service.build_swap_transaction(&quote, "user_public_key").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All methods return `Result<T, AppError>`. Common errors:
//! - `AppError::InvalidInput` - Invalid swap parameters
//! - `AppError::Internal` - Failed to fetch quote or build transaction
//!
//! ## Architecture
//!
//! The service uses Jupiter Aggregator for swap routing:
//!
//! ```text
//! SwapService → JupiterClient → Jupiter Aggregator API
//! ```

use lib_core::AppError;
use lib_solana::SolanaState;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Route information extracted from Jupiter quote.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RouteInfo {
    /// DEX name (e.g., "Orca", "Raydium")
    pub dex: String,
    /// Input token mint address
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    /// Output token mint address
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    /// Input amount (string representation)
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    /// Output amount (string representation)
    #[serde(rename = "outAmount")]
    pub out_amount: String,
}

/// Swap quote response with route information.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SwapQuoteResult {
    /// Input token mint address
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    /// Output token mint address
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    /// Input amount (string representation)
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    /// Output amount (string representation)
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    /// Price impact percentage
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: f64,
    /// Routing steps through different DEXs
    pub routes: Vec<RouteInfo>,
}

/// Swap transaction response.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SwapTransactionResult {
    /// Base64-encoded unsigned transaction
    #[serde(rename = "transaction")]
    pub transaction: String,
    /// Last valid block height
    #[serde(rename = "lastValidBlockHeight")]
    pub last_valid_block_height: u64,
    /// Input token mint address
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    /// Output token mint address
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    /// Input amount (string representation)
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    /// Output amount (string representation)
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    /// Price impact percentage
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: f64,
}

/// Service for swap operations.
///
/// This service provides business logic for token swaps, including quote fetching
/// and transaction building. It coordinates with Jupiter Aggregator to find optimal
/// swap routes.
pub struct SwapService {
    solana: Arc<SolanaState>,
}

impl SwapService {
    /// Create a new swap service.
    ///
    /// # Arguments
    ///
    /// * `solana` - Shared Solana state containing Jupiter client
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::swap::SwapService;
    /// use backend::solana::SolanaState;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let service = SwapService::new(solana);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(solana: Arc<SolanaState>) -> Self {
        Self { solana }
    }

    /// Get a swap quote from Jupiter Aggregator.
    ///
    /// Fetches an optimal swap quote that finds the best route across multiple DEXs
    /// for the given swap parameters.
    ///
    /// # Arguments
    ///
    /// * `input_mint` - Input token mint address
    /// * `output_mint` - Output token mint address
    /// * `amount` - Amount to swap in smallest unit (e.g., lamports for SOL)
    /// * `slippage_bps` - Slippage tolerance in basis points (e.g., 50 = 0.5%)
    ///
    /// # Returns
    ///
    /// * `Ok(SwapQuoteResult)` - Quote with route information
    /// * `Err(AppError::InvalidInput)` - Invalid swap parameters
    /// * `Err(AppError::Internal)` - Failed to fetch quote from Jupiter
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::swap::SwapService;
    /// use backend::solana::SolanaState;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let service = SwapService::new(solana);
    ///
    /// let quote = service.get_swap_quote(
    ///     "So11111111111111111111111111111111111111112", // SOL
    ///     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
    ///     1000000000, // 1 SOL
    ///     50, // 0.5% slippage
    /// ).await?;
    ///
    /// println!("Output amount: {}", quote.out_amount);
    /// println!("Price impact: {:.2}%", quote.price_impact_pct);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Notes
    ///
    /// - Slippage is specified in basis points (1 bps = 0.01%)
    /// - Quote includes routing information showing which DEXs will be used
    /// - Price impact indicates how much the swap will affect the market price
    #[instrument(skip(self), fields(input_mint = %input_mint, output_mint = %output_mint, amount, slippage_bps))]
    pub async fn get_swap_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<SwapQuoteResult, AppError> {
        debug!(
            "Getting swap quote: {} -> {} (amount: {}, slippage: {} bps)",
            input_mint, output_mint, amount, slippage_bps
        );

        let quote = self
            .solana
            .jupiter
            .get_swap_quote(input_mint, output_mint, amount, slippage_bps)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get swap quote: {}", e)))?;

        // Extract route information
        let routes: Vec<RouteInfo> = quote
            .route_plan
            .iter()
            .map(|step| RouteInfo {
                dex: step.swap_info.label.clone().unwrap_or_else(|| "Unknown".to_string()),
                input_mint: step.swap_info.input_mint.clone(),
                output_mint: step.swap_info.output_mint.clone(),
                in_amount: step.swap_info.in_amount.clone(),
                out_amount: step.swap_info.out_amount.clone(),
            })
            .collect();

        Ok(SwapQuoteResult {
            input_mint: quote.input_mint,
            output_mint: quote.output_mint,
            in_amount: quote.in_amount,
            out_amount: quote.out_amount,
            price_impact_pct: quote.price_impact_pct,
            routes,
        })
    }

    /// Build an unsigned swap transaction from swap parameters.
    ///
    /// This method combines getting a quote and building a transaction in one call.
    /// It first fetches a quote from Jupiter, then builds the transaction using that quote.
    ///
    /// # Arguments
    ///
    /// * `input_mint` - Input token mint address
    /// * `output_mint` - Output token mint address
    /// * `amount` - Amount to swap in smallest unit (e.g., lamports for SOL)
    /// * `slippage_bps` - Slippage tolerance in basis points (e.g., 50 = 0.5%)
    /// * `user_public_key` - User's wallet public key that will sign the transaction
    ///
    /// # Returns
    ///
    /// * `Ok(SwapTransactionResult)` - Unsigned transaction data with quote information
    /// * `Err(AppError::InvalidInput)` - Invalid swap parameters
    /// * `Err(AppError::Internal)` - Failed to get quote or build transaction
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::services::swap::SwapService;
    /// use backend::solana::SolanaState;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Arc::new(SolanaState::new(/* ... */).await?);
    /// let service = SwapService::new(solana);
    ///
    /// let tx = service.execute_swap(
    ///     "So11111111111111111111111111111111111111112", // SOL
    ///     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
    ///     1000000000, // 1 SOL
    ///     50, // 0.5% slippage
    ///     "user_public_key",
    /// ).await?;
    ///
    /// println!("Transaction: {}", tx.transaction);
    /// println!("Output amount: {}", tx.out_amount);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Notes
    ///
    /// - Transaction is unsigned and must be signed by the client
    /// - Transaction includes recent blockhash and expires after ~60 seconds
    /// - Client should submit the signed transaction via the transaction service
    #[instrument(skip(self), fields(input_mint = %input_mint, output_mint = %output_mint, amount, user_public_key = %user_public_key))]
    pub async fn execute_swap(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
        user_public_key: &str,
    ) -> Result<SwapTransactionResult, AppError> {
        debug!("Executing swap for user: {}", user_public_key);

        // Step 1: Get quote from Jupiter
        let quote = self
            .solana
            .jupiter
            .get_swap_quote(input_mint, output_mint, amount, slippage_bps)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get swap quote: {}", e)))?;

        // Step 2: Get swap transaction from Jupiter
        let swap_tx = self
            .solana
            .jupiter
            .get_swap_transaction(&quote, user_public_key)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to build swap transaction: {}", e)))?;

        Ok(SwapTransactionResult {
            transaction: swap_tx.swap_transaction,
            last_valid_block_height: swap_tx.last_valid_block_height,
            input_mint: quote.input_mint,
            output_mint: quote.output_mint,
            in_amount: quote.in_amount,
            out_amount: quote.out_amount,
            price_impact_pct: quote.price_impact_pct,
        })
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests would require mocking SolanaState
    // For now, we'll add integration tests in the handlers

    #[tokio::test]
    #[ignore] // Requires SolanaState setup
    async fn test_get_swap_quote() {
        // TODO: Add test with mock SolanaState
    }

    #[tokio::test]
    #[ignore] // Requires SolanaState setup
    async fn test_execute_swap() {
        // TODO: Add test with mock SolanaState
    }
}

