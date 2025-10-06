use serde::{Deserialize, Serialize};
use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdrConfig {
    pub contract_id: String,
    pub network_passphrase: String,
    pub rpc_url: String,
}

impl XdrConfig {
    pub fn validate(&self) -> Result<()> {
        if self.contract_id.is_empty() {
            return Err(AppError::Config("Contract ID cannot be empty".to_string()));
        }
        if self.network_passphrase.is_empty() {
            return Err(AppError::Config("Network passphrase cannot be empty".to_string()));
        }
        if self.rpc_url.is_empty() {
            return Err(AppError::Config("RPC URL cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl Default for XdrConfig {
    fn default() -> Self {
        Self {
            contract_id: "CBGTG6AXOYEWEH34QO5VKIL5CCUI3O24MWSQ6PLGS3WWQW2T4U3NLEXI".to_string(), // Example contract
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        }
    }
}
