//! # Batch Swap Validation
//!
//! Input validation and error handling for batch swap operations.

use super::types::{BatchSwapRequest, ExecuteSwapRequest};
use crate::contracts::plugin::PluginError;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Validate batch swap request
pub fn validate_batch_swap_request(request: &BatchSwapRequest) -> Result<(), PluginError> {
    // Validate batch is not empty
    if request.swaps.is_empty() {
        return Err(PluginError::ContractError(
            "Batch swap must contain at least one swap".to_string()
        ));
    }

    // Validate batch size (max 10 swaps)
    const MAX_BATCH_SIZE: usize = 10;
    if request.swaps.len() > MAX_BATCH_SIZE {
        return Err(PluginError::ContractError(
            format!("Batch swap exceeds maximum size of {} swaps", MAX_BATCH_SIZE)
        ));
    }

    // Validate user public key
    Pubkey::from_str(&request.user_public_key)
        .map_err(|e| PluginError::ContractError(
            format!("Invalid user public key: {}", e)
        ))?;

    // Validate each swap
    for (index, swap) in request.swaps.iter().enumerate() {
        // Validate input mint
        Pubkey::from_str(&swap.input_mint)
            .map_err(|e| PluginError::ContractError(
                format!("Invalid input_mint at index {}: {}", index, e)
            ))?;

        // Validate output mint
        Pubkey::from_str(&swap.output_mint)
            .map_err(|e| PluginError::ContractError(
                format!("Invalid output_mint at index {}: {}", index, e)
            ))?;

        // Validate mints are different
        if swap.input_mint == swap.output_mint {
            return Err(PluginError::ContractError(
                format!("Input and output mints must differ at index {}", index)
            ));
        }

        // Validate amount
        const MIN_SWAP_AMOUNT: u64 = 1;
        if swap.amount < MIN_SWAP_AMOUNT {
            return Err(PluginError::ContractError(
                format!("Invalid amount at index {}: must be >= {}", index, MIN_SWAP_AMOUNT)
            ));
        }

        // Validate min_output_amount
        if swap.min_output_amount == 0 {
            return Err(PluginError::ContractError(
                format!("Invalid min_output_amount at index {}: must be > 0", index)
            ));
        }
    }

    Ok(())
}

/// Validate execute swap request
pub fn validate_execute_swap_request(request: &ExecuteSwapRequest) -> Result<(), PluginError> {
    // Validate amount
    const MIN_SWAP_AMOUNT: u64 = 1;
    if request.amount < MIN_SWAP_AMOUNT {
        return Err(PluginError::ContractError(
            format!("Invalid amount: must be >= {}", MIN_SWAP_AMOUNT)
        ));
    }

    // Validate min_output_amount
    if request.min_output_amount == 0 {
        return Err(PluginError::ContractError(
            "Invalid min_output_amount: must be > 0".to_string()
        ));
    }

    // Validate expected_output
    if request.expected_output == 0 {
        return Err(PluginError::ContractError(
            "Invalid expected_output: must be > 0".to_string()
        ));
    }

    // Validate token accounts
    Pubkey::from_str(&request.from)
        .map_err(|e| PluginError::ContractError(
            format!("Invalid from token account: {}", e)
        ))?;

    Pubkey::from_str(&request.to)
        .map_err(|e| PluginError::ContractError(
            format!("Invalid to token account: {}", e)
        ))?;

    // Validate user public key
    Pubkey::from_str(&request.user_public_key)
        .map_err(|e| PluginError::ContractError(
            format!("Invalid user public key: {}", e)
        ))?;

    Ok(())
}

