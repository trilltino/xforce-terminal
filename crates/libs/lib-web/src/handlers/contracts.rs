//! # Contract Handlers
//!
//! HTTP handlers for contract-related API endpoints.
//!
//! This module provides HTTP handlers for interacting with Solana contract plugins.
//! It includes generic contract management endpoints (list, metadata, health) and
//! contract-specific endpoints (batch swap, execute swap).

use axum::{
    extract::State,
    response::Json,
};
use serde::Serialize;
use std::sync::Arc;
use tracing::instrument;
use lib_solana::contracts::ContractRegistry;

/// Contract route handlers
///
/// These handlers are registered directly in server.rs with AppState.
/// They extract Arc<ContractRegistry> from AppState via FromRef implementation.

#[derive(Serialize)]
pub struct ContractListResponse {
    contracts: Vec<String>,
}

#[derive(Serialize)]
pub struct ContractInfo {
    name: String,
    version: String,
    program_id: String,
    description: String,
}

// Handler functions that extract from AppState (via FromRef)
// These can be used directly in the main router with AppState

#[instrument(skip(registry))]
pub async fn list_contracts_handler(
    State(registry): State<Arc<ContractRegistry>>,
) -> Json<ContractListResponse> {
    let contracts = registry.list().await;
    Json(ContractListResponse { contracts })
}

#[instrument(skip(registry), fields(contract_name = %name))]
pub async fn get_contract_handler(
    State(registry): State<Arc<ContractRegistry>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<ContractInfo>, axum::response::Response> {
    let plugin = registry.get(&name).await
        .ok_or_else(|| {
            axum::response::Response::builder()
                .status(axum::http::StatusCode::NOT_FOUND)
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(serde_json::json!({ "error": "Contract not found" }).to_string().into())
                .expect("Failed to build error response")
        })?;
    
    let metadata = plugin.metadata();
    Ok(Json(ContractInfo {
        name: metadata.name,
        version: metadata.version,
        program_id: metadata.program_id.to_string(),
        description: metadata.description,
    }))
}

#[instrument(skip(registry), fields(contract_name = %name))]
pub async fn health_check_handler(
    State(registry): State<Arc<ContractRegistry>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    let plugin = registry.get(&name).await
        .ok_or_else(|| {
            axum::response::Response::builder()
                .status(axum::http::StatusCode::NOT_FOUND)
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(serde_json::json!({ "error": "Contract not found" }).to_string().into())
                .expect("Failed to build error response")
        })?;
    
    match plugin.health_check().await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "healthy",
            "name": plugin.name(),
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "status": "unhealthy",
            "name": plugin.name(),
            "error": e.to_string(),
        }))),
    }
}

#[instrument(skip(registry), fields(contract_name = %name))]
pub async fn get_metadata_handler(
    State(registry): State<Arc<ContractRegistry>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<lib_solana::contracts::plugin::ContractMetadata>, axum::response::Response> {
    let plugin = registry.get(&name).await
        .ok_or_else(|| {
            axum::response::Response::builder()
                .status(axum::http::StatusCode::NOT_FOUND)
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(serde_json::json!({ "error": "Contract not found" }).to_string().into())
                .expect("Failed to build error response")
        })?;
    
    Ok(Json(plugin.metadata()))
}

