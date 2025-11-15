//! # GUI Theme
//!
//! Xterminal-style dark theme with red, white, and black colors for egui.
//! Professional trading terminal aesthetic with high contrast and sharp edges.

use egui::{Color32, Visuals, Stroke, Context};
use egui::Theme as EguiTheme;
use serde::{Serialize, Deserialize};
use std::path::Path;

/// Serializable theme configuration for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Pure black background
    pub background: [u8; 3],
    /// Bright white text
    pub text: [u8; 3],
    /// Bloomberg red (primary accent)
    pub red_primary: [u8; 3],
    /// Dark red (secondary accent)
    pub red_dark: [u8; 3],
    /// Red highlight for hover/selection
    pub red_highlight: [u8; 3],
    /// Dark gray borders
    pub border_dark: [u8; 3],
    /// Dark gray with red tint
    pub border_red_tint: [u8; 3],
    /// Success green (gains)
    pub green_success: [u8; 3],
    /// Error red (losses)
    pub red_error: [u8; 3],
    /// Warning yellow/orange
    pub yellow_warning: [u8; 3],
    /// Info blue
    pub blue_info: [u8; 3],
    /// Dark gray for inactive elements
    pub gray_inactive: [u8; 3],
    /// Medium gray for secondary text
    pub gray_secondary: [u8; 3],
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig {
            background: [0, 0, 0],
            text: [255, 255, 255],
            red_primary: [204, 0, 0],
            red_dark: [153, 0, 0],
            red_highlight: [204, 0, 0],
            border_dark: [51, 51, 51],
            border_red_tint: [68, 0, 0],
            green_success: [0, 255, 0],
            red_error: [255, 0, 0],
            yellow_warning: [255, 170, 0],
            blue_info: [100, 150, 255],
            gray_inactive: [26, 26, 26],
            gray_secondary: [150, 150, 150],
        }
    }
}

impl ThemeConfig {
    /// Load theme configuration from a JSON file
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let content = std::fs::read_to_string(path)?;
        let config: ThemeConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save theme configuration to a JSON file
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Convert ThemeConfig to XterminalColors
    pub fn to_xterminal_colors(&self) -> XterminalColors {
        XterminalColors {
            background: Color32::from_rgb(self.background[0], self.background[1], self.background[2]),
            text: Color32::from_rgb(self.text[0], self.text[1], self.text[2]),
            red_primary: Color32::from_rgb(self.red_primary[0], self.red_primary[1], self.red_primary[2]),
            red_dark: Color32::from_rgb(self.red_dark[0], self.red_dark[1], self.red_dark[2]),
            red_highlight: Color32::from_rgb(self.red_highlight[0], self.red_highlight[1], self.red_highlight[2]),
            border_dark: Color32::from_rgb(self.border_dark[0], self.border_dark[1], self.border_dark[2]),
            border_red_tint: Color32::from_rgb(self.border_red_tint[0], self.border_red_tint[1], self.border_red_tint[2]),
            green_success: Color32::from_rgb(self.green_success[0], self.green_success[1], self.green_success[2]),
            red_error: Color32::from_rgb(self.red_error[0], self.red_error[1], self.red_error[2]),
            yellow_warning: Color32::from_rgb(self.yellow_warning[0], self.yellow_warning[1], self.yellow_warning[2]),
            blue_info: Color32::from_rgb(self.blue_info[0], self.blue_info[1], self.blue_info[2]),
            gray_inactive: Color32::from_rgb(self.gray_inactive[0], self.gray_inactive[1], self.gray_inactive[2]),
            gray_secondary: Color32::from_rgb(self.gray_secondary[0], self.gray_secondary[1], self.gray_secondary[2]),
        }
    }

    /// Convert XterminalColors to ThemeConfig
    pub fn from_xterminal_colors(colors: &XterminalColors) -> Self {
        ThemeConfig {
            background: [colors.background.r(), colors.background.g(), colors.background.b()],
            text: [colors.text.r(), colors.text.g(), colors.text.b()],
            red_primary: [colors.red_primary.r(), colors.red_primary.g(), colors.red_primary.b()],
            red_dark: [colors.red_dark.r(), colors.red_dark.g(), colors.red_dark.b()],
            red_highlight: [colors.red_highlight.r(), colors.red_highlight.g(), colors.red_highlight.b()],
            border_dark: [colors.border_dark.r(), colors.border_dark.g(), colors.border_dark.b()],
            border_red_tint: [colors.border_red_tint.r(), colors.border_red_tint.g(), colors.border_red_tint.b()],
            green_success: [colors.green_success.r(), colors.green_success.g(), colors.green_success.b()],
            red_error: [colors.red_error.r(), colors.red_error.g(), colors.red_error.b()],
            yellow_warning: [colors.yellow_warning.r(), colors.yellow_warning.g(), colors.yellow_warning.b()],
            blue_info: [colors.blue_info.r(), colors.blue_info.g(), colors.blue_info.b()],
            gray_inactive: [colors.gray_inactive.r(), colors.gray_inactive.g(), colors.gray_inactive.b()],
            gray_secondary: [colors.gray_secondary.r(), colors.gray_secondary.g(), colors.gray_secondary.b()],
        }
    }
}

/// Xterminal color palette
#[derive(Clone)]
pub struct XterminalColors {
    /// Pure black background
    pub background: Color32,
    /// Bright white text
    pub text: Color32,
    /// Bloomberg red (primary accent)
    pub red_primary: Color32,
    /// Dark red (secondary accent)
    pub red_dark: Color32,
    /// Red highlight for hover/selection
    pub red_highlight: Color32,
    /// Dark gray borders
    pub border_dark: Color32,
    /// Dark gray with red tint
    pub border_red_tint: Color32,
    /// Success green (gains)
    pub green_success: Color32,
    /// Error red (losses)
    pub red_error: Color32,
    /// Warning yellow/orange
    pub yellow_warning: Color32,
    /// Info blue
    pub blue_info: Color32,
    /// Dark gray for inactive elements
    pub gray_inactive: Color32,
    /// Medium gray for secondary text
    pub gray_secondary: Color32,
}

impl Default for XterminalColors {
    fn default() -> Self {
        XterminalColors {
            // Primary colors
            background: Color32::from_rgb(0, 0, 0),           // #000000 - Pure black
            text: Color32::from_rgb(255, 255, 255),           // #FFFFFF - Bright white
            red_primary: Color32::from_rgb(204, 0, 0),        // #CC0000 - Xterminal red
            red_dark: Color32::from_rgb(153, 0, 0),           // #990000 - Dark red
            red_highlight: Color32::from_rgb(204, 0, 0),      // #CC0000 - Red highlight
            
            // Borders
            border_dark: Color32::from_rgb(51, 51, 51),       // #333333 - Dark gray
            border_red_tint: Color32::from_rgb(68, 0, 0),     // #440000 - Dark gray with red tint
            
            // Status colors
            green_success: Color32::from_rgb(0, 255, 0),      // #00FF00 - Green for gains
            red_error: Color32::from_rgb(255, 0, 0),          // #FF0000 - Red for losses
            yellow_warning: Color32::from_rgb(255, 170, 0),   // #FFAA00 - Yellow/orange
            blue_info: Color32::from_rgb(100, 150, 255),      // #6496FF - Blue
            
            // Grays
            gray_inactive: Color32::from_rgb(26, 26, 26),     // #1A1A1A - Dark gray for inactive
            gray_secondary: Color32::from_rgb(150, 150, 150), // #969696 - Medium gray for secondary text
        }
    }
}

/// Application theme with Xterminal-inspired colors
pub struct Theme {
    /// Color palette
    pub colors: XterminalColors,
    /// Normal text color
    pub normal: Color32,
    /// Selected/highlighted items (Xterminal red)
    pub selected: Color32,
    /// Border color
    pub border: Color32,
    /// Dimmed/secondary text
    pub dim: Color32,
    /// Success/positive (green for gains)
    pub success: Color32,
    /// Error/negative (red for losses)
    pub error: Color32,
    /// Warning/attention (yellow)
    pub warning: Color32,
    /// Information (blue)
    pub info: Color32,
    /// Price up (green)
    pub price_up: Color32,
    /// Price down (red)
    pub price_down: Color32,
    /// Background color
    pub background: Color32,
}

impl Default for Theme {
    fn default() -> Self {
        let colors = XterminalColors::default();
        Theme {
            colors: colors.clone(),
            normal: colors.text,
            selected: colors.red_primary,
            border: colors.border_dark,
            dim: colors.gray_secondary,
            success: colors.green_success,
            error: colors.red_error,
            warning: colors.yellow_warning,
            info: colors.blue_info,
            price_up: colors.green_success,
            price_down: colors.red_error,
            background: colors.background,
        }
    }
}

impl Theme {
    /// Get color for price change percentage
    pub fn price_change_color(&self, change: f64) -> Color32 {
        if change > 0.0 {
            self.price_up
        } else if change < 0.0 {
            self.price_down
        } else {
            self.dim
    }
    }

    /// Format price change with color
    pub fn format_price_change(&self, change: f64) -> (String, Color32) {
        let text = if change >= 0.0 {
            format!("+{:.2}%", change)
        } else {
            format!("{:.2}%", change)
        };
        (text, self.price_change_color(change))
    }

    /// Create Xterminal-style egui Visuals from ThemeConfig
    pub fn xterminal_visuals_from_config(config: &ThemeConfig) -> Visuals {
        let colors = config.to_xterminal_colors();
        let mut visuals = Visuals::dark();
        
        // Override text color
        visuals.override_text_color = Some(colors.text);
        
        // Background colors - Complete black
        visuals.faint_bg_color = Color32::from_rgb(0, 0, 0);
        visuals.extreme_bg_color = Color32::from_rgb(0, 0, 0);

        // Panel colors - Complete black
        visuals.panel_fill = Color32::from_rgb(0, 0, 0);
        visuals.window_fill = Color32::from_rgb(0, 0, 0);
        visuals.window_stroke = Stroke::new(1.0, colors.border_dark);
        
        // Non-interactive widgets
        visuals.widgets.noninteractive.bg_fill = colors.gray_inactive;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors.border_dark);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.text);
        
        // Inactive widgets
        visuals.widgets.inactive.bg_fill = colors.gray_inactive;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors.border_dark);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(30, 30, 30);
        
        // Hovered widgets - Red highlight
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(51, 0, 0); // Dark red background
        visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, colors.red_primary);
        visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(40, 0, 0);
        
        // Active/pressed widgets - Bright red
        visuals.widgets.active.bg_fill = Color32::from_rgb(102, 0, 0); // Medium red
        visuals.widgets.active.bg_stroke = Stroke::new(2.0, colors.red_primary);
        visuals.widgets.active.weak_bg_fill = Color32::from_rgb(76, 0, 0);
        
        // Open (expanded) state
        visuals.widgets.open.bg_fill = Color32::from_rgb(51, 0, 0);
        visuals.widgets.open.bg_stroke = Stroke::new(2.0, colors.red_primary);
        
        // Selection highlight - Red with transparency
        visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(204, 0, 0, 76); // 30% opacity red
        visuals.selection.stroke = Stroke::new(2.0, colors.red_primary);
        
        // Hyperlinks
        visuals.hyperlink_color = colors.blue_info;
        
        // Window and popup styling - Sharp corners for terminal aesthetic
        // Note: window_rounding and menu_rounding fields removed in egui 0.33+
        // Corner radius is now controlled via window styling
        
        // Resize handle
        visuals.resize_corner_size = 8.0;
        
        // Clip rect margin
        visuals.clip_rect_margin = 2.0;
        
        // Slider
        visuals.slider_trailing_fill = true;
        
        visuals
    }

    /// Create Xterminal-style egui Visuals (uses default theme)
    pub fn xterminal_visuals() -> Visuals {
        Self::xterminal_visuals_from_config(&ThemeConfig::default())
    }

    /// Apply custom theme to an egui context
    pub fn apply_custom_theme(ctx: &Context, config: &ThemeConfig) {
        // Verify that required text styles exist (defensive check)
        let style_check = ctx.style();
        let required_styles = [
            egui::TextStyle::Heading,
            egui::TextStyle::Body,
            egui::TextStyle::Button,
            egui::TextStyle::Small,
            egui::TextStyle::Monospace,
        ];
        
        let missing_styles: Vec<_> = required_styles
            .iter()
            .filter(|style| !style_check.text_styles.contains_key(style))
            .collect();
        
        if !missing_styles.is_empty() {
            tracing::warn!(
                "Missing text styles before theme application: {:?}. \
                Font initialization may not have been called. \
                Application may panic when rendering text.",
                missing_styles
            );
        }
        
        let visuals = Self::xterminal_visuals_from_config(config);
        
        // Use style_mut_of instead of set_visuals to avoid panic in egui 0.33
        ctx.style_mut_of(EguiTheme::Dark, |style| {
            // Apply visuals
            style.visuals = visuals.clone();
            
            // Apply spacing modifications for terminal-like appearance
            style.spacing.item_spacing = egui::Vec2::new(4.0, 2.0);
            style.spacing.window_margin = egui::Margin::same(4);
            style.spacing.button_padding = egui::Vec2::new(8.0, 4.0);
            style.spacing.indent = 12.0;
            style.spacing.interact_size = egui::Vec2::new(32.0, 32.0);
            style.spacing.menu_margin = egui::Margin::same(2);
            style.spacing.tooltip_width = 400.0;
        });
        
        // Also apply to light theme (in case user switches)
        ctx.style_mut_of(EguiTheme::Light, |style| {
            style.visuals = visuals;
            style.spacing.item_spacing = egui::Vec2::new(4.0, 2.0);
            style.spacing.window_margin = egui::Margin::same(4);
            style.spacing.button_padding = egui::Vec2::new(8.0, 4.0);
            style.spacing.indent = 12.0;
            style.spacing.interact_size = egui::Vec2::new(32.0, 32.0);
            style.spacing.menu_margin = egui::Margin::same(2);
            style.spacing.tooltip_width = 400.0;
        });
        
        tracing::debug!("Applied custom theme visuals and spacing using style_mut_of API");
    }

    /// Apply Xterminal theme to an egui context (uses default theme)
    /// Uses style_mut_of API which is the safe way to modify styles in egui 0.33
    /// 
    /// # Panic Safety
    /// 
    /// This function assumes that fonts have been initialized (via `FontConfig::setup_terminal_fonts`)
    /// before calling this function. Font initialization registers all required TextStyles.
    /// If fonts are not initialized, TextStyle resolution may panic when rendering text.
    pub fn apply_xterminal_theme(ctx: &Context) {
        Self::apply_custom_theme(ctx, &ThemeConfig::default());
    }

    /// Get Xterminal color palette
    pub fn xterminal_colors() -> XterminalColors {
        XterminalColors::default()
    }
    
    /// Apply Xterminal theme (alias for compatibility)
    #[deprecated(note = "Use apply_xterminal_theme instead")]
    pub fn apply_bloomberg_theme(ctx: &Context) {
        Self::apply_xterminal_theme(ctx);
    }
}
