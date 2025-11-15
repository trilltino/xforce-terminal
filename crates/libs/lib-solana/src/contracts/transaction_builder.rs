//! # Transaction Builder
//!
//! Builds Solana transactions that combine Jupiter swap instructions with
//! batch swap router contract instructions.

use base64::{Engine as _, engine::general_purpose};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    transaction::Transaction,
};
use thiserror::Error;
use tracing::{info, warn};

use crate::contracts::idl_handler::{IdlHandler, load_batch_swap_idl, get_default_idl_path};
use crate::contracts::SwapParams;
use std::str::FromStr;

/// Transaction building errors
#[derive(Debug, Error)]
pub enum TransactionBuilderError {
    #[error("Failed to decode Jupiter transaction: {0}")]
    DecodeError(String),
    
    #[error("Failed to build instruction: {0}")]
    InstructionError(String),
    
    #[error("Failed to serialize transaction: {0}")]
    SerializeError(String),
    
    #[error("IDL not available: {0}")]
    IdlError(String),
    
    #[error("Invalid account: {0}")]
    InvalidAccount(String),
}

/// Transaction builder for batch swap router
pub struct BatchSwapTransactionBuilder {
    /// Program ID
    program_id: Pubkey,
    /// IDL handler
    #[allow(dead_code)] // Stored for future use in transaction building
    idl_handler: IdlHandler,
    /// Token program ID
    token_program_id: Pubkey,
}

impl BatchSwapTransactionBuilder {
    /// Create a new transaction builder
    ///
    /// # Arguments
    ///
    /// * `program_id` - The batch swap router program ID
    ///
    /// # Returns
    ///
    /// A new `BatchSwapTransactionBuilder` instance
    pub fn new(program_id: Pubkey) -> Self {
        let idl_handler = load_batch_swap_idl(program_id)
            .unwrap_or_else(|_| {
                warn!("Failed to load IDL, will use placeholder");
                IdlHandler::new(program_id, get_default_idl_path())
            });

        Self {
            program_id,
            idl_handler,
            token_program_id: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
                .expect("Invalid token program ID"), // SPL Token program ID
        }
    }

    /// Build a batch swap transaction that combines Jupiter swap instructions
    /// with the batch swap router instruction
    ///
    /// This function takes a Jupiter transaction (which contains swap instructions)
    /// and adds the batch swap router instruction to validate and track the swaps.
    ///
    /// # Arguments
    ///
    /// * `jupiter_tx_base64` - Base64-encoded Jupiter swap transaction
    /// * `user_pubkey` - User's public key (authority)
    /// * `swaps` - Vector of swap parameters
    /// * `fee_recipient` - Optional fee recipient
    /// * `recent_blockhash` - Recent blockhash for the transaction
    /// * `last_valid_block_height` - Last valid block height from Jupiter
    ///
    /// # Returns
    ///
    /// Base64-encoded transaction ready for signing and last valid block height
    pub fn build_batch_swap_transaction(
        &self,
        jupiter_tx_base64: &str,
        user_pubkey: &Pubkey,
        swaps: Vec<SwapParams>,
        fee_recipient: Option<Pubkey>,
        recent_blockhash: solana_sdk::hash::Hash,
        _last_valid_block_height: u64,
    ) -> Result<String, TransactionBuilderError> {
        info!("Building batch swap transaction with {} swaps", swaps.len());

        // Step 1: Decode Jupiter transaction
        let jupiter_tx_bytes = general_purpose::STANDARD
            .decode(jupiter_tx_base64)
            .map_err(|e| TransactionBuilderError::DecodeError(format!("Failed to decode Jupiter transaction: {}", e)))?;

        let jupiter_tx: Transaction = bincode::deserialize(&jupiter_tx_bytes)
            .map_err(|e| TransactionBuilderError::DecodeError(format!("Failed to deserialize Jupiter transaction: {}", e)))?;

        // Step 2: Extract instructions from Jupiter transaction
        // Convert CompiledInstructions to Instructions so we can add our instruction
        // Message::new will recompile everything properly
        let mut instructions: Vec<Instruction> = jupiter_tx.message.instructions.iter()
            .map(|compiled_ix| {
                Instruction {
                    program_id: jupiter_tx.message.account_keys[compiled_ix.program_id_index as usize],
                    accounts: compiled_ix.accounts.iter()
                        .map(|&idx| {
                            let pubkey = jupiter_tx.message.account_keys[idx as usize];
                            let is_signer = jupiter_tx.message.is_signer(idx as usize);
                            // Check if account is writable by checking if it's in the writable accounts list
                            let is_writable = jupiter_tx.message.is_maybe_writable(idx as usize, None);
                            if is_writable {
                                AccountMeta::new(pubkey, is_signer)
                            } else {
                                AccountMeta::new_readonly(pubkey, is_signer)
                            }
                        })
                        .collect(),
                    data: compiled_ix.data.clone(),
                }
            })
            .collect();

        // Step 3: Build batch swap router instruction
        // Create a temporary account_keys vec for the instruction builder
        // Message::new will handle the final account ordering
        let mut temp_account_keys = jupiter_tx.message.account_keys.clone();
        let batch_swap_instruction = self.build_batch_swap_instruction(
            user_pubkey,
            &swaps,
            fee_recipient,
            &mut temp_account_keys,
        )?;

        // Step 4: Add batch swap router instruction at the beginning
        // This allows the contract to validate parameters before swaps execute
        instructions.insert(0, batch_swap_instruction);

        // Step 5: Create new transaction with combined instructions
        // Use Message::new to compile instructions properly - this handles account key ordering
        let message = solana_sdk::message::Message::new(&instructions, Some(user_pubkey));
        let combined_tx = Transaction {
            signatures: vec![solana_sdk::signature::Signature::default(); message.header.num_required_signatures as usize],
            message,
        };
        // Override the blockhash with the one we want to use
        let mut combined_tx = combined_tx;
        combined_tx.message.recent_blockhash = recent_blockhash;

        // Step 6: Serialize transaction
        let tx_bytes = bincode::serialize(&combined_tx)
            .map_err(|e| TransactionBuilderError::SerializeError(format!("Failed to serialize transaction: {}", e)))?;

        let tx_base64 = general_purpose::STANDARD.encode(&tx_bytes);

        info!("Batch swap transaction built successfully");

        Ok(tx_base64)
    }

    /// Build batch swap instruction manually
    ///
    /// This builds the instruction using manual account and data construction.
    /// The instruction uses account indices that reference accounts in the transaction.
    ///
    /// # Arguments
    ///
    /// * `authority` - Authority public key
    /// * `swaps` - Vector of swap parameters
    /// * `fee_recipient` - Optional fee recipient
    /// * `account_keys` - Mutable reference to account keys list (will add missing accounts)
    ///
    /// # Returns
    ///
    /// The batch swap instruction with account indices
    fn build_batch_swap_instruction(
        &self,
        authority: &Pubkey,
        swaps: &[SwapParams],
        fee_recipient: Option<Pubkey>,
        account_keys: &mut Vec<Pubkey>,
    ) -> Result<Instruction, TransactionBuilderError> {
        // Build instruction data using Anchor serialization
        // Format: [discriminator (8 bytes)] + [swaps vector]
        // Discriminator: first 8 bytes of sha256("global:batch_swap")
        // For now, use placeholder - will be correct once IDL is generated
        let mut instruction_data = Vec::with_capacity(8 + 4 + (swaps.len() * (32 + 32 + 8 + 8)));
        
        // Add discriminator (placeholder)
        // TODO: Calculate actual discriminator from IDL once available
        // Discriminator = first 8 bytes of sha256("global:batch_swap")
        instruction_data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);

        // Serialize swaps vector using Anchor's vector serialization
        // Format: length (u32, 4 bytes) + swap data
        let swaps_len = swaps.len() as u32;
        instruction_data.extend_from_slice(&swaps_len.to_le_bytes());

        for swap in swaps {
            // Serialize SwapParams struct
            // Format: input_mint (32 bytes) + output_mint (32 bytes) + amount (u64, 8 bytes) + min_output_amount (u64, 8 bytes)
            instruction_data.extend_from_slice(&swap.input_mint.to_bytes());
            instruction_data.extend_from_slice(&swap.output_mint.to_bytes());
            instruction_data.extend_from_slice(&swap.amount.to_le_bytes());
            instruction_data.extend_from_slice(&swap.min_output_amount.to_le_bytes());
        }

        // Build account metas
        // Note: We need to ensure accounts are in the account_keys list first
        let authority_key = *authority;
        let token_program_key = self.token_program_id;
        // System program ID: 11111111111111111111111111111111
        let system_program_key = Pubkey::from_str("11111111111111111111111111111111")
            .expect("Invalid system program ID");
        let program_id_key = self.program_id;

        // Get or add accounts to the list
        let _authority_index = self.get_or_add_account(authority_key, account_keys);
        let _token_program_index = self.get_or_add_account(token_program_key, account_keys);
        let _system_program_index = self.get_or_add_account(system_program_key, account_keys);
        let _program_id_index = self.get_or_add_account(program_id_key, account_keys);

        // Build account metas in the correct order:
        // 1. authority (signer, writable)
        // 2. fee_recipient (optional, writable)
        // 3. token_program (readonly)
        // 4. system_program (readonly)
        let mut accounts = vec![
            AccountMeta::new(authority_key, true), // authority (signer, writable)
        ];

        // Add fee recipient if provided
        if let Some(fee_recipient_key) = fee_recipient {
            let _fee_recipient_index = self.get_or_add_account(fee_recipient_key, account_keys);
            accounts.push(AccountMeta::new(fee_recipient_key, false)); // fee_recipient (writable)
        }

        // Add token program and system program
        accounts.push(AccountMeta::new_readonly(token_program_key, false)); // token_program
        accounts.push(AccountMeta::new_readonly(system_program_key, false)); // system_program

        // Create instruction
        // Note: The Instruction struct uses AccountMeta, and Solana will handle
        // the mapping to account indices when the transaction is built
        Ok(Instruction {
            program_id: program_id_key,
            accounts,
            data: instruction_data,
        })
    }

    /// Get account index or add account to list
    ///
    /// # Arguments
    ///
    /// * `pubkey` - Account public key
    /// * `account_keys` - Mutable reference to account keys list
    ///
    /// # Returns
    ///
    /// Index of the account in the list
    fn get_or_add_account(&self, pubkey: Pubkey, account_keys: &mut Vec<Pubkey>) -> usize {
        account_keys.iter()
            .position(|&key| key == pubkey)
            .unwrap_or_else(|| {
                let index = account_keys.len();
                account_keys.push(pubkey);
                index
            })
    }

    /// Build execute swap instruction manually
    ///
    /// # Arguments
    ///
    /// * `authority` - Authority public key
    /// * `input_token_account` - Input token account
    /// * `output_token_account` - Output token account
    /// * `input_mint` - Input mint
    /// * `output_mint` - Output mint
    /// * `amount` - Swap amount
    /// * `min_output_amount` - Minimum output amount
    /// * `expected_output` - Expected output amount
    /// * `fee_recipient` - Optional fee recipient
    ///
    /// # Returns
    ///
    /// The execute swap instruction
    pub fn build_execute_swap_instruction(
        &self,
        authority: &Pubkey,
        input_token_account: &Pubkey,
        output_token_account: &Pubkey,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        min_output_amount: u64,
        expected_output: u64,
        fee_recipient: Option<Pubkey>,
    ) -> Result<Instruction, TransactionBuilderError> {
        // Build instruction data
        // Format: [discriminator (8 bytes)] + amount (8 bytes) + min_output_amount (8 bytes) + expected_output (8 bytes)
        let mut instruction_data = vec![];
        
        // Add discriminator (placeholder)
        instruction_data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]); // Placeholder

        // Add arguments
        instruction_data.extend_from_slice(&amount.to_le_bytes());
        instruction_data.extend_from_slice(&min_output_amount.to_le_bytes());
        instruction_data.extend_from_slice(&expected_output.to_le_bytes());

        // Build accounts (using AccountMeta for simplicity)
        // In a real transaction, these would be converted to indices
        let mut accounts = vec![
            AccountMeta::new(*authority, true), // authority (signer, writable)
            AccountMeta::new(*input_token_account, false), // input_token_account (writable)
            AccountMeta::new(*output_token_account, false), // output_token_account (writable)
            AccountMeta::new_readonly(*input_mint, false), // input_mint
            AccountMeta::new_readonly(*output_mint, false), // output_mint
        ];

        // Add fee recipient if provided
        if let Some(fee_recipient) = fee_recipient {
            accounts.push(AccountMeta::new(fee_recipient, false)); // fee_recipient (writable)
        }

        // Add token program and system program
        accounts.push(AccountMeta::new_readonly(self.token_program_id, false)); // token_program
        // System program ID: 11111111111111111111111111111111
        let system_program_id = Pubkey::from_str("11111111111111111111111111111111")
            .expect("Invalid system program ID");
        accounts.push(AccountMeta::new_readonly(system_program_id, false)); // system_program

        // Create instruction
        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: instruction_data,
        })
    }

    /// Build execute swap transaction that combines Jupiter swap instruction
    /// with the execute swap instruction
    ///
    /// # Arguments
    ///
    /// * `jupiter_tx_base64` - Base64-encoded Jupiter swap transaction
    /// * `user_pubkey` - User's public key (authority)
    /// * `input_token_account` - Input token account
    /// * `output_token_account` - Output token account
    /// * `input_mint` - Input mint
    /// * `output_mint` - Output mint
    /// * `amount` - Swap amount
    /// * `min_output_amount` - Minimum output amount
    /// * `expected_output` - Expected output amount
    /// * `fee_recipient` - Optional fee recipient
    /// * `recent_blockhash` - Recent blockhash
    /// * `last_valid_block_height` - Last valid block height from Jupiter
    ///
    /// # Returns
    ///
    /// Base64-encoded transaction ready for signing
    pub fn build_execute_swap_transaction(
        &self,
        jupiter_tx_base64: &str,
        user_pubkey: &Pubkey,
        input_token_account: &Pubkey,
        output_token_account: &Pubkey,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        min_output_amount: u64,
        expected_output: u64,
        fee_recipient: Option<Pubkey>,
        recent_blockhash: solana_sdk::hash::Hash,
        _last_valid_block_height: u64,
    ) -> Result<String, TransactionBuilderError> {
        info!("Building execute swap transaction");

        // Step 1: Decode Jupiter transaction
        let jupiter_tx_bytes = general_purpose::STANDARD
            .decode(jupiter_tx_base64)
            .map_err(|e| TransactionBuilderError::DecodeError(format!("Failed to decode Jupiter transaction: {}", e)))?;

        let jupiter_tx: Transaction = bincode::deserialize(&jupiter_tx_bytes)
            .map_err(|e| TransactionBuilderError::DecodeError(format!("Failed to deserialize Jupiter transaction: {}", e)))?;

        // Step 2: Extract instructions from Jupiter transaction
        // Convert CompiledInstructions to Instructions so we can add our instruction
        // Message::new will recompile everything properly
        let mut instructions: Vec<Instruction> = jupiter_tx.message.instructions.iter()
            .map(|compiled_ix| {
                Instruction {
                    program_id: jupiter_tx.message.account_keys[compiled_ix.program_id_index as usize],
                    accounts: compiled_ix.accounts.iter()
                        .map(|&idx| {
                            let pubkey = jupiter_tx.message.account_keys[idx as usize];
                            let is_signer = jupiter_tx.message.is_signer(idx as usize);
                            // Check if account is writable by checking if it's in the writable accounts list
                            let is_writable = jupiter_tx.message.is_maybe_writable(idx as usize, None);
                            if is_writable {
                                AccountMeta::new(pubkey, is_signer)
                            } else {
                                AccountMeta::new_readonly(pubkey, is_signer)
                            }
                        })
                        .collect(),
                    data: compiled_ix.data.clone(),
                }
            })
            .collect();

        // Step 3: Build execute swap instruction (use the private method with account_keys)
        // Create a temporary account_keys vec for the instruction builder
        let mut temp_account_keys = jupiter_tx.message.account_keys.clone();
        let execute_swap_instruction = self.build_execute_swap_instruction_with_accounts(
            user_pubkey,
            input_token_account,
            output_token_account,
            input_mint,
            output_mint,
            amount,
            min_output_amount,
            expected_output,
            fee_recipient,
            &mut temp_account_keys,
        )?;

        // Step 4: Add execute swap instruction at the beginning
        instructions.insert(0, execute_swap_instruction);

        // Step 5: Create new transaction
        // Use Message::new to compile instructions properly - this handles account key ordering
        let message = solana_sdk::message::Message::new(&instructions, Some(user_pubkey));
        let mut combined_tx = Transaction {
            signatures: vec![solana_sdk::signature::Signature::default(); message.header.num_required_signatures as usize],
            message,
        };
        // Override the blockhash with the one we want to use
        combined_tx.message.recent_blockhash = recent_blockhash;

        // Step 6: Serialize transaction
        let tx_bytes = bincode::serialize(&combined_tx)
            .map_err(|e| TransactionBuilderError::SerializeError(format!("Failed to serialize transaction: {}", e)))?;

        let tx_base64 = general_purpose::STANDARD.encode(&tx_bytes);

        info!("Execute swap transaction built successfully");

        Ok(tx_base64)
    }

    /// Build execute swap instruction with account indices
    ///
    /// # Arguments
    ///
    /// * `authority` - Authority public key
    /// * `input_token_account` - Input token account
    /// * `output_token_account` - Output token account
    /// * `input_mint` - Input mint
    /// * `output_mint` - Output mint
    /// * `amount` - Swap amount
    /// * `min_output_amount` - Minimum output amount
    /// * `expected_output` - Expected output amount
    /// * `fee_recipient` - Optional fee recipient
    /// * `account_keys` - Mutable reference to account keys list
    ///
    /// # Returns
    ///
    /// The execute swap instruction
    fn build_execute_swap_instruction_with_accounts(
        &self,
        authority: &Pubkey,
        input_token_account: &Pubkey,
        output_token_account: &Pubkey,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        min_output_amount: u64,
        expected_output: u64,
        fee_recipient: Option<Pubkey>,
        account_keys: &mut Vec<Pubkey>,
    ) -> Result<Instruction, TransactionBuilderError> {
        // Build instruction data
        let mut instruction_data = Vec::with_capacity(8 + 8 + 8 + 8);
        
        // Add discriminator (placeholder)
        instruction_data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);

        // Add arguments
        instruction_data.extend_from_slice(&amount.to_le_bytes());
        instruction_data.extend_from_slice(&min_output_amount.to_le_bytes());
        instruction_data.extend_from_slice(&expected_output.to_le_bytes());

        // Build account metas
        let authority_index = self.get_or_add_account(*authority, account_keys);
        let input_token_account_index = self.get_or_add_account(*input_token_account, account_keys);
        let output_token_account_index = self.get_or_add_account(*output_token_account, account_keys);
        let input_mint_index = self.get_or_add_account(*input_mint, account_keys);
        let output_mint_index = self.get_or_add_account(*output_mint, account_keys);

        let mut accounts = vec![
            AccountMeta::new(account_keys[authority_index], true), // authority
            AccountMeta::new(account_keys[input_token_account_index], false), // input_token_account
            AccountMeta::new(account_keys[output_token_account_index], false), // output_token_account
            AccountMeta::new_readonly(account_keys[input_mint_index], false), // input_mint
            AccountMeta::new_readonly(account_keys[output_mint_index], false), // output_mint
        ];

        // Add fee recipient if provided
        if let Some(fee_recipient) = fee_recipient {
            let fee_recipient_index = self.get_or_add_account(fee_recipient, account_keys);
            accounts.push(AccountMeta::new(account_keys[fee_recipient_index], false));
        }

        // Add token program and system program
        let token_program_index = self.get_or_add_account(self.token_program_id, account_keys);
        // System program ID: 11111111111111111111111111111111
        let system_program_id = Pubkey::from_str("11111111111111111111111111111111")
            .expect("Invalid system program ID");
        let system_program_index = self.get_or_add_account(system_program_id, account_keys);
        accounts.push(AccountMeta::new_readonly(account_keys[token_program_index], false));
        accounts.push(AccountMeta::new_readonly(account_keys[system_program_index], false));

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: instruction_data,
        })
    }
}


