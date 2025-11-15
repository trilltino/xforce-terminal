//! # Solana RPC Client
//!
//! Provides a high-level wrapper around the Solana RPC client with network management
//! and connection pooling.
//!
//! ## Features
//!
//! - **Network Selection**: Easy switching between Mainnet and Devnet
//! - **Helius Integration**: Support for premium RPC endpoints with API keys
//! - **Account Queries**: Retrieve account data by public key
//! - **Transaction Submission**: Send and confirm transactions on-chain
//! - **Epoch Information**: Query current blockchain epoch and slot details
//! - **Health Checks**: Verify RPC endpoint connectivity
//!
//! ## Network Selection
//!
//! The client supports two Solana networks:
//! - **Mainnet**: Production network with real assets
//! - **Devnet**: Test network with free test tokens
//!
//! ## RPC Endpoints
//!
//! ### Mainnet (with Helius API key)
//! - URL: `https://mainnet.helius-rpc.com/?api-key={key}`
//! - Rate Limit: 100+ req/sec (depends on plan)
//! - Recommended for: Production applications
//!
//! ### Mainnet (without API key)
//! - URL: `https://api.mainnet-beta.solana.com`
//! - Rate Limit: ~10 req/sec
//! - Recommended for: Testing and development
//!
//! ### Devnet
//! - URL: `https://api.devnet.solana.com`
//! - Rate Limit: ~10 req/sec
//! - Recommended for: Development and integration testing
//!
//! ## Example
//!
//! ```rust
//! use backend::solana::client::{SolanaClient, Network};
//! use solana_sdk::pubkey::Pubkey;
//! use std::str::FromStr;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create mainnet client with Helius
//! let client = SolanaClient::new(
//!     Network::Mainnet,
//!     Some("your-helius-api-key".to_string())
//! );
//!
//! // Query account data
//! let pubkey = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
//! let account = client.get_account(&pubkey).await?;
//! println!("Account lamports: {}", account.lamports);
//!
//! // Check RPC health
//! client.health_check().await?;
//!
//! // Get current epoch info
//! let epoch_info = client.get_epoch_info().await?;
//! println!("Current epoch: {}", epoch_info.epoch);
//! # Ok(())
//! # }
//! ```

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use solana_sdk::{clock::Epoch, pubkey::Pubkey};
use std::sync::Arc;
use tracing::info;

/// Simplified epoch information from the Solana blockchain.
///
/// Contains the current epoch number and slot position within that epoch.
/// Epochs on Solana last approximately 2-3 days.
///
/// # Fields
///
/// * `epoch` - Current epoch number (increments every ~2-3 days)
/// * `slot_index` - Current slot position within the epoch (0 to slots_in_epoch-1)
/// * `slots_in_epoch` - Total number of slots in this epoch (typically ~432,000)
///
/// # Example
///
/// ```rust
/// # use backend::solana::client::EpochInfo;
/// let epoch_info = EpochInfo {
///     epoch: 500,
///     slot_index: 100_000,
///     slots_in_epoch: 432_000,
/// };
///
/// let progress_pct = (epoch_info.slot_index as f64 / epoch_info.slots_in_epoch as f64) * 100.0;
/// println!("Epoch {} is {:.1}% complete", epoch_info.epoch, progress_pct);
/// ```
#[derive(Debug, Clone)]
pub struct EpochInfo {
    pub epoch: Epoch,
    pub slot_index: u64,
    pub slots_in_epoch: u64,
}

/// Solana network selection.
///
/// Determines which Solana cluster the client connects to. Each network
/// has different characteristics:
///
/// - **Mainnet**: Production network with real economic value
/// - **Devnet**: Test network for development with free test tokens
///
/// # Example
///
/// ```rust
/// use backend::solana::client::Network;
///
/// let mainnet = Network::Mainnet;
/// let devnet = Network::Devnet;
///
/// // Use in client creation
/// // let client = SolanaClient::new(Network::Mainnet, None);
/// ```
#[derive(Debug, Clone)]
pub enum Network {
    /// Solana mainnet-beta (production network)
    Mainnet,
    /// Solana devnet (test network)
    Devnet,
}

/// High-level Solana RPC client wrapper.
///
/// Provides a convenient interface to the Solana blockchain with automatic
/// network configuration and connection pooling. All methods are async and
/// return descriptive errors.
///
/// The client wraps the official `solana_client::RpcClient` with additional
/// network management and error handling.
///
/// # Example
///
/// ```rust
/// use backend::solana::client::{SolanaClient, Network};
///
/// # async fn example() -> anyhow::Result<()> {
/// // Create client
/// let client = SolanaClient::new(Network::Devnet, None);
///
/// // Use client methods
/// client.health_check().await?;
/// let epoch = client.get_epoch_info().await?;
/// println!("Current epoch: {}", epoch.epoch);
/// # Ok(())
/// # }
/// ```
pub struct SolanaClient {
    rpc: Arc<RpcClient>,
    network: Network,
}

/// Builder for configuring SolanaClient.
///
/// Allows fluent configuration of client settings before building.
#[derive(Debug, Clone)]
pub struct SolanaClientBuilder {
    network: Option<Network>,
    helius_api_key: Option<String>,
    custom_rpc_url: Option<String>,
}

impl Default for SolanaClientBuilder {
    fn default() -> Self {
        Self {
            network: Some(Network::Devnet),
            helius_api_key: None,
            custom_rpc_url: None,
        }
    }
}

impl SolanaClientBuilder {
    /// Set the Solana network.
    pub fn network(mut self, network: Network) -> Self {
        self.network = Some(network);
        self
    }

    /// Set the Helius API key for premium RPC access.
    pub fn helius_api_key(mut self, key: String) -> Self {
        self.helius_api_key = Some(key);
        self
    }

    /// Set a custom RPC URL (overrides network-based URL).
    pub fn custom_rpc_url(mut self, url: String) -> Self {
        self.custom_rpc_url = Some(url);
        self
    }

    /// Build the SolanaClient with configured settings.
    pub fn build(self) -> SolanaClient {
        let network = self.network.unwrap_or(Network::Devnet);
        let rpc_url = if let Some(custom_url) = self.custom_rpc_url {
            custom_url
        } else {
            match network {
                Network::Mainnet => {
                    if let Some(key) = self.helius_api_key {
                        format!("https://mainnet.helius-rpc.com/?api-key={}", key)
                    } else {
                        "https://api.mainnet-beta.solana.com".to_string()
                    }
                }
                Network::Devnet => "https://api.devnet.solana.com".to_string(),
            }
        };

        let rpc = Arc::new(RpcClient::new(rpc_url));
        SolanaClient { rpc, network }
    }
}

impl SolanaClient {
    /// Create a new Solana RPC client using a builder for configuration.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use lib_solana::client::{SolanaClient, Network};
    ///
    /// let client = SolanaClient::builder()
    ///     .network(Network::Mainnet)
    ///     .helius_api_key("your-api-key".to_string())
    ///     .build();
    /// ```
    pub fn builder() -> SolanaClientBuilder {
        SolanaClientBuilder::default()
    }

    /// Create a new Solana RPC client.
    ///
    /// Initializes a connection to the specified Solana network. The connection
    /// is lazy - actual network requests only happen when methods are called.
    ///
    /// # RPC Endpoint Selection
    ///
    /// The RPC URL is determined by network and API key:
    /// - Mainnet + API key: `https://mainnet.helius-rpc.com/?api-key={key}`
    /// - Mainnet without key: `https://api.mainnet-beta.solana.com`
    /// - Devnet: `https://api.devnet.solana.com`
    ///
    /// # Arguments
    ///
    /// * `network` - Target network (Mainnet or Devnet)
    /// * `helius_api_key` - Optional Helius API key for premium RPC access
    ///
    /// # Returns
    ///
    /// A configured `SolanaClient` ready to make RPC calls.
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::solana::client::{SolanaClient, Network};
    ///
    /// // Production client with Helius (recommended)
    /// let mainnet = SolanaClient::new(
    ///     Network::Mainnet,
    ///     Some("your-helius-api-key".to_string())
    /// );
    ///
    /// // Development client (free)
    /// let devnet = SolanaClient::new(Network::Devnet, None);
    /// ```
    ///
    /// # Performance
    ///
    /// - With Helius: 100+ req/sec rate limit
    /// - Public endpoints: ~10 req/sec rate limit
    /// - Connection pooling: HTTP connections are reused automatically
    pub fn new(network: Network, helius_api_key: Option<String>) -> Self {
        let url = match network {
            Network::Mainnet => {
                if let Some(key) = helius_api_key {
                    format!("https://mainnet.helius-rpc.com/?api-key={}", key)
                } else {
                    "https://api.mainnet-beta.solana.com".to_string()
                }
            }
            Network::Devnet => "https://api.devnet.solana.com".to_string(),
        };

        info!("🔗 Connecting to Solana RPC: {}", url);

        Self {
            rpc: Arc::new(RpcClient::new(url)),
            network,
        }
    }

    /// Retrieve account data from the blockchain.
    ///
    /// Fetches the complete account state including balance (lamports),
    /// owner program, data, and executable flag.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - Public key of the account to query
    ///
    /// # Returns
    ///
    /// * `Ok(Account)` - Account data including lamports, owner, and data bytes
    /// * `Err(_)` - If account doesn't exist or RPC request fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::solana::client::{SolanaClient, Network};
    /// use solana_sdk::pubkey::Pubkey;
    /// use std::str::FromStr;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = SolanaClient::new(Network::Mainnet, None);
    ///
    /// // Query SOL system program account
    /// let pubkey = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    /// let account = client.get_account(&pubkey).await?;
    ///
    /// println!("Balance: {} SOL", account.lamports as f64 / 1e9);
    /// println!("Owner: {}", account.owner);
    /// println!("Data size: {} bytes", account.data.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_account(&self, pubkey: &Pubkey) -> anyhow::Result<solana_sdk::account::Account> {
        self.rpc.get_account(pubkey).await
            .map_err(|e| anyhow::anyhow!("RPC error: {}", e))
    }

    /// Get transaction signatures for an address.
    ///
    /// Returns a list of transaction signatures involving the specified address,
    /// ordered from most recent to oldest. Each entry includes the signature,
    /// slot number, block time, and status.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - Address to query transaction history for
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<...>)` - List of transaction signatures with metadata
    /// * `Err(_)` - If RPC request fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::solana::client::{SolanaClient, Network};
    /// use solana_sdk::pubkey::Pubkey;
    /// use std::str::FromStr;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = SolanaClient::new(Network::Mainnet, None);
    /// let pubkey = Pubkey::from_str("YourAddressHere")?;
    ///
    /// let signatures = client.get_signatures_for_address(&pubkey).await?;
    /// for sig in signatures.iter().take(5) {
    ///     println!("Signature: {}", sig.signature);
    ///     println!("  Slot: {}", sig.slot);
    ///     if let Some(err) = &sig.err {
    ///         println!("  Error: {:?}", err);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Limitations
    ///
    /// - Default limit: 1000 most recent signatures
    /// - Historical data may be pruned after ~2 epochs on some RPC nodes
    pub async fn get_signatures_for_address(
        &self,
        pubkey: &Pubkey,
    ) -> anyhow::Result<Vec<RpcConfirmedTransactionStatusWithSignature>> {
        self.rpc
            .get_signatures_for_address(pubkey)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get signatures: {}", e))
    }

    /// Get current blockchain epoch information.
    ///
    /// Returns the current epoch number, slot position within the epoch,
    /// and total slots in the epoch. Useful for understanding blockchain
    /// timing and scheduling.
    ///
    /// # Returns
    ///
    /// * `Ok(EpochInfo)` - Current epoch details
    /// * `Err(_)` - If RPC request fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::solana::client::{SolanaClient, Network};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = SolanaClient::new(Network::Mainnet, None);
    /// let epoch_info = client.get_epoch_info().await?;
    ///
    /// println!("Current epoch: {}", epoch_info.epoch);
    /// println!("Slot: {}/{}", epoch_info.slot_index, epoch_info.slots_in_epoch);
    ///
    /// let progress = (epoch_info.slot_index as f64 / epoch_info.slots_in_epoch as f64) * 100.0;
    /// println!("Epoch progress: {:.2}%", progress);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_epoch_info(&self) -> anyhow::Result<EpochInfo> {
        let epoch_info = self.rpc.get_epoch_info().await
            .map_err(|e| anyhow::anyhow!("Failed to get epoch info: {}", e))?;

        Ok(EpochInfo {
            epoch: epoch_info.epoch,
            slot_index: epoch_info.slot_index,
            slots_in_epoch: epoch_info.slots_in_epoch,
        })
    }

    /// Check if the RPC endpoint is healthy and responsive.
    ///
    /// Makes a lightweight request to verify the RPC connection is working.
    /// Useful for health checks in monitoring systems or before making
    /// critical requests.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - RPC endpoint is healthy
    /// * `Err(_)` - RPC endpoint is unreachable or unhealthy
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::solana::client::{SolanaClient, Network};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = SolanaClient::new(Network::Mainnet, None);
    ///
    /// match client.health_check().await {
    ///     Ok(_) => println!("RPC is healthy"),
    ///     Err(e) => println!("RPC is down: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn health_check(&self) -> anyhow::Result<()> {
        let _ = self.rpc.get_version().await
            .map_err(|e| anyhow::anyhow!("Health check failed: {}", e))?;
        Ok(())
    }

    /// Get the network this client is connected to.
    ///
    /// Returns a reference to the network enum (Mainnet or Devnet).
    ///
    /// # Returns
    ///
    /// Reference to the `Network` enum indicating which network this client uses.
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::solana::client::{SolanaClient, Network};
    ///
    /// let client = SolanaClient::new(Network::Devnet, None);
    ///
    /// match client.network() {
    ///     Network::Mainnet => println!("Using mainnet"),
    ///     Network::Devnet => println!("Using devnet"),
    /// }
    /// ```
    pub fn network(&self) -> &Network {
        &self.network
    }

    /// Send a signed transaction to the Solana blockchain.
    ///
    /// Submits a fully signed transaction to the network and waits for
    /// confirmation. The transaction must be properly signed before calling
    /// this method.
    ///
    /// # Transaction Confirmation
    ///
    /// This method uses `send_and_confirm_transaction` which:
    /// - Submits the transaction to the network
    /// - Waits for confirmation (typically 1-3 seconds)
    /// - Returns the transaction signature on success
    ///
    /// # Arguments
    ///
    /// * `transaction` - Signed Solana transaction ready for submission
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Transaction signature as base58 string
    /// * `Err(_)` - If transaction fails simulation, submission, or confirmation
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::solana::client::{SolanaClient, Network};
    /// use solana_sdk::transaction::Transaction;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = SolanaClient::new(Network::Devnet, None);
    ///
    /// // Assume we have a signed transaction
    /// # let transaction = Transaction::default(); // placeholder
    /// // let transaction = create_and_sign_transaction(...);
    ///
    /// let signature = client.send_transaction(&transaction).await?;
    /// println!("Transaction confirmed: {}", signature);
    /// println!("View on explorer: https://explorer.solana.com/tx/{}", signature);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Error Conditions
    ///
    /// - Transaction simulation fails (invalid instructions)
    /// - Insufficient balance for fees
    /// - Invalid signatures
    /// - Network timeout or congestion
    /// - Blockhash expired (transaction too old)
    pub async fn send_transaction(&self, transaction: &solana_sdk::transaction::Transaction) -> anyhow::Result<String> {
        let signature = self.rpc
            .send_and_confirm_transaction(transaction)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;

        Ok(signature.to_string())
    }

    /// Get the latest blockhash from the blockchain.
    ///
    /// Returns the most recent blockhash, which is required for building transactions.
    /// Blockhashes expire after ~60 seconds, so they should be fetched close to when
    /// the transaction will be submitted.
    ///
    /// # Returns
    ///
    /// * `Ok(Hash)` - Latest blockhash
    /// * `Err(_)` - If RPC request fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::solana::client::{SolanaClient, Network};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = SolanaClient::new(Network::Mainnet, None);
    /// let blockhash = client.get_latest_blockhash().await?;
    /// println!("Latest blockhash: {}", blockhash);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_latest_blockhash(&self) -> anyhow::Result<solana_sdk::hash::Hash> {
        self.rpc.get_latest_blockhash().await
            .map_err(|e| anyhow::anyhow!("Failed to get latest blockhash: {}", e))
    }
}
