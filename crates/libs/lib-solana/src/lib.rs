//! # Solana Library
//!
//! Solana blockchain integration including RPC client, Jupiter, Pyth, and contract plugins.

// Declare all modules
pub mod client;
pub mod jupiter;
pub mod contracts;
pub mod pyth;
pub mod types;
pub mod cache;
pub mod candle_aggregator;
pub mod price_stream;
pub mod spl_token;

// Import mod_rs to re-export its content
pub mod mod_rs;

// Create solana module alias by re-exporting mod_rs
pub use mod_rs as solana;

// Re-export commonly used types from root for convenience
pub use mod_rs::{SolanaState, Network};
pub use contracts::{ContractRegistry, PluginLoader};
pub use price_stream::PriceStreamServer;

