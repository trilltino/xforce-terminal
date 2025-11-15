//! # Wallet Service
//!
//! Manages Solana wallet connections, keypair loading, and transaction signing.
//!
//! ## Features
//! - Load keypair from file or environment variable
//! - Generate new keypairs
//! - Sign transactions
//! - Query wallet balance
//! - RPC connection management

use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use solana_client::rpc_client::RpcClient;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Wallet connection errors
#[derive(Debug)]
pub enum WalletError {
    /// Failed to load keypair from file
    KeypairLoadError(String),
    /// Invalid keypair format
    InvalidKeypair(String),
    /// RPC connection error
    RpcError(String),
    /// Transaction signing error
    SigningError(String),
    /// Balance query error
    BalanceError(String),
    /// File I/O error
    IoError(std::io::Error),
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::KeypairLoadError(msg) => write!(f, "Keypair load error: {}", msg),
            WalletError::InvalidKeypair(msg) => write!(f, "Invalid keypair: {}", msg),
            WalletError::RpcError(msg) => write!(f, "RPC error: {}", msg),
            WalletError::SigningError(msg) => write!(f, "Signing error: {}", msg),
            WalletError::BalanceError(msg) => write!(f, "Balance error: {}", msg),
            WalletError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl Error for WalletError {}

impl From<std::io::Error> for WalletError {
    fn from(err: std::io::Error) -> Self {
        WalletError::IoError(err)
    }
}

/// Wallet connection status
#[derive(Debug, Clone, PartialEq)]
pub enum WalletStatus {
    /// Not connected
    Disconnected,
    /// Connected with wallet address
    Connected(String),
    /// Connecting in progress
    Connecting,
    /// Error state with message
    Error(String),
}

impl WalletStatus {
    pub fn is_connected(&self) -> bool {
        matches!(self, WalletStatus::Connected(_))
    }

    pub fn address(&self) -> Option<&str> {
        match self {
            WalletStatus::Connected(addr) => Some(addr),
            _ => None,
        }
    }
}

/// Wallet service for managing Solana wallet operations
pub struct WalletService {
    /// Optional keypair (None if not loaded)
    keypair: Option<Keypair>,
    /// RPC client for blockchain operations
    rpc_client: RpcClient,
    /// Current connection status
    status: WalletStatus,
}

impl WalletService {
    /// Create a new wallet service with RPC endpoint
    ///
    /// # Arguments
    /// * `rpc_url` - Solana RPC endpoint URL (e.g., "https://api.devnet.solana.com")
    ///
    /// # Example
    /// ```
    /// let wallet = WalletService::new("https://api.devnet.solana.com");
    /// ```
    pub fn new(rpc_url: &str) -> Self {
        // In Solana SDK 3.0, just use the simple constructor
        let rpc_client = RpcClient::new(rpc_url.to_string());

        Self {
            keypair: None,
            rpc_client,
            status: WalletStatus::Disconnected,
        }
    }

    /// Create a new wallet service from an existing keypair
    pub fn from_keypair(rpc_url: &str, keypair: Keypair) -> Self {
        let rpc_client = RpcClient::new(rpc_url.to_string());
        let pubkey = keypair.pubkey().to_string();
        
        Self {
            keypair: Some(keypair),
            rpc_client,
            status: WalletStatus::Connected(pubkey),
        }
    }

    /// Load keypair from file
    ///
    /// Supports multiple formats:
    /// - JSON array format: [1,2,3,...]
    /// - Solana CLI format: path to keypair file
    ///
    /// # Arguments
    /// * `path` - Path to keypair file
    ///
    /// # Example
    /// ```
    /// wallet.load_keypair_from_file("~/.config/solana/id.json")?;
    /// ```
    pub fn load_keypair_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), WalletError> {
        self.status = WalletStatus::Connecting;

        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .map_err(|e| WalletError::KeypairLoadError(format!("Failed to read file: {}", e)))?;

        // Try to parse as JSON array
        let keypair = if contents.trim().starts_with('[') {
            let bytes: Vec<u8> = serde_json::from_str(&contents)
                .map_err(|e| WalletError::InvalidKeypair(format!("Invalid JSON format: {}", e)))?;

            if bytes.len() != 32 {
                return Err(WalletError::InvalidKeypair(format!("Expected 32 bytes, got {}", bytes.len())));
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes[..32]);

            Keypair::new_from_array(arr)
        } else {
            // Try to parse as base58
            let bytes = bs58::decode(contents.trim())
                .into_vec()
                .map_err(|e| WalletError::InvalidKeypair(format!("Invalid base58: {}", e)))?;

            if bytes.len() != 32 {
                return Err(WalletError::InvalidKeypair(format!("Expected 32 bytes, got {}", bytes.len())));
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes[..32]);

            Keypair::new_from_array(arr)
        };

        let pubkey = keypair.pubkey().to_string();
        self.keypair = Some(keypair);
        self.status = WalletStatus::Connected(pubkey);

        Ok(())
    }

    /// Load keypair from base58 string
    ///
    /// # Arguments
    /// * `base58_key` - Base58 encoded private key
    pub fn load_keypair_from_base58(&mut self, base58_key: &str) -> Result<(), WalletError> {
        self.status = WalletStatus::Connecting;

        let bytes = bs58::decode(base58_key.trim())
            .into_vec()
            .map_err(|e| WalletError::InvalidKeypair(format!("Invalid base58: {}", e)))?;

        if bytes.len() != 32 {
            return Err(WalletError::InvalidKeypair(format!("Expected 32 bytes, got {}", bytes.len())));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes[..32]);

        let keypair = Keypair::new_from_array(arr);

        let pubkey = keypair.pubkey().to_string();
        self.keypair = Some(keypair);
        self.status = WalletStatus::Connected(pubkey);

        Ok(())
    }

    /// Generate a new random keypair
    ///
    /// # Returns
    /// The public key as a string
    pub fn generate_new_keypair(&mut self) -> String {
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey().to_string();

        self.keypair = Some(keypair);
        self.status = WalletStatus::Connected(pubkey.clone());

        pubkey
    }

    /// Get current wallet public key
    ///
    /// # Returns
    /// Public key as string if wallet is connected, None otherwise
    pub fn get_public_key(&self) -> Option<String> {
        self.keypair.as_ref().map(|kp| kp.pubkey().to_string())
    }

    /// Get current wallet status
    pub fn get_status(&self) -> &WalletStatus {
        &self.status
    }

    /// Check if wallet is connected
    pub fn is_connected(&self) -> bool {
        self.keypair.is_some()
    }

    /// Sign a transaction
    ///
    /// # Arguments
    /// * `transaction` - Transaction to sign (must be mutable)
    ///
    /// # Returns
    /// Signature of the transaction
    pub fn sign_transaction(&self, transaction: &mut Transaction) -> Result<Signature, WalletError> {
        let keypair = self.keypair.as_ref()
            .ok_or_else(|| WalletError::SigningError("No keypair loaded".to_string()))?;

        // Get recent blockhash
        let recent_blockhash = self.rpc_client
            .get_latest_blockhash()
            .map_err(|e| WalletError::RpcError(format!("Failed to get blockhash: {}", e)))?;

        transaction.sign(&[keypair], recent_blockhash);

        transaction.signatures.first()
            .copied()
            .ok_or_else(|| WalletError::SigningError("No signature generated".to_string()))
    }

    /// Get wallet balance in SOL
    ///
    /// # Returns
    /// Balance in SOL (converted from lamports)
    pub async fn get_balance(&self) -> Result<f64, WalletError> {
        let pubkey = self.keypair.as_ref()
            .ok_or_else(|| WalletError::BalanceError("No keypair loaded".to_string()))?
            .pubkey();

        let lamports = self.rpc_client
            .get_balance(&pubkey)
            .map_err(|e| WalletError::BalanceError(format!("Failed to get balance: {}", e)))?;

        Ok(lamports as f64 / 1_000_000_000.0)
    }

    /// Get SPL token balance
    ///
    /// # Arguments
    /// * `token_mint` - Token mint address as string
    ///
    /// # Returns
    /// Token balance as f64
    pub async fn get_token_balance(&self, token_mint: &str) -> Result<f64, WalletError> {
        let wallet_pubkey = self.keypair.as_ref()
            .ok_or_else(|| WalletError::BalanceError("No keypair loaded".to_string()))?
            .pubkey();

        let mint_pubkey = Pubkey::from_str(token_mint)
            .map_err(|e| WalletError::BalanceError(format!("Invalid mint address: {}", e)))?;

        // Use solana_sdk::Pubkey directly (spl_associated_token_account accepts it)
        let wallet_spl_pubkey = &wallet_pubkey;
        let mint_spl_pubkey = &mint_pubkey;

        // Get associated token account - returns Pubkey directly
        let token_account = spl_associated_token_account::get_associated_token_address(
            wallet_spl_pubkey,
            mint_spl_pubkey,
        );

        // Get token account balance
        let balance = self.rpc_client
            .get_token_account_balance(&token_account)
            .map_err(|e| WalletError::BalanceError(format!("Failed to get token balance: {}", e)))?;

        // Parse UI amount
        balance.ui_amount
            .ok_or_else(|| WalletError::BalanceError("No UI amount in response".to_string()))
    }

    /// Export keypair to base58 string (for backup)
    ///
    /// # Returns
    /// Base58 encoded private key
    ///
    /// # Security Warning
    /// This exposes the private key! Handle with extreme care.
    pub fn export_keypair_base58(&self) -> Result<String, WalletError> {
        let keypair = self.keypair.as_ref()
            .ok_or_else(|| WalletError::KeypairLoadError("No keypair loaded".to_string()))?;

        Ok(bs58::encode(keypair.to_bytes()).into_string())
    }

    /// Disconnect wallet
    pub fn disconnect(&mut self) {
        self.keypair = None;
        self.status = WalletStatus::Disconnected;
    }

    /// Get RPC client reference for advanced operations
    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    /// Take the keypair from the wallet service (consumes the service)
    pub fn take_keypair(&mut self) -> Option<Keypair> {
        self.keypair.take()
    }
}

/// Load default keypair from standard Solana CLI location
///
/// # Returns
/// WalletService with loaded keypair
pub fn load_default_keypair() -> Result<WalletService, WalletError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| WalletError::KeypairLoadError("Cannot determine home directory".to_string()))?;

    let default_path = Path::new(&home)
        .join(".config")
        .join("solana")
        .join("id.json");

    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let mut wallet = WalletService::new(&rpc_url);
    wallet.load_keypair_from_file(default_path)?;

    Ok(wallet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = WalletService::new("https://api.devnet.solana.com");
        assert!(!wallet.is_connected());
        assert_eq!(wallet.get_status(), &WalletStatus::Disconnected);
    }

    #[test]
    fn test_generate_new_keypair() {
        let mut wallet = WalletService::new("https://api.devnet.solana.com");
        let pubkey = wallet.generate_new_keypair();

        assert!(wallet.is_connected());
        assert_eq!(wallet.get_public_key(), Some(pubkey.clone()));
        assert_eq!(wallet.get_status(), &WalletStatus::Connected(pubkey));
    }

    #[test]
    fn test_disconnect() {
        let mut wallet = WalletService::new("https://api.devnet.solana.com");
        wallet.generate_new_keypair();
        assert!(wallet.is_connected());

        wallet.disconnect();
        assert!(!wallet.is_connected());
        assert_eq!(wallet.get_status(), &WalletStatus::Disconnected);
    }

    #[test]
    fn test_wallet_status_methods() {
        let status = WalletStatus::Connected("test_address".to_string());
        assert!(status.is_connected());
        assert_eq!(status.address(), Some("test_address"));

        let status = WalletStatus::Disconnected;
        assert!(!status.is_connected());
        assert_eq!(status.address(), None);
    }
}
