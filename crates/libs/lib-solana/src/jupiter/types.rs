//! # Jupiter API Types
//!
//! Type definitions for Jupiter Aggregator API responses.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response from Jupiter price API
#[derive(Debug, Deserialize)]
pub struct JupiterPriceResponse {
    pub data: HashMap<String, JupiterPriceData>,
}

/// Price data for a single token
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JupiterPriceData {
    pub id: String,
    #[serde(rename = "mintSymbol")]
    pub mint_symbol: String,
    #[serde(rename = "vsToken")]
    pub vs_token: String,
    #[serde(rename = "vsTokenSymbol")]
    pub vs_token_symbol: String,
    pub price: f64,
}

/// Token information from Jupiter token list
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub tags: Vec<String>,
}

/// Response from Jupiter quote API
#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteResponse {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: f64,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RoutePlanStep>,
}

/// A step in Jupiter's routing plan
#[derive(Debug, Serialize, Deserialize)]
pub struct RoutePlanStep {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
}

/// Details about a single swap operation within a route
#[derive(Debug, Serialize, Deserialize)]
pub struct SwapInfo {
    #[serde(rename = "ammKey")]
    pub amm_key: String,
    pub label: Option<String>,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
    #[serde(rename = "feeMint")]
    pub fee_mint: String,
}

/// Response from Jupiter swap API
#[derive(Debug, Serialize, Deserialize)]
pub struct SwapTransactionResponse {
    /// Base64-encoded serialized Solana transaction
    #[serde(rename = "swapTransaction")]
    pub swap_transaction: String,
    /// Block height after which transaction is invalid
    #[serde(rename = "lastValidBlockHeight")]
    pub last_valid_block_height: u64,
    /// Optional priority fee in lamports
    #[serde(rename = "prioritizationFeeLamports")]
    pub prioritization_fee_lamports: Option<u64>,
}

