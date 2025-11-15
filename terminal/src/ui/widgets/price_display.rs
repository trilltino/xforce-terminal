//! # Price Display Widget
//!
//! Price display with animations and flash effects for live updates.

use egui;
use crate::ui::theme::Theme;

/// Price change direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceDirection {
    Up,
    Down,
    Neutral,
}

impl PriceDirection {
    pub fn from_change(old_price: f64, new_price: f64) -> Self {
        if (new_price - old_price).abs() < 0.0001 {
            PriceDirection::Neutral
        } else if new_price > old_price {
            PriceDirection::Up
        } else {
            PriceDirection::Down
        }
    }
    
    pub fn arrow(&self) -> &'static str {
        match self {
            PriceDirection::Up => "↑",
            PriceDirection::Down => "↓",
            PriceDirection::Neutral => "→",
        }
    }
}

/// Render price with flash effect
pub fn render_price_with_flash(
    ui: &mut egui::Ui,
    price: f64,
    previous_price: Option<f64>,
    recently_updated: bool,
    theme: &Theme,
) {
    // Determine flash color based on direction
    let flash_color = if let Some(prev) = previous_price {
        let direction = PriceDirection::from_change(prev, price);
        if recently_updated {
            match direction {
                PriceDirection::Up => theme.success,
                PriceDirection::Down => theme.error,
                PriceDirection::Neutral => theme.selected,
            }
        } else {
            theme.normal
        }
    } else {
        if recently_updated {
            theme.selected
        } else {
            theme.normal
        }
    };
    
    // Animate flash effect
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    // Flash lasts for 500ms
    let flash_intensity = if recently_updated && now % 1000 < 500 {
        // Bright flash
        1.0
    } else if recently_updated {
        // Fading flash
        0.5
    } else {
        0.0
    };
    
    let final_color = if flash_intensity > 0.0 {
        // Blend with flash color
        flash_color.linear_multiply(0.7 + flash_intensity * 0.3)
    } else {
        flash_color
    };
    
    ui.colored_label(final_color, format!("${:.4}", price));
    
    // Show direction arrow if recently updated
    if recently_updated && previous_price.is_some() {
        let direction = PriceDirection::from_change(
            previous_price.unwrap(),
            price,
        );
        let arrow_color = match direction {
            PriceDirection::Up => theme.success,
            PriceDirection::Down => theme.error,
            PriceDirection::Neutral => theme.normal,
        };
        ui.colored_label(arrow_color, direction.arrow());
    }
}

/// Render price change with color coding
pub fn render_price_change(
    ui: &mut egui::Ui,
    change: f64,
    change_percent: f64,
    theme: &Theme,
) {
    let (text, color) = theme.format_price_change(change_percent);
    let direction = if change >= 0.0 {
        PriceDirection::Up
    } else {
        PriceDirection::Down
    };
    
    ui.horizontal(|ui| {
        ui.colored_label(color, direction.arrow());
        ui.colored_label(color, text);
        ui.label(format!("({:+.4})", change));
    });
}

