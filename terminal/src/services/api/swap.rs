//! # Swap Endpoints
//!
//! Handles swap operations (quote, execute, submit, history).

use serde::{Deserialize, Serialize};
use shared::ErrorResponse;
use super::client::ApiClient;

/// Get swap quote from Jupiter.
pub async fn get_swap_quote(
    client: &ApiClient,
    input_mint: &str,
    output_mint: &str,
    amount: u64,
    slippage_bps: u16,
) -> Result<SwapQuoteResponse, String> {
    let url = format!(
        "{}/api/swap/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
        ApiClient::base_url(), input_mint, output_mint, amount, slippage_bps
    );

    let response = client
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json::<SwapQuoteResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        let error = response
            .json::<ErrorResponse>()
            .await
            .map_err(|e| format!("Failed to parse error: {}", e))?;
        Err(error.error)
    }
}

/// Execute swap - get unsigned transaction.
#[tracing::instrument(skip(client, jwt_token), fields(
    input_mint = %input_mint,
    output_mint = %output_mint,
    amount = amount,
    slippage_bps = slippage_bps,
    user = %user_public_key
))]
pub async fn execute_swap(
    client: &ApiClient,
    input_mint: &str,
    output_mint: &str,
    amount: u64,
    slippage_bps: u16,
    user_public_key: &str,
    jwt_token: &str,
) -> Result<SwapExecuteResponse, String> {
    tracing::info!("Executing swap");
    let start = std::time::Instant::now();

    let request = SwapExecuteRequest {
        input_mint: input_mint.to_string(),
        output_mint: output_mint.to_string(),
        amount,
        slippage_bps,
        user_public_key: user_public_key.to_string(),
    };

    let response = client
        .client
        .post(format!("{}/api/swap/execute", ApiClient::base_url()))
        .header("Authorization", format!("Bearer {}", jwt_token))
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Swap execution network error");
            format!("Network error: {}", e)
        })?;

    let duration = start.elapsed();
    let status = response.status();

    if status.is_success() {
        let result = response
            .json::<SwapExecuteResponse>()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Swap response parse error");
                format!("Failed to parse response: {}", e)
            });

        if result.is_ok() {
            tracing::info!(duration_ms = duration.as_millis(), "Swap executed successfully");
        }
        result
    } else {
        let error = response
            .json::<ErrorResponse>()
            .await
            .map_err(|e| format!("Failed to parse error: {}", e))?;

        tracing::warn!(
            status = status.as_u16(),
            error = %error.error,
            duration_ms = duration.as_millis(),
            "Swap execution failed"
        );
        Err(error.error)
    }
}

/// Submit signed transaction.
pub async fn submit_transaction(
    client: &ApiClient,
    signed_transaction: String,
    input_mint: String,
    output_mint: String,
    input_amount: i64,
    output_amount: i64,
    price_impact: Option<f64>,
    slippage_bps: Option<i32>,
    jwt_token: &str,
) -> Result<TransactionSubmitResponse, String> {
    let request = TransactionSubmitRequest {
        signed_transaction,
        input_mint,
        output_mint,
        input_amount,
        output_amount,
        price_impact,
        slippage_bps,
    };

    let response = client
        .client
        .post(format!("{}/api/transactions/submit", ApiClient::base_url()))
        .header("Authorization", format!("Bearer {}", jwt_token))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json::<TransactionSubmitResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        let error = response
            .json::<ErrorResponse>()
            .await
            .map_err(|e| format!("Failed to parse error: {}", e))?;
        Err(error.error)
    }
}

/// Get swap history for user.
pub async fn get_swap_history(
    client: &ApiClient,
    jwt_token: &str,
    limit: usize,
) -> Result<Vec<SwapHistoryItem>, String> {
    let url = format!("{}/api/swap/history?limit={}", ApiClient::base_url(), limit);

    let response = client
        .client
        .get(&url)
        .header("Authorization", format!("Bearer {}", jwt_token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json::<SwapHistoryResponse>()
            .await
            .map(|resp| resp.swaps)
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        Err(format!("Failed to fetch swap history: {}", response.status()))
    }
}

// ==================== SWAP TYPES ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuoteResponse {
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
    pub routes: Vec<RouteInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    pub dex: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapExecuteRequest {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    pub amount: u64,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapExecuteResponse {
    pub transaction: String, // Base64-encoded unsigned transaction
    #[serde(rename = "lastValidBlockHeight")]
    pub last_valid_block_height: u64,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSubmitRequest {
    #[serde(rename = "signedTransaction")]
    pub signed_transaction: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inputAmount")]
    pub input_amount: i64,
    #[serde(rename = "outputAmount")]
    pub output_amount: i64,
    #[serde(rename = "priceImpact")]
    pub price_impact: Option<f64>,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSubmitResponse {
    pub signature: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapHistoryItem {
    pub id: i64,
    pub signature: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inputAmount")]
    pub input_amount: i64,
    #[serde(rename = "outputAmount")]
    pub output_amount: i64,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapHistoryResponse {
    pub swaps: Vec<SwapHistoryItem>,
}

