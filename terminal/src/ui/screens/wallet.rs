//! # Wallet Screen
//!
//! Display wallet address and token balances using egui widgets.

use egui;
use crate::app::AppState;
use crate::ui::theme::Theme;
use crate::ui::widgets::icons::{Icons, material, size};

/// Render wallet screen
pub fn render(ui: &mut egui::Ui, state: &AppState, app: &mut impl crate::app::AppLike) {
    let theme = Theme::default();

    if let Some(wallet) = &state.wallet {
        render_wallet_info(ui, wallet, app, &theme);
    } else {
        render_no_wallet(ui, app, &theme);
    }
}

/// Render wallet information
fn render_wallet_info(
    ui: &mut egui::Ui,
    wallet: &crate::app::WalletState,
    app: &mut impl crate::app::AppLike,
    theme: &Theme,
) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(Icons::icon_success(material::WALLET, size::MEDIUM));
            ui.heading("Connected Wallet");
        });
        ui.add_space(10.0);

        // Wallet address
        ui.horizontal(|ui| {
            ui.label(Icons::icon_dim(material::TOKEN, size::SMALL));
            ui.label("Address:");
            ui.monospace(&wallet.address);
        });
        ui.add_space(10.0);

        // SOL balance
        ui.horizontal(|ui| {
            ui.label(Icons::icon_red(material::TOKEN, size::SMALL));
            ui.label("SOL Balance:");
            ui.colored_label(theme.selected, format!("{:.6}", wallet.sol_balance));
        });
        ui.add_space(10.0);

        ui.separator();
        ui.add_space(10.0);

        // Token balances table
        ui.horizontal(|ui| {
            ui.label(Icons::icon_red(material::TOKEN, size::MEDIUM));
            ui.heading("Token Balances");
        });
        ui.add_space(5.0);

        // Enhanced table with scroll area
        use crate::ui::widgets::tables;
        let config = tables::TableConfig {
            num_columns: 4,
            spacing: [10.0, 5.0],
            striped: true,
            scrollable: true,
        };

        tables::render_table(
            ui,
            "token_balances",
            config,
            &["Token", "Amount", "USD Value", "Value"],
            theme,
            |ui| {
                // Data rows
                for balance in &wallet.token_balances {
                    ui.label(&balance.symbol);
                    ui.monospace(format!("{:.6}", balance.amount));
                    ui.colored_label(theme.success, format!("${:.2}", balance.usd_value));
                    let value_icon = if balance.usd_value > 0.0 {
                        material::ARROW_UP
                    } else {
                        ""
                    };
                    if !value_icon.is_empty() {
                        ui.label(Icons::icon_success(value_icon, size::SMALL));
                    } else {
                        ui.label("");
                    }
                    ui.end_row();
                }
            },
        );

        ui.add_space(10.0);

        // Disconnect button with icon
        if ui.add(egui::Button::new(format!("{} Disconnect Wallet", material::CLOSE)).fill(theme.error)).clicked() {
            app.handle_wallet_disconnect_click();
        }
    });
}

/// Render no wallet connected message
fn render_no_wallet(ui: &mut egui::Ui, app: &mut impl crate::app::AppLike, theme: &Theme) {
    use crate::ui::widgets::{layouts, forms};
    
    layouts::render_centered(ui, |ui| {
        ui.add_space(20.0);
        ui.label(Icons::icon_error(material::WALLET, size::XLARGE));
        ui.add_space(10.0);
        ui.colored_label(theme.error, "No Wallet Connected");
        ui.add_space(10.0);
        forms::render_hint(ui, "Connect or generate a Solana wallet to get started", theme);
        ui.add_space(20.0);

        ui.horizontal(|ui| {
            if forms::render_button(ui, "Connect Wallet", Some(material::WALLET), theme, Some(theme.selected), None).clicked() {
                app.handle_wallet_connect_click();
            }

            if forms::render_button(ui, "Generate New Wallet", Some(material::SETTINGS), theme, None, None).clicked() {
                app.handle_wallet_generate_click();
            }
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(Icons::icon_info(material::INFO, size::SMALL));
            forms::render_hint(ui, "Tip: Create a wallet with: solana-keygen new", theme);
        });
    });
}
