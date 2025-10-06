use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use serde_json::{json, Value};
use tracing::{info, error, debug};

use crate::soroban::client;
use shared::dto::soroban::FunctionParameter;

const REFLECTOR_ORACLE_ID: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";
const SOROBAN_RPC_URL: &str = "https://soroban-testnet.stellar.org";
const NETWORK_PASSPHRASE: &str = "Test SDF Network ; September 2015";

/// In-memory cache for Reflector Oracle prices
#[derive(Clone)]
pub struct OracleCache {
    prices: Arc<RwLock<HashMap<String, Value>>>,
    last_update: Arc<RwLock<u64>>,
}

impl OracleCache {
    pub fn new() -> Self {
        Self {
            prices: Arc::new(RwLock::new(HashMap::new())),
            last_update: Arc::new(RwLock::new(0)),
        }
    }

    /// Get all cached prices
    pub async fn get_all(&self) -> (HashMap<String, Value>, u64) {
        let prices = self.prices.read().await.clone();
        let timestamp = *self.last_update.read().await;
        (prices, timestamp)
    }

    /// Update cache with new prices
    async fn update(&self, new_prices: HashMap<String, Value>) {
        let mut prices = self.prices.write().await;
        *prices = new_prices;

        let mut last_update = self.last_update.write().await;
        *last_update = chrono::Utc::now().timestamp() as u64;

        info!("📦 Cache updated with {} prices", prices.len());
    }

    /// Fetch a single asset price from Reflector Oracle
    async fn fetch_asset_price(&self, asset: String) -> Option<(String, Value)> {
        let params = vec![
            FunctionParameter::Enum(
                "Other".to_string(),
                Some(Box::new(FunctionParameter::Symbol(asset.clone())))
            )
        ];

        match client::call_contract_function(
            REFLECTOR_ORACLE_ID,
            "lastprice",
            params,
            None,
            SOROBAN_RPC_URL,
            NETWORK_PASSPHRASE,
        ).await {
            Ok(response) if response.success => {
                if let Some(result) = response.result {
                    if let Some(price_str) = result.get("price").and_then(|p| p.as_str()) {
                        if let Ok(price_raw) = price_str.parse::<u128>() {
                            let price = price_raw as f64 / 100_000_000_000_000.0; // 14 decimals
                            let timestamp = result.get("timestamp").and_then(|t| t.as_u64())
                                .unwrap_or_else(|| chrono::Utc::now().timestamp() as u64);

                            let price_data = json!({
                                "price": price,
                                "price_raw": price_str,
                                "timestamp": timestamp,
                                "symbol": asset,
                            });

                            debug!("✅ Fetched {}: ${:.6}", asset, price);
                            return Some((asset, price_data));
                        }
                    }
                }
                error!("❌ Failed to parse price for {}: invalid response format", asset);
            }
            Ok(response) => {
                error!("❌ RPC call failed for {}: {:?}", asset, response.error);
            }
            Err(e) => {
                error!("❌ RPC call error for {}: {}", asset, e);
            }
        }
        None
    }

    /// Fetch all prices from Reflector Oracle
    async fn fetch_all_prices(&self) -> HashMap<String, Value> {
        info!("🔮 Fetching fresh prices from Reflector Oracle...");

        let assets = vec![
            "BTC", "ETH", "XLM", "SOL", "USDT", "USDC", "XRP", "ADA",
            "AVAX", "DOT", "MATIC", "LINK", "DAI", "ATOM", "UNI", "EURC",
        ];

        let mut tasks = Vec::new();

        // Spawn concurrent fetch tasks
        for asset in assets.iter() {
            let asset = asset.to_string();
            let cache = self.clone();
            let task = tokio::spawn(async move {
                cache.fetch_asset_price(asset).await
            });
            tasks.push(task);
        }

        // Collect results
        let mut new_prices = HashMap::new();
        for task in tasks {
            if let Ok(Some((asset, price_data))) = task.await {
                new_prices.insert(asset, price_data);
            }
        }

        info!("✅ Fetched {}/{} prices successfully", new_prices.len(), assets.len());
        new_prices
    }

    /// Start background refresh task (updates cache every N seconds)
    pub fn start_background_refresh(self, interval_secs: u64) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            info!("🔄 Starting Reflector Oracle cache refresh (every {}s)", interval_secs);

            loop {
                ticker.tick().await;

                let new_prices = self.fetch_all_prices().await;

                if !new_prices.is_empty() {
                    self.update(new_prices).await;
                } else {
                    error!("⚠️  Failed to fetch any prices - keeping old cache");
                }
            }
        });
    }
}
