//! # SPL Tokens Screen
//!
//! Display and manage SPL token accounts, balances, and associated token accounts.

use egui;
use crate::app::AppState;
use crate::ui::theme::Theme;
use crate::ui::widgets::icons::{Icons, material, size};

/// Render SPL tokens screen
pub fn render(ui: &mut egui::Ui, state: &AppState, app: &mut impl crate::app::AppLike) {
    let theme = Theme::default();

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(Icons::icon_red(material::TOKEN, size::MEDIUM));
            ui.heading("SPL Token Management");
        });
        ui.add_space(10.0);

        // Show wallet connection status
        if let Some(wallet) = &state.wallet {
            render_token_accounts(ui, wallet, state, app, &theme);
        } else {
            render_no_wallet(ui, app, &theme);
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Token utilities section
        render_token_utilities(ui, state, app, &theme);
    });
}

/// Render token accounts for connected wallet
fn render_token_accounts(
    ui: &mut egui::Ui,
    wallet: &crate::app::WalletState,
    state: &AppState,
    _app: &mut impl crate::app::AppLike,
    theme: &Theme,
) {
    ui.horizontal(|ui| {
        ui.label(Icons::icon_dim(material::WALLET, size::SMALL));
        ui.label("Wallet:");
        ui.monospace(&wallet.address);
    });
    ui.add_space(10.0);

    // Token balances table
    ui.horizontal(|ui| {
        ui.label(Icons::icon_red(material::TOKEN, size::MEDIUM));
        ui.heading("Token Accounts");
    });
    ui.add_space(5.0);

    use crate::ui::widgets::tables;
    let config = tables::TableConfig {
        num_columns: 5,
        spacing: [10.0, 5.0],
        striped: true,
        scrollable: true,
    };

    // Check if prices were recently updated for live USD values
    let recently_updated = state.last_price_update_time.elapsed().as_millis() < 500;
    
    // Clone state for closure
    let state_clone = state.clone();
    
    tables::render_table(
        ui,
        "token_accounts",
        config,
        &["Token", "Mint Address", "Balance", "USD Value", "Price", "Actions"],
        theme,
        |ui| {
            // Data rows
            for balance in &wallet.token_balances {
                // Find live price for this token
                let live_price = state_clone.terminal.prices
                    .iter()
                    .find(|p| p.symbol == balance.symbol);
                
                ui.label(&balance.symbol);
                ui.monospace({
                    // Truncate long addresses
                    let mint = &balance.symbol; // Use symbol for now, could add mint field
                    if mint.len() > 20 {
                        format!("{}...{}", &mint[..10], &mint[mint.len()-10..])
                    } else {
                        mint.clone()
                    }
                });
                ui.monospace(format!("{:.6}", balance.amount));
                
                // USD value - update with live price if available
                let usd_value = if let Some(price) = live_price {
                    balance.amount * price.price
                } else {
                    balance.usd_value
                };
                let usd_color = if recently_updated && live_price.is_some() {
                    theme.selected
                } else {
                    theme.success
                };
                ui.colored_label(usd_color, format!("${:.2}", usd_value));
                
                // Live price display
                if let Some(price) = live_price {
                    let price_color = if recently_updated {
                        theme.selected
                    } else {
                        theme.normal
                    };
                    ui.colored_label(price_color, format!("${:.4}", price.price));
                } else {
                    ui.colored_label(theme.dim, "-");
                }
                
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.small_button("View").clicked() {
                        // TODO: Show token details
                    }
                });
                ui.end_row();
            }
            
            // Show message if no tokens
            if wallet.token_balances.is_empty() {
                ui.label("");
                ui.label("");
                ui.colored_label(theme.error, "No token accounts found");
                ui.label("");
                ui.label("");
                ui.end_row();
            }
        },
    );
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
        forms::render_hint(ui, "Connect a wallet to view and manage SPL tokens", theme);
        ui.add_space(20.0);

        if forms::render_button(ui, "Go to Wallet", Some(material::WALLET), theme, Some(theme.selected), None).clicked() {
            app.handle_screen_change(crate::app::Screen::Wallet);
        }
    });
}

/// Render token utilities section
fn render_token_utilities(
    ui: &mut egui::Ui,
    state: &AppState,
    _app: &mut impl crate::app::AppLike,
    _theme: &Theme,
) {
    ui.horizontal(|ui| {
        ui.label(Icons::icon_dim(material::SETTINGS, size::MEDIUM));
        ui.heading("Token Utilities");
    });
    ui.add_space(10.0);

    ui.group(|ui| {
        ui.vertical(|ui| {
            // Calculate ATA address
            ui.horizontal(|ui| {
                ui.label("Calculate Associated Token Account (ATA):");
            });
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Wallet:");
                let mut wallet_input = if let Some(wallet) = &state.wallet {
                    wallet.address.clone()
                } else {
                    String::new()
                };
                ui.text_edit_singleline(&mut wallet_input);
            });
            
            ui.horizontal(|ui| {
                ui.label("Mint:");
                let mut mint_input = String::new();
                ui.text_edit_singleline(&mut mint_input);
                
                if ui.button("Calculate ATA").clicked() && !mint_input.is_empty() {
                    // TODO: Calculate and display ATA address
                }
            });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Token info section
            ui.horizontal(|ui| {
                ui.label(Icons::icon_info(material::INFO, size::SMALL));
                ui.label("About SPL Tokens:");
            });
            ui.add_space(5.0);
            ui.label("• SPL Tokens are the standard for fungible tokens on Solana");
            ui.label("• Each token has a unique mint address");
            ui.label("• Associated Token Accounts (ATAs) are canonical token accounts");
            ui.label("• Use ATAs for standard token operations");
        });
    });
}

