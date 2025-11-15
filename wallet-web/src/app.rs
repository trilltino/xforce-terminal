//! XForce Terminal Wallet Web App - Leptos Frontend
//!
//! Deep space themed wallet connection interface

use leptos::prelude::*;
use leptos_router::{
    components::{A, Route, Router, Routes},
    path,
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::pages::{
    ConnectPage,
    StatusPage,
    WalletSetupPage,
    TransactionSignPage,
    AboutPage,
};
use crate::components::{Navbar, Starfield};
use crate::state::wallet::provide_wallet_context;

#[component]
pub fn App() -> impl IntoView {
    // Only wallet context - no auth, no swap
    provide_wallet_context();

    // Hide loading screen once app is mounted (backup in case main() didn't catch it)
    Effect::new(move || {
        web_sys::console::log_1(&"[DEBUG] App::Effect::new() fired - Leptos app component mounted".into());
        log::info!("[DEBUG] App::Effect::new() fired");
        
        // Hide immediately - create_effect fires after component is in DOM
        let window = match web_sys::window() {
            Some(w) => {
                web_sys::console::log_1(&"[DEBUG] App::create_effect - Window obtained".into());
                w
            }
            None => {
                web_sys::console::error_1(&"[DEBUG] App::create_effect - ERROR: No window".into());
                return;
            }
        };
        
        let document = match window.document() {
            Some(d) => {
                web_sys::console::log_1(&"[DEBUG] App::create_effect - Document obtained".into());
                d
            }
            None => {
                web_sys::console::error_1(&"[DEBUG] App::create_effect - ERROR: No document".into());
                return;
            }
        };
        
        // Hide synchronously first, then also try async as backup
        web_sys::console::log_1(&"[DEBUG] App::create_effect - Looking for loading element".into());
        if let Some(loading_element) = document.get_element_by_id("leptos-loading") {
            web_sys::console::log_1(&"[DEBUG] App::create_effect - Loading element found!".into());
            
            // Check current state
            if let Some(html_element) = loading_element.dyn_ref::<HtmlElement>() {
                let current_classes = html_element.class_name();
                web_sys::console::log_1(&format!("[DEBUG] App::create_effect - Current classes: {}", current_classes).into());
                
                // Try adding hidden class first (preferred method)
                match html_element.class_list().add_1("hidden") {
                    Ok(_) => web_sys::console::log_1(&"[DEBUG] App::create_effect - Added 'hidden' class".into()),
                    Err(e) => web_sys::console::error_1(&format!("[DEBUG] App::create_effect - ERROR adding class: {:?}", e).into()),
                }
                
                let new_classes = html_element.class_name();
                web_sys::console::log_1(&format!("[DEBUG] App::create_effect - New classes: {}", new_classes).into());
            }
            
            // Also set display:none as backup
            match loading_element.set_attribute("style", "display: none !important;") {
                Ok(_) => web_sys::console::log_1(&"[DEBUG] App::create_effect - Set style attribute".into()),
                Err(e) => web_sys::console::error_1(&format!("[DEBUG] App::create_effect - ERROR setting style: {:?}", e).into()),
            }
            
            // Verify
            if let Some(html_element) = loading_element.dyn_ref::<HtmlElement>() {
                let computed_style = window
                    .get_computed_style(&html_element)
                    .ok()
                    .flatten();
                if let Some(style) = computed_style {
                    let display = style.get_property_value("display").unwrap_or_default();
                    web_sys::console::log_1(&format!("[DEBUG] App::create_effect - Computed display: {}", display).into());
                }
            }
            
            log::info!("[DEBUG] Loading screen hidden via create_effect - Leptos app mounted");
            web_sys::console::log_1(&"[DEBUG] App::create_effect - Synchronous hide completed".into());
        } else {
            web_sys::console::warn_1(&"[DEBUG] App::create_effect - Loading element not found!".into());
        }
        
        // Also try async as additional backup
        web_sys::console::log_1(&"[DEBUG] App::create_effect - Spawning async backup task".into());
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(100).await;
            web_sys::console::log_1(&"[DEBUG] App::create_effect - Async backup task (100ms) executing".into());
            
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(loading_element) = document.get_element_by_id("leptos-loading") {
                        web_sys::console::log_1(&"[DEBUG] App::create_effect - Async backup found loading element".into());
                        if let Some(html_element) = loading_element.dyn_ref::<HtmlElement>() {
                            html_element.class_list().add_1("hidden").ok();
                        }
                        loading_element.set_attribute("style", "display: none !important;").ok();
                        web_sys::console::log_1(&"[DEBUG] App::create_effect - Async backup completed".into());
                    } else {
                        web_sys::console::log_1(&"[DEBUG] App::create_effect - Async backup: element not found".into());
                    }
                }
            }
        });
    });

    view! {
        <Router>
            <div class="app-container">
                <Starfield/>
                <Navbar/>
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=ConnectPage/>
                    <Route path=path!("/about") view=AboutPage/>
                    <Route path=path!("/status") view=StatusPage/>
                    <Route path=path!("/wallet-setup") view=WalletSetupPage/>
                    <Route path=path!("/sign-transaction") view=TransactionSignPage/>
                </Routes>
            </div>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="app-container" style="display: flex; justify-content: center; align-items: center; min-height: calc(100vh - 60px);">
            <div class="card" style="max-width: 500px; text-align: center;">
                <h1 style="color: #ffffff; margin-bottom: 16px; font-size: 32px; font-weight: 700;">"404 - Page Not Found"</h1>
                <p style="color: #cccccc; margin-bottom: 24px;">"The page you're looking for doesn't exist."</p>
                <A href="/">
                    <span class="btn" style="margin-top: 20px; display: inline-block;">
                        "Go to Home"
                    </span>
                </A>
            </div>
        </div>
    }
}
