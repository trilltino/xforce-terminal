//! Minimal Wallet Helper for Phantom/Solflare Connection
//!
//! The Ratatui terminal is the main app. This is just a browser wallet helper.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

mod app;
mod components;
mod pages;
mod services;
mod state;
mod utils;

use app::App;

#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook for better error messages in WASM
    console_error_panic_hook::set_once();

    // Initialize logger
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("[DEBUG] Solana DeFi Terminal starting...");
    web_sys::console::log_1(&"[DEBUG] main() function called - WASM loaded".into());

    // Hide loading screen immediately when WASM loads
    web_sys::console::log_1(&"[DEBUG] Calling hide_loading_screen() from main()".into());
    hide_loading_screen();

    // Mount the Leptos app
    web_sys::console::log_1(&"[DEBUG] Mounting Leptos app to body".into());
    leptos::mount::mount_to_body(|| view! { <App/> });
    web_sys::console::log_1(&"[DEBUG] leptos::mount::mount_to_body() called".into());
}

/// Hide the loading screen element
fn hide_loading_screen() {
    web_sys::console::log_1(&"[DEBUG] hide_loading_screen() function entered".into());
    
    let window = match web_sys::window() {
        Some(w) => {
            web_sys::console::log_1(&"[DEBUG] Window obtained successfully".into());
            w
        }
        None => {
            web_sys::console::error_1(&"[DEBUG] ERROR: No window available".into());
            return;
        }
    };
    
    let document = match window.document() {
        Some(d) => {
            web_sys::console::log_1(&"[DEBUG] Document obtained successfully".into());
            d
        }
        None => {
            web_sys::console::error_1(&"[DEBUG] ERROR: No document available".into());
            return;
        }
    };
    
    web_sys::console::log_1(&"[DEBUG] Looking for element with id 'leptos-loading'".into());
    if let Some(loading_element) = document.get_element_by_id("leptos-loading") {
        web_sys::console::log_1(&"[DEBUG] Loading element found!".into());
        
        // Check current state
        if let Some(html_element) = loading_element.dyn_ref::<HtmlElement>() {
            let current_classes = html_element.class_name();
            web_sys::console::log_1(&format!("[DEBUG] Current classes: {}", current_classes).into());
            
            // Add hidden class
            match html_element.class_list().add_1("hidden") {
                Ok(_) => web_sys::console::log_1(&"[DEBUG] Successfully added 'hidden' class".into()),
                Err(e) => web_sys::console::error_1(&format!("[DEBUG] ERROR adding 'hidden' class: {:?}", e).into()),
            }
            
            let new_classes = html_element.class_name();
            web_sys::console::log_1(&format!("[DEBUG] New classes after adding 'hidden': {}", new_classes).into());
        } else {
            web_sys::console::warn_1(&"[DEBUG] WARNING: Loading element is not an HtmlElement".into());
        }
        
        // Also set display:none as backup
        match loading_element.set_attribute("style", "display: none !important;") {
            Ok(_) => web_sys::console::log_1(&"[DEBUG] Successfully set style='display: none !important;'".into()),
            Err(e) => web_sys::console::error_1(&format!("[DEBUG] ERROR setting style: {:?}", e).into()),
        }
        
        // Verify the element is actually hidden
        if let Some(html_element) = loading_element.dyn_ref::<HtmlElement>() {
            let computed_style = window
                .get_computed_style(&html_element)
                .ok()
                .flatten();
            if let Some(style) = computed_style {
                let display = style.get_property_value("display").unwrap_or_default();
                web_sys::console::log_1(&format!("[DEBUG] Computed display style: {}", display).into());
            }
        }
        
        log::info!("[DEBUG] Loading screen hidden from main()");
        web_sys::console::log_1(&"[DEBUG] hide_loading_screen() completed successfully".into());
    } else {
        web_sys::console::warn_1(&"[DEBUG] WARNING: Loading element with id 'leptos-loading' not found!".into());
        log::warn!("[DEBUG] Loading element not found");
    }
}
