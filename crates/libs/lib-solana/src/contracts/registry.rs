//! # Contract Registry
//!
//! Manages registered contract plugins and provides access to them.

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::contracts::plugin::{ContractPlugin, PluginError};

/// Contract plugin registry
pub struct ContractRegistry {
    plugins: Arc<RwLock<Vec<Arc<dyn ContractPlugin>>>>,
}

impl ContractRegistry {
    /// Create a new contract registry
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Register a contract plugin
    pub async fn register(&self, plugin: Arc<dyn ContractPlugin>) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write().await;
        plugins.push(plugin);
        Ok(())
    }
    
    /// Get a plugin by name
    pub async fn get(&self, name: &str) -> Option<Arc<dyn ContractPlugin>> {
        let plugins = self.plugins.read().await;
        plugins.iter().find(|p| p.name() == name).map(Arc::clone)
    }
    
    /// List all registered plugins
    pub async fn list(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.iter().map(|p| p.name().to_string()).collect()
    }
    
    /// Get all plugins
    pub async fn get_all(&self) -> Vec<Arc<dyn ContractPlugin>> {
        let plugins = self.plugins.read().await;
        plugins.clone()
    }
}

impl Default for ContractRegistry {
    fn default() -> Self {
        Self::new()
    }
}

