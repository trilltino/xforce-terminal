//! # Wallet Handlers
//!
//! HTTP endpoints for querying Solana wallet information including balances and token holdings.
//!
//! ## Endpoints
//!
//! - `GET /api/wallet/balance` - Get SOL balance for a wallet address
//! - `GET /api/wallet/info` - Get full wallet info including SOL and token balances
//! - `GET /api/wallet/tokens` - Get SPL token balances for a wallet
//!
//! ## Authentication
//!
//! These endpoints are public and do not require authentication.
//! Any valid Solana wallet address can be queried.
//!
//! ## Request Examples
//!
//! ```bash
//! # Get SOL balance
//! curl "http://localhost:3001/api/wallet/balance?address=8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL"
//!
//! # Get full wallet info
//! curl "http://localhost:3001/api/wallet/info?address=8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL"
//!
//! # Get token balances
//! curl "http://localhost:3001/api/wallet/tokens?address=8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL"
//! ```
//!
//! ## Address Validation
//!
//! All endpoints validate that the provided address is a valid Solana public key.
//! Invalid addresses will return a 400 Bad Request error.

use crate::services::wallet::WalletService;
use lib_solana::SolanaState;
use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use lib_core::dto::ErrorResponse;
use std::sync::Arc;
use tracing::{error, info, instrument};

#[derive(Debug, Deserialize)]
pub struct WalletQuery {
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct WalletBalance {
    pub address: String,
    pub balance_sol: f64,
    pub balance_lamports: u64,
}

#[derive(Debug, Serialize)]
pub struct TokenBalance {
    pub mint: String,
    pub symbol: Option<String>,
    pub balance: f64,
    pub ui_amount: String,
}

#[derive(Debug, Serialize)]
pub struct WalletInfo {
    pub address: String,
    pub balance_sol: f64,
    pub balance_lamports: u64,
    pub token_accounts: Vec<TokenBalance>,
}

/// Get SOL balance for a Solana wallet.
///
/// **Route**: `GET /api/wallet/balance`
///
/// # Parameters
///
/// - `address` (query) - Solana wallet public key address
///
/// # Returns
///
/// Success (200): `Json<WalletBalance>` - Wallet balance information:
/// - `address`: The queried wallet address
/// - `balance_sol`: Balance in SOL (human-readable)
/// - `balance_lamports`: Balance in lamports (smallest unit, 1 SOL = 1B lamports)
///
/// Error (400): Invalid Solana address format
/// Error (500): Failed to query Solana RPC
///
/// # Example
///
/// ```bash
/// curl "http://localhost:3001/api/wallet/balance?address=8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL"
/// ```
///
/// Response:
/// ```json
/// {
///   "address": "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL",
///   "balance_sol": 1.5,
///   "balance_lamports": 1500000000
/// }
/// ```
#[instrument(skip(solana))]
pub async fn get_wallet_balance(
    State(solana): State<Arc<SolanaState>>,
    Query(params): Query<WalletQuery>,
) -> Result<(StatusCode, Json<WalletBalance>), (StatusCode, Json<ErrorResponse>)> {
    info!("Wallet balance request: {}", params.address);

    let service = WalletService::new(solana);
    let balance = service.get_wallet_balance(&params.address).await.map_err(|e| {
        error!("Failed to get wallet balance: {}", e);
        let status = if e.to_string().contains("Invalid") {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(ErrorResponse {
            error: e.to_string(),
        }))
    })?;

    Ok((
        StatusCode::OK,
        Json(WalletBalance {
            address: balance.address,
            balance_sol: balance.balance_sol,
            balance_lamports: balance.balance_lamports,
        }),
    ))
}

/// Get comprehensive wallet information including SOL and SPL token balances.
///
/// **Route**: `GET /api/wallet/info`
///
/// # Parameters
///
/// - `address` (query) - Solana wallet public key address
///
/// # Returns
///
/// Success (200): `Json<WalletInfo>` - Complete wallet information:
/// - `address`: The queried wallet address
/// - `balance_sol`: SOL balance (human-readable)
/// - `balance_lamports`: SOL balance in lamports
/// - `token_accounts`: Array of SPL token balances, each containing:
///   - `mint`: Token mint address
///   - `symbol`: Token symbol (e.g., "USDC") if available
///   - `balance`: Raw token balance
///   - `ui_amount`: Human-readable token amount (respecting decimals)
///
/// Error (400): Invalid Solana address format
/// Error (500): Failed to query Solana RPC or token accounts
///
/// # Example
///
/// ```bash
/// curl "http://localhost:3001/api/wallet/info?address=8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL"
/// ```
///
/// Response:
/// ```json
/// {
///   "address": "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL",
///   "balance_sol": 1.5,
///   "balance_lamports": 1500000000,
///   "token_accounts": [
///     {
///       "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
///       "symbol": "USDC",
///       "balance": 1000000,
///       "ui_amount": "1.0"
///     }
///   ]
/// }
/// ```
#[instrument(skip(solana))]
pub async fn get_wallet_info(
    State(solana): State<Arc<SolanaState>>,
    Query(params): Query<WalletQuery>,
) -> Result<(StatusCode, Json<WalletInfo>), (StatusCode, Json<ErrorResponse>)> {
    info!("Full wallet info request: {}", params.address);

    let service = WalletService::new(solana);
    let info = service.get_wallet_info(&params.address).await.map_err(|e| {
        error!("Failed to get wallet info: {}", e);
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
    let token_accounts: Vec<TokenBalance> = info
        .token_accounts
        .into_iter()
        .map(|tb| TokenBalance {
            mint: tb.mint,
            symbol: tb.symbol,
            balance: tb.balance,
            ui_amount: tb.ui_amount,
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(WalletInfo {
            address: info.address,
            balance_sol: info.balance_sol,
            balance_lamports: info.balance_lamports,
            token_accounts,
        }),
    ))
}

/// Get SPL token balances for a Solana wallet.
///
/// **Route**: `GET /api/wallet/tokens`
///
/// # Parameters
///
/// - `address` (query) - Solana wallet public key address
///
/// # Returns
///
/// Success (200): `Json<Vec<TokenBalance>>` - Array of token balances:
/// - `mint`: Token mint address
/// - `symbol`: Token symbol (e.g., "USDC", "BONK") if available from Jupiter token list
/// - `balance`: Raw token balance (without decimal adjustment)
/// - `ui_amount`: Human-readable amount adjusted for token decimals
///
/// Error (400): Invalid Solana address format
/// Error (500): Failed to fetch token accounts from Solana RPC
///
/// # Notes
///
/// - Only returns accounts with non-zero balances
/// - Token symbols are enriched from Jupiter token list when available
/// - Returns empty array if wallet has no token accounts
///
/// # Example
///
/// ```bash
/// curl "http://localhost:3001/api/wallet/tokens?address=8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL"
/// ```
///
/// Response:
/// ```json
/// [
///   {
///     "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
///     "symbol": "USDC",
///     "balance": 1000000,
///     "ui_amount": "1.0"
///   },
///   {
///     "mint": "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
///     "symbol": "BONK",
///     "balance": 1000000000,
///     "ui_amount": "1000.0"
///   }
/// ]
/// ```
#[instrument(skip(solana))]
pub async fn get_token_balances(
    State(solana): State<Arc<SolanaState>>,
    Query(params): Query<WalletQuery>,
) -> Result<(StatusCode, Json<Vec<TokenBalance>>), (StatusCode, Json<ErrorResponse>)> {
    info!(" Token balances request: {}", params.address);

    let service = WalletService::new(solana);
    let balances = service.get_token_balances(&params.address).await.map_err(|e| {
        error!("Failed to fetch token balances: {}", e);
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
    let token_accounts: Vec<TokenBalance> = balances
        .into_iter()
        .map(|tb| TokenBalance {
            mint: tb.mint,
            symbol: tb.symbol,
            balance: tb.balance,
            ui_amount: tb.ui_amount,
        })
        .collect();

    Ok((StatusCode::OK, Json(token_accounts)))
}
