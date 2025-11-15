//! # Jupiter Quote API
//!
//! Quote API integration for getting swap quotes from Jupiter.

use super::client::JupiterHttpClient;
use super::types::QuoteResponse;
use tracing::debug;

impl JupiterHttpClient {
    /// Get a swap quote from Jupiter Aggregator V6
    pub async fn get_swap_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> anyhow::Result<QuoteResponse> {
        let quote_url = "https://quote-api.jup.ag/v6";
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            quote_url, input_mint, output_mint, amount, slippage_bps
        );

        debug!("Jupiter swap quote request: {}", url);

        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Jupiter quote failed: {}", error_text));
        }

        let quote: QuoteResponse = response.json().await?;

        debug!(
            "Jupiter quote: {} lamports -> {} lamports (impact: {:.2}%)",
            quote.in_amount, quote.out_amount, quote.price_impact_pct
        );

        Ok(quote)
    }
}

