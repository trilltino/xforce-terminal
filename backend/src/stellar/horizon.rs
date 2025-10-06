use serde::Deserialize;
use super::types::*;
use reqwest::Client;
use tracing::{debug, error};

const HORIZON_URL: &str = "https://horizon.stellar.org";
const HORIZON_TESTNET_URL: &str = "https://horizon-testnet.stellar.org";

pub struct HorizonClient {
    client: Client,
    base_url: String,
}

impl HorizonClient {
    pub fn new(testnet: bool) -> Self {
        Self {
            client: Client::new(),
            base_url: if testnet {
                HORIZON_TESTNET_URL.to_string()
            } else {
                HORIZON_URL.to_string()
            },
        }
    }

    /// Get recent trades for XLM/USDC pair
    pub async fn get_xlm_trades(&self, limit: u32) -> Result<Vec<Trade>, String> {
        let url = format!(
            "{}/trades?base_asset_type=native&counter_asset_code=USDC&counter_asset_issuer=GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN&order=desc&limit={}",
            self.base_url, limit
        );

        debug!("Fetching XLM trades from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            error!("Horizon API error: {}", response.status());
            return Err(format!("API error: {}", response.status()));
        }

        let trades_response: TradesResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(trades_response.embedded.records)
    }

    /// Get order book for XLM/USDC
    pub async fn get_xlm_orderbook(&self) -> Result<OrderBook, String> {
        let url = format!(
            "{}/order_book?selling_asset_type=native&buying_asset_code=USDC&buying_asset_issuer=GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN",
            self.base_url
        );

        debug!("Fetching XLM orderbook from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Get 24h ticker data for XLM/USDC
    pub async fn get_xlm_ticker(&self) -> Result<Ticker, String> {
        let url = format!(
            "{}/trade_aggregations?base_asset_type=native&counter_asset_code=USDC&counter_asset_issuer=GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN&resolution=900000&limit=1&order=desc",
            self.base_url
        );

        debug!("Fetching XLM ticker from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct AggregationsResponse {
            #[serde(rename = "_embedded")]
            embedded: EmbeddedAggregations,
        }

        #[derive(Deserialize)]
        struct EmbeddedAggregations {
            records: Vec<Ticker>,
        }

        let agg_response: AggregationsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        agg_response
            .embedded
            .records
            .into_iter()
            .next()
            .ok_or_else(|| "No ticker data available".to_string())
    }

    /// Calculate current XLM price from recent trades
    pub async fn get_current_xlm_price(&self) -> Result<f64, String> {
        let trades = self.get_xlm_trades(10).await?;

        if trades.is_empty() {
            return Err("No trades available".to_string());
        }

        // Calculate average price from recent trades
        let mut total_price = 0.0;
        let mut count = 0;

        for trade in trades.iter() {
            // Price = counter_amount / base_amount
            if let (Ok(base), Ok(counter)) = (
                trade.base_amount.parse::<f64>(),
                trade.counter_amount.parse::<f64>(),
            ) {
                if base > 0.0 {
                    total_price += counter / base;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return Err("Could not calculate price".to_string());
        }

        Ok(total_price / count as f64)
    }

    /// Get price history for chart (using trade aggregations)
    pub async fn get_price_history(&self, resolution: u64, limit: u32) -> Result<Vec<(i64, f64)>, String> {
        let url = format!(
            "{}/trade_aggregations?base_asset_type=native&counter_asset_code=USDC&counter_asset_issuer=GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN&resolution={}&limit={}&order=asc",
            self.base_url, resolution, limit
        );

        debug!("Fetching price history from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct AggregationsResponse {
            #[serde(rename = "_embedded")]
            embedded: EmbeddedAggregations,
        }

        #[derive(Deserialize)]
        struct EmbeddedAggregations {
            records: Vec<AggregationRecord>,
        }

        #[derive(Deserialize)]
        struct AggregationRecord {
            timestamp: String,
            close: String,
        }

        let agg_response: AggregationsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let mut price_history = Vec::new();

        for record in agg_response.embedded.records {
            if let (Ok(timestamp), Ok(price)) = (
                record.timestamp.parse::<i64>(),
                record.close.parse::<f64>(),
            ) {
                price_history.push((timestamp, price));
            }
        }

        Ok(price_history)
    }
}
