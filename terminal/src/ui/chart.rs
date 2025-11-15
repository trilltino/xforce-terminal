//! # Chart Module
//!
//! Chart rendering using egui_plot for candlestick and line charts.

use egui;
use egui_plot::{Plot, PlotPoints, Line};
use std::collections::VecDeque;

/// OHLCV candlestick data point
#[derive(Debug, Clone)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Chart timeframe
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timeframe {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    FourHours,
    OneDay,
}

impl Timeframe {
    pub fn label(&self) -> &'static str {
        match self {
            Timeframe::OneMinute => "1m",
            Timeframe::FiveMinutes => "5m",
            Timeframe::FifteenMinutes => "15m",
            Timeframe::OneHour => "1h",
            Timeframe::FourHours => "4h",
            Timeframe::OneDay => "1d",
        }
    }

    pub fn seconds(&self) -> i64 {
        match self {
            Timeframe::OneMinute => 60,
            Timeframe::FiveMinutes => 300,
            Timeframe::FifteenMinutes => 900,
            Timeframe::OneHour => 3600,
            Timeframe::FourHours => 14400,
            Timeframe::OneDay => 86400,
        }
    }
}

/// Chart state and data
#[derive(Debug, Clone)]
pub struct ChartData {
    pub candles: VecDeque<Candle>,
    pub timeframe: Timeframe,
    pub symbol: String,
    pub max_candles: usize,
}

impl ChartData {
    pub fn new(symbol: String) -> Self {
        Self {
            candles: VecDeque::with_capacity(100),
            timeframe: Timeframe::OneHour,
            symbol,
            max_candles: 100,
        }
    }

    /// Add a new candle
    pub fn add_candle(&mut self, candle: Candle) {
        self.candles.push_back(candle);
        while self.candles.len() > self.max_candles {
            self.candles.pop_front();
        }
    }

    /// Get price range for visible candles
    pub fn price_range(&self) -> (f64, f64) {
        if self.candles.is_empty() {
            return (0.0, 100.0);
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for candle in &self.candles {
            if candle.low < min {
                min = candle.low;
            }
            if candle.high > max {
                max = candle.high;
            }
        }

        // Add 5% padding
        let padding = (max - min) * 0.05;
        (min - padding, max + padding)
    }

    /// Generate demo/random candles for testing
    pub fn generate_demo_data(&mut self, count: usize) {
        use rand::Rng;
        let mut rng = rand::rng();

        let base_price: f64 = 150.0;
        let mut current_price: f64 = base_price;
        let now = chrono::Utc::now().timestamp();

        for i in 0..count {
            let timestamp = now - ((count - i - 1) as i64 * self.timeframe.seconds());

            // Random walk
            let change: f64 = rng.random_range(-5.0..5.0);
            current_price += change;
            current_price = current_price.max(base_price * 0.8).min(base_price * 1.2);

            let open: f64 = current_price;
            let close: f64 = open + rng.random_range(-3.0..3.0);
            let high: f64 = open.max(close) + rng.random_range(0.0..2.0);
            let low: f64 = open.min(close) - rng.random_range(0.0..2.0);
            let volume: f64 = rng.random_range(1000.0..10000.0);

            self.add_candle(Candle {
                timestamp,
                open,
                high,
                low,
                close,
                volume,
            });

            current_price = close;
        }
    }
}

/// Render candlestick chart using egui_plot
pub fn render_chart(ui: &mut egui::Ui, chart_data: &ChartData) {
    if chart_data.candles.is_empty() {
        ui.label("No chart data available");
        return;
    }

    // Prepare data for plotting
    let mut close_points = Vec::new();
    let mut high_points = Vec::new();
    let mut low_points = Vec::new();

    for (idx, candle) in chart_data.candles.iter().enumerate() {
        let x = idx as f64;
        close_points.push([x, candle.close]);
        high_points.push([x, candle.high]);
        low_points.push([x, candle.low]);
    }

    // Create plot
    Plot::new("candlestick_chart")
        .view_aspect(2.0)
        .show(ui, |plot_ui| {
            // Close price line
            if !close_points.is_empty() {
                plot_ui.line(
                    Line::new("Close Price", PlotPoints::from(close_points))
                        .color(egui::Color32::from_rgb(0, 255, 0))
                        .width(2.0)
                );
            }

            // High/Low range (optional, can be shown as error bars or area)
            // For simplicity, just show close price line for now
        });

    // Show latest price info
    if let Some(last_candle) = chart_data.candles.back() {
        ui.horizontal(|ui| {
            ui.label(format!("Current: ${:.2}", last_candle.close));
            let price_change = last_candle.close - last_candle.open;
            let percent_change = (price_change / last_candle.open) * 100.0;
            if price_change >= 0.0 {
                ui.colored_label(egui::Color32::from_rgb(0, 255, 0), format!("▲ {:.2}%", percent_change.abs()));
            } else {
                ui.colored_label(egui::Color32::from_rgb(255, 0, 0), format!("▼ {:.2}%", percent_change.abs()));
            }
        });
    }
}

/// Render candlestick chart from real OHLC data
pub fn render_candlestick_chart(ui: &mut egui::Ui, candles: &[shared::dto::OHLC], _theme: &crate::ui::theme::Theme) {
    use egui_plot::{Plot, PlotPoints, Line};
    use tracing::trace;
    
    if candles.is_empty() {
        ui.label("No chart data available");
        return;
    }
    
    trace!(candle_count = candles.len(), "Rendering candlestick chart");

    // Prepare data for plotting
    let mut close_points = Vec::new();
    
    // Calculate price range for proper scaling
    let mut min_price = f64::MAX;
    let mut max_price = f64::MIN;

    for candle in candles.iter() {
        min_price = min_price.min(candle.low);
        max_price = max_price.max(candle.high);
    }

    // Add padding
    let price_range = max_price - min_price;
    let padding = price_range * 0.1;
    min_price -= padding;
    max_price += padding;

    // Create plot with proper aspect ratio
    Plot::new("candlestick_chart")
        .view_aspect(2.5)
        .include_y(min_price)
        .include_y(max_price)
        .show(ui, |plot_ui| {
            // Draw candlesticks using lines
            for (idx, candle) in candles.iter().enumerate() {
                let x = idx as f64;
                let is_bullish = candle.is_bullish();
                let color = if is_bullish {
                    egui::Color32::from_rgb(0, 200, 0) // Green for bullish
                } else {
                    egui::Color32::from_rgb(200, 0, 0) // Red for bearish
                };
                
                // Draw wick (high-low line)
                plot_ui.line(
                    Line::new("wick", PlotPoints::from(vec![[x, candle.low], [x, candle.high]]))
                        .color(color)
                        .width(1.0)
                );
                
                // Draw body (open-close rectangle) using lines
                let body_top = candle.open.max(candle.close);
                let body_bottom = candle.open.min(candle.close);
                let body_width = 0.4; // Width of candle body
                
                // Draw body as a filled rectangle using multiple lines
                if body_top > body_bottom {
                    // Draw top line
                    plot_ui.line(
                        Line::new("body_top", PlotPoints::from(vec![[x - body_width / 2.0, body_top], [x + body_width / 2.0, body_top]]))
                            .color(color)
                            .width(3.0)
                    );
                    // Draw bottom line
                    plot_ui.line(
                        Line::new("body_bottom", PlotPoints::from(vec![[x - body_width / 2.0, body_bottom], [x + body_width / 2.0, body_bottom]]))
                            .color(color)
                            .width(3.0)
                    );
                    // Draw left edge
                    plot_ui.line(
                        Line::new("body_left", PlotPoints::from(vec![[x - body_width / 2.0, body_bottom], [x - body_width / 2.0, body_top]]))
                            .color(color)
                            .width(2.0)
                    );
                    // Draw right edge
                    plot_ui.line(
                        Line::new("body_right", PlotPoints::from(vec![[x + body_width / 2.0, body_bottom], [x + body_width / 2.0, body_top]]))
                            .color(color)
                            .width(2.0)
                    );
                } else {
                    // Doji (open == close) - draw a horizontal line
                    plot_ui.line(
                        Line::new("doji", PlotPoints::from(vec![[x - body_width / 2.0, candle.close], [x + body_width / 2.0, candle.close]]))
                            .color(color)
                            .width(2.0)
                    );
                }
                
                close_points.push([x, candle.close]);
            }
            
            // Optional: Draw close price line overlay
            if !close_points.is_empty() {
                plot_ui.line(
                    Line::new("Close", PlotPoints::from(close_points))
                        .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100))
                        .width(1.0)
                );
            }
        });
}