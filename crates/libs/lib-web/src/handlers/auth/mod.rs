//! # Authentication Handlers
//!
//! HTTP request handlers for user authentication endpoints.
//!
//! ## Overview
//!
//! This module implements the authentication flow including:
//! - User signup with email/password
//! - User login with email or username
//! - JWT token generation
//! - Wallet setup token generation
//!
//! ## Example
//!
//! ```rust,no_run
//! use axum::{Router, routing::post};
//! use backend::handlers::auth::{signup, login};
//!
//! let app = Router::new()
//!     .route("/signup", post(signup))
//!     .route("/login", post(login));
//! ```

use lib_auth::{encode_jwt, hash_password, verify_password};
use lib_core::{Config, DbPool, dto::{AuthResponse, ErrorResponse, LoginRequest, SignupRequest, UserInfo}};
use lib_core::model::store::user_repository::UserRepository;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use tracing::{debug, error, info, warn, instrument};

/// Signup handler - creates a new user account.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `config` - Application configuration (JWT secret, expiration)
/// * `req` - Signup request containing username, email, and password
///
/// # Returns
///
/// * `Ok((StatusCode::CREATED, AuthResponse))` - User created successfully with JWT token and wallet setup token
/// * `Err((StatusCode, ErrorResponse))` - Validation error, duplicate user, or server error
///
/// # Validation
///
/// - Username must be at least 3 characters
/// - Email must contain '@' symbol
/// - Email must be unique
/// - Username must be unique
/// - Password must be at least 8 characters (validated in hash_password)
///
/// # Example
///
/// ```rust,no_run
/// use axum::Json;
/// use shared::SignupRequest;
/// use backend::handlers::auth::signup;
///
/// # async fn example() {
/// let request = SignupRequest {
///     username: "alice".to_string(),
///     email: "alice@example.com".to_string(),
///     password: "SecurePassword123!".to_string(),
/// };
/// # }
/// ```
#[instrument(skip(pool, config), fields(username = %req.username, email = %req.email))]
pub async fn signup(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(req): Json<SignupRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)> {
    info!("[SIGNUP]  NEW USER SIGNUP REQUEST");
    debug!("   Username: {}", req.username);
    debug!("   Email: {}", req.email);

    if req.username.len() < 3 {
        warn!("[SIGNUP]  Username too short");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Username must be at least 3 characters".to_string(),
            }),
        ));
    }

    if !req.email.contains('@') {
        warn!("[SIGNUP]  Invalid email format");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid email format".to_string(),
            }),
        ));
    }

    match UserRepository::find_by_email(&pool, &req.email).await {
        Ok(Some(_)) => {
            warn!("[SIGNUP]  Email already registered: {}", req.email);
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Email already registered".to_string(),
                }),
            ));
        }
        Ok(None) => {}
        Err(e) => {
            error!("[SIGNUP]  Database error checking email: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            ));
        }
    }

    match UserRepository::find_by_username(&pool, &req.username).await {
        Ok(Some(_)) => {
            warn!("[SIGNUP]  Username already taken: {}", req.username);
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Username already taken".to_string(),
                }),
            ));
        }
        Ok(None) => {}
        Err(e) => {
            error!("[SIGNUP]  Database error checking username: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            ));
        }
    }

    // Hash password
    debug!("[SIGNUP] Hashing password...");
    let password_hash = match hash_password(&req.password) {
        Ok(hash) => hash,
        Err(e) => {
            warn!("[SIGNUP]  Password hashing failed: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e }),
            ));
        }
    };

    // Create user
    debug!("[SIGNUP] Creating user in database...");
    let user = match UserRepository::create(&pool, &req.username, &req.email, &password_hash).await
    {
        Ok(user) => user,
        Err(e) => {
            error!("[SIGNUP]  Failed to create user: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create user".to_string(),
                }),
            ));
        }
    };

    // Generate JWT
    debug!("[SIGNUP] Generating JWT token...");
    let token = match encode_jwt(
        user.id,
        user.username.clone(),
        &config.jwt_secret,
        config.jwt_expiration_hours,
    ) {
        Ok(token) => token,
        Err(e) => {
            error!("[SIGNUP]  JWT encoding failed: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to generate token".to_string(),
                }),
            ));
        }
    };

    // Generate wallet setup token
    debug!("[SIGNUP] Generating wallet setup token...");
    let setup_token = uuid::Uuid::new_v4().to_string();

    if let Err(e) = lib_core::model::store::users::set_wallet_setup_token(&pool, user.id, &setup_token).await {
        error!("[SIGNUP]  Failed to set wallet setup token: {}", e);
        // Don't fail signup, just log the error
    }

    info!("[SIGNUP] User created and authenticated!");
    info!("   User ID: {}", user.id);
    info!("   Username: {}", user.username);
    // Safely get token prefix for logging (prevent panic on short token)
    let token_prefix = if setup_token.len() > 8 {
        &setup_token[..8]
    } else {
        &setup_token[..]
    };
    info!("   Wallet setup token: {}...", token_prefix);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            user: UserInfo {
                id: user.id.to_string(),
                username: user.username.clone(),
                email: user.email,
                created_at: user.created_at.to_string(),
                wallet_address: None,
            },
            token,
            message: "Signup successful".to_string(),
            wallet_setup_required: Some(true),
            wallet_setup_token: Some(setup_token),
        }),
    ))
}

/// Login handler - authenticates existing user.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `config` - Application configuration (JWT secret, expiration)
/// * `req` - Login request containing email/username and password
///
/// # Returns
///
/// * `Ok((StatusCode::OK, AuthResponse))` - Authentication successful with JWT token
/// * `Err((StatusCode, ErrorResponse))` - Invalid credentials, inactive account, or server error
///
/// # Authentication
///
/// - Accepts either email (contains '@') or username
/// - Verifies password using Argon2
/// - Checks if account is active
/// - Updates last_login timestamp
/// - Generates JWT token with user claims
///
/// # Example
///
/// ```rust,no_run
/// use axum::Json;
/// use shared::LoginRequest;
/// use backend::handlers::auth::login;
///
/// # async fn example() {
/// let request = LoginRequest {
///     email_or_username: "alice@example.com".to_string(),
///     password: "SecurePassword123!".to_string(),
/// };
/// // login(pool, config, Json(request)).await;
/// # }
/// ```
pub async fn login(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)> {
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("[LOGIN]  LOGIN ATTEMPT");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    debug!("   Identifier: {}", req.email_or_username);

    // Find user by email or username
    let user = if req.email_or_username.contains('@') {
        debug!("[LOGIN] Looking up by email...");
        UserRepository::find_by_email(&pool, &req.email_or_username).await
    } else {
        debug!("[LOGIN] Looking up by username...");
        UserRepository::find_by_username(&pool, &req.email_or_username).await
    };

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("[LOGIN]  User not found: {}", req.email_or_username);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid credentials".to_string(),
                }),
            ));
        }
        Err(e) => {
            error!("[LOGIN]  Database error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            ));
        }
    };

    // Check if user is active
    if !user.is_active {
        warn!("[LOGIN]  Account deactivated: {}", user.username);
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Account is deactivated".to_string(),
            }),
        ));
    }

    // Verify password
    debug!("[LOGIN] Verifying password...");
    let is_valid = match verify_password(&req.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            error!("[LOGIN]  Password verification error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Authentication error".to_string(),
                }),
            ));
        }
    };

    if !is_valid {
        warn!("[LOGIN] Invalid password for user: {}", user.username);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        ));
    }

    // Update last login
    debug!("[LOGIN] Updating last login timestamp...");
    let _ = UserRepository::update_last_login(&pool, user.id).await;

    // Generate JWT
    debug!("[LOGIN] Generating JWT token...");
    let token = match encode_jwt(
        user.id,
        user.username.clone(),
        &config.jwt_secret,
        config.jwt_expiration_hours,
    ) {
        Ok(token) => token,
        Err(e) => {
            error!("[LOGIN]  JWT encoding failed: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to generate token".to_string(),
                }),
            ));
        }
    };

    info!("[LOGIN]  User authenticated successfully!");
    info!("   User ID: {}", user.id);
    info!("   Username: {}", user.username);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            user: UserInfo {
                id: user.id.to_string(),
                username: user.username.clone(),
                email: user.email,
                created_at: user.created_at.to_string(),
                wallet_address: user.wallet_address,
            },
            token,
            message: "Login successful".to_string(),
            wallet_setup_required: None,
            wallet_setup_token: None,
        }),
    ))
}

#[cfg(test)]
mod tests;

