//! # Screen Modules
//!
//! Each screen module contains the rendering logic for a specific screen in the application.
//! Screens are organized by functionality and follow consistent rendering patterns.
//!
//! ## Screen Organization
//!
//! The application has multiple screens, each with its own rendering module:
//!
//! - **[`landing`]**: Welcome/splash screen with branding (rotating 3D icosahedron)
//! - **[`auth`]**: Authentication screen with login and signup forms
//! - **[`terminal`]**: Main trading terminal with swaps, charts, and price feeds
//! - **[`wallet`]**: Wallet management screen (connect, generate, view balance)
//! - **[`transactions`]**: Transaction history and monitoring screen
//! - **[`swap_history`]**: Swap transaction history view
//! - **[`token_explorer`]**: Token search and selection interface
//!
//! ## Rendering Pattern
//!
//! All screen modules follow a consistent pattern:
//!
//! ```rust,ignore
//! pub fn render(
//!     ui: &mut egui::Ui,
//!     state: &AppState,
//!     app: &mut App,
//!     cube: &mut RotatingCube, // Only on landing/auth screens
//! ) {
//!     // Screen-specific rendering logic
//!     // - Read from state
//!     // - Handle user input
//!     // - Call app.handle_* methods for actions
//! }
//! ```
//!
//! ## State Access Pattern
//!
//! Screens receive a **cloned state snapshot** for rendering:
//!
//! - State is cloned before rendering (no locks during UI rendering)
//! - User actions call `app.handle_*` methods which acquire locks internally
//! - This prevents UI freezing from lock contention
//!
//! ## Navigation
//!
//! Screens navigate using [`App::handle_screen_change`]:
//!
//! ```rust,ignore
//! if button.clicked() {
//!     app.handle_screen_change(Screen::Terminal);
//! }
//! ```
//!
//! ## Related Types
//!
//! - [`crate::app::Screen`]: Screen enum variants
//! - [`crate::app::AppState`]: Application state types
//! - [`crate::app::App`]: Application orchestrator

pub mod auth;
pub mod landing;
pub mod messaging;
pub mod ai_chat;
pub mod terminal;
pub mod transactions;
pub mod wallet;
pub mod swap_history;
pub mod token_explorer;
pub mod pyth_feed;
pub mod jupiter_feed;
pub mod tokens;
pub mod settings;
pub mod live_chart;
pub mod live_assets;
pub mod live_table;
