//! # Swap Tasks
//!
//! Async tasks for swap operations including quote fetching and swap execution.

use crate::app::state::{AppState, SwapQuote};
use crate::app::events::AppEvent;
use crate::core::service::ApiService;
use async_channel::Sender;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::spawn;

/// Trigger async swap quote fetch with debouncing
///
/// Internal task function - spawns async task to fetch swap quote and send results via event channel.
pub(crate) fn trigger_quote_fetch(
    state: Arc<RwLock<AppState>>,
    event_tx: Sender<AppEvent>,
) {
    let state_guard = state.read();

    // Only fetch if we have a valid amount
    let amount_str = &state_guard.terminal.swap.amount;
    if amount_str.is_empty() {
        return;
    }

    // Parse amount to lamports (assuming 9 decimals for SOL)
    let amount_f64: f64 = match amount_str.parse() {
        Ok(amt) => amt,
        Err(_) => return, // Invalid number, skip
    };

    // Convert to lamports (9 decimals)
    let amount_lamports = (amount_f64 * 1_000_000_000.0) as u64;

    // Debounce: only fetch if 500ms elapsed since last fetch
    if state_guard.terminal.swap.last_quote_fetch.elapsed().as_millis() < 500 {
        return;
    }

    let input_mint = state_guard.terminal.swap.input_mint.clone();
    let output_mint = state_guard.terminal.swap.output_mint.clone();
    let slippage_bps = state_guard.terminal.swap.slippage_bps;
    let api_client = match &state_guard.api_client {
        Some(client) => client.clone(),
        None => return,
    };

    drop(state_guard); // Release lock

    // Update last fetch time
    {
        let mut state = state.write();
        state.terminal.swap.quote_loading = true;
        state.terminal.swap.last_quote_fetch = std::time::Instant::now();
    }

    spawn(async move {
        match api_client.get_swap_quote(&input_mint, &output_mint, amount_lamports, slippage_bps).await {
            Ok(quote_response) => {
                // Convert API response to our SwapQuote
                let input_amount: f64 = quote_response.in_amount.parse().unwrap_or(0.0) / 1_000_000_000.0;
                let output_amount: f64 = quote_response.out_amount.parse().unwrap_or(0.0) / 1_000_000_000.0;

                let quote = SwapQuote {
                    input_amount,
                    output_amount,
                    price_impact: quote_response.price_impact_pct,
                    estimated_fee: 0.000005, // TODO: Calculate from routes
                };

                let _ = event_tx.send(AppEvent::SwapQuoteResult(Ok(quote))).await;
            }
            Err(e) => {
                let _ = event_tx.send(AppEvent::SwapQuoteResult(Err(e))).await;
            }
        }
    });
}

/// Execute swap transaction
///
/// Internal task function - spawns async task to execute swap and send results via event channel.
pub(crate) fn execute_swap(
    state: Arc<RwLock<AppState>>,
    event_tx: Sender<AppEvent>,
) {
    // Get necessary data for swap
    let (quote, input_mint, output_mint, amount_str, slippage_bps, wallet_pubkey, auth_token, api_client) = {
        let state_guard = state.read();
        
        // Check if wallet is connected
        let wallet_pubkey = match state_guard.wallet_service.as_ref().and_then(|ws| ws.get_public_key()) {
            Some(pk) => pk,
            None => {
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(AppEvent::Loading("ERROR: Cannot execute swap - wallet not connected!".to_string())).await;
                });
                return;
            }
        };

        // Check if we have a quote
        let quote = match &state_guard.terminal.swap.quote {
            Some(q) => q.clone(),
            None => {
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(AppEvent::Loading("ERROR: Cannot execute swap - no quote available!".to_string())).await;
                });
                return;
            }
        };

        // Get auth token
        let auth_token = match &state_guard.auth_token {
            Some(token) => token.clone(),
            None => {
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(AppEvent::Loading("ERROR: Not authenticated - please login first".to_string())).await;
                });
                return;
            }
        };

        // Get API client
        let api_client = match &state_guard.api_client {
            Some(client) => client.clone(),
            None => {
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(AppEvent::Loading("ERROR: API client not available".to_string())).await;
                });
                return;
            }
        };

        (
            quote,
            state_guard.terminal.swap.input_mint.clone(),
            state_guard.terminal.swap.output_mint.clone(),
            state_guard.terminal.swap.amount.clone(),
            state_guard.terminal.swap.slippage_bps,
            wallet_pubkey,
            auth_token,
            api_client,
        )
    };

    // Parse amount to lamports
    let amount_f64: f64 = match amount_str.parse() {
        Ok(amt) => amt,
        Err(_) => {
            let tx = event_tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(AppEvent::Loading("ERROR: Invalid amount format".to_string())).await;
            });
            return;
        }
    };
    let amount_lamports = (amount_f64 * 1_000_000_000.0) as u64;

    // Clone state reference for async task
    let state_clone = state.clone();

    // Spawn async task to execute swap
    spawn(async move {
        eprintln!("Starting swap execution...");
        eprintln!("  Input: {} {} ({})", amount_f64, input_mint, amount_lamports);
        eprintln!("  Output: {} (expected)", quote.output_amount);
        eprintln!("  Slippage: {} bps", slippage_bps);

        // Step 1: Get unsigned transaction from backend
        let swap_response = match api_client
            .execute_swap(
                &input_mint,
                &output_mint,
                amount_lamports,
                slippage_bps,
                &wallet_pubkey,
                &auth_token,
            )
            .await
        {
            Ok(resp) => {
                eprintln!("Received unsigned transaction from backend");
                resp
            }
            Err(e) => {
                eprintln!("Failed to get transaction: {}", e);
                let _ = event_tx.send(AppEvent::Loading(format!("Swap failed: {}", e))).await;
                return;
            }
        };

        // Step 2: Deserialize transaction from base64
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        use solana_sdk::transaction::Transaction;

        let tx_bytes = match BASE64.decode(&swap_response.transaction) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed to decode transaction: {}", e);
                let _ = event_tx.send(AppEvent::Loading(format!("Decode failed: {}", e))).await;
                return;
            }
        };

        let mut transaction: Transaction = match bincode::deserialize(&tx_bytes) {
            Ok(tx) => tx,
            Err(e) => {
                eprintln!("Failed to deserialize transaction: {}", e);
                let _ = event_tx.send(AppEvent::Loading(format!("Deserialize failed: {}", e))).await;
                return;
            }
        };

        eprintln!("Transaction deserialized successfully");
        eprintln!("  Instructions: {}", transaction.message.instructions.len());

        // Step 3: Sign transaction with wallet
        let sign_result = {
            let state_write = state_clone.write();
            match &state_write.wallet_service {
                Some(wallet_service) => {
                    wallet_service.sign_transaction(&mut transaction)
                }
                None => {
                    Err(crate::services::wallet::WalletError::SigningError("Wallet service not available".to_string()))
                }
            }
            // Lock released here before any .await
        };

        let _signature = match sign_result {
            Ok(sig) => {
                eprintln!("Transaction signed successfully");
                eprintln!("  Signature: {}", sig);
                sig
            }
            Err(e) => {
                eprintln!("Failed to sign transaction: {}", e);
                let _ = event_tx.send(AppEvent::Loading(format!("Signing failed: {}", e))).await;
                return;
            }
        };

        // Step 4: Serialize signed transaction back to base64
        let signed_bytes = match bincode::serialize(&transaction) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed to serialize signed transaction: {}", e);
                let _ = event_tx.send(AppEvent::Loading(format!("Serialize failed: {}", e))).await;
                return;
            }
        };
        let signed_b64 = BASE64.encode(&signed_bytes);

        eprintln!("Signed transaction serialized");

        // Step 5: Submit signed transaction to backend
        let submit_result = api_client
            .submit_transaction(
                signed_b64,
                input_mint.clone(),
                output_mint.clone(),
                amount_lamports as i64,
                (quote.output_amount * 1_000_000_000.0) as i64, // Convert to lamports
                Some(quote.price_impact),
                Some(slippage_bps as i32),
                &auth_token,
            )
            .await;

        match submit_result {
            Ok(response) => {
                eprintln!("Swap executed successfully!");
                eprintln!("  Transaction signature: {}", response.signature);
                eprintln!("  Explorer: https://explorer.solana.com/tx/{}?cluster=devnet", response.signature);

                // Send success notification with trade details
                let success_msg = format!("Swap successful! Signature: {}", response.signature);
                let _ = event_tx.send(AppEvent::Loading(success_msg)).await;
                
                // Also trigger a trade confirmation notification
                let trade_msg = format!("Trade Confirmed: {} → {} | Sig: {}", 
                    input_mint, output_mint, response.signature);
                let _ = event_tx.send(AppEvent::Loading(format!("NOTIFY_SUCCESS:{}", trade_msg))).await;
            }
            Err(e) => {
                eprintln!("Failed to submit transaction: {}", e);
                let error_msg = format!("Submit failed: {}", e);
                let _ = event_tx.send(AppEvent::Loading(format!("NOTIFY_ERROR:{}", error_msg))).await;
            }
        }
    });
}

