//! # Plugin Loader
//!
//! Loads and initializes Solana contract plugins from configuration.
//!
//! This module provides functionality to dynamically load and register contract plugins
//! based on configuration. It serves as the entry point for initializing contract integrations
//! at application startup.
//!
//! ## Architecture
//!
//! ```text
//! Application Startup
//!     ↓
//! PluginLoader::load_from_config()
//!     ↓
//! For each enabled contract:
//!     ├─> Create plugin instance
//!     ├─> Initialize with config
//!     └─> Register in ContractRegistry
//!     ↓
//! Plugins available via ContractRegistry
//! ```
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use backend::solana::contracts::{PluginLoader, ContractRegistry};
//! use std::collections::HashMap;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = Arc::new(ContractRegistry::new());
//! let loader = PluginLoader::new(registry.clone());
//!
//! // Build configuration map
//! let mut contracts = HashMap::new();
//! contracts.insert("batch-swap-router".to_string(), ContractConfig {
//!     program_id: /* ... */,
//!     enabled: true,
//!     cluster: Some(Cluster::Devnet),
//!     rpc_url: None,
//! });
//!
//! // Load all enabled plugins
//! loader.load_from_config(
//!     contracts,
//!     Cluster::Devnet,
//!     "https://api.devnet.solana.com".to_string(),
//! ).await?;
//!
//! // Plugins are now available via registry
//! let plugin = registry.get("batch-swap-router").await;
//! # Ok(())
//! # }
//! ```
//!
//! ## Configuration Format
//!
//! Contracts are configured using a `HashMap<String, ContractConfig>`:
//!
//! - **Key**: Contract identifier (e.g., "batch-swap-router")
//! - **Value**: [`ContractConfig`] with program ID, cluster, RPC URL, etc.
//!
//! ## Supported Plugins
//!
//! - **batch-swap-router**: Batch swap router contract (see [`BatchSwapRouterPlugin`])
//!
//! Additional plugins can be added by extending `load_from_config()` method.

use std::collections::HashMap;
use crate::contracts::{
    plugin::{ContractPlugin, PluginConfig, PluginError, Cluster, CommitmentLevel},
    registry::ContractRegistry,
    batch_swap::BatchSwapRouterPlugin,
};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

/// Configuration for a Solana contract plugin.
///
/// Specifies how a contract plugin should be initialized, including the program ID,
/// network cluster, RPC endpoint, and whether it should be enabled.
///
/// # Fields
///
/// * `program_id` - The on-chain program ID (Pubkey) for this contract
/// * `enabled` - Whether this plugin should be loaded and registered
/// * `cluster` - Optional network cluster (overrides default if set)
/// * `rpc_url` - Optional RPC URL (overrides default if set)
///
/// # Example
///
/// ```rust
/// use backend::solana::contracts::loader::ContractConfig;
/// use backend::solana::contracts::plugin::Cluster;
/// use solana_sdk::pubkey::Pubkey;
/// use std::str::FromStr;
///
/// let config = ContractConfig {
///     program_id: Pubkey::from_str("HS63bw1V1qTM5uWf92q3uaFdqogrc4SN9qUJSR8aqBMx").unwrap(),
///     enabled: true,
///     cluster: Some(Cluster::Devnet),
///     rpc_url: Some("https://api.devnet.solana.com".to_string()),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ContractConfig {
    /// The Solana program ID for this contract.
    pub program_id: Pubkey,
    
    /// Whether this plugin should be loaded and registered.
    ///
    /// If `false`, the plugin will be skipped even if present in the configuration map.
    pub enabled: bool,
    
    /// Optional network cluster (overrides default if set).
    ///
    /// If `None`, uses the default cluster from `load_from_config()`.
    pub cluster: Option<Cluster>,
    
    /// Optional RPC URL (overrides default if set).
    ///
    /// If `None`, uses the default RPC URL from `load_from_config()`.
    pub rpc_url: Option<String>,
}

/// Loads and initializes contract plugins from configuration.
///
/// The plugin loader reads configuration and dynamically loads contract plugins,
/// initializes them, and registers them in the contract registry for use throughout
/// the application.
///
/// # Fields
///
/// * `registry` - The contract registry where plugins are registered
///
/// # Example
///
/// ```rust,no_run
/// use backend::solana::contracts::{PluginLoader, ContractRegistry};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let registry = Arc::new(ContractRegistry::new());
/// let loader = PluginLoader::new(registry);
/// // Load plugins...
/// # Ok(())
/// # }
/// ```
pub struct PluginLoader {
    /// The contract registry where plugins are registered after initialization.
    registry: Arc<ContractRegistry>,
}

impl PluginLoader {
    /// Create a new plugin loader with the given contract registry.
    ///
    /// # Arguments
    ///
    /// * `registry` - The contract registry where plugins will be registered
    ///
    /// # Returns
    ///
    /// A new `PluginLoader` instance ready to load plugins.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::solana::contracts::{PluginLoader, ContractRegistry};
    /// use std::sync::Arc;
    ///
    /// let registry = Arc::new(ContractRegistry::new());
    /// let loader = PluginLoader::new(registry);
    /// ```
    pub fn new(registry: Arc<ContractRegistry>) -> Self {
        Self { registry }
    }
    
    /// Load all enabled plugins from configuration.
    ///
    /// Iterates through the configuration map and loads each enabled plugin.
    /// Each plugin is initialized with its configuration and registered in the
    /// contract registry.
    ///
    /// # Arguments
    ///
    /// * `contracts` - Map of contract identifiers to their configuration
    /// * `default_cluster` - Default network cluster (used if config doesn't specify)
    /// * `default_rpc_url` - Default RPC URL (used if config doesn't specify)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All enabled plugins loaded successfully
    /// * `Err(PluginError)` - If any plugin fails to initialize
    ///
    /// # Configuration Precedence
    ///
    /// For each contract, configuration values are resolved in this order:
    /// 1. Contract-specific values in `ContractConfig` (highest priority)
    /// 2. Default values passed to this method (fallback)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::solana::contracts::loader::{PluginLoader, ContractConfig};
    /// use backend::solana::contracts::{ContractRegistry, plugin::Cluster};
    /// use std::collections::HashMap;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = Arc::new(ContractRegistry::new());
    /// let loader = PluginLoader::new(registry);
    ///
    /// let mut contracts = HashMap::new();
    /// // Add contract configurations...
    ///
    /// loader.load_from_config(
    ///     contracts,
    ///     Cluster::Devnet,
    ///     "https://api.devnet.solana.com".to_string(),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Error Handling
    ///
    /// If any plugin fails to initialize, this method returns an error immediately.
    /// Previously loaded plugins remain registered in the registry.
    pub async fn load_from_config(
        &self,
        contracts: HashMap<String, ContractConfig>,
        default_cluster: Cluster,
        default_rpc_url: String,
    ) -> Result<(), PluginError> {
        // Load batch swap router plugin if configured and enabled
        if let Some(config) = contracts.get("batch-swap-router") {
            if config.enabled {
                self.load_batch_swap_router(config, &default_cluster, &default_rpc_url).await?;
            }
        }
        
        // TODO: Add other plugins here as they are implemented
        // Example for future plugins:
        // if let Some(config) = contracts.get("limit-orders") {
        //     if config.enabled {
        //         self.load_limit_orders(config, &default_cluster, &default_rpc_url).await?;
        //     }
        // }
        
        Ok(())
    }
    
    /// Load and initialize the batch swap router plugin.
    ///
    /// This is an internal helper method that creates a `BatchSwapRouterPlugin` instance,
    /// initializes it with the provided configuration, and registers it in the registry.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the batch swap router contract
    /// * `default_cluster` - Default cluster if config doesn't specify
    /// * `default_rpc_url` - Default RPC URL if config doesn't specify
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Plugin loaded and registered successfully
    /// * `Err(PluginError)` - If plugin initialization fails
    ///
    /// # Configuration Resolution
    ///
    /// Uses `config.cluster` if present, otherwise `default_cluster`.
    /// Uses `config.rpc_url` if present, otherwise `default_rpc_url`.
    async fn load_batch_swap_router(
        &self,
        config: &ContractConfig,
        default_cluster: &Cluster,
        default_rpc_url: &str,
    ) -> Result<(), PluginError> {
        // Create plugin instance
        let mut plugin = BatchSwapRouterPlugin::new();
        
        // Resolve configuration values (contract-specific overrides defaults)
        let plugin_config = PluginConfig {
            program_id: config.program_id,
            cluster: config.cluster.clone().unwrap_or_else(|| default_cluster.clone()),
            rpc_url: config.rpc_url.clone().unwrap_or_else(|| default_rpc_url.to_string()),
            commitment: CommitmentLevel::Confirmed, // Always use Confirmed commitment
            enabled: config.enabled,
        };
        
        // Initialize plugin (connects to RPC, validates program ID, etc.)
        plugin.initialize(plugin_config).await?;
        
        // Register plugin in registry (makes it available via registry.get())
        self.registry.register(Arc::new(plugin)).await?;
        
        Ok(())
    }
    
    /// Get a reference to the contract registry.
    ///
    /// Returns a cloned `Arc` to the registry, allowing access to all registered plugins.
    ///
    /// # Returns
    ///
    /// An `Arc<ContractRegistry>` that can be used to query and access registered plugins.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use backend::solana::contracts::PluginLoader;
    ///
    /// # async fn example(loader: &PluginLoader) -> Option<()> {
    /// let registry = loader.registry();
    /// let plugin = registry.get("batch-swap-router").await?;
    /// # Some(())
    /// # }
    /// ```
    pub fn registry(&self) -> Arc<ContractRegistry> {
        Arc::clone(&self.registry)
    }
}

