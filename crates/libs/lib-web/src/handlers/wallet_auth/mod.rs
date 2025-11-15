//! # Wallet Authentication Handlers
//!
//! HTTP request handlers for Solana wallet-based authentication.
//!
//! ## Overview
//!
//! This module implements wallet-based authentication flow:
//! - Wallet setup validation (verifying setup tokens)
//! - Wallet linking (connecting Solana wallet to user account via signature verification)
//! - Wallet-based login (sign-in using Solana wallet signature)
//!
//! All wallet operations use Ed25519 signature verification to prove wallet ownership.
//!
//! ## Security
//!
//! - Challenge-response mechanism prevents replay attacks
//! - Ed25519 signature verification using Solana SDK
//! - Setup tokens expire after 30 minutes
//! - One wallet can only be linked to one account
//!
//! ## Example Flow
//!
//! ```text
//! 1. User signs up → receives setup token
//! 2. User calls /validate with setup token → receives challenge
//! 3. User signs challenge with wallet → sends to /complete
//! 4. Server verifies signature → links wallet to account
//! 5. User can now login with wallet signature
//! ```

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use lib_core::dto::{
    ErrorResponse, WalletLoginRequest, WalletSetupCompleteRequest, WalletSetupCompleteResponse,
    WalletSetupValidateRequest, WalletSetupValidateResponse, AuthResponse, UserInfo,
};
use lib_auth::encode_jwt;
use lib_core::{Config, DbPool};
use lib_core::model::store::users;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::str::FromStr;
use tracing::{error, info, warn, instrument};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(FromRow)]
struct TokenUser {
    id: i64,
    username: String,
    wallet_setup_token_expires_at: Option<chrono::NaiveDateTime>,
}

#[derive(FromRow)]
struct BasicUser {
    id: i64,
    username: String,
}

#[derive(FromRow)]
struct UsernameOnly {
    username: String,
}

/// Validate wallet setup token and return challenge for signing.
///
/// # Arguments
///
/// * `db` - Database connection pool
/// * `req` - Query parameters containing the setup token
///
/// # Returns
///
/// * `Ok(WalletSetupValidateResponse)` - Token is valid, returns username and challenge
/// * `Err((StatusCode, ErrorResponse))` - Invalid token, expired token, or database error
///
/// # Security
///
/// - Setup tokens expire after 30 minutes
/// - Returns unique challenge (UUID) for signature verification
/// - Challenge should be signed by wallet and sent to `/complete`
///
/// # Example
///
/// ```text
/// GET /api/wallet/setup/validate?token=abc123...
/// Response: { "valid": true, "username": "alice", "challenge": "uuid-here" }
/// ```
#[instrument(skip(db))]
pub async fn validate_wallet_setup(
    State(db): State<DbPool>,
    Query(req): Query<WalletSetupValidateRequest>,
) -> Result<Json<WalletSetupValidateResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("[WALLET AUTH] ========== validate_wallet_setup HANDLER CALLED ==========");
    info!("[WALLET AUTH] Endpoint: /api/wallet/setup/validate");
    info!("[WALLET AUTH] Token length: {}", req.token.len());
    // Safely get token prefix for logging (prevent panic on empty token)
    let token_prefix = if req.token.len() > 20 {
        &req.token[..20]
    } else {
        &req.token[..]
    };
    info!("[WALLET AUTH] Token prefix: {}...", token_prefix);
    info!("[WALLET AUTH] Full token received: {}", req.token);
    info!("[WALLET AUTH] Query params parsed successfully");

    // Find user by setup token
    info!("[WALLET AUTH] Executing database query for token...");
    let user: Option<TokenUser> = sqlx::query_as(
        "SELECT id, username, wallet_setup_token_expires_at FROM users WHERE wallet_setup_token = ?"
    )
    .bind(&req.token)
    .fetch_optional(&db)
    .await
    .map_err(|e: sqlx::Error| {
        error!("[WALLET AUTH] Database error: {}", e);
        error!("[WALLET AUTH] Database error details: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
    })?;
    
    info!("[WALLET AUTH] Database query completed. User found: {}", user.is_some());

    let user = match user {
        Some(u) => {
            info!("[WALLET AUTH] User found: id={}, username={}", u.id, u.username);
            u
        },
        None => {
            // Safely get token prefix for logging
            let token_prefix = if req.token.len() > 20 {
                &req.token[..20]
            } else {
                &req.token[..]
            };
            warn!("[WALLET AUTH] No user found with token: {}...", token_prefix);
            warn!("[WALLET AUTH] Token does not exist in database");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid or expired setup token".into(),
                }),
            ));
        }
    };

    // Check if token is expired (30 minutes validity)
    let now = chrono::Utc::now().naive_utc();
    info!("[WALLET AUTH] Checking token expiration. Current time: {}", now);
    if let Some(expires_at) = user.wallet_setup_token_expires_at {
        info!("[WALLET AUTH] Token expires at: {}", expires_at);
        if now > expires_at {
            warn!("[WALLET AUTH] Setup token expired. Now: {}, Expires: {}", now, expires_at);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Setup token expired. Please signup again.".into(),
                }),
            ));
        }
        info!("[WALLET AUTH] Token is still valid (not expired)");
    } else {
        warn!("[WALLET AUTH] Token has no expiration time set");
    }

    // Generate challenge for signing
    let challenge = Uuid::new_v4().to_string();
    info!("[WALLET AUTH] Generated challenge: {}", challenge);

    info!("[WALLET AUTH] VALIDATION SUCCESSFUL");
    info!("[WALLET AUTH] Returning response for user: {}", user.username);

    Ok(Json(WalletSetupValidateResponse {
        valid: true,
        username: user.username,
        challenge,
    }))
}

/// Complete wallet setup by verifying signature and linking wallet to user.
///
/// # Arguments
///
/// * `db` - Database connection pool
/// * `req` - Request containing setup token, wallet address, signature, and challenge
///
/// # Returns
///
/// * `Ok(WalletSetupCompleteResponse)` - Wallet successfully linked to account
/// * `Err((StatusCode, ErrorResponse))` - Invalid signature, duplicate wallet, or other error
///
/// # Security
///
/// - Verifies Ed25519 signature using Solana SDK
/// - Message format: "Connect wallet to XForce Terminal\n\nChallenge: {challenge}"
/// - Prevents one wallet from being linked to multiple accounts
/// - Clears setup token after successful linking
///
/// # Validation
///
/// - Wallet address must be valid Solana public key
/// - Signature must be valid base58 encoded Ed25519 signature
/// - Signature must match the challenge message
/// - Setup token must exist and be valid
/// - Wallet must not already be linked to another account
///
/// # Example
///
/// ```text
/// POST /api/wallet/setup/complete
/// Body: {
///   "setup_token": "abc123...",
///   "wallet_address": "7xKXtg...",
///   "signature": "5J7B...",
///   "challenge": "uuid-from-validate"
/// }
/// ```
#[instrument(skip(db))]
pub async fn complete_wallet_setup(
    State(db): State<DbPool>,
    Json(req): Json<WalletSetupCompleteRequest>,
) -> Result<Json<WalletSetupCompleteResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "[WALLET AUTH] Completing wallet setup for address: {}",
        req.wallet_address
    );

    // 1. Verify wallet address format
    let wallet_pubkey = Pubkey::from_str(&req.wallet_address).map_err(|e| {
        warn!("[WALLET AUTH] Invalid wallet address format: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid Solana wallet address".into(),
            }),
        )
    })?;

    // 2. Verify signature
    let signature = Signature::from_str(&req.signature).map_err(|e| {
        warn!("[WALLET AUTH] Invalid signature format: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid signature format".into(),
            }),
        )
    })?;

    // 3. Construct message that was signed
    let message = format!(
        "Connect wallet to XForce Terminal\n\nChallenge: {}",
        req.challenge
    );

    // 4. Verify Ed25519 signature
    if !signature.verify(wallet_pubkey.as_ref(), message.as_bytes()) {
        warn!("[WALLET AUTH] Signature verification failed");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Signature verification failed".into(),
            }),
        ));
    }

    info!("[WALLET AUTH] Signature verified!");

    // 5. Find user by setup token
    let user: Option<BasicUser> = sqlx::query_as(
        "SELECT id, username FROM users WHERE wallet_setup_token = ?"
    )
    .bind(&req.setup_token)
    .fetch_optional(&db)
    .await
    .map_err(|e: sqlx::Error| {
        error!("[WALLET AUTH] Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".into(),
            }),
        )
    })?;

    let user = match user {
        Some(u) => u,
        None => {
            warn!("[WALLET AUTH] Invalid setup token during completion");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid setup token".into(),
                }),
            ));
        }
    };

    // 6. Check if wallet is already linked to another user
    let existing_wallet: Option<UsernameOnly> = sqlx::query_as(
        "SELECT username FROM users WHERE wallet_address = ?"
    )
    .bind(&req.wallet_address)
    .fetch_optional(&db)
    .await
    .map_err(|e: sqlx::Error| {
        error!("[WALLET AUTH] Database error checking wallet: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".into(),
            }),
        )
    })?;

    if let Some(existing) = existing_wallet {
        warn!(
            "[WALLET AUTH] Wallet already linked to user: {}",
            existing.username
        );
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!(
                    "This wallet is already connected to another account: {}",
                    existing.username
                ),
            }),
        ));
    }

    // 7. Link wallet to user account
    let now = chrono::Utc::now().naive_utc();
    sqlx::query(
        "UPDATE users SET wallet_address = ?, wallet_connected_at = ?, wallet_setup_token = NULL, wallet_setup_token_expires_at = NULL WHERE id = ?"
    )
    .bind(&req.wallet_address)
    .bind(now)
    .bind(user.id)
    .execute(&db)
    .await
    .map_err(|e: sqlx::Error| {
        error!("[WALLET AUTH] Failed to link wallet: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to link wallet to account".into(),
            }),
        )
    })?;

    info!(
        "[WALLET AUTH] Wallet {} successfully linked to user {}",
        req.wallet_address, user.username
    );

    Ok(Json(WalletSetupCompleteResponse {
        success: true,
        message: format!("Wallet successfully connected to {}", user.username),
    }))
}

/// Login with wallet by verifying signature to prove wallet ownership.
///
/// # Arguments
///
/// * `db` - Database connection pool
/// * `config` - Application configuration (JWT secret and expiration)
/// * `req` - Request containing wallet address, signature, and challenge
///
/// # Returns
///
/// * `Ok(AuthResponse)` - Authentication successful with JWT token and user info
/// * `Err((StatusCode, ErrorResponse))` - Invalid signature, wallet not linked, or other error
///
/// # Security
///
/// - Verifies Ed25519 signature using Solana SDK
/// - Message format: "Login to XForce Terminal\n\nChallenge: {challenge}"
/// - Challenge should be generated client-side (UUID) to prevent replay attacks
/// - Generates JWT token for stateless authentication
///
/// # Validation
///
/// - Wallet address must be valid Solana public key
/// - Signature must be valid Ed25519 signature
/// - Signature must match the challenge message
/// - Wallet must be linked to an existing user account
///
/// # Example
///
/// ```text
/// POST /api/wallet/login
/// Body: {
///   "wallet_address": "7xKXtg...",
///   "signature": "5J7B...",
///   "challenge": "client-generated-uuid"
/// }
/// Response: {
///   "user": { ... },
///   "token": "jwt-token-here",
///   "message": "Successfully logged in with wallet"
/// }
/// ```
#[instrument(skip(db, config))]
pub async fn wallet_login(
    State(db): State<DbPool>,
    State(config): State<Config>,
    Json(req): Json<WalletLoginRequest>,
) -> Result<
    Json<AuthResponse>,
    (StatusCode, Json<ErrorResponse>),
> {
    info!("[WALLET LOGIN] Attempting wallet login: {}", req.wallet_address);

    // 1. Verify wallet address format
    let wallet_pubkey = Pubkey::from_str(&req.wallet_address).map_err(|e| {
        warn!("[WALLET LOGIN] Invalid wallet address: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid wallet address".into(),
            }),
        )
    })?;

    // 2. Verify signature
    let signature = Signature::from_str(&req.signature).map_err(|e| {
        warn!("[WALLET LOGIN] Invalid signature: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid signature".into(),
            }),
        )
    })?;

    // 3. Construct message
    let message = format!("Login to XForce Terminal\n\nChallenge: {}", req.challenge);

    // 4. Verify signature
    if !signature.verify(wallet_pubkey.as_ref(), message.as_bytes()) {
        warn!("[WALLET LOGIN] Signature verification failed");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid signature".into(),
            }),
        ));
    }

    info!("[WALLET LOGIN] Signature verified");

    // 5. Find user by wallet address
    let user = users::find_by_wallet(&db, &req.wallet_address)
        .await
        .map_err(|e| {
            error!("[WALLET LOGIN] Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".into(),
                }),
            )
        })?;

    let user = match user {
        Some(u) => u,
        None => {
            warn!("[WALLET LOGIN] No user found with wallet: {}", req.wallet_address);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "No account found with this wallet. Please connect your wallet to an account first.".into(),
                }),
            ));
        }
    };

    // 6. Generate JWT token
    let token = encode_jwt(
        user.id,
        user.username.clone(),
        &config.jwt_secret,
        config.jwt_expiration_hours,
    ).map_err(|e| {
        error!("[WALLET LOGIN] JWT error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to generate auth token".into(),
            }),
        )
    })?;

    info!("[WALLET LOGIN] User {} logged in via wallet", user.username);

        Ok(Json(AuthResponse {
            user: UserInfo {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            created_at: user.created_at.to_string(),
            wallet_address: Some(req.wallet_address),
        },
        token,
        message: "Successfully logged in with wallet".into(),
        wallet_setup_required: None,
        wallet_setup_token: None,
    }))
}

#[cfg(test)]
mod tests;

