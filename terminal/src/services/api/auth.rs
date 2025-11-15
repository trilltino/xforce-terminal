//! # Authentication Endpoints
//!
//! Handles user authentication (login and signup).

use shared::{AuthResponse, ErrorResponse, LoginRequest, SignupRequest};
use super::client::ApiClient;

/// Login with username/email and password.
#[tracing::instrument(skip(client, password), fields(email_or_username = %email_or_username))]
pub async fn login(
    client: &ApiClient,
    email_or_username: String,
    password: String,
) -> Result<AuthResponse, String> {
    tracing::info!("Attempting login");
    let start = std::time::Instant::now();

    let request = LoginRequest {
        email_or_username,
        password,
    };

    let response = client
        .client
        .post(format!("{}/api/auth/login", ApiClient::base_url()))
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Login network error");
            format!("Network error: {}", e)
        })?;

    let status = response.status();
    let duration = start.elapsed();

    if status.is_success() {
        let result = response
            .json::<AuthResponse>()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Login response parse error");
                format!("Failed to parse response: {}", e)
            });

        if result.is_ok() {
            tracing::info!(duration_ms = duration.as_millis(), "Login successful");
        }
        result
    } else {
        let error = response
            .json::<ErrorResponse>()
            .await
            .map_err(|e| format!("Failed to parse error: {}", e))?;

        tracing::warn!(
            status = status.as_u16(),
            error = %error.error,
            duration_ms = duration.as_millis(),
            "Login failed"
        );
        Err(error.error)
    }
}

/// Sign up a new user.
pub async fn signup(
    client: &ApiClient,
    username: String,
    email: String,
    password: String,
) -> Result<AuthResponse, String> {
    let request = SignupRequest {
        username,
        email,
        password,
    };

    let response = client
        .client
        .post(format!("{}/api/auth/signup", ApiClient::base_url()))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json::<AuthResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        let error = response
            .json::<ErrorResponse>()
            .await
            .map_err(|e| format!("Failed to parse error: {}", e))?;
        Err(error.error)
    }
}

