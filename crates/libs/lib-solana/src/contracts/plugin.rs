//! # Contract Plugin Trait
//!
//! Defines the interface that all Solana contract plugins must implement.

use async_trait::async_trait;
use axum::Router;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

/// Contract plugin trait that all contract integrations must implement
#[async_trait]
pub trait ContractPlugin: Send + Sync {
    /// Get the contract name
    fn name(&self) -> &str;
    
    /// Get the program ID
    fn program_id(&self) -> Pubkey;
    
    /// Get the contract version
    fn version(&self) -> &str;
    
    /// Initialize the plugin with configuration
    async fn initialize(&mut self, config: PluginConfig) -> Result<(), PluginError>;
    
    /// Register API routes for this contract
    fn register_routes(&self, router: Router) -> Router;
    
    /// Get contract metadata
    fn metadata(&self) -> ContractMetadata;
    
    /// Health check
    async fn health_check(&self) -> Result<(), PluginError>;
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub program_id: Pubkey,
    pub cluster: Cluster,
    pub rpc_url: String,
    pub commitment: CommitmentLevel,
    pub enabled: bool,
}

/// Network cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cluster {
    Localnet,
    Devnet,
    Mainnet,
}

impl Cluster {
    pub fn as_str(&self) -> &str {
        match self {
            Cluster::Localnet => "localnet",
            Cluster::Devnet => "devnet",
            Cluster::Mainnet => "mainnet",
        }
    }
}

/// Commitment level for Solana transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommitmentLevel {
    Processed,
    Confirmed,
    Finalized,
}

/// Contract metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub name: String,
    pub version: String,
    pub program_id: Pubkey,
    pub description: String,
    pub instructions: Vec<String>,
    pub events: Vec<String>,
}

/// Plugin errors
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Contract error: {0}")]
    ContractError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid program ID: {0}")]
    InvalidProgramId(String),
}

