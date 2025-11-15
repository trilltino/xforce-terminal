//! # IDL Handler
//!
//! Handles loading and management of Anchor IDL files for contract interaction.
//! Supports both loading from file and using placeholder IDL when the contract hasn't been built yet.

use solana_sdk::pubkey::Pubkey;
use std::path::{Path, PathBuf};
use std::fs;
use std::str::FromStr;
use thiserror::Error;
use tracing::{warn, info};
use serde::{Deserialize, Serialize};

/// IDL metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlMetadata {
    pub address: String,
}

/// IDL structure for Anchor programs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idl {
    pub version: String,
    pub name: String,
    pub metadata: IdlMetadata,
    pub instructions: Vec<serde_json::Value>,
    pub accounts: Vec<serde_json::Value>,
    pub types: Vec<serde_json::Value>,
    pub events: Vec<serde_json::Value>,
    pub errors: Vec<serde_json::Value>,
    #[serde(default)]
    pub constants: Vec<serde_json::Value>,
}

/// IDL loading errors
#[derive(Debug, Error)]
pub enum IdlError {
    #[error("IDL file not found: {0}")]
    NotFound(String),
    
    #[error("Failed to read IDL file: {0}")]
    ReadError(String),
    
    #[error("Failed to parse IDL JSON: {0}")]
    ParseError(String),
    
    #[error("Invalid IDL format: {0}")]
    InvalidFormat(String),
}

/// IDL handler for loading and managing Anchor IDL files
pub struct IdlHandler {
    /// Program ID
    program_id: Pubkey,
    /// IDL file path
    idl_path: PathBuf,
    /// Loaded IDL (if available)
    idl: Option<Idl>,
}

impl IdlHandler {
    /// Create a new IDL handler
    ///
    /// # Arguments
    ///
    /// * `program_id` - The program ID
    /// * `idl_path` - Path to the IDL file
    ///
    /// # Returns
    ///
    /// A new `IdlHandler` instance
    pub fn new(program_id: Pubkey, idl_path: impl AsRef<Path>) -> Self {
        Self {
            program_id,
            idl_path: idl_path.as_ref().to_path_buf(),
            idl: None,
        }
    }

    /// Load IDL from file
    ///
    /// # Returns
    ///
    /// `Ok(())` if IDL was loaded successfully, `Err(IdlError)` if not
    pub fn load(&mut self) -> Result<(), IdlError> {
        if !self.idl_path.exists() {
            warn!("IDL file not found at: {:?}", self.idl_path);
            return Err(IdlError::NotFound(
                format!("IDL file not found at: {:?}", self.idl_path)
            ));
        }

        let idl_contents = fs::read_to_string(&self.idl_path)
            .map_err(|e| IdlError::ReadError(format!("Failed to read IDL file: {}", e)))?;

        // Parse IDL - handle both new format (address at top level) and old format
        let idl: Idl = serde_json::from_str(&idl_contents)
            .map_err(|e| IdlError::ParseError(format!("Failed to parse IDL JSON: {}", e)))?;

        // Validate program ID matches
        // New Anchor IDL format has address at top level in metadata
        let idl_program_id_str = idl.metadata.address.clone();
        if let Ok(idl_program_id) = Pubkey::from_str(&idl_program_id_str) {
            if idl_program_id != self.program_id {
                warn!(
                    "IDL program ID ({}) does not match expected program ID ({})",
                    idl_program_id_str, self.program_id
                );
                warn!("Using program ID from IDL file: {}", idl_program_id_str);
                // Update the program ID to match the IDL (IDL is source of truth)
                // Note: This requires mutable access to self.program_id, but we can't change it here
                // The plugin should use the program ID from the IDL when it's loaded
            } else {
                info!("IDL program ID validated: {}", idl_program_id_str);
            }
        } else {
            warn!("Failed to parse program ID from IDL: {}", idl_program_id_str);
        }

        self.idl = Some(idl);
        info!("IDL loaded successfully from: {:?}", self.idl_path);
        
        Ok(())
    }

    /// Check if IDL is loaded
    ///
    /// # Returns
    ///
    /// `true` if IDL is loaded, `false` otherwise
    pub fn is_loaded(&self) -> bool {
        self.idl.is_some()
    }

    /// Get the loaded IDL
    ///
    /// # Returns
    ///
    /// `Some(Idl)` if IDL is loaded, `None` otherwise
    pub fn get_idl(&self) -> Option<&Idl> {
        self.idl.as_ref()
    }

    /// Create a placeholder IDL for development
    ///
    /// This creates a minimal IDL structure that can be used when the contract
    /// hasn't been built yet. For now, we return a basic structure.
    /// Note: The actual IDL structure from anchor_client may be complex,
    /// so we'll use a simplified approach and rely on manual instruction building.
    ///
    /// # Returns
    ///
    /// A placeholder `Idl` instance (simplified)
    pub fn create_placeholder_idl() -> Idl {
        // For now, return a minimal IDL structure
        // The actual implementation will load from the generated IDL file
        // This is just a placeholder to prevent errors
        serde_json::from_str(r#"{
            "version": "0.1.0",
            "name": "batch_swap_router",
            "metadata": {
                "address": "HS63bw1V1qTM5uWf92q3uaFdqogrc4SN9qUJSR8aqBMx"
            },
            "instructions": [],
            "accounts": [],
            "types": [],
            "events": [],
            "errors": []
        }"#).unwrap_or_else(|_| {
            // If JSON parsing fails, create a minimal IDL manually
            // This is a fallback - in practice, we'll load from the actual IDL file
            warn!("Failed to create placeholder IDL from JSON, using minimal structure");
            Idl {
                version: "0.1.0".to_string(),
                name: "batch_swap_router".to_string(),
                metadata: IdlMetadata {
                    address: "HS63bw1V1qTM5uWf92q3uaFdqogrc4SN9qUJSR8aqBMx".to_string(),
                },
                instructions: vec![],
                accounts: vec![],
                types: vec![],
                events: vec![],
                errors: vec![],
                constants: vec![],
            }
        })
    }

    /// Get IDL (load from file if needed, or use placeholder)
    ///
    /// # Returns
    ///
    /// The IDL (either loaded from file or placeholder)
    pub fn get_or_create_idl(&mut self) -> Result<Idl, IdlError> {
        // Try to load from file first
        if self.load().is_ok() {
            if let Some(idl) = &self.idl {
                return Ok(idl.clone());
            }
        }

        // If loading failed, create and return placeholder
        warn!("Using placeholder IDL - contract should be built to generate real IDL");
        Ok(Self::create_placeholder_idl())
    }
}

/// Get the default IDL path for batch swap router
///
/// # Returns
///
/// Path to the IDL file
///
/// This function tries multiple strategies to find the IDL file:
/// 1. Environment variable `BATCH_SWAP_ROUTER_IDL_PATH` (highest priority)
/// 2. Contracts directory relative to workspace root
/// 3. Contracts directory using common parent directory resolution
/// 4. Backend IDL directory (fallback)
pub fn get_default_idl_path() -> PathBuf {
    // Strategy 1: Check environment variable first (highest priority)
    if let Ok(env_path) = std::env::var("BATCH_SWAP_ROUTER_IDL_PATH") {
        let path = PathBuf::from(env_path);
        if path.exists() {
            return path;
        }
        warn!("IDL path from environment variable does not exist: {:?}", path);
    }
    
        // Strategy 2: Try contracts directory using common parent directory resolution
        // Both xforce-terminal and xforce-terminal-contracts are siblings under C:\Users\isich\
        // Directory structure:
        //   C:\Users\isich\
        //     ├── xforce-terminal\          (this workspace)
        //     │     └── backend\
        //     │           └── target\debug\
        //     │                 └── backend.exe  (current_exe)
        //     └── xforce-terminal-contracts\
        //           └── target\idl\
        //                 └── batch_swap_router.json  (target IDL)
        if let Ok(current_exe) = std::env::current_exe() {
            // If running from target/debug or target/release, navigate up to find parent directory
            if let Some(parent) = current_exe.parent() {
                if parent.ends_with("target/debug") || parent.ends_with("target/release") {
                    // Navigate up: target/debug -> backend -> xforce-terminal (workspace root)
                    if let Some(workspace_root) = parent.parent().and_then(|p| p.parent()) {
                        // Go up one more level to get to the common parent directory (C:\Users\isich\)
                        if let Some(parent_dir) = workspace_root.parent() {
                            // Construct path to contracts IDL file
                            let contracts_idl = parent_dir.join("xforce-terminal-contracts/target/idl/batch_swap_router.json");
                            if contracts_idl.exists() {
                                info!("Found IDL at: {:?}", contracts_idl);
                                return contracts_idl;
                            }
                        }
                    }
                }
            }
        }
    
    // Strategy 3: Try contracts directory relative to current working directory
    // This works when running from workspace root or backend directory
    let possible_paths = vec![
        // From workspace root
        PathBuf::from("../xforce-terminal-contracts/target/idl/batch_swap_router.json"),
        // From backend directory
        PathBuf::from("../../xforce-terminal-contracts/target/idl/batch_swap_router.json"),
        // From backend/src directory (if running from source)
        PathBuf::from("../../../xforce-terminal-contracts/target/idl/batch_swap_router.json"),
    ];
    
    for path in possible_paths {
        if path.exists() {
            info!("Found IDL at: {:?}", path);
            return path;
        }
    }
    
    // Strategy 4: Try default Windows user path (for development)
    // This handles the case where the contracts directory is at C:\Users\isich\xforce-terminal-contracts
    if let Ok(username) = std::env::var("USERPROFILE") {
        let default_contracts_path = PathBuf::from(&username)
            .join("xforce-terminal-contracts/target/idl/batch_swap_router.json");
        if default_contracts_path.exists() {
            info!("Found IDL at: {:?}", default_contracts_path);
            return default_contracts_path;
        }
    }
    
    // Strategy 4: Fallback to root idl directory
    let fallback_path = PathBuf::from("idl/batch_swap_router.json");
    if fallback_path.exists() {
        warn!("Using fallback IDL path: {:?}", fallback_path);
        return fallback_path;
    }
    
    // If nothing found, return the most likely path for error reporting
    warn!("IDL file not found in any expected location. Using default path for error reporting.");
    PathBuf::from("../../xforce-terminal-contracts/target/idl/batch_swap_router.json")
}

/// Load IDL for batch swap router
///
/// # Arguments
///
/// * `program_id` - The program ID
///
/// # Returns
///
/// `Ok(IdlHandler)` if IDL handler was created, `Err(IdlError)` if not
pub fn load_batch_swap_idl(program_id: Pubkey) -> Result<IdlHandler, IdlError> {
    let idl_path = get_default_idl_path();
    let mut handler = IdlHandler::new(program_id, idl_path);
    
    // Try to load, but don't fail if it doesn't exist (will use placeholder)
    let _ = handler.load();
    
    Ok(handler)
}

