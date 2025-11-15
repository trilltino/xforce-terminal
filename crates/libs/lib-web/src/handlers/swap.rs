//! # Swap Handlers
//!
//! HTTP endpoints for token swaps using Jupiter Aggregator on Solana.
//!
//! ## Endpoints
//!
//! - `GET /api/swap/quote` - Get a swap quote for token exchange
//! - `POST /api/swap/execute` - Build an unsigned swap transaction (requires auth)
//! - `POST /api/transactions/submit` - Submit a signed swap transaction (requires auth)
//!
//! ## Authentication
//!
//! - Quote endpoint is public and does not require authentication
//! - Execute and submit endpoints require valid JWT authentication
//!
//! ## Request Examples
//!
//! ```bash
//! # Get swap quote (SOL to USDC)
//! curl "http://localhost:3001/api/swap/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=1000000000&slippageBps=50"
//!
//! # Execute swap (requires auth)
//! curl -X POST http://localhost:3001/api/swap/execute \
//!   -H "Authorization: Bearer YOUR_JWT_TOKEN" \
//!   -H "Content-Type: application/json" \
//!   -d '{
//!     "inputMint": "So11111111111111111111111111111111111111112",
//!     "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
//!     "amount": 1000000000,
//!     "slippageBps": 50,
//!     "userPublicKey": "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL"
//!   }'
//!
//! # Submit signed transaction (requires auth)
//! curl -X POST http://localhost:3001/api/transactions/submit \
//!   -H "Authorization: Bearer YOUR_JWT_TOKEN" \
//!   -H "Content-Type: application/json" \
//!   -d '{
//!     "signedTransaction": "BASE64_ENCODED_SIGNED_TX",
//!     "inputMint": "So11111111111111111111111111111111111111112",
//!     "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
//!     "inputAmount": 1000000000,
//!     "outputAmount": 24500000,
//!     "priceImpact": 0.05,
//!     "slippageBps": 50
//!   }'
//! ```
//!
//! ## Swap Flow
//!
//! 1. **Get Quote**: Query Jupiter for best swap route and estimated output
//! 2. **Execute**: Build unsigned transaction with user's wallet
//! 3. **Sign**: Client signs transaction with user's private key
//! 4. **Submit**: Send signed transaction to Solana and record in database
//!
//! ## Jupiter Integration
//!
//! This module integrates with Jupiter Aggregator v6 API:
//! - Aggregates liquidity from multiple Solana DEXs
//! - Finds optimal routing for best prices
//! - Supports partial fills and multi-hop swaps
//! - Provides price impact and slippage protection

use axum::{extract::{Query, State}, http::StatusCode, Json, Extension};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

use crate::services::swap::SwapService;
use lib_auth::Claims;
use lib_core::AppError;
use lib_solana::SolanaState;

#[derive(Debug, Deserialize)]
pub struct SwapQuoteQuery {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    pub amount: u64,
    #[serde(rename = "slippageBps", default = "default_slippage")]
    pub slippage_bps: u16,
}

fn default_slippage() -> u16 {
    50 // 0.5% default slippage
}

#[derive(Debug, Serialize)]
pub struct SwapQuoteResponse {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: f64,
    pub routes: Vec<RouteInfo>,
}

#[derive(Debug, Serialize)]
pub struct RouteInfo {
    pub dex: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
}

#[derive(Debug, Serialize)]
pub struct SwapErrorResponse {
    pub error: String,
}

fn app_error_to_swap_response(err: AppError) -> (StatusCode, Json<SwapErrorResponse>) {
    let status = match &err {
        AppError::NotFound(_) => StatusCode::NOT_FOUND,
        AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status, Json(SwapErrorResponse {
        error: err.to_string(),
    }))
}

/// Get a swap quote from Jupiter Aggregator for token exchange.
///
/// **Route**: `GET /api/swap/quote`
///
/// # Parameters
///
/// - `inputMint` (query) - Input token mint address (token to swap from)
/// - `outputMint` (query) - Output token mint address (token to swap to)
/// - `amount` (query) - Amount to swap in smallest unit (e.g., lamports for SOL)
/// - `slippageBps` (query, optional) - Slippage tolerance in basis points (default: 50 = 0.5%)
///
/// # Returns
///
/// Success (200): `Json<SwapQuoteResponse>` - Quote information:
/// - `inputMint`: Input token mint address
/// - `outputMint`: Output token mint address
/// - `inAmount`: Amount being swapped (string)
/// - `outAmount`: Estimated output amount (string)
/// - `priceImpactPct`: Estimated price impact as percentage
/// - `routes`: Array of routing steps through different DEXs
///
/// Error (400): Invalid parameters
/// Error (500): Failed to fetch quote from Jupiter
///
/// # Slippage
///
/// Slippage is specified in basis points (bps):
/// - 1 bps = 0.01%
/// - 50 bps = 0.5% (default)
/// - 100 bps = 1%
///
/// # Example
///
/// ```bash
/// # Swap 1 SOL to USDC with 0.5% slippage
/// curl "http://localhost:3001/api/swap/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=1000000000&slippageBps=50"
/// ```
///
/// Response:
/// ```json
/// {
///   "inputMint": "So11111111111111111111111111111111111111112",
///   "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
///   "inAmount": "1000000000",
///   "outAmount": "24500000",
///   "priceImpactPct": 0.05,
///   "routes": [
///     {
///       "dex": "Orca",
///       "inputMint": "So11111111111111111111111111111111111111112",
///       "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
///       "inAmount": "1000000000",
///       "outAmount": "24500000"
///     }
///   ]
/// }
/// ```
#[instrument(skip(solana), fields(input_mint = %params.input_mint, output_mint = %params.output_mint, amount = params.amount))]
pub async fn get_swap_quote(
    State(solana): State<Arc<SolanaState>>,
    Query(params): Query<SwapQuoteQuery>,
) -> Result<(StatusCode, Json<SwapQuoteResponse>), (StatusCode, Json<SwapErrorResponse>)> {
    let service = SwapService::new(solana);
    let quote_result = service
        .get_swap_quote(
            &params.input_mint,
            &params.output_mint,
            params.amount,
            params.slippage_bps,
        )
        .await
        .map_err(|e| {
            // Use AppError's status_code and user_message for proper error handling
            let status = e.status_code();
            (status, Json(SwapErrorResponse {
                error: e.user_message(),
            }))
        })?;

    // Convert service result to handler response
    let routes: Vec<RouteInfo> = quote_result
        .routes
        .into_iter()
        .map(|r| RouteInfo {
            dex: r.dex,
            input_mint: r.input_mint,
            output_mint: r.output_mint,
            in_amount: r.in_amount,
            out_amount: r.out_amount,
        })
        .collect();

    let response = SwapQuoteResponse {
        input_mint: quote_result.input_mint,
        output_mint: quote_result.output_mint,
        in_amount: quote_result.in_amount,
        out_amount: quote_result.out_amount,
        price_impact_pct: quote_result.price_impact_pct,
        routes,
    };

    Ok((StatusCode::OK, Json(response)))
}

// ==================== SWAP EXECUTION ENDPOINTS ====================

#[derive(Debug, Deserialize)]
pub struct SwapExecuteRequest {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    pub amount: u64,
    #[serde(rename = "slippageBps", default = "default_slippage")]
    pub slippage_bps: u16,
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
}

#[derive(Debug, Serialize)]
pub struct SwapExecuteResponse {
    #[serde(rename = "transaction")]
    pub transaction: String, // Base64-encoded unsigned transaction
    #[serde(rename = "lastValidBlockHeight")]
    pub last_valid_block_height: u64,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: f64,
}

/// Build an unsigned swap transaction ready for client-side signing.
///
/// **Route**: `POST /api/swap/execute`
///
/// **Authentication**: Required (JWT token in Authorization header)
///
/// # Parameters
///
/// - `inputMint` (body) - Input token mint address
/// - `outputMint` (body) - Output token mint address
/// - `amount` (body) - Amount to swap in smallest unit
/// - `slippageBps` (body, optional) - Slippage tolerance in basis points (default: 50)
/// - `userPublicKey` (body) - User's wallet public key that will sign the transaction
///
/// # Returns
///
/// Success (200): `Json<SwapExecuteResponse>` - Unsigned transaction data:
/// - `transaction`: Base64-encoded unsigned transaction
/// - `lastValidBlockHeight`: Block height until which transaction is valid
/// - `inputMint`: Input token mint address
/// - `outputMint`: Output token mint address
/// - `inAmount`: Amount being swapped
/// - `outAmount`: Estimated output amount
/// - `priceImpactPct`: Estimated price impact
///
/// Error (400): Invalid parameters
/// Error (401): Unauthorized (missing or invalid JWT)
/// Error (500): Failed to build transaction
///
/// # Workflow
///
/// 1. Server fetches quote from Jupiter
/// 2. Server builds unsigned transaction
/// 3. Client receives base64 transaction
/// 4. Client deserializes and signs transaction
/// 5. Client submits signed transaction via `/api/transactions/submit`
///
/// # Security
///
/// - Transaction is unsigned and cannot be submitted without user's private key
/// - Server never has access to user's private key
/// - Transaction includes recent blockhash and expires after ~60 seconds
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:3001/api/swap/execute \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN" \
///   -H "Content-Type: application/json" \
///   -d '{
///     "inputMint": "So11111111111111111111111111111111111111112",
///     "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
///     "amount": 1000000000,
///     "slippageBps": 50,
///     "userPublicKey": "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL"
///   }'
/// ```
///
/// Response:
/// ```json
/// {
///   "transaction": "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDoQ...",
///   "lastValidBlockHeight": 123456789,
///   "inputMint": "So11111111111111111111111111111111111111112",
///   "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
///   "inAmount": "1000000000",
///   "outAmount": "24500000",
///   "priceImpactPct": 0.05
/// }
/// ```
#[instrument(skip(solana), fields(input_mint = %payload.input_mint, output_mint = %payload.output_mint))]
pub async fn execute_swap(
    State(solana): State<Arc<SolanaState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<SwapExecuteRequest>,
) -> Result<(StatusCode, Json<SwapExecuteResponse>), (StatusCode, Json<SwapErrorResponse>)> {
    let service = SwapService::new(solana);
    let tx_result = service
        .execute_swap(
            &payload.input_mint,
            &payload.output_mint,
            payload.amount,
            payload.slippage_bps,
            &payload.user_public_key,
        )
        .await
        .map_err(|e| {
            // Use AppError's status_code and user_message for proper error handling
            let status = e.status_code();
            (status, Json(SwapErrorResponse {
                error: e.user_message(),
            }))
        })?;

    // Convert service result to handler response
    let response = SwapExecuteResponse {
        transaction: tx_result.transaction,
        last_valid_block_height: tx_result.last_valid_block_height,
        input_mint: tx_result.input_mint,
        output_mint: tx_result.output_mint,
        in_amount: tx_result.in_amount,
        out_amount: tx_result.out_amount,
        price_impact_pct: tx_result.price_impact_pct,
    };

    Ok((StatusCode::OK, Json(response)))
}

#[derive(Debug, Deserialize)]
pub struct TransactionSubmitRequest {
    #[serde(rename = "signedTransaction")]
    pub signed_transaction: String, // Base64-encoded signed transaction
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inputAmount")]
    pub input_amount: i64,
    #[serde(rename = "outputAmount")]
    pub output_amount: i64,
    #[serde(rename = "priceImpact")]
    pub price_impact: Option<f64>,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TransactionSubmitResponse {
    pub signature: String,
    pub status: String,
}

/// Submit a signed swap transaction to the Solana blockchain and record it in the database.
///
/// **Route**: `POST /api/transactions/submit`
///
/// **Authentication**: Required (JWT token in Authorization header)
///
/// # Parameters
///
/// - `signedTransaction` (body) - Base64-encoded signed transaction
/// - `inputMint` (body) - Input token mint address
/// - `outputMint` (body) - Output token mint address
/// - `inputAmount` (body) - Amount swapped in (smallest unit)
/// - `outputAmount` (body) - Amount received out (smallest unit)
/// - `priceImpact` (body, optional) - Price impact percentage
/// - `slippageBps` (body, optional) - Slippage tolerance used
///
/// # Returns
///
/// Success (200): `Json<TransactionSubmitResponse>` - Submission result:
/// - `signature`: Transaction signature (unique identifier on Solana)
/// - `status`: Transaction status ("pending" initially)
///
/// Error (400): Invalid transaction format or base64 encoding
/// Error (401): Unauthorized (missing or invalid JWT)
/// Error (500): Failed to submit transaction or record in database
///
/// # Database Recording
///
/// All submitted swaps are recorded in the `swaps` table with:
/// - User ID (from JWT claims)
/// - Transaction signature
/// - Input/output token mints
/// - Input/output amounts
/// - Price impact and slippage
/// - Initial status ("pending")
/// - Timestamp
///
/// # Transaction Lifecycle
///
/// 1. Transaction submitted to Solana RPC with `send_transaction`
/// 2. Signature returned immediately (transaction is pending)
/// 3. Swap recorded in database with "pending" status
/// 4. Transaction processed by Solana validators (~400ms)
/// 5. Client should poll Solana to confirm transaction status
///
/// # Notes
///
/// - Returned signature can be used to track transaction on Solana explorers
/// - Transaction may still fail during processing (check status on-chain)
/// - Recent blockhash must be valid (transactions expire after ~60 seconds)
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:3001/api/transactions/submit \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN" \
///   -H "Content-Type: application/json" \
///   -d '{
///     "signedTransaction": "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDoQ...",
///     "inputMint": "So11111111111111111111111111111111111111112",
///     "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
///     "inputAmount": 1000000000,
///     "outputAmount": 24500000,
///     "priceImpact": 0.05,
///     "slippageBps": 50
///   }'
/// ```
///
/// Response:
/// ```json
/// {
///   "signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW",
///   "status": "pending"
/// }
/// ```
#[instrument(skip(solana, pool, claims), fields(user_id = %claims.sub))]
pub async fn submit_transaction(
    State(solana): State<Arc<SolanaState>>,
    State(pool): State<lib_core::DbPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<TransactionSubmitRequest>,
) -> Result<(StatusCode, Json<TransactionSubmitResponse>), (StatusCode, Json<SwapErrorResponse>)> {
    use crate::services::transaction::{TransactionService, SwapTransactionSubmitRequest};

    let service = TransactionService::new(solana, pool);
    
    // Convert handler request to service request
    let request = SwapTransactionSubmitRequest {
        signed_transaction: payload.signed_transaction,
        input_mint: payload.input_mint,
        output_mint: payload.output_mint,
        input_amount: payload.input_amount,
        output_amount: payload.output_amount,
        price_impact: payload.price_impact,
        slippage_bps: payload.slippage_bps,
    };

    let result = service.submit_swap_transaction(request, &claims.sub).await
        .map_err(|e| {
            // Use AppError's status_code method for proper HTTP status mapping
            let status = e.status_code();
            (status, Json(SwapErrorResponse {
                error: e.user_message(),
            }))
        })?;

    let response = TransactionSubmitResponse {
        signature: result.signature,
        status: result.status,
    };

    Ok((StatusCode::OK, Json(response)))
}
