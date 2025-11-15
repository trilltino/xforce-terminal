//! # Solana Contracts Module
//!
//! Provides a plugin system for integrating Solana smart contracts into the Axum backend.
//! Contracts are loaded as plugins that can be dynamically registered and managed.

pub mod plugin;
pub mod registry;
pub mod loader;
pub mod batch_swap;
pub mod idl_handler;
pub mod transaction_builder;

// Use the real contracts client library
pub use xforce_terminal_contracts_client::{
    SwapParams,
    get_batch_swap_router_program_id,
};

pub use plugin::{ContractPlugin, PluginConfig, PluginError, ContractMetadata, Cluster, CommitmentLevel};
pub use registry::ContractRegistry;
pub use loader::PluginLoader;
pub use batch_swap::{
    BatchSwapRouterPlugin, 
    create_batch_swap_routes,
};
pub use batch_swap::routes::{
    handle_batch_swap_app_state,
    handle_execute_swap_app_state,
    handle_health_app_state,
    handle_metadata_app_state,
};
pub use transaction_builder::BatchSwapTransactionBuilder;
pub use idl_handler::{IdlHandler, load_batch_swap_idl};

