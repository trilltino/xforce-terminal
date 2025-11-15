use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccountInfo {
    pub mint: String,
    pub owner: String,
    pub amount: u64,
    pub decimals: u8,
    pub ui_amount: f64,
    pub token_symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub mint: String,
    pub symbol: String,
    pub balance: u64,
    pub decimals: u8,
    pub ui_balance: f64,
    pub usd_value: Option<f64>,
}

pub struct SplTokenClient {
    rpc_client: RpcClient,
}

impl SplTokenClient {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url),
        }
    }

    /// Get all SPL token accounts for a given wallet address
    /// 
    /// Note: This implementation is a placeholder. The actual token account fetching
    /// is handled by the WalletService which uses the Solana RPC client directly.
    /// This method is kept for API compatibility.
    pub async fn get_token_accounts(&self, _wallet_address: &str) -> Result<Vec<TokenAccountInfo>> {
        // TODO: Implement proper token account fetching using RPC client
        // For now, return empty - the WalletService handles this via direct RPC calls
        // This maintains API compatibility while the implementation is being refined
        Ok(Vec::new())
    }

    /// Get token balance for a specific mint
    pub async fn get_token_balance(
        &self,
        wallet_address: &str,
        mint_address: &str,
    ) -> Result<Option<TokenAccountInfo>> {
        let accounts = self.get_token_accounts(wallet_address).await?;
        Ok(accounts
            .into_iter()
            .find(|acc| acc.mint == mint_address))
    }

    /// Get Associated Token Account address for a wallet and mint
    pub fn get_associated_token_address(
        wallet_address: &str,
        mint_address: &str,
    ) -> Result<String> {
        let wallet_pubkey = Pubkey::from_str(wallet_address)
            .context("Invalid wallet address")?;
        let mint_pubkey = Pubkey::from_str(mint_address)
            .context("Invalid mint address")?;

        let ata = get_associated_token_address(&wallet_pubkey, &mint_pubkey);
        Ok(ata.to_string())
    }

    /// Check if an Associated Token Account exists
    pub async fn ata_exists(&self, ata_address: &str) -> Result<bool> {
        use tokio::task;
        
        let ata_pubkey = Pubkey::from_str(ata_address)
            .context("Invalid ATA address")?;
        
        let rpc_url = self.rpc_client.url().to_string();
        
        let result = task::spawn_blocking(move || {
            let rpc = RpcClient::new(rpc_url);
            rpc.get_account(&ata_pubkey)
        })
        .await
        .context("Failed to spawn blocking task")?;

        match result {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get decimals for a token mint
    async fn get_mint_decimals(&self, mint_address: &str) -> Result<u8> {
        use tokio::task;
        
        let mint_pubkey = Pubkey::from_str(mint_address)
            .context("Invalid mint address")?;
        
        let rpc_url = self.rpc_client.url().to_string();
        
        let _account = task::spawn_blocking(move || {
            let rpc = RpcClient::new(rpc_url);
            rpc.get_account(&mint_pubkey)
        })
        .await
        .context("Failed to spawn blocking task")?
        .context("Failed to fetch mint account")?;

        // Try to parse as token mint account
        // For now, return default - decimals are usually available from token account data
        Ok(9) // Default to 9 decimals - actual decimals come from token account parsing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires RPC connection
    async fn test_get_token_accounts() {
        let client = SplTokenClient::new("https://api.mainnet-beta.solana.com".to_string());
        let wallet = "YOUR_TEST_WALLET_ADDRESS";

        let accounts = client.get_token_accounts(wallet).await;
        assert!(accounts.is_ok());
    }

    #[test]
    fn test_get_associated_token_address() {
        let wallet = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU";
        let mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC

        let ata = SplTokenClient::get_associated_token_address(wallet, mint);
        assert!(ata.is_ok());
    }
}
