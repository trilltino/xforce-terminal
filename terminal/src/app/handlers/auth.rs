//! # Authentication Handlers
//!
//! Handlers for login, signup, and authentication-related actions.

use crate::app::state::{AppState, AuthState, LoginField, SignupField};
use crate::app::events::AppEvent;
use crate::core::service::ApiService;
use async_channel::Sender;
use parking_lot::RwLock;
use std::sync::Arc;

/// Handle login button click
///
/// Internal handler function - use [`crate::app::App::handle_login_click`] instead.
pub(crate) fn handle_login_click(
    state: Arc<RwLock<AppState>>,
    event_tx: Sender<AppEvent>,
    username: String,
    password: String,
) {
    if username.is_empty() || password.is_empty() {
        let mut state = state.write();
        if let AuthState::Login { error, .. } = &mut state.auth {
            *error = Some("Username and password required".to_string());
        }
        return;
    }

    let api_client = match state.read().api_client.as_ref() {
        Some(client) => client.clone(),
        None => {
            let mut state = state.write();
            if let AuthState::Login { error, .. } = &mut state.auth {
                *error = Some("API client not available".to_string());
            }
            return;
        }
    };

    let tx = event_tx.clone();
    tokio::spawn(async move {
        let _ = tx.send(AppEvent::Loading("Logging in...".to_string())).await;
        let result = api_client.login(username, password).await;
        let _ = tx.send(AppEvent::LoginResult(result)).await;
    });

    let mut state = state.write();
    if let AuthState::Login { error, .. } = &mut state.auth {
        *error = Some("Logging in...".to_string());
    }
}

/// Handle signup button click
///
/// Internal handler function - use [`crate::app::App::handle_signup_click`] instead.
pub(crate) fn handle_signup_click(
    state: Arc<RwLock<AppState>>,
    event_tx: Sender<AppEvent>,
    username: String,
    email: String,
    password: String,
    confirm_password: String,
) {
    if username.is_empty() || email.is_empty() || password.is_empty() {
        let mut state = state.write();
        if let AuthState::Signup { error, .. } = &mut state.auth {
            *error = Some("All fields required".to_string());
        }
        return;
    }

    if password != confirm_password {
        let mut state = state.write();
        if let AuthState::Signup { error, .. } = &mut state.auth {
            *error = Some("Passwords don't match".to_string());
        }
        return;
    }

    let api_client = match state.read().api_client.as_ref() {
        Some(client) => client.clone(),
        None => {
            let mut state = state.write();
            if let AuthState::Signup { error, .. } = &mut state.auth {
                *error = Some("API client not available".to_string());
            }
            return;
        }
    };

    let tx = event_tx.clone();
    tokio::spawn(async move {
        let _ = tx.send(AppEvent::Loading("Signing up...".to_string())).await;
        let result = api_client.signup(username, email, password).await;
        let _ = tx.send(AppEvent::SignupResult(result)).await;
    });

    let mut state = state.write();
    if let AuthState::Signup { error, .. } = &mut state.auth {
        *error = Some("Signing up...".to_string());
    }
}

/// Switch to login form
///
/// Internal handler function - use [`crate::app::App::handle_switch_to_login`] instead.
pub(crate) fn handle_switch_to_login(state: Arc<RwLock<AppState>>) {
    let mut state = state.write();
    state.auth = AuthState::Login {
        username: String::new(),
        password: String::new(),
        error: None,
        active_field: LoginField::Username,
    };
}

/// Switch to signup form
///
/// Internal handler function - use [`crate::app::App::handle_switch_to_signup`] instead.
pub(crate) fn handle_switch_to_signup(state: Arc<RwLock<AppState>>) {
    let mut state = state.write();
    state.auth = AuthState::Signup {
        username: String::new(),
        email: String::new(),
        password: String::new(),
        confirm_password: String::new(),
        error: None,
        active_field: SignupField::Username,
    };
}

