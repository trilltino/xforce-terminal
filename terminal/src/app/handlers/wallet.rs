//! # Wallet Handlers
//!
//! Handlers for wallet connection, generation, and disconnection.

use crate::app::state::{AppState, WalletState};
use crate::app::events::AppEvent;
use async_channel::Sender;
use parking_lot::RwLock;
use std::sync::Arc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use std::str::FromStr;

/// Handle wallet connect button click
///
/// Internal handler function - use [`crate::app::App::handle_wallet_connect_click`] instead.
pub(crate) fn handle_wallet_connect_click(
    state: Arc<RwLock<AppState>>,
    event_tx: Sender<AppEvent>,
) {
    // Use default path on Unix, or Windows path
    #[cfg(unix)]
    let default_path = format!("{}/.config/solana/id.json", std::env::var("HOME").unwrap_or_else(|_| "~".to_string()));
    #[cfg(windows)]
    let default_path = format!("{}\\solana\\id.json", std::env::var("APPDATA").unwrap_or_else(|_| "~".to_string()));
    
    let path = default_path;
    let state_clone = state.clone();
    let tx = event_tx.clone();
    
    // Load keypair synchronously before spawning async task
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let mut wallet_service = crate::services::wallet::WalletService::new(&rpc_url);
    
    let keypair_result = wallet_service.load_keypair_from_file(&path);
    let keypair = match keypair_result {
        Ok(_) => wallet_service.take_keypair(),
        Err(e) => {
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                let _ = tx_clone.send(AppEvent::Loading(format!("Failed to load wallet: {}", e))).await;
            });
            return;
        }
    };
    
    let pubkey = keypair.as_ref().map(|kp| kp.pubkey().to_string());
    
    if let (Some(keypair), Some(pubkey)) = (keypair, pubkey) {
        let pubkey_clone = pubkey.clone();
        let rpc_url_clone = rpc_url.clone();
        
        tokio::spawn(async move {
            let _ = tx.send(AppEvent::Loading("Connecting wallet...".to_string())).await;
            
            // Get balance using spawn_blocking to avoid holding wallet_service across await
            let pubkey_for_balance = pubkey_clone.clone();
            let rpc_url_for_balance = rpc_url_clone.clone();
            let balance_result = match tokio::task::spawn_blocking(move || {
                let rpc_client = RpcClient::new(rpc_url_for_balance);
                let pubkey_pk = Pubkey::from_str(&pubkey_for_balance)
                    .map_err(|e| crate::services::wallet::WalletError::BalanceError(format!("Invalid pubkey: {}", e)))?;
                rpc_client.get_balance(&pubkey_pk)
                    .map(|lamports| lamports as f64 / 1_000_000_000.0)
                    .map_err(|e| crate::services::wallet::WalletError::BalanceError(format!("Failed to get balance: {}", e)))
            }).await {
                Ok(result) => result,
                Err(e) => Err(crate::services::wallet::WalletError::BalanceError(format!("Task join error: {}", e))),
            };

            match balance_result {
                Ok(balance) => {
                    // Reconstruct wallet_service from keypair
                    let wallet_service = crate::services::wallet::WalletService::from_keypair(&rpc_url_clone, keypair);
                    {
                        let mut state = state_clone.write();
                        state.wallet_service = Some(wallet_service);
                        state.wallet = Some(WalletState {
                            address: pubkey_clone.clone(),
                            sol_balance: balance,
                            token_balances: Vec::new(),
                        });
                    } // Drop the lock guard before await
                    let _ = tx.send(AppEvent::Loading(format!("Wallet connected: {}", pubkey_clone))).await;
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Loading(format!("Failed to get balance: {}", e))).await;
                }
            }
        });
    } else {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let _ = tx_clone.send(AppEvent::Loading("Failed to get public key".to_string())).await;
        });
    }
}

/// Handle wallet generate button click
///
/// Internal handler function - use [`crate::app::App::handle_wallet_generate_click`] instead.
pub(crate) fn handle_wallet_generate_click(
    state: Arc<RwLock<AppState>>,
    event_tx: Sender<AppEvent>,
) {
    let state_clone = state.clone();
    let tx = event_tx.clone();
    
    // Generate keypair synchronously before spawning async task
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let mut wallet_service = crate::services::wallet::WalletService::new(&rpc_url);
    let pubkey = wallet_service.generate_new_keypair();
    let keypair = wallet_service.take_keypair();
    
    if let Some(keypair) = keypair {
        let pubkey_clone = pubkey.clone();
        let rpc_url_clone = rpc_url.clone();
        
        tokio::spawn(async move {
            let _ = tx.send(AppEvent::Loading("Generating wallet...".to_string())).await;
            
            // Get balance using spawn_blocking to avoid holding wallet_service across await
            let pubkey_for_balance = pubkey_clone.clone();
            let rpc_url_for_balance = rpc_url_clone.clone();
            let balance_result = match tokio::task::spawn_blocking(move || {
                let rpc_client = RpcClient::new(rpc_url_for_balance);
                let pubkey_pk = Pubkey::from_str(&pubkey_for_balance)
                    .map_err(|e| crate::services::wallet::WalletError::BalanceError(format!("Invalid pubkey: {}", e)))?;
                rpc_client.get_balance(&pubkey_pk)
                    .map(|lamports| lamports as f64 / 1_000_000_000.0)
                    .map_err(|e| crate::services::wallet::WalletError::BalanceError(format!("Failed to get balance: {}", e)))
            }).await {
                Ok(result) => result,
                Err(e) => Err(crate::services::wallet::WalletError::BalanceError(format!("Task join error: {}", e))),
            };

            match balance_result {
                Ok(balance) => {
                    // Reconstruct wallet_service from keypair
                    let wallet_service = crate::services::wallet::WalletService::from_keypair(&rpc_url_clone, keypair);
                    {
                        let mut state = state_clone.write();
                        state.wallet_service = Some(wallet_service);
                        state.wallet = Some(WalletState {
                            address: pubkey_clone.clone(),
                            sol_balance: balance,
                            token_balances: Vec::new(),
                        });
                    } // Drop the lock guard before await
                    let _ = tx.send(AppEvent::Loading(format!("Wallet generated: {}", pubkey_clone))).await;
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Loading(format!("Failed to get balance: {}", e))).await;
                }
            }
        });
    }
}

/// Handle wallet disconnect button click
///
/// Internal handler function - use [`crate::app::App::handle_wallet_disconnect_click`] instead.
pub(crate) fn handle_wallet_disconnect_click(state: Arc<RwLock<AppState>>) {
    let mut state = state.write();
    if let Some(ref mut wallet_service) = state.wallet_service {
        wallet_service.disconnect();
    }
    state.wallet_service = None;
    state.wallet = None;
}

