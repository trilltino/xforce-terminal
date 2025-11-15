//! Multi-Wallet Integration via wasm-bindgen
//!
//! This module provides JavaScript interop for multiple Solana wallet providers.
//! Supports Phantom, Solflare, Backpack, and other Wallet Standard compatible wallets.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// Supported wallet provider types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WalletProvider {
    Phantom,
    Solflare,
    Backpack,
    Sollet,
    Ledger,
}

impl WalletProvider {
    pub fn name(&self) -> &'static str {
        match self {
            WalletProvider::Phantom => "Phantom",
            WalletProvider::Solflare => "Solflare",
            WalletProvider::Backpack => "Backpack",
            WalletProvider::Sollet => "Sollet",
            WalletProvider::Ledger => "Ledger",
        }
    }
}

// ============================================================================
// MULTI-WALLET DETECTION AND CONNECTION (JavaScript Interop)
// ============================================================================

/// Check which wallets are installed and available
#[wasm_bindgen(inline_js = "
export function detectWallets() {
    const wallets = [];
    
    // Check Phantom
    if (window.solana && window.solana.isPhantom) {
        wallets.push({ name: 'Phantom', provider: 'phantom', installed: true });
    }
    
    // Check Solflare - multiple detection methods for better compatibility
    let solflareDetected = false;
    
    // Method 1: Check for window.solflare directly
    if (window.solflare) {
        // Check if it has connect method (Solflare extension)
        if (typeof window.solflare.connect === 'function' || window.solflare.isConnected) {
            wallets.push({ name: 'Solflare', provider: 'solflare', installed: true });
            solflareDetected = true;
        }
    }
    
    // Method 2: Check for Solflare via Wallet Standard (window.solana)
    if (!solflareDetected && window.solana) {
        // Check if solana object identifies as Solflare
        if (window.solana.isSolflare === true) {
            wallets.push({ name: 'Solflare', provider: 'solflare', installed: true });
            solflareDetected = true;
        }
        // Also check by name property
        if (!solflareDetected && window.solana.name && 
            (window.solana.name.toLowerCase().includes('solflare') || 
             window.solana.name === 'Solflare')) {
            wallets.push({ name: 'Solflare', provider: 'solflare', installed: true });
            solflareDetected = true;
        }
    }
    
    // Method 3: Check for Solflare in window object with different names
    if (!solflareDetected) {
        // Some versions might expose it differently
        if (window.Solflare || window.__solflare) {
            wallets.push({ name: 'Solflare', provider: 'solflare', installed: true });
        }
    }
    
    // Check Backpack
    if (window.backpack) {
        wallets.push({ name: 'Backpack', provider: 'backpack', installed: true });
    } else if (window.solana && window.solana.isBackpack) {
        wallets.push({ name: 'Backpack', provider: 'backpack', installed: true });
    }
    
    // Check Sollet
    if (window.sollet) {
        wallets.push({ name: 'Sollet', provider: 'sollet', installed: true });
    }
    
    return wallets;
}

export function getWalletAdapter(provider) {
    switch(provider) {
        case 'phantom':
            return window.solana && window.solana.isPhantom ? window.solana : null;
        case 'solflare':
            // Try multiple methods to get Solflare adapter
            // Method 1: Direct window.solflare
            if (window.solflare) {
                // Check if it has the expected methods
                if (typeof window.solflare.connect === 'function' || 
                    window.solflare.isConnected !== undefined ||
                    window.solflare.publicKey !== undefined) {
                    return window.solflare;
                }
                // Some versions expose the adapter differently
                if (window.solflare.adapter) {
                    return window.solflare.adapter;
                }
            }
            // Method 2: Via window.solana with isSolflare flag
            if (window.solana && window.solana.isSolflare === true) {
                return window.solana;
            }
            // Method 3: Check by name
            if (window.solana && window.solana.name) {
                const name = window.solana.name.toLowerCase();
                if (name.includes('solflare') || name === 'solflare') {
                    return window.solana;
                }
            }
            // Method 4: Check alternative locations
            if (window.Solflare && window.Solflare.adapter) {
                return window.Solflare.adapter;
            }
            if (window.__solflare) {
                return window.__solflare;
            }
            return null;
        case 'backpack':
            if (window.backpack) {
                return window.backpack;
            }
            if (window.solana && window.solana.isBackpack) {
                return window.solana;
            }
            return null;
        case 'sollet':
            return window.sollet || null;
        default:
            return window.solana || null;
    }
}

export async function connectWallet(provider) {
    const adapter = getWalletAdapter(provider);
    if (!adapter) {
        throw new Error(provider + ' wallet not found. Please install the Solflare extension from https://solflare.com');
    }
    
    try {
        // Solflare-specific connection handling
        if (provider === 'solflare') {
            // Check if already connected
            if (adapter.isConnected && adapter.isConnected === true) {
                // Get public key from connected wallet
                if (adapter.publicKey) {
                    return {
                        publicKey: adapter.publicKey.toString(),
                        provider: provider
                    };
                }
                // Try to get account
                if (adapter.account && adapter.account.publicKey) {
                    return {
                        publicKey: adapter.account.publicKey.toString(),
                        provider: provider
                    };
                }
            }
            
            // Try connecting with multiple methods
            let response;
            try {
                // Method 1: Standard connect() without options first
                if (typeof adapter.connect === 'function') {
                    try {
                        response = await adapter.connect();
                    } catch (err) {
                        // Method 2: Try connect() with options if first attempt fails
                        try {
                            response = await adapter.connect({ onlyIfTrusted: false });
                        } catch (err2) {
                            // Method 3: Try enable() method
                            if (adapter.enable && typeof adapter.enable === 'function') {
                                await adapter.enable();
                                if (adapter.publicKey) {
                                    response = { publicKey: adapter.publicKey };
                                } else {
                                    throw new Error('Connected but could not get public key');
                                }
                            } else {
                                throw err2;
                            }
                        }
                    }
                } else if (adapter.enable && typeof adapter.enable === 'function') {
                    // Method 3: Try enable() if connect() doesn't exist
                    await adapter.enable();
                    if (adapter.publicKey) {
                        response = { publicKey: adapter.publicKey };
                    } else {
                        throw new Error('Connected but could not get public key');
                    }
                } else {
                    throw new Error('No connect method available on Solflare adapter');
                }
            } catch (connectError) {
                // Last resort: check if already has publicKey
                if (adapter.publicKey) {
                    response = { publicKey: adapter.publicKey };
                } else {
                    throw connectError;
                }
            }
            
            // Extract public key from response
            let publicKey;
            if (response.publicKey) {
                publicKey = response.publicKey;
            } else if (response.account && response.account.publicKey) {
                publicKey = response.account.publicKey;
            } else if (adapter.publicKey) {
                publicKey = adapter.publicKey;
            } else {
                throw new Error('Connected but could not retrieve public key');
            }
            
            // Convert to string if it's an object
            const publicKeyStr = publicKey.toString ? publicKey.toString() : String(publicKey);
            
            return {
                publicKey: publicKeyStr,
                provider: provider
            };
        } else {
            // Standard connection for other wallets
            const response = await adapter.connect();
            return {
                publicKey: response.publicKey.toString(),
                provider: provider
            };
        }
    } catch (error) {
        const errorMsg = error.message || String(error);
        throw new Error('Failed to connect to ' + provider + ': ' + errorMsg + 
            '. Make sure the Solflare extension is installed and unlocked.');
    }
}

export function getWalletAddress(provider) {
    const adapter = getWalletAdapter(provider);
    if (!adapter || !adapter.publicKey) {
        return null;
    }
    return adapter.publicKey.toString();
}

export async function signMessageWithWallet(provider, messageBytes, display) {
    const adapter = getWalletAdapter(provider);
    if (!adapter) {
        throw new Error(provider + ' wallet not found');
    }
    
    try {
        // Convert message bytes to Uint8Array
        const message = new Uint8Array(messageBytes);
        
        // Different wallets have slightly different APIs
        if (adapter.signMessage) {
            return await adapter.signMessage(message, display || 'utf8');
        } else if (adapter.sign && adapter.signMessage) {
            // Some wallets use sign() method
            return await adapter.sign(message, display || 'utf8');
        } else {
            throw new Error('Wallet does not support message signing');
        }
    } catch (error) {
        throw new Error('Failed to sign message: ' + error.message);
    }
}
")]
extern "C" {
    /// Detect all installed wallets
    pub fn detectWallets() -> JsValue;
    
    /// Get wallet adapter for a specific provider
    pub fn getWalletAdapter(provider: &str) -> Option<JsValue>;
    
    /// Connect to a specific wallet provider
    #[wasm_bindgen(catch)]
    pub async fn connectWallet(provider: &str) -> Result<JsValue, JsValue>;
    
    /// Get the address from a connected wallet
    pub fn getWalletAddress(provider: &str) -> Option<String>;
    
    /// Sign a message with a specific wallet
    #[wasm_bindgen(catch)]
    pub async fn signMessageWithWallet(provider: &str, message_bytes: &[u8], display: &str) -> Result<JsValue, JsValue>;
}

// ============================================================================
// PHANTOM WALLET BINDINGS (Legacy - for backward compatibility)
// ============================================================================

/// Check if Phantom wallet extension is installed
#[wasm_bindgen(inline_js = "
export function isPhantomInstalled() {
    return window.solana && window.solana.isPhantom === true;
}
")]
extern "C" {
    pub fn isPhantomInstalled() -> bool;
}

/// Connect to Phantom wallet (legacy function)
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "solana"], js_name = connect, catch)]
    pub async fn phantom_connect() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "solana"], js_name = signMessage, catch)]
    pub async fn phantom_sign_message(message: &[u8], display: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "solana"], js_name = signTransaction, catch)]
    pub async fn phantom_sign_transaction(transaction: &JsValue) -> Result<JsValue, JsValue>;
}

/// Helper functions for Phantom wallet operations
#[wasm_bindgen(inline_js = "
export function getPhantomAddress() {
    if (window.solana && window.solana.publicKey) {
        return window.solana.publicKey.toString();
    }
    return null;
}

export function extractSignature(signResponse) {
    if (signResponse && signResponse.signature) {
        // Convert Uint8Array to regular array
        return Array.from(signResponse.signature);
    }
    return null;
}

export function deserializeTransaction(base64Tx) {
    try {
        const txBuffer = Uint8Array.from(atob(base64Tx), c => c.charCodeAt(0));
        const tx = window.solanaWeb3.Transaction.from(txBuffer);
        return tx;
    } catch (e) {
        console.error('Failed to deserialize transaction:', e);
        return null;
    }
}

export function serializeSignedTransaction(signedTx) {
    try {
        const serialized = signedTx.serialize();
        return btoa(String.fromCharCode.apply(null, serialized));
    } catch (e) {
        console.error('Failed to serialize transaction:', e);
        return null;
    }
}

export async function signTransactionWithProvider(provider, base64Tx) {
    try {
        // Get wallet adapter
        let adapter = null;
        switch(provider) {
            case 'phantom':
                adapter = window.solana && window.solana.isPhantom ? window.solana : null;
                break;
            case 'solflare':
                adapter = window.solflare || (window.solana && window.solana.isSolflare ? window.solana : null);
                break;
            case 'backpack':
                adapter = window.backpack || (window.solana && window.solana.isBackpack ? window.solana : null);
                break;
            case 'sollet':
                adapter = window.sollet || null;
                break;
            default:
                adapter = window.solana || null;
        }
        
        if (!adapter) {
            throw new Error('Wallet adapter not found');
        }
        
        // Deserialize transaction from base64
        const txBuffer = Uint8Array.from(atob(base64Tx), c => c.charCodeAt(0));
        
        // Use Solana Web3.js if available
        if (typeof window.solanaWeb3 !== 'undefined') {
            const tx = window.solanaWeb3.Transaction.from(txBuffer);
            
            // Sign transaction
            if (adapter.signTransaction) {
                const signedTx = await adapter.signTransaction(tx);
                const serialized = signedTx.serialize();
                return btoa(String.fromCharCode.apply(null, serialized));
            } else if (adapter.sendTransaction) {
                // Some wallets only support sendTransaction
                // This will submit directly, so we return the signature as a string
                const result = await adapter.sendTransaction(tx, 'confirmed');
                // Ensure we return a string - result might be a signature object or string
                if (typeof result === 'string') {
                    return result;
                } else if (result && typeof result === 'object' && result.signature) {
                    return String(result.signature);
                } else {
                    return String(result);
                }
            } else {
                throw new Error('Wallet does not support transaction signing');
            }
        } else {
            throw new Error('Solana Web3.js not loaded');
        }
    } catch (error) {
        // Convert error to string for wasm_bindgen
        const errorMsg = error instanceof Error ? error.message : String(error);
        throw new Error(errorMsg);
    }
}
")]
extern "C" {
    /// Get connected Phantom wallet address as Base58 string (returns null if not connected)
    pub fn getPhantomAddress() -> Option<String>;

    /// Extract signature bytes from Phantom sign response (returns null array if no signature)
    pub fn extractSignature(val: &JsValue) -> Option<Vec<u8>>;

    /// Deserialize base64 transaction for Phantom signing (returns null on error)
    #[wasm_bindgen(catch)]
    pub fn deserializeTransaction(base64_tx: &str) -> Result<JsValue, JsValue>;

    /// Serialize signed transaction back to base64 (returns null on error)
    pub fn serializeSignedTransaction(signed_tx: &JsValue) -> Option<String>;
    
    /// Sign a transaction with a wallet provider (returns base64 signed transaction as JsValue)
    #[wasm_bindgen(catch)]
    pub async fn signTransactionWithProvider(provider: &str, base64_tx: &str) -> Result<JsValue, JsValue>;
}

// ============================================================================
// WALLET SERVICE
// ============================================================================

/// Wallet connection state with provider information
#[derive(Clone, PartialEq)]
pub enum WalletState {
    Disconnected,
    Connecting,
    Connected { address: String, provider: WalletProvider },
    Error(String),
}

impl WalletState {
    pub fn is_connected(&self) -> bool {
        matches!(self, WalletState::Connected { .. })
    }

    pub fn address(&self) -> Option<&str> {
        match self {
            WalletState::Connected { address, .. } => Some(address),
            _ => None,
        }
    }
    
    pub fn provider(&self) -> Option<WalletProvider> {
        match self {
            WalletState::Connected { provider, .. } => Some(provider.clone()),
            _ => None,
        }
    }
}

/// Detected wallet information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectedWallet {
    pub name: String,
    pub provider: String,
    pub installed: bool,
}

/// Get list of available wallets
pub fn get_available_wallets() -> Vec<DetectedWallet> {
    let wallets_js = detectWallets();
    let wallets: Vec<DetectedWallet> = serde_wasm_bindgen::from_value(wallets_js)
        .unwrap_or_else(|_| vec![]);
    wallets
}

/// Check if a specific wallet is installed
pub fn is_wallet_installed(provider: &WalletProvider) -> bool {
    let provider_str = match provider {
        WalletProvider::Phantom => "phantom",
        WalletProvider::Solflare => "solflare",
        WalletProvider::Backpack => "backpack",
        WalletProvider::Sollet => "sollet",
        WalletProvider::Ledger => "ledger",
    };
    getWalletAdapter(provider_str).is_some()
}

/// Connect to a wallet provider
pub async fn connect_wallet_provider(provider: &WalletProvider) -> Result<(String, WalletProvider), String> {
    let provider_str = match provider {
        WalletProvider::Phantom => "phantom",
        WalletProvider::Solflare => "solflare",
        WalletProvider::Backpack => "backpack",
        WalletProvider::Sollet => "sollet",
        WalletProvider::Ledger => "ledger",
    };
    
    match connectWallet(provider_str).await {
        Ok(result) => {
            // Extract publicKey directly from JS object using js_sys::Reflect
            use js_sys::Reflect;
            let pk_val = Reflect::get(&result, &JsValue::from_str("publicKey"))
                .map_err(|_| "Failed to get publicKey from result".to_string())?;
            
            let address = pk_val.as_string()
                .ok_or_else(|| "PublicKey is not a string".to_string())?;
            
            Ok((address, provider.clone()))
        }
        Err(e) => {
            let error_msg = if let Some(err_str) = e.as_string() {
                err_str
            } else {
                format!("Connection error: {:?}", e)
            };
            Err(error_msg)
        }
    }
}

/// Get address from connected wallet
pub fn get_connected_wallet_address(provider: &WalletProvider) -> Option<String> {
    let provider_str = match provider {
        WalletProvider::Phantom => "phantom",
        WalletProvider::Solflare => "solflare",
        WalletProvider::Backpack => "backpack",
        WalletProvider::Sollet => "sollet",
        WalletProvider::Ledger => "ledger",
    };
    getWalletAddress(provider_str)
}

/// Sign a message with a wallet provider
pub async fn sign_message_provider(
    provider: &WalletProvider,
    message: &[u8],
    display: &str,
) -> Result<JsValue, String> {
    let provider_str = match provider {
        WalletProvider::Phantom => "phantom",
        WalletProvider::Solflare => "solflare",
        WalletProvider::Backpack => "backpack",
        WalletProvider::Sollet => "sollet",
        WalletProvider::Ledger => "ledger",
    };
    
    signMessageWithWallet(provider_str, message, display)
        .await
        .map_err(|e| {
            if let Some(err_str) = e.as_string() {
                err_str
            } else {
                format!("Sign error: {:?}", e)
            }
        })
}
