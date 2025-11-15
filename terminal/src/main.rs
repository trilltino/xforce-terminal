//! # Solana DeFi Trading Terminal (egui GUI)
//!
//! A high-performance **native desktop GUI** for Solana DeFi trading.
//! Built with egui for cross-platform compatibility (Linux, Windows, macOS).
//!
//! ## Features
//!
//! - **Real-time Price Feeds**: Jupiter aggregator + Pyth oracle integration
//! - **Wallet Management**: Local keypair signing with Solana SDK
//! - **DEX Swap Execution**: Jupiter aggregator for best swap routes
//! - **Transaction History**: Monitor and track all swap transactions
//! - **Native GUI Window**: Full control without terminal limitations
//!
//! ## Architecture
//!
//! ### Technology Stack
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    GUI Application (This Binary)            │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │  egui Framework                                     │   │
//! │  │  - eframe (native window)                           │   │
//! │  │  - Event loop (update/render)                       │   │
//! │  │  - Widget system (buttons, tables, charts)          │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │  Application Layer (app.rs)                         │   │
//! │  │  - State management (Arc<RwLock<AppState>>)         │   │
//! │  │  - Event handling (async channels)                  │   │
//! │  │  - Screen navigation                                │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │  Services Layer                                     │   │
//! │  │  - ApiClient: Backend HTTP client                   │   │
//! │  │  - WalletService: Solana transaction signing        │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//!           │                                       │
//!           │ HTTP (reqwest)                        │ Solana RPC
//!           ▼                                       ▼
//! ┌──────────────────────┐               ┌─────────────────────┐
//! │  Backend API Server  │               │   Solana Network    │
//! │  (Axum + PostgreSQL) │               │   (Devnet/Mainnet)  │
//! └──────────────────────┘               └─────────────────────┘
//! ```
//!
//! ### Key Components
//!
//! - **GUI Framework**: egui + eframe for native window rendering
//! - **Backend API**: Axum REST API (runs separately on port 3001)
//! - **Blockchain**: Solana SDK for transaction signing
//! - **Async Runtime**: Tokio for non-blocking I/O
//!
//! ## Event Loop Architecture
//!
//! The application runs egui's event loop that processes:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │                    egui Event Loop                      │
//! │                   (eframe::App::update)                 │
//! └──────────────────┬───────────────────────────────────────┘
//!                    │
//!                    ├──> UPDATE (every frame, ~60 FPS)
//!                    │    ├─> app.on_tick() (process async events)
//!                    │    ├─> Handle GUI events (button clicks, input)
//!                    │    └─> Request repaint for animations
//!                    │
//!                    └──> RENDER (every frame)
//!                         ├─> ui::render(ui, &app)
//!                         │   └─> Screen-specific rendering
//!                         └─> Display to native window
//! ```

use eframe::egui;
use std::time::{Duration, Instant};
use crate::app::{App, show_deferred_viewport};

mod app;
mod core;
mod debug;
mod services;
mod ui;
mod utils;

use ui::cube::RotatingCube;

/// Application entry point
#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Initialize debug system (file-based logging)
    // This also sets up the panic hook that logs to files
    debug::init();
    debug::init_metrics();

    // Note: Panic hook is already set up by debug::init() which logs to both
    // console (stderr) and log files. We don't need to set it up again here.
    // The debug system's panic hook is more comprehensive and includes:
    // - File logging via tracing
    // - Backtrace capture
    // - Trace ID support

    tracing::info!("Starting Solana DeFi Trading Terminal (GUI)");
    tracing::info!("Terminal startup - Debug viewer should be tracking logs from this point");
    tracing::debug!("Main function entry point - Application initialization beginning");

    // Create app state with error handling
    let app = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| App::new())) {
        Ok(app) => app,
        Err(e) => {
            let error_msg = if let Some(s) = e.downcast_ref::<&str>() {
                format!("Panic during App::new(): {}", s)
            } else if let Some(s) = e.downcast_ref::<String>() {
                format!("Panic during App::new(): {}", s)
            } else {
                "Unknown panic during App::new()".to_string()
            };
            tracing::error!("{}", error_msg);
            eprintln!("ERROR: {}", error_msg);
            eprintln!("The application will now exit. Check the logs for more details.");
            // Exit with error code instead of returning eframe::Error
            std::process::exit(1);
        }
    };

    tracing::info!("App state created successfully");

    // Native options for window - with title bar for window movement
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Solana DeFi Trading Terminal")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_decorations(true)  // Show title bar for window movement
            .with_transparent(false),
        ..Default::default()
    };

    tracing::info!("Starting eframe GUI application...");

    // Run the GUI application with error handling
    let result = eframe::run_native(
        "Solana DeFi Trading Terminal",
        native_options,
        Box::new(|_cc| {
            tracing::info!("eframe app creation callback called");
            
            // Theme application deferred to first update() call to avoid egui 0.33 initialization panic
            // Material Design icons initialization also deferred
            
            // Register root viewport in window manager
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut window_manager = app.window_manager.write();
                let current_screen = {
                    let state = app.state.read();
                    state.current_screen
                };
                window_manager.register_root(egui::ViewportId::ROOT, current_screen);
            })) {
                Ok(_) => {
                    tracing::info!("Root viewport registered successfully");
                }
                Err(e) => {
                    let error_msg = if let Some(s) = e.downcast_ref::<&str>() {
                        format!("Panic during viewport registration: {}", s)
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        format!("Panic during viewport registration: {}", s)
                    } else {
                        "Unknown panic during viewport registration".to_string()
                    };
                    tracing::error!("{}", error_msg);
                    eprintln!("ERROR: {}", error_msg);
                    // Return error using String conversion (eframe::Error implements From<String>)
                    return Err(error_msg.into());
                }
            }
            
            tracing::info!("Creating GuiApp instance...");
            Ok(Box::new(GuiApp { 
                app,
                cube: RotatingCube::new(),
                last_frame_time: Instant::now(),
                notifications: crate::ui::widgets::notifications::NotificationManager::new(),
                theme_applied: false,
                viewport_fullscreen: std::collections::HashMap::new(),
            }))
        }),
    );

    match &result {
        Ok(_) => tracing::info!("Application exited normally"),
        Err(e) => {
            tracing::error!("Application exited with error: {:?}", e);
            eprintln!("ERROR: Application failed to start: {:?}", e);
            eprintln!("Check logs/debug-realtime.log for more details.");
        }
    }

    result
}

/// GUI application wrapper that implements eframe::App
struct GuiApp {
    app: App,
    cube: RotatingCube,
    last_frame_time: Instant,
    notifications: crate::ui::widgets::notifications::NotificationManager,
    /// Flag to ensure theme is only applied once after egui is fully initialized
    theme_applied: bool,
    /// Track fullscreen state per viewport (viewport_id -> is_fullscreen)
    viewport_fullscreen: std::collections::HashMap<egui::ViewportId, bool>,
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Initialize fonts and apply theme on first update (after egui is fully initialized)
        // This prevents panic in egui 0.33's style.rs during initialization
        // Fonts must be initialized BEFORE theme application to ensure text styles exist
        if !self.theme_applied {
            // Initialize fonts first - this registers all required text styles
            crate::ui::fonts::FontConfig::setup_terminal_fonts(ctx);
            
            // Verify JetBrains Mono font is loaded (already loaded in setup_terminal_fonts)
            crate::ui::fonts::FontConfig::load_custom_fonts(ctx);
            
            // Then apply theme - use loaded theme config from app state
            let theme_config = {
                let state = self.app.state.read();
                state.settings.theme_config.clone()
            };
            crate::ui::theme::Theme::apply_custom_theme(ctx, &theme_config);
            
            // Initialize Material Design icons
            crate::ui::widgets::icons::initialize_material_icons(ctx);
            
            self.theme_applied = true;
            tracing::info!("Initialized fonts and applied theme successfully");
        }
        
        // Update cube rotation
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        // Clamp delta_time to prevent large jumps on first frame or lag spikes
        let delta_time = delta_time.min(0.1); // Max 100ms per frame
        self.last_frame_time = now;
        self.cube.update(delta_time);

        // Handle Ctrl+N to create new window
        if ctx.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.ctrl) {
            let current_screen = {
                let state = self.app.state.read();
                state.current_screen
            };
            crate::ui::widgets::window_controls::create_new_window(&mut self.app, current_screen);
        }
        
        // Handle F11 to toggle fullscreen for the focused viewport
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            // Get the currently focused viewport (defaults to ROOT)
            // TODO: Implement proper focused viewport detection when egui API is clearer
            let focused_viewport = egui::ViewportId::ROOT;
            
            let is_fullscreen = self.viewport_fullscreen.get(&focused_viewport).copied().unwrap_or(false);
            let new_fullscreen = !is_fullscreen;
            self.viewport_fullscreen.insert(focused_viewport, new_fullscreen);
            
            // Update window manager
            {
                let mut window_manager = self.app.window_manager.write();
                if let Some(window) = window_manager.get_window_by_viewport_mut(focused_viewport) {
                    window.is_fullscreen = new_fullscreen;
                }
            }
            
            // Send fullscreen command to viewport
            ctx.send_viewport_cmd_to(
                focused_viewport,
                egui::ViewportCommand::Fullscreen(new_fullscreen),
            );
        }
        
        // Render all secondary windows
        self.render_secondary_windows(ctx);

        // Process async events on every frame (this processes events from event_rx)
        self.app.on_tick();
        
        // Process pending notifications from app state
        self.process_notifications();

        // CRITICAL: Check if immediate repaint is needed (real-time price updates)
        // Process this FIRST before any other checks to minimize latency
        let (needs_immediate_repaint, is_receiving_updates, ws_connected) = {
            let state = self.app.state.read();
            (
                state.needs_immediate_repaint,
                state.websocket_connected 
                    && matches!(state.websocket_status.state, crate::app::WebSocketState::Connected)
                    && state.last_price_update_time.elapsed().as_millis() < 1000, // Updated in last second
                state.websocket_connected 
                    && matches!(state.websocket_status.state, crate::app::WebSocketState::Connected),
            )
        };
        
        // CRITICAL: Immediate repaint for price updates (0ms delay for instant updates)
        if needs_immediate_repaint {
            // Instant repaint - no delay for Bloomberg-style real-time updates
            ctx.request_repaint();
            // Clear the flag immediately
            {
                let mut state = self.app.state.write();
                state.needs_immediate_repaint = false;
            }
        } else if is_receiving_updates {
            // When actively receiving WebSocket updates, use 0ms delay for instant updates
            // This ensures <10ms latency for price changes (Bloomberg-style)
            ctx.request_repaint();
        } else if ws_connected {
            // WebSocket connected but not receiving updates recently - use minimal delay
            ctx.request_repaint_after(Duration::from_millis(100)); // Check every 100ms
        } else {
            // Normal refresh rate for animations (target 60 FPS)
            ctx.request_repaint_after(Duration::from_millis(16));
        }

        // Render UI (pass frame for window controls, notifications for potential in-render notifications)
        ui::render(ctx, &mut self.app, &mut self.notifications, &mut self.cube, frame);
        
        // Show notifications (rendered on top of everything)
        self.notifications.show(ctx);
    }
}

impl GuiApp {
    /// Process pending notifications from app state
    fn process_notifications(&mut self) {
        let notifications = {
            let mut state = self.app.state.write();
            let pending = state.pending_notifications.clone();
            state.pending_notifications.clear(); // Clear after reading
            pending
        };
        
        // Show all pending notifications
        for (level, message) in notifications {
            match level.as_str() {
                "success" => self.notifications.success(message),
                "error" => self.notifications.error(message),
                "warning" => self.notifications.warning(message),
                "info" => self.notifications.info(message),
                _ => self.notifications.info(message),
            }
        }
    }

    /// Render all secondary windows as deferred viewports
    fn render_secondary_windows(&self, ctx: &egui::Context) {
        let windows_to_render: Vec<_> = {
            let window_manager = self.app.window_manager.read();
            window_manager.all_windows()
                .iter()
                .filter(|w| w.id.0 != 0) // Exclude root
                .map(|w| (w.viewport_id, w.id, w.title.clone()))
                .collect()
        };
        
        let state = std::sync::Arc::clone(&self.app.state);
        let window_manager = std::sync::Arc::clone(&self.app.window_manager);
        let event_tx = self.app.event_tx();
        
        for (viewport_id, window_id, window_title) in windows_to_render {
            show_deferred_viewport(
                ctx,
                viewport_id,
                window_id,
                window_title,
                state.clone(),
                window_manager.clone(),
                event_tx.clone(),
            );
        }
    }
}
