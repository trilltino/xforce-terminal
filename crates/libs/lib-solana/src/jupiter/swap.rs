//! # Jupiter Swap Transaction Building
//!
//! Swap transaction building from Jupiter quotes.

use super::client::JupiterHttpClient;
use super::types::{QuoteResponse, SwapTransactionResponse};
use tracing::debug;

impl JupiterHttpClient {
    /// Build an unsigned swap transaction from a quote
    pub async fn get_swap_transaction(
        &self,
        quote_response: &QuoteResponse,
        user_public_key: &str,
    ) -> anyhow::Result<SwapTransactionResponse> {
        let swap_url = "https://quote-api.jup.ag/v6/swap";

        let request_body = serde_json::json!({
            "quoteResponse": quote_response,
            "userPublicKey": user_public_key,
            "wrapAndUnwrapSol": true,
            "dynamicComputeUnitLimit": true,
            "prioritizationFeeLamports": "auto",
        });

        debug!("Jupiter swap transaction request for user: {}", user_public_key);

        let response = self
            .http
            .post(swap_url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Jupiter swap transaction failed: {}", error_text));
        }

        let swap_response: SwapTransactionResponse = response.json().await?;

        debug!("Jupiter swap transaction received");

        Ok(swap_response)
    }
}

