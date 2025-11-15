//! # GUI Rendering Framework
//!
//! This module orchestrates the complete UI rendering pipeline using **egui widgets**.
//! It implements a layout system with theme support and visual effects.

pub mod chart;
pub mod cube;
pub mod debug_overlay;
pub mod effects;
pub mod fonts;
pub mod screens;
pub mod theme;
pub mod widgets;

use egui;
use crate::app::{App, AppState, Screen};

/// Main render function - called every frame by egui
pub fn render(ctx: &egui::Context, app: &mut App, _notifications: &mut crate::ui::widgets::notifications::NotificationManager, cube: &mut crate::ui::cube::RotatingCube, _frame: &mut eframe::Frame) {
    // Read state for rendering
    let state = {
        match app.state.try_read() {
            Some(state_guard) => state_guard.clone(),
            None => {
                // Lock is held by another task, skip this frame
                return;
            }
        }
    }; // Lock released here - rendering happens without holding lock

    // Central panel - Main content area
    egui::CentralPanel::default().show(ctx, |ui| {
        // Check authentication before rendering protected screens
        let current_screen = state.current_screen;
        let is_authenticated = state.is_authenticated();
        
        // Redirect to Auth if trying to access protected screen without authentication
        if AppState::requires_auth(current_screen) && !is_authenticated {
            // Redirect to Auth screen
            app.handle_screen_change(Screen::Auth);
            // Render Auth screen instead
            screens::auth::render(ui, &state, app, cube);
            return;
        }
        
        // Render Bloomberg-style navigation bar (only when authenticated)
        if is_authenticated {
            widgets::nav_bar::render_nav_bar(ui, &state, app);
            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);
        }
        
        // Handle Tab key for screen navigation (excludes Messaging and Settings)
        if ctx.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.shift) {
            app.next_screen();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.shift) {
            app.previous_screen();
        }
        
        // Handle Ctrl+D to toggle debug overlay
        if ctx.input(|i| i.key_pressed(egui::Key::D) && i.modifiers.ctrl) {
            let mut state_write = app.state.write();
            state_write.debug_overlay_visible = !state_write.debug_overlay_visible;
        }

        match current_screen {
            Screen::Landing => screens::landing::render(ui, &state, app, cube),
            Screen::Auth => screens::auth::render(ui, &state, app, cube),
            Screen::Terminal => {
                screens::terminal::render(ui, &state, app);
                // Status bar at bottom
                ui.add_space(10.0);
                ui.separator();
                render_status_bar(ui, &state);
            },
            Screen::PythFeed => {
                screens::pyth_feed::render(ui, &state, app);
                // Status bar at bottom
                ui.add_space(10.0);
                ui.separator();
                render_status_bar(ui, &state);
            },
            Screen::JupiterFeed => {
                screens::jupiter_feed::render(ui, &state, app);
                // Status bar at bottom
                ui.add_space(10.0);
                ui.separator();
                render_status_bar(ui, &state);
            },
            Screen::Wallet => screens::wallet::render(ui, &state, app),
            Screen::Transactions => screens::transactions::render(ui, &state),
            Screen::Tokens => screens::tokens::render(ui, &state, app),
            Screen::Settings => screens::settings::render(ui, &state, app),
            Screen::Messaging => {
                screens::messaging::render(ui, &state, app);
                // Load friends list when messaging screen is opened
                // This is handled inside the messaging screen render function
            },
            Screen::AIChat => {
                screens::ai_chat::render(ui, &state, app);
            },
            Screen::LiveChart => {
                screens::live_chart::render(ui, &state, app);
                // Status bar at bottom
                ui.add_space(10.0);
                ui.separator();
                render_status_bar(ui, &state);
            },
            Screen::LiveAssets => {
                screens::live_assets::render(ui, &state, app);
                // Status bar at bottom
                ui.add_space(10.0);
                ui.separator();
                render_status_bar(ui, &state);
            },
            Screen::LiveTable => {
                screens::live_table::render(ui, &state, app);
                // Status bar at bottom
                ui.add_space(10.0);
                ui.separator();
                render_status_bar(ui, &state);
            },
        }
    });

    // Token picker popup (if active) - rendered as a window
    if state.terminal.swap.show_token_picker {
        widgets::token_picker::render_token_picker(ctx, &state, app);
    }

    // Debug overlay (if enabled) - rendered as a window on top
    if debug_overlay::should_show_overlay(&state) {
        debug_overlay::render_debug_overlay(ctx, &state);
    }
}

/// Render status bar at the bottom (public version)
pub fn render_status_bar(ui: &mut egui::Ui, state: &crate::app::AppState) {
    render_status_bar_impl(ui, state);
}

/// Render status bar implementation
fn render_status_bar_impl(ui: &mut egui::Ui, state: &crate::app::AppState) {
    render_status_bar_bottom(ui, state);
}

// Status bar implementation (keep this one, remove duplicate)
fn render_status_bar_bottom(ui: &mut egui::Ui, state: &crate::app::AppState) {
    use crate::ui::widgets::icons::{Icons, material, size};
    use crate::ui::theme::Theme;
    
    let theme = Theme::default();
    
    ui.horizontal(|ui| {
        // Wallet connection status with icon (first)
        if let Some(wallet) = &state.wallet {
            ui.label(Icons::icon_success(material::WALLET, size::SMALL));
            let short_addr = if wallet.address.len() > 8 {
                format!("{}...{}", &wallet.address[..4], &wallet.address[wallet.address.len()-4..])
            } else {
                wallet.address.clone()
            };
            ui.colored_label(
                theme.success,
                format!("Wallet: {} ({:.4} SOL)", short_addr, wallet.sol_balance)
            );
        } else {
            ui.label(Icons::icon_dim(material::WALLET, size::SMALL));
            ui.colored_label(theme.dim, "No Wallet");
        }

        ui.separator();
        
        // Status indicator with red accent
        ui.label(Icons::icon_red(material::INFO, size::SMALL));
        ui.colored_label(theme.selected, "Ready");

        ui.separator();

        // WebSocket connection status with icon and details
        use crate::app::WebSocketState;
        match &state.websocket_status.state {
            WebSocketState::Connected => {
                ui.label(Icons::icon_success(material::NETWORK, size::SMALL));
                let msg_count = state.websocket_status.messages_received;
                if msg_count > 0 {
                    ui.colored_label(theme.success, format!("WS: {} msgs", msg_count));
                } else {
                    ui.colored_label(theme.success, "WS Connected");
                }
            }
            WebSocketState::Connecting | WebSocketState::Reconnecting => {
                ui.label(Icons::icon_dim(material::NETWORK, size::SMALL));
                let attempts = state.websocket_status.connection_attempts;
                ui.colored_label(
                    theme.warning,
                    format!("WS: Connecting... ({})", attempts)
                );
            }
            WebSocketState::Disabled => {
                ui.label(Icons::icon_dim(material::NETWORK, size::SMALL));
                ui.colored_label(theme.error, "WS: Disabled");
            }
            WebSocketState::Disconnected => {
                ui.label(Icons::icon_dim(material::NETWORK, size::SMALL));
                if let Some(err) = &state.websocket_status.last_error {
                    let short_err = if err.len() > 30 {
                        format!("{}...", &err[..30])
                    } else {
                        err.clone()
                    };
                    ui.colored_label(theme.error, format!("WS: {}", short_err));
                } else {
                    ui.colored_label(theme.dim, "WS Disconnected");
                }
            }
        }

        ui.separator();

        // API connection status with icon (moved to bottom)
        if state.auth_token.is_some() {
            ui.label(Icons::icon_success(material::NETWORK, size::SMALL));
            ui.colored_label(theme.success, "API Connected");
        } else {
            ui.label(Icons::icon_dim(material::NETWORK, size::SMALL));
            ui.colored_label(theme.dim, "API Disconnected");
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.colored_label(theme.dim, "Q: Quit | Tab: Navigate | Enter: Select | Esc: Back");
        });
    });
}
