//! # Notifications Widget
//!
//! Toast notification system using egui-notify for trade confirmations and status updates.
//! Provides Xterminal-style notifications with red accent colors.

use egui_notify::Toasts;

/// Notification manager for the application
pub struct NotificationManager {
    /// Toast notification system
    pub toasts: Toasts,
}

impl Default for NotificationManager {
    fn default() -> Self {
        let toasts = Toasts::default();
        
        Self { toasts }
    }
}

impl NotificationManager {
    /// Create a new notification manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Show a success notification (green for successful trades)
    pub fn success(&mut self, message: String) {
        self.toasts.success(message);
    }

    /// Show an error notification (red for failed trades)
    pub fn error(&mut self, message: String) {
        self.toasts.error(message);
    }

    /// Show a warning notification (yellow for warnings)
    pub fn warning(&mut self, message: String) {
        self.toasts.warning(message);
    }

    /// Show an info notification (blue for informational messages)
    pub fn info(&mut self, message: String) {
        self.toasts.info(message);
    }

    /// Show a trade confirmation notification
    pub fn trade_confirmed(&mut self, input_token: &str, output_token: &str, amount: f64, signature: &str) {
        let message = format!(
            "Trade Confirmed\n{} {} → {} {}\nSignature: {}",
            amount, input_token, amount, output_token, signature
        );
        self.success(message);
    }

    /// Show a trade execution started notification
    pub fn trade_executing(&mut self, input_token: &str, output_token: &str, amount: f64) {
        let message = format!(
            "Executing Trade\n{} {} → {}",
            amount, input_token, output_token
        );
        self.info(message);
    }

    /// Show a trade failure notification
    pub fn trade_failed(&mut self, reason: &str) {
        let message = format!("Trade Failed\n{}", reason);
        self.error(message);
    }

    /// Render notifications in the UI context
    pub fn show(&mut self, ctx: &egui::Context) {
        // In egui-notify 0.21.0, show expects &egui::Context which is compatible
        // The type alias should work, but we need to ensure proper type matching
        let ctx_ref: &egui::Context = ctx;
        self.toasts.show(ctx_ref);
    }
}
