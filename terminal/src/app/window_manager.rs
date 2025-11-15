//! # Window Manager
//!
//! Manages multiple independent windows (viewports) in the Bloomberg-style terminal application.
//! Each window can display different screens, be moved, resized, and fullscreened independently.
//!
//! ## Architecture
//!
//! - **Main Window**: Root viewport (ViewportId::ROOT) - cannot be closed
//! - **Secondary Windows**: Deferred viewports created on demand
//! - **Window State**: Each window has independent screen, position, size, and fullscreen state
//! - **Shared App State**: All windows share the same `Arc<RwLock<AppState>>` for data synchronization

use egui::ViewportId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for a window/viewport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

/// Window state tracking position, size, and content
#[derive(Debug, Clone)]
pub struct WindowState {
    /// Unique window identifier
    pub id: WindowId,
    /// Viewport ID from egui
    pub viewport_id: ViewportId,
    /// Current screen displayed in this window
    pub screen: crate::app::Screen,
    /// Window title
    pub title: String,
    /// Whether this window is fullscreened
    pub is_fullscreen: bool,
    /// Window position (x, y)
    pub position: Option<(f32, f32)>,
    /// Window size (width, height)
    pub size: Option<(f32, f32)>,
}

/// Window manager that tracks all open windows
pub struct WindowManager {
    /// Next window ID to assign
    next_window_id: AtomicU64,
    /// All open windows by window ID
    windows: HashMap<WindowId, WindowState>,
    /// All windows by viewport ID for quick lookup
    viewport_to_window: HashMap<ViewportId, WindowId>,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        Self {
            next_window_id: AtomicU64::new(1), // Start at 1, 0 is reserved for ROOT
            windows: HashMap::new(),
            viewport_to_window: HashMap::new(),
        }
    }

    /// Register the root viewport (main window)
    pub fn register_root(&mut self, viewport_id: ViewportId, screen: crate::app::Screen) -> WindowId {
        let window_id = WindowId(0); // ROOT always has ID 0
        
        let window_state = WindowState {
            id: window_id,
            viewport_id,
            screen,
            title: "Solana DeFi Trading Terminal".to_string(),
            is_fullscreen: false,
            position: None,
            size: None,
        };
        
        self.windows.insert(window_id, window_state);
        self.viewport_to_window.insert(viewport_id, window_id);
        
        window_id
    }

    /// Create a new window and return its ID
    pub fn create_window(
        &mut self,
        viewport_id: ViewportId,
        screen: crate::app::Screen,
        title: Option<String>,
    ) -> WindowId {
        let window_id = WindowId(self.next_window_id.fetch_add(1, Ordering::Relaxed));
        
        let title = title.unwrap_or_else(|| format!("Terminal Window #{}", window_id.0));
        
        let window_state = WindowState {
            id: window_id,
            viewport_id,
            screen,
            title: title.clone(),
            is_fullscreen: false,
            position: None,
            size: None,
        };
        
        self.windows.insert(window_id, window_state);
        self.viewport_to_window.insert(viewport_id, window_id);
        
        window_id
    }

    /// Remove a window (called when window is closed)
    pub fn remove_window(&mut self, viewport_id: ViewportId) -> Option<WindowId> {
        if let Some(window_id) = self.viewport_to_window.remove(&viewport_id) {
            self.windows.remove(&window_id);
            Some(window_id)
        } else {
            None
        }
    }

    /// Get window state by window ID
    pub fn get_window(&self, window_id: WindowId) -> Option<&WindowState> {
        self.windows.get(&window_id)
    }

    /// Get window state by viewport ID
    pub fn get_window_by_viewport(&self, viewport_id: ViewportId) -> Option<&WindowState> {
        self.viewport_to_window
            .get(&viewport_id)
            .and_then(|window_id| self.windows.get(window_id))
    }

    /// Get mutable window state by window ID
    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut WindowState> {
        self.windows.get_mut(&window_id)
    }

    /// Get mutable window state by viewport ID
    pub fn get_window_by_viewport_mut(&mut self, viewport_id: ViewportId) -> Option<&mut WindowState> {
        self.viewport_to_window
            .get(&viewport_id)
            .and_then(|window_id| self.windows.get_mut(window_id))
    }

    /// Update window screen
    pub fn set_window_screen(&mut self, window_id: WindowId, screen: crate::app::Screen) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.screen = screen;
            window.title = format!("Terminal - {}", screen.title());
        }
    }

    /// Update window screen by viewport ID
    pub fn set_window_screen_by_viewport(&mut self, viewport_id: ViewportId, screen: crate::app::Screen) {
        if let Some(window) = self.get_window_by_viewport_mut(viewport_id) {
            window.screen = screen;
            window.title = format!("Terminal - {}", screen.title());
        }
    }

    /// Toggle fullscreen for a window
    pub fn toggle_fullscreen(&mut self, window_id: WindowId) -> bool {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.is_fullscreen = !window.is_fullscreen;
            window.is_fullscreen
        } else {
            false
        }
    }

    /// Toggle fullscreen by viewport ID
    pub fn toggle_fullscreen_by_viewport(&mut self, viewport_id: ViewportId) -> bool {
        if let Some(window) = self.get_window_by_viewport_mut(viewport_id) {
            window.is_fullscreen = !window.is_fullscreen;
            window.is_fullscreen
        } else {
            false
        }
    }

    /// Get all open windows
    pub fn all_windows(&self) -> Vec<&WindowState> {
        self.windows.values().collect()
    }

    /// Get window count (excluding root)
    pub fn window_count(&self) -> usize {
        self.windows.len().saturating_sub(1) // Exclude root
    }

    /// Check if a viewport ID belongs to a managed window
    pub fn is_managed(&self, viewport_id: ViewportId) -> bool {
        self.viewport_to_window.contains_key(&viewport_id)
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

