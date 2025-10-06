use crate::stellar::HorizonClient;
use axum::{extract::State, http::StatusCode, Json};
use shared::{ErrorResponse, MarketDataResponse, PriceData};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Get XLM price history for chart
pub async fn get_xlm_price_history(
    State(horizon): State<Arc<HorizonClient>>,
) -> Result<(StatusCode, Json<MarketDataResponse>), (StatusCode, Json<ErrorResponse>)> {
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("[MARKET] 📊 XLM PRICE HISTORY REQUEST");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Get last 30 data points with 15-minute resolution (900000ms)
    debug!("[MARKET] Fetching 30 data points from Horizon...");

    match horizon.get_price_history(900000, 30).await {
        Ok(history) => {
            let prices: Vec<PriceData> = history
                .into_iter()
                .map(|(timestamp, price)| PriceData {
                    timestamp: timestamp / 1000, // Convert ms to seconds
                    price,
                })
                .collect();

            info!("[MARKET] ✅ Fetched {} price points", prices.len());
            debug!("[MARKET] Latest price: ${:.4}", prices.last().map(|p| p.price).unwrap_or(0.0));
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            Ok((
                StatusCode::OK,
                Json(MarketDataResponse {
                    asset: "XLM".to_string(),
                    prices,
                }),
            ))
        }
        Err(e) => {
            error!("[MARKET] ❌ Failed to fetch price history: {}", e);
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to fetch price data: {}", e),
                }),
            ))
        }
    }
}

/// Get current XLM price
pub async fn get_xlm_current_price(
    State(horizon): State<Arc<HorizonClient>>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorResponse>)> {
    info!("[MARKET] 💰 Current XLM price request");

    match horizon.get_current_xlm_price().await {
        Ok(price) => {
            debug!("[MARKET] Current price: ${:.4}", price);
            Ok((
                StatusCode::OK,
                Json(serde_json::json!({
                    "asset": "XLM",
                    "price": price,
                    "timestamp": chrono::Utc::now().timestamp()
                })),
            ))
        }
        Err(e) => {
            error!("[MARKET] Failed to fetch current price: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to fetch price: {}", e),
                }),
            ))
        }
    }
}

/// Get XLM orderbook
pub async fn get_xlm_orderbook(
    State(horizon): State<Arc<HorizonClient>>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorResponse>)> {
    info!("[MARKET] 📖 XLM orderbook request");

    match horizon.get_xlm_orderbook().await {
        Ok(orderbook) => {
            debug!(
                "[MARKET] Orderbook: {} bids, {} asks",
                orderbook.bids.len(),
                orderbook.asks.len()
            );
            Ok((StatusCode::OK, Json(serde_json::to_value(orderbook).unwrap())))
        }
        Err(e) => {
            error!("[MARKET] Failed to fetch orderbook: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to fetch orderbook: {}", e),
                }),
            ))
        }
    }
}
