//! # Icons Helper Module
//!
//! Provides Material Design and Phosphor icons integration for the Xterminal-style terminal.
//! Icons are used throughout the UI for enhanced visual communication.

use egui::{RichText, Color32};
use crate::ui::theme::XterminalColors;

/// Icon size constants
pub mod size {
    pub const SMALL: f32 = 16.0;
    pub const MEDIUM: f32 = 24.0;
    pub const LARGE: f32 = 32.0;
    pub const XLARGE: f32 = 48.0;
}

/// Material Design Icons
pub mod material {
    // Note: egui_material_icons provides icon constants
    // Common icons used in the terminal
    
    /// Terminal/Console icon
    pub const TERMINAL: &str = "\u{e8af}"; // terminal
    /// Wallet icon
    pub const WALLET: &str = "\u{e227}"; // account_balance_wallet
    /// Swap/Exchange icon
    pub const SWAP: &str = "\u{e8d4}"; // swap_horiz
    /// Chart/Graph icon
    pub const CHART: &str = "\u{e24b}"; // show_chart
    /// Settings icon
    pub const SETTINGS: &str = "\u{e8b8}"; // settings
    /// Close icon
    pub const CLOSE: &str = "\u{e5cd}"; // close
    /// Check/Success icon
    pub const CHECK: &str = "\u{e5ca}"; // check
    /// Error icon
    pub const ERROR: &str = "\u{e000}"; // error
    /// Warning icon
    pub const WARNING: &str = "\u{e002}"; // warning
    /// Info icon
    pub const INFO: &str = "\u{e88e}"; // info
    /// Refresh icon
    pub const REFRESH: &str = "\u{e5d5}"; // refresh
    /// Search icon
    pub const SEARCH: &str = "\u{e8b6}"; // search
    /// Arrow up icon
    pub const ARROW_UP: &str = "\u{e5ce}"; // arrow_upward
    /// Arrow down icon
    pub const ARROW_DOWN: &str = "\u{e5db}"; // arrow_downward
    /// Arrow right icon
    pub const ARROW_RIGHT: &str = "\u{e5c8}"; // arrow_forward
    /// History icon
    pub const HISTORY: &str = "\u{e889}"; // history
    /// Token/Coin icon
    pub const TOKEN: &str = "\u{e227}"; // account_balance
    /// Network/Connection icon
    pub const NETWORK: &str = "\u{e1be}"; // network_check
    /// Lock icon
    pub const LOCK: &str = "\u{e897}"; // lock
    /// Unlock icon
    pub const UNLOCK: &str = "\u{e898}"; // lock_open
    /// Send icon
    pub const SEND: &str = "\u{e163}"; // send
    /// Receive icon
    pub const RECEIVE: &str = "\u{e2c4}"; // call_received
    /// Message/Chat icon
    pub const MESSAGE: &str = "\u{e0c9}"; // message
    /// Save icon
    pub const SAVE: &str = "\u{e161}"; // save
    /// Palette/Color icon
    pub const PALETTE: &str = "\u{e40a}"; // palette
}

/// Icon helper functions for rendering icons with Bloomberg theme
pub struct Icons;

impl Icons {
    /// Render an icon with default styling
    pub fn icon(icon: &str, size: f32) -> RichText {
        RichText::new(icon).size(size)
    }

    /// Render an icon with Xterminal red accent color
    pub fn icon_red(icon: &str, size: f32) -> RichText {
        let colors = XterminalColors::default();
        RichText::new(icon).size(size).color(colors.red_primary)
    }

    /// Render an icon with success green color
    pub fn icon_success(icon: &str, size: f32) -> RichText {
        let colors = XterminalColors::default();
        RichText::new(icon).size(size).color(colors.green_success)
    }

    /// Render an icon with error red color
    pub fn icon_error(icon: &str, size: f32) -> RichText {
        let colors = XterminalColors::default();
        RichText::new(icon).size(size).color(colors.red_error)
    }

    /// Render an icon with warning yellow color
    pub fn icon_warning(icon: &str, size: f32) -> RichText {
        let colors = XterminalColors::default();
        RichText::new(icon).size(size).color(colors.yellow_warning)
    }

    /// Render an icon with info blue color
    pub fn icon_info(icon: &str, size: f32) -> RichText {
        let colors = XterminalColors::default();
        RichText::new(icon).size(size).color(colors.blue_info)
    }

    /// Render an icon with custom color
    pub fn icon_color(icon: &str, size: f32, color: Color32) -> RichText {
        RichText::new(icon).size(size).color(color)
    }

    /// Render an icon with dim/secondary color
    pub fn icon_dim(icon: &str, size: f32) -> RichText {
        let colors = XterminalColors::default();
        RichText::new(icon).size(size).color(colors.gray_secondary)
    }
}

/// Helper trait for rendering icons in UI
pub trait IconRenderer {
    /// Render an icon with default styling
    fn icon(&mut self, icon: &str, size: f32);
    
    /// Render an icon with Xterminal red accent
    fn icon_red(&mut self, icon: &str, size: f32);
    
    /// Render an icon with success green
    fn icon_success(&mut self, icon: &str, size: f32);
    
    /// Render an icon with error red
    fn icon_error(&mut self, icon: &str, size: f32);
    
    /// Render an icon with warning yellow
    fn icon_warning(&mut self, icon: &str, size: f32);
    
    /// Render an icon with info blue
    fn icon_info(&mut self, icon: &str, size: f32);
}

impl IconRenderer for egui::Ui {
    fn icon(&mut self, icon: &str, size: f32) {
        self.label(Icons::icon(icon, size));
    }

    fn icon_red(&mut self, icon: &str, size: f32) {
        self.label(Icons::icon_red(icon, size));
    }

    fn icon_success(&mut self, icon: &str, size: f32) {
        self.label(Icons::icon_success(icon, size));
    }

    fn icon_error(&mut self, icon: &str, size: f32) {
        self.label(Icons::icon_error(icon, size));
    }

    fn icon_warning(&mut self, icon: &str, size: f32) {
        self.label(Icons::icon_warning(icon, size));
    }

    fn icon_info(&mut self, icon: &str, size: f32) {
        self.label(Icons::icon_info(icon, size));
    }
}

/// Initialize Material Design icons in the egui context
/// This should be called during application initialization
pub fn initialize_material_icons(_ctx: &egui::Context) {
    // egui_material_icons provides font initialization
    // For now, we'll use Unicode/emoji fallbacks until the font is properly loaded
    // In a real implementation, you would call:
    // egui_material_icons::initialize(ctx);
    // Note: Material icons are currently using Unicode codepoints as fallback
}

