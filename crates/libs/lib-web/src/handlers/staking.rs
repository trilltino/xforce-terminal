//! # Staking Handlers
//!
//! HTTP endpoints for querying Solana staking information and epoch data.
//!
//! ## Endpoints
//!
//! - `GET /api/staking/info` - Get basic staking information for a wallet
//!
//! ## Authentication
//!
//! These endpoints are public and do not require authentication.
//! Any valid Solana wallet address can be queried.
//!
//! ## Request Examples
//!
//! ```bash
//! # Get staking info
//! curl "http://localhost:3001/api/staking/info?address=8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL"
//! ```
//!
//! ## Implementation Notes
//!
//! This is a simplified staking endpoint that provides epoch information.
//! Full staking account details (stake accounts, validators, rewards) require
//! additional RPC calls and parsing of stake account data structures.
//!
//! Future enhancements could include:
//! - Detailed stake account information
//! - Active/inactive stake amounts
//! - Validator information
//! - Historical staking rewards

use crate::services::staking::StakingService;
use lib_solana::SolanaState;
use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

use lib_core::AppError;

#[derive(Debug, Deserialize)]
pub struct StakingQuery {
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct StakingInfo {
    pub wallet_address: String,
    pub epoch: u64,
    pub rent_epoch: u64,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct StakingErrorResponse {
    pub error: String,
}

fn app_error_to_staking_response(err: AppError) -> (StatusCode, Json<StakingErrorResponse>) {
    let status = match &err {
        AppError::NotFound(_) => StatusCode::NOT_FOUND,
        AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status, Json(StakingErrorResponse {
        error: err.to_string(),
    }))
}

/// Get basic staking and epoch information for a Solana wallet.
///
/// **Route**: `GET /api/staking/info`
///
/// # Parameters
///
/// - `address` (query) - Solana wallet public key address
///
/// # Returns
///
/// Success (200): `Json<StakingInfo>` - Staking information containing:
/// - `wallet_address`: The queried wallet address
/// - `epoch`: Current epoch number on Solana
/// - `rent_epoch`: Rent epoch for the wallet account
/// - `message`: Informational message about staking status
///
/// Error (400): Invalid Solana address format
/// Error (404): Wallet account not found
/// Error (500): Failed to fetch account or epoch information
///
/// # Notes
///
/// - This is a simplified endpoint showing basic epoch information
/// - Full staking details require parsing stake account program data
/// - Rent epoch indicates when rent was last collected for the account
/// - Current epoch shows the network's current epoch cycle
///
/// # Solana Epochs
///
/// Solana epochs are time periods of approximately 2-3 days where:
/// - Validator schedules are determined
/// - Staking rewards are calculated
/// - Network parameters can be updated
///
/// # Example
///
/// ```bash
/// curl "http://localhost:3001/api/staking/info?address=8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL"
/// ```
///
/// Response:
/// ```json
/// {
///   "wallet_address": "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL",
///   "epoch": 450,
///   "rent_epoch": 450,
///   "message": "Current epoch: 450, Wallet rent epoch: 450. Staking details require additional RPC calls."
/// }
/// ```
#[instrument(skip(solana), fields(address = %params.address))]
pub async fn get_staking_info(
    State(solana): State<Arc<SolanaState>>,
    Query(params): Query<StakingQuery>,
) -> Result<(StatusCode, Json<StakingInfo>), (StatusCode, Json<StakingErrorResponse>)> {
    let service = StakingService::new(solana);
    let info = service.get_staking_info(&params.address).await.map_err(|e| {
        let status = match &e {
            AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(StakingErrorResponse {
            error: e.to_string(),
        }))
    })?;

    // Convert service result to handler response
    // Note: The service currently returns a placeholder, so we'll create a basic response
    // When staking is fully implemented, this will return actual staking data
    let staking_info = StakingInfo {
        wallet_address: info.address,
        epoch: 0, // Placeholder - will be populated when staking is implemented
        rent_epoch: 0, // Placeholder - will be populated when staking is implemented
        message: "Staking service is not yet fully implemented".to_string(),
    };

    Ok((StatusCode::OK, Json(staking_info)))
}
