//! # Transaction Handlers
//!
//! HTTP endpoints for querying Solana transaction history and details.
//!
//! ## Endpoints
//!
//! - `GET /api/transactions/history` - Get recent transaction history for a wallet
//!
//! ## Authentication
//!
//! These endpoints are public and do not require authentication.
//! Any valid Solana wallet address can be queried.
//!
//! ## Request Examples
//!
//! ```bash
//! # Get last 10 transactions (default)
//! curl "http://localhost:3001/api/transactions/history?address=8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL"
//!
//! # Get last 25 transactions
//! curl "http://localhost:3001/api/transactions/history?address=8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL&limit=25"
//! ```
//!
//! ## Rate Limits
//!
//! Transaction queries are rate-limited by the Solana RPC endpoint.
//! Consider caching results for frequently accessed wallets.

use crate::services::transaction::TransactionService;
use lib_solana::SolanaState;
use lib_core::DbPool;
use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use shared::{ErrorResponse, SubmitTransactionRequest, SubmitTransactionResponse};
use std::sync::Arc;
use tracing::{error, info, instrument};

#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    pub address: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Serialize)]
pub struct TransactionSummary {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct TransactionHistory {
    pub address: String,
    pub transactions: Vec<TransactionSummary>,
}

/// Get recent transaction history for a Solana wallet.
///
/// **Route**: `GET /api/transactions/history`
///
/// # Parameters
///
/// - `address` (query) - Solana wallet public key address
/// - `limit` (query, optional) - Maximum number of transactions to return (default: 10, max recommended: 100)
///
/// # Returns
///
/// Success (200): `Json<TransactionHistory>` - Transaction history containing:
/// - `address`: The queried wallet address
/// - `transactions`: Array of transaction summaries, each with:
///   - `signature`: Transaction signature (unique identifier)
///   - `slot`: Blockchain slot number where transaction was processed
///   - `block_time`: Unix timestamp of when transaction was confirmed (optional)
///   - `status`: Transaction status ("Success" or "Failed")
///
/// Error (400): Invalid Solana address format
/// Error (500): Failed to fetch transaction signatures from Solana RPC
///
/// # Notes
///
/// - Transactions are returned in reverse chronological order (newest first)
/// - Only transaction signatures and basic info are returned, not full transaction details
/// - Failed transactions are included with status "Failed"
/// - Block time may be null for very recent transactions
///
/// # Example
///
/// ```bash
/// curl "http://localhost:3001/api/transactions/history?address=8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL&limit=5"
/// ```
///
/// Response:
/// ```json
/// {
///   "address": "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL",
///   "transactions": [
///     {
///       "signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW",
///       "slot": 123456789,
///       "block_time": 1729857600,
///       "status": "Success"
///     },
///     {
///       "signature": "4hXTCkRzt9WyecNzV1XPgCDfGAZzQKNxLXgynz5QDuWWPSAZBZSHptvWRL3BjCvzUXRdKvHL2b7yGrRQcWyaqsaBrq",
///       "slot": 123456780,
///       "block_time": 1729857500,
///       "status": "Failed"
///     }
///   ]
/// }
/// ```
#[instrument(skip(solana, pool))]
pub async fn get_transaction_history(
    State(solana): State<Arc<SolanaState>>,
    State(pool): State<DbPool>,
    Query(params): Query<TransactionQuery>,
) -> Result<(StatusCode, Json<TransactionHistory>), (StatusCode, Json<ErrorResponse>)> {
    info!("📜 Transaction history request: {} (limit: {})", params.address, params.limit);

    let service = TransactionService::new(solana, pool);
    let history = service.get_transaction_history(&params.address, params.limit).await.map_err(|e| {
        error!("Failed to fetch transaction history: {}", e);
        let status = if e.to_string().contains("Invalid") {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(ErrorResponse {
            error: e.to_string(),
        }))
    })?;

    // Convert service types to handler types
    let transactions: Vec<TransactionSummary> = history
        .transactions
        .into_iter()
        .map(|tx| TransactionSummary {
            signature: tx.signature,
            slot: tx.slot,
            block_time: tx.block_time,
            status: tx.status,
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(TransactionHistory {
            address: history.address,
            transactions,
        }),
    ))
}

/// Submit a signed transaction to Solana.
///
/// **Route**: `POST /api/transaction/submit`
///
/// # Request Body
///
/// - `transaction` - Base64-encoded signed transaction bytes
/// - `wallet_address` - Solana wallet address that signed the transaction
/// - `transaction_type` - Type of transaction (e.g., "swap", "transfer", "stake")
///
/// # Returns
///
/// Success (200): `Json<SubmitTransactionResponse>` - Transaction submission result:
/// - `success`: Whether the transaction was successfully submitted
/// - `signature`: Transaction signature if successful
/// - `message`: Human-readable status message
///
/// Error (400): Invalid transaction format or wallet address
/// Error (500): Failed to submit transaction to Solana
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:3001/api/transaction/submit \
///   -H "Content-Type: application/json" \
///   -d '{
///     "transaction": "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDArczNgag3jGgUOGF4R8d4p4k3gV4qJ3p5k2jL3nP...",
///     "wallet_address": "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde",
///     "transaction_type": "swap"
///   }'
/// ```
///
/// Response:
/// ```json
/// {
///   "success": true,
///   "signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW",
///   "message": "Transaction submitted successfully"
/// }
/// ```
#[instrument(skip(solana))]
pub async fn submit_transaction(
    State(solana): State<Arc<SolanaState>>,
    Json(req): Json<SubmitTransactionRequest>,
) -> Result<Json<SubmitTransactionResponse>, (StatusCode, Json<ErrorResponse>)> {
    use base64::{Engine as _, engine::general_purpose};
    use solana_sdk::transaction::Transaction;

    info!("Submitting transaction: type={}, wallet={}", req.transaction_type, req.wallet_address);

    // Decode the base64 transaction
    let tx_bytes = general_purpose::STANDARD
        .decode(&req.transaction)
        .map_err(|e| {
            error!("Invalid base64 transaction: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid transaction format: {}", e),
                }),
            )
        })?;

    // Deserialize transaction
    let transaction: Transaction = bincode::deserialize(&tx_bytes)
        .map_err(|e| {
            error!("Failed to deserialize transaction: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid transaction format: {}", e),
                }),
            )
        })?;

    // Submit transaction to Solana
    let signature = solana
        .rpc
        .send_transaction(&transaction)
        .await
        .map_err(|e| {
            error!("Failed to submit transaction: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to submit transaction: {}", e),
                }),
            )
        })?;

    info!("Transaction submitted successfully: {}", signature);

    Ok(Json(SubmitTransactionResponse {
        success: true,
        signature: Some(signature.to_string()),
        message: "Transaction submitted successfully".to_string(),
    }))
}
