//! # Viewport Rendering
//!
//! Handles rendering of secondary windows (deferred viewports) with full
//! screen navigation support. Each window can independently cycle through screens.

use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;
use async_channel::Sender;
use crate::app::{
    AppState, Screen,
    events::AppEvent,
    window_manager::{WindowManager, WindowId},
    window_app::WindowApp,
};

/// Handle Tab key navigation for a window, updating the window's screen.
pub fn handle_window_navigation(
    ctx: &egui::Context,
    window_id: WindowId,
    current_screen: Screen,
    state: &Arc<RwLock<AppState>>,
    window_manager: &Arc<RwLock<WindowManager>>,
    forward: bool,
) -> Option<Screen> {
    if !ctx.input(|i| {
        let tab_pressed = i.key_pressed(egui::Key::Tab);
        let shift = i.modifiers.shift;
        tab_pressed && ((forward && !shift) || (!forward && shift))
    }) {
        return None;
    }

    let screens = Screen::all();
    let current_idx = screens
        .iter()
        .position(|&s| s == current_screen)
        .unwrap_or(0);
    
    let is_authenticated = {
        let state = state.read();
        state.is_authenticated()
    };
    
    // Find next/previous screen, skipping protected screens if not authenticated
    let mut new_idx = if forward {
        (current_idx + 1) % screens.len()
    } else {
        if current_idx == 0 { screens.len() - 1 } else { current_idx - 1 }
    };
    
    let mut attempts = 0;
    while attempts < screens.len() {
        let screen = screens[new_idx];
        if !AppState::requires_auth(screen) || is_authenticated {
            // Update window's screen
            let mut window_manager = window_manager.write();
            if let Some(window) = window_manager.get_window_mut(window_id) {
                window.screen = screen;
                window.title = format!("Terminal - {}", screen.title());
            }
            return Some(screen);
        }
        new_idx = if forward {
            (new_idx + 1) % screens.len()
        } else {
            if new_idx == 0 { screens.len() - 1 } else { new_idx - 1 }
        };
        attempts += 1;
    }
    
    None
}

/// Render a screen in a viewport window.
/// 
/// Now uses the AppLike trait, allowing full rendering of all screens in secondary windows.
pub fn render_viewport_screen(
    ui: &mut egui::Ui,
    screen: Screen,
    state: &AppState,
    window_app: &mut WindowApp,
    cube: &mut crate::ui::cube::RotatingCube,
) {
    if AppState::requires_auth(screen) && !state.is_authenticated() {
        // Redirect to Auth screen for this window
        window_app.handle_screen_change(Screen::Auth);
        crate::ui::screens::auth::render(ui, state, window_app, cube);
        return;
    }
    
    // Render the full screen using the same renderers as main window
    // All screen renderers now accept impl AppLike, so WindowApp works seamlessly
    match screen {
        Screen::Landing => {
            crate::ui::screens::landing::render(ui, state, window_app, cube);
        },
        Screen::Auth => {
            crate::ui::screens::auth::render(ui, state, window_app, cube);
        },
        Screen::Terminal => {
            crate::ui::screens::terminal::render(ui, state, window_app);
        },
        Screen::PythFeed => {
            crate::ui::screens::pyth_feed::render(ui, state, window_app);
        },
        Screen::JupiterFeed => {
            crate::ui::screens::jupiter_feed::render(ui, state, window_app);
        },
        Screen::Wallet => {
            crate::ui::screens::wallet::render(ui, state, window_app);
        },
        Screen::Transactions => {
            crate::ui::screens::transactions::render(ui, state);
        },
        Screen::Tokens => {
            crate::ui::screens::tokens::render(ui, state, window_app);
        },
        Screen::Settings => {
            crate::ui::screens::settings::render(ui, state, window_app);
        },
        Screen::Messaging => {
            crate::ui::screens::messaging::render(ui, state, window_app);
        },
        Screen::AIChat => {
            crate::ui::screens::ai_chat::render(ui, state, window_app);
        },
        Screen::LiveChart => {
            crate::ui::screens::live_chart::render(ui, state, window_app);
        },
        Screen::LiveAssets => {
            crate::ui::screens::live_assets::render(ui, state, window_app);
        },
        Screen::LiveTable => {
            crate::ui::screens::live_table::render(ui, state, window_app);
        },
    }
}

/// Create and show a deferred viewport for a secondary window.
pub fn show_deferred_viewport(
    ctx: &egui::Context,
    viewport_id: egui::ViewportId,
    window_id: WindowId,
    window_title: String,
    state: Arc<RwLock<AppState>>,
    window_manager: Arc<RwLock<WindowManager>>,
    event_tx: Sender<AppEvent>,
) {
    let viewport_builder = egui::ViewportBuilder::default()
        .with_title(window_title.clone())
        .with_inner_size([1000.0, 700.0])
        .with_min_inner_size([600.0, 400.0])
        .with_decorations(true)
        .with_transparent(false);
    
    ctx.show_viewport_deferred(viewport_id, viewport_builder, move |ctx, class| {
        if class != egui::ViewportClass::Deferred {
            return;
        }

        // Get current window screen from window manager
        let current_screen = {
            let window_manager = window_manager.read();
            window_manager.get_window(window_id)
                .map(|w| w.screen)
                .unwrap_or(Screen::Terminal)
        };
        
        // Update window title based on current screen
        let title = format!("Terminal - {}", current_screen.title());
        ctx.send_viewport_cmd_to(viewport_id, egui::ViewportCommand::Title(title));
        
        // Handle Tab key navigation (forward and backward)
        let state_clone = state.clone();
        let window_manager_clone = window_manager.clone();
        handle_window_navigation(
            ctx,
            window_id,
            current_screen,
            &state_clone,
            &window_manager_clone,
            true,
        );
        
        let state_clone = state.clone();
        let window_manager_clone = window_manager.clone();
        handle_window_navigation(
            ctx,
            window_id,
            current_screen,
            &state_clone,
            &window_manager_clone,
            false,
        );
        
        // Get updated screen after potential navigation
        let screen_to_render = {
            let window_manager = window_manager.read();
            window_manager.get_window(window_id)
                .map(|w| w.screen)
                .unwrap_or(Screen::Terminal)
        };
        
        // Read state for rendering
        let state_for_render = state.read().clone();
        
        // Create WindowApp wrapper
        let mut window_app = WindowApp::new(
            state.clone(),
            window_manager.clone(),
            event_tx.clone(),
            window_id,
        );
        
        // Create a cube for screens that need it
        let mut cube = crate::ui::cube::RotatingCube::new();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            render_viewport_screen(ui, screen_to_render, &state_for_render, &mut window_app, &mut cube);
        });
    });
}

