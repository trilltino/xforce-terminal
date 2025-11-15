//! # Batch Swap Transaction Builder
//!
//! Transaction building logic for batch swap operations.

use super::types::{BatchSwapRequest, ExecuteSwapRequest};
use crate::contracts::transaction_builder::BatchSwapTransactionBuilder;
use crate::contracts::SwapParams as ClientSwapParams;
use crate::mod_rs::SolanaState;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;

/// Build a batch swap transaction
pub async fn build_batch_swap_transaction(
    solana_state: &Arc<SolanaState>,
    plugin_program_id: Pubkey,
    request: &BatchSwapRequest,
) -> Result<(String, u64), String> {
    // Parse user public key
    let user_pubkey = Pubkey::from_str(&request.user_public_key)
        .map_err(|e| format!("Invalid user public key: {}", e))?;

    // Convert swap requests to client swap params
    let swaps: Vec<ClientSwapParams> = request.swaps
        .iter()
        .map(|swap| {
            ClientSwapParams {
                input_mint: Pubkey::from_str(&swap.input_mint)
                    .expect("Input mint validation should have caught this"),
                output_mint: Pubkey::from_str(&swap.output_mint)
                    .expect("Output mint validation should have caught this"),
                amount: swap.amount,
                min_output_amount: swap.min_output_amount,
            }
        })
        .collect();

    if swaps.is_empty() {
        return Err("No swaps provided".to_string());
    }

    // Get Jupiter quote and transaction for the first swap
    let first_swap = &swaps[0];
    let slippage_bps = 50; // Default slippage tolerance
    
    // Get Jupiter quote
    let quote = solana_state.jupiter.get_swap_quote(
        &first_swap.input_mint.to_string(),
        &first_swap.output_mint.to_string(),
        first_swap.amount,
        slippage_bps,
    ).await.map_err(|e| format!("Failed to get Jupiter quote: {}", e))?;

    // Get Jupiter swap transaction
    let jupiter_tx_response = solana_state.jupiter.get_swap_transaction(
        &quote,
        &request.user_public_key,
    ).await.map_err(|e| format!("Failed to get Jupiter transaction: {}", e))?;

    // Get recent blockhash from RPC
    let recent_blockhash = solana_state.rpc.get_latest_blockhash().await
        .map_err(|e| format!("Failed to get recent blockhash: {}", e))?;

    // Build batch swap transaction using transaction builder
    let tx_builder = BatchSwapTransactionBuilder::new(plugin_program_id);
    
    // Convert ClientSwapParams to SwapParams for transaction builder
    let swap_params: Vec<ClientSwapParams> = swaps
        .into_iter()
        .map(|s| ClientSwapParams {
            input_mint: s.input_mint,
            output_mint: s.output_mint,
            amount: s.amount,
            min_output_amount: s.min_output_amount,
        })
        .collect();

    let combined_tx_base64 = tx_builder.build_batch_swap_transaction(
        &jupiter_tx_response.swap_transaction,
        &user_pubkey,
        swap_params,
        None, // fee_recipient - can be added later
        recent_blockhash,
        jupiter_tx_response.last_valid_block_height,
    ).map_err(|e| format!("Failed to build batch swap transaction: {}", e))?;

    Ok((combined_tx_base64, jupiter_tx_response.last_valid_block_height))
}

/// Build an execute swap transaction
pub async fn build_execute_swap_transaction(
    solana_state: &Arc<SolanaState>,
    plugin_program_id: Pubkey,
    request: &ExecuteSwapRequest,
) -> Result<(String, u64), String> {
    // Parse user public key
    let user_pubkey = Pubkey::from_str(&request.user_public_key)
        .map_err(|e| format!("Invalid user public key: {}", e))?;

    // Parse mints (from and to are mint addresses)
    let input_mint = Pubkey::from_str(&request.from)
        .map_err(|e| format!("Invalid input mint: {}", e))?;
    let output_mint = Pubkey::from_str(&request.to)
        .map_err(|e| format!("Invalid output mint: {}", e))?;

    // Get Jupiter quote for the swap
    let slippage_bps = 50; // Default slippage tolerance
    let quote = solana_state.jupiter.get_swap_quote(
        &input_mint.to_string(),
        &output_mint.to_string(),
        request.amount,
        slippage_bps,
    ).await.map_err(|e| format!("Failed to get Jupiter quote: {}", e))?;

    // Get Jupiter swap transaction
    let jupiter_tx_response = solana_state.jupiter.get_swap_transaction(
        &quote,
        &request.user_public_key,
    ).await.map_err(|e| format!("Failed to get Jupiter transaction: {}", e))?;

    // Get recent blockhash from RPC
    let recent_blockhash = solana_state.rpc.get_latest_blockhash().await
        .map_err(|e| format!("Failed to get recent blockhash: {}", e))?;

    // Derive token accounts from mints and user public key
    let input_token_account = Pubkey::try_from(input_mint.as_ref())
        .map_err(|_| "Invalid mint format".to_string())?;
    
    let output_token_account = Pubkey::try_from(output_mint.as_ref())
        .map_err(|_| "Invalid mint format".to_string())?;

    // Build execute swap transaction using transaction builder
    let tx_builder = BatchSwapTransactionBuilder::new(plugin_program_id);
    
    let combined_tx_base64 = tx_builder.build_execute_swap_transaction(
        &jupiter_tx_response.swap_transaction,
        &user_pubkey,
        &input_token_account,
        &output_token_account,
        &input_mint,
        &output_mint,
        request.amount,
        request.min_output_amount,
        request.expected_output,
        None, // fee_recipient - can be added later
        recent_blockhash,
        jupiter_tx_response.last_valid_block_height,
    ).map_err(|e| format!("Failed to build execute swap transaction: {}", e))?;

    Ok((combined_tx_base64, jupiter_tx_response.last_valid_block_height))
}

