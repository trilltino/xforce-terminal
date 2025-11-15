//! # Batch Swap Routes
//!
//! HTTP route handlers for batch swap operations.

use super::types::{BatchSwapRequest, BatchSwapResponse, ExecuteSwapRequest};
use super::validator::{validate_batch_swap_request, validate_execute_swap_request};
use super::builder::{build_batch_swap_transaction, build_execute_swap_transaction};
use super::BatchSwapRouterPlugin;
use crate::contracts::plugin::{ContractMetadata, ContractPlugin};
use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use std::sync::Arc;
use tracing::{error, info};

/// Handle batch swap request
pub async fn handle_batch_swap_app_state(
    State(plugin): State<Arc<BatchSwapRouterPlugin>>,
    Json(request): Json<BatchSwapRequest>,
) -> Result<(StatusCode, Json<BatchSwapResponse>), (StatusCode, Json<BatchSwapResponse>)> {
    info!("Batch swap request received: {} swaps", request.swaps.len());

    // Validate request
    if let Err(e) = validate_batch_swap_request(&request) {
        error!("Batch swap validation failed: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(BatchSwapResponse {
                signature: None,
                transaction: None,
                status: "error".to_string(),
                last_valid_block_height: None,
                error: Some(e.to_string()),
            })
        ));
    }

    // Get Solana state
    let solana_state = plugin.solana_state.as_ref()
        .ok_or_else(|| {
            error!("Plugin not initialized");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BatchSwapResponse {
                    signature: None,
                    transaction: None,
                    status: "error".to_string(),
                    last_valid_block_height: None,
                    error: Some("Plugin not initialized".to_string()),
                })
            )
        })?;

    // Build transaction
    match build_batch_swap_transaction(solana_state, plugin.program_id(), &request).await {
        Ok((transaction, last_valid_block_height)) => {
            info!("Batch swap transaction built successfully");
            Ok((
                StatusCode::OK,
                Json(BatchSwapResponse {
                    signature: None,
                    transaction: Some(transaction),
                    status: "success".to_string(),
                    last_valid_block_height: Some(last_valid_block_height),
                    error: None,
                })
            ))
        }
        Err(e) => {
            error!("Failed to build batch swap transaction: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BatchSwapResponse {
                    signature: None,
                    transaction: None,
                    status: "error".to_string(),
                    last_valid_block_height: None,
                    error: Some(e),
                })
            ))
        }
    }
}

/// Handle execute swap request
pub async fn handle_execute_swap_app_state(
    State(plugin): State<Arc<BatchSwapRouterPlugin>>,
    Json(request): Json<ExecuteSwapRequest>,
) -> Result<(StatusCode, Json<BatchSwapResponse>), (StatusCode, Json<BatchSwapResponse>)> {
    info!("Execute swap request received");

    // Validate request
    if let Err(e) = validate_execute_swap_request(&request) {
        error!("Execute swap validation failed: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(BatchSwapResponse {
                signature: None,
                transaction: None,
                status: "error".to_string(),
                last_valid_block_height: None,
                error: Some(e.to_string()),
            })
        ));
    }

    // Get Solana state
    let solana_state = plugin.solana_state.as_ref()
        .ok_or_else(|| {
            error!("Plugin not initialized");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BatchSwapResponse {
                    signature: None,
                    transaction: None,
                    status: "error".to_string(),
                    last_valid_block_height: None,
                    error: Some("Plugin not initialized".to_string()),
                })
            )
        })?;

    // Build transaction
    match build_execute_swap_transaction(solana_state, plugin.program_id(), &request).await {
        Ok((transaction, last_valid_block_height)) => {
            info!("Execute swap transaction built successfully");
            Ok((
                StatusCode::OK,
                Json(BatchSwapResponse {
                    signature: None,
                    transaction: Some(transaction),
                    status: "success".to_string(),
                    last_valid_block_height: Some(last_valid_block_height),
                    error: None,
                })
            ))
        }
        Err(e) => {
            error!("Failed to build execute swap transaction: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BatchSwapResponse {
                    signature: None,
                    transaction: None,
                    status: "error".to_string(),
                    last_valid_block_height: None,
                    error: Some(e),
                })
            ))
        }
    }
}

/// Handle health check request
pub async fn handle_health_app_state(
    State(plugin): State<Arc<BatchSwapRouterPlugin>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match plugin.health_check().await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "healthy",
            "name": plugin.name(),
            "version": plugin.version(),
            "program_id": plugin.program_id().to_string(),
        }))),
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "unhealthy",
                "error": e.to_string(),
            })),
        )),
    }
}

/// Handle metadata request
pub async fn handle_metadata_app_state(
    State(plugin): State<Arc<BatchSwapRouterPlugin>>,
) -> Json<ContractMetadata> {
    Json(plugin.metadata())
}

/// Create routes for batch swap router plugin
pub fn create_batch_swap_routes(plugin: Arc<BatchSwapRouterPlugin>) -> axum::Router<Arc<BatchSwapRouterPlugin>> {
    axum::Router::new()
        .route("/batch-swap", axum::routing::post(handle_batch_swap_app_state))
        .route("/execute-swap", axum::routing::post(handle_execute_swap_app_state))
        .route("/health", axum::routing::get(handle_health_app_state))
        .route("/metadata", axum::routing::get(handle_metadata_app_state))
        .with_state(plugin)
}

