//! # Batch Swap Router Plugin
//!
//! Plugin implementation for the batch swap router contract.

// region: --- Modules
pub mod types;
pub mod validator;
pub mod builder;
pub mod routes;
// endregion: --- Modules

// region: --- Plugin Implementation
use async_trait::async_trait;
use axum::Router;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tracing::info;
use crate::contracts::plugin::{
    ContractPlugin, PluginConfig, PluginError, ContractMetadata, Cluster,
};
use crate::mod_rs::{SolanaState, Network};
use crate::contracts::get_batch_swap_router_program_id;

/// Batch swap router contract plugin
#[derive(Clone)]
pub struct BatchSwapRouterPlugin {
    name: String,
    program_id: Pubkey,
    version: String,
    config: Option<PluginConfig>,
    solana_state: Option<Arc<SolanaState>>,
    rpc_url: String,
}

impl BatchSwapRouterPlugin {
    /// Create a new batch swap router plugin
    pub fn new() -> Self {
        // Get program ID from environment variable first (allows override)
        let program_id = if let Ok(env_program_id) = std::env::var("BATCH_SWAP_ROUTER_PROGRAM_ID") {
            env_program_id
                .parse()
                .unwrap_or_else(|_| {
                    tracing::warn!("Failed to parse BATCH_SWAP_ROUTER_PROGRAM_ID, using default");
                    let contract_pubkey = get_batch_swap_router_program_id();
                    Pubkey::try_from(contract_pubkey.as_ref()).unwrap_or_else(|_| {
                        let bytes: [u8; 32] = contract_pubkey.to_bytes();
                        Pubkey::from(bytes)
                    })
                })
        } else {
            let contract_pubkey = get_batch_swap_router_program_id();
            Pubkey::try_from(contract_pubkey.as_ref()).unwrap_or_else(|_| {
                let bytes: [u8; 32] = contract_pubkey.to_bytes();
                Pubkey::from(bytes)
            })
        };
        
        info!("Batch swap router plugin created with program_id: {}", program_id);
        
        Self {
            name: "batch-swap-router".to_string(),
            program_id,
            version: "0.1.0".to_string(),
            config: None,
            solana_state: None,
            rpc_url: String::new(),
        }
    }

    /// Get the RPC URL from Solana state
    #[allow(dead_code)]
    fn get_rpc_url(&self) -> String {
        if !self.rpc_url.is_empty() {
            return self.rpc_url.clone();
        }
        
        self.solana_state.as_ref()
            .map(|_state| {
                std::env::var("SOLANA_RPC_URL")
                    .unwrap_or_else(|_| {
                        if let Ok(key) = std::env::var("HELIUS_API_KEY") {
                            format!("https://mainnet.helius-rpc.com/?api-key={}", key)
                        } else {
                            "https://api.devnet.solana.com".to_string()
                        }
                    })
            })
            .unwrap_or_else(|| "https://api.devnet.solana.com".to_string())
    }
}

#[async_trait]
impl ContractPlugin for BatchSwapRouterPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn program_id(&self) -> Pubkey {
        self.program_id
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    async fn initialize(&mut self, config: PluginConfig) -> Result<(), PluginError> {
        self.config = Some(config.clone());
        self.rpc_url = config.rpc_url.clone();
        
        // Initialize Solana state
        let helius_api_key = if config.rpc_url.contains("helius-rpc.com") {
            config.rpc_url.split("api-key=").nth(1).map(|s| s.to_string())
        } else {
            std::env::var("HELIUS_API_KEY").ok()
        };
        
        let network = match config.cluster {
            Cluster::Localnet => Network::Devnet,
            Cluster::Devnet => Network::Devnet,
            Cluster::Mainnet => Network::Mainnet,
        };
        
        let network_clone = network.clone();
        let solana_state = Arc::new(
            SolanaState::new(network, helius_api_key)
                .await
                .map_err(|e| PluginError::InitializationFailed(e.to_string()))?
        );
        
        self.solana_state = Some(solana_state);
        
        info!("Batch swap router plugin initialized: program_id={}, network={:?}", 
            self.program_id, network_clone);
        
        Ok(())
    }
    
    fn register_routes(&self, router: Router) -> Router {
        // Routes are registered separately in the main router
        router
    }
    
    fn metadata(&self) -> ContractMetadata {
        ContractMetadata {
            name: self.name.clone(),
            version: self.version.clone(),
            program_id: self.program_id,
            description: "Batch swap router for executing multiple swaps in a single transaction".to_string(),
            instructions: vec![
                "batch_swap".to_string(),
                "execute_swap".to_string(),
            ],
            events: vec![
                "BatchSwapEvent".to_string(),
                "SwapExecutedEvent".to_string(),
            ],
        }
    }
    
    async fn health_check(&self) -> Result<(), PluginError> {
        if self.config.is_none() || self.solana_state.is_none() {
            return Err(PluginError::ContractError("Plugin not initialized".to_string()));
        }
        
        if let Some(solana_state) = &self.solana_state {
            solana_state.rpc.get_epoch_info().await
                .map_err(|e| PluginError::NetworkError(format!("RPC health check failed: {}", e)))?;
        }
        
        Ok(())
    }
}

impl Default for BatchSwapRouterPlugin {
    fn default() -> Self {
        Self::new()
    }
}
// endregion: --- Plugin Implementation

// Re-export commonly used types
pub use types::*;
pub use routes::*;

