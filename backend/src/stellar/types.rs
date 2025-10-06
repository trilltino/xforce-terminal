use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub asset_type: String,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub ledger_close_time: String,
    pub trade_type: String,
    pub base_asset_type: String,
    pub base_asset_code: Option<String>,
    pub base_amount: String,
    pub counter_asset_type: String,
    pub counter_asset_code: Option<String>,
    pub counter_amount: String,
    pub price: PriceRatio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRatio {
    pub n: u64,
    pub d: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesResponse {
    #[serde(rename = "_embedded")]
    pub embedded: EmbeddedTrades,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedTrades {
    pub records: Vec<Trade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
    pub base: Asset,
    pub counter: Asset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    pub price: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub base_volume: String,
    pub counter_volume: String,
    pub trade_count: u64,
    pub open: String,
    pub low: String,
    pub high: String,
    pub close: String,
}
