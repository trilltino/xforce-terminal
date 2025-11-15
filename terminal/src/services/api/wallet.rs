//! # Wallet Query Endpoints
//!
//! Handles wallet-related queries (balance, token balances, transaction history).

use serde::{Deserialize, Serialize};
use super::client::ApiClient;

/// Get wallet SOL balance.
pub async fn get_wallet_balance(
    client: &ApiClient,
    address: &str,
) -> Result<WalletBalance, String> {
    let url = format!("{}/api/wallet/balance?address={}", ApiClient::base_url(), address);

    let response = client
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json::<WalletBalance>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        Err(format!("Failed to fetch wallet balance: {}", response.status()))
    }
}

/// Get transaction history for an address.
pub async fn get_transaction_history(
    client: &ApiClient,
    address: &str,
    limit: usize,
) -> Result<TransactionHistory, String> {
    let url = format!("{}/api/transactions?address={}&limit={}", ApiClient::base_url(), address, limit);

    let response = client
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json::<TransactionHistory>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        Err(format!("Failed to fetch transactions: {}", response.status()))
    }
}

/// Get SPL token balances for an address.
pub async fn get_token_balances(
    client: &ApiClient,
    address: &str,
) -> Result<Vec<TokenBalance>, String> {
    let url = format!("{}/api/wallet/tokens?address={}", ApiClient::base_url(), address);

    let response = client
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json::<Vec<TokenBalance>>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        Err(format!("Failed to fetch token balances: {}", response.status()))
    }
}

// ==================== WALLET TYPES ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub address: String,
    pub balance_sol: f64,
    pub balance_lamports: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistory {
    pub address: String,
    pub transactions: Vec<TransactionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub mint: String,
    pub symbol: Option<String>,
    pub balance: f64,
    pub ui_amount: String,
}

