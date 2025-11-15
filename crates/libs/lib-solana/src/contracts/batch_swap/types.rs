//! # Batch Swap Types
//!
//! Request and response types for batch swap operations.

use serde::{Deserialize, Serialize};

/// Request to execute a batch of swaps
#[derive(Debug, Deserialize, Clone)]
pub struct BatchSwapRequest {
    /// List of swaps to execute (max 10)
    pub swaps: Vec<SwapRequest>,
    /// User's public key (will sign the transaction)
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
}

/// Parameters for a single swap in a batch
#[derive(Debug, Deserialize, Clone)]
pub struct SwapRequest {
    /// Input token mint address
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    /// Output token mint address
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    /// Amount of input tokens to swap (in token's smallest unit)
    pub amount: u64,
    /// Minimum output amount (slippage protection)
    #[serde(rename = "minOutputAmount")]
    pub min_output_amount: u64,
}

/// Response from batch swap execution
#[derive(Debug, Serialize)]
pub struct BatchSwapResponse {
    /// Transaction signature (if executed) or transaction data (if unsigned)
    pub signature: Option<String>,
    /// Unsigned transaction (base64 encoded) for client-side signing
    #[serde(rename = "transaction", skip_serializing_if = "Option::is_none")]
    pub transaction: Option<String>,
    /// Status of the operation
    pub status: String,
    /// Last valid block height (for transaction expiration)
    #[serde(rename = "lastValidBlockHeight", skip_serializing_if = "Option::is_none")]
    pub last_valid_block_height: Option<u64>,
    /// Error message (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Request to execute a single swap
#[derive(Debug, Deserialize)]
pub struct ExecuteSwapRequest {
    /// Input token account address
    pub from: String,
    /// Output token account address
    pub to: String,
    /// Amount to swap
    pub amount: u64,
    /// Minimum output amount (slippage protection)
    #[serde(rename = "minOutputAmount")]
    pub min_output_amount: u64,
    /// Expected output amount (from Jupiter quote)
    #[serde(rename = "expectedOutput")]
    pub expected_output: u64,
    /// User's public key (will sign the transaction)
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
}

