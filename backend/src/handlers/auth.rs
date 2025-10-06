use crate::{
    auth::{encode_jwt, hash_password, verify_password},
    config::Config,
    database::{repository::UserRepository, DbPool},
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use shared::{AuthResponse, ErrorResponse, LoginRequest, SignupRequest, UserInfo};
use tracing::{debug, error, info, warn};

/// Signup handler - creates a new user account
pub async fn signup(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(req): Json<SignupRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)> {
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("[SIGNUP] 🔐 NEW USER SIGNUP REQUEST");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    debug!("   Username: {}", req.username);
    debug!("   Email: {}", req.email);

    // Validate input
    if req.username.len() < 3 {
        warn!("[SIGNUP] ❌ Username too short");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Username must be at least 3 characters".to_string(),
            }),
        ));
    }

    if !req.email.contains('@') {
        warn!("[SIGNUP] ❌ Invalid email format");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid email format".to_string(),
            }),
        ));
    }

    // Check if email already exists
    match UserRepository::find_by_email(&pool, &req.email).await {
        Ok(Some(_)) => {
            warn!("[SIGNUP] ❌ Email already registered: {}", req.email);
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Email already registered".to_string(),
                }),
            ));
        }
        Ok(None) => {}
        Err(e) => {
            error!("[SIGNUP] ❌ Database error checking email: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            ));
        }
    }

    // Check if username already exists
    match UserRepository::find_by_username(&pool, &req.username).await {
        Ok(Some(_)) => {
            warn!("[SIGNUP] ❌ Username already taken: {}", req.username);
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Username already taken".to_string(),
                }),
            ));
        }
        Ok(None) => {}
        Err(e) => {
            error!("[SIGNUP] ❌ Database error checking username: {}", e);
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
            warn!("[SIGNUP] ❌ Password hashing failed: {}", e);
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
            error!("[SIGNUP] ❌ Failed to create user: {}", e);
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
            error!("[SIGNUP] ❌ JWT encoding failed: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to generate token".to_string(),
                }),
            ));
        }
    };

    info!("[SIGNUP] ✅ User created and authenticated!");
    info!("   User ID: {}", user.id);
    info!("   Username: {}", user.username);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            user: UserInfo {
                id: user.id.to_string(),
                username: user.username,
                email: user.email,
                created_at: user.created_at.to_string(),
            },
            token,
            message: "Signup successful".to_string(),
        }),
    ))
}

/// Login handler - authenticates existing user
pub async fn login(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)> {
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("[LOGIN] 🔓 LOGIN ATTEMPT");
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
            warn!("[LOGIN] ❌ User not found: {}", req.email_or_username);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid credentials".to_string(),
                }),
            ));
        }
        Err(e) => {
            error!("[LOGIN] ❌ Database error: {}", e);
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
        warn!("[LOGIN] ❌ Account deactivated: {}", user.username);
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
            error!("[LOGIN] ❌ Password verification error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Authentication error".to_string(),
                }),
            ));
        }
    };

    if !is_valid {
        warn!("[LOGIN] ❌ Invalid password for user: {}", user.username);
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
            error!("[LOGIN] ❌ JWT encoding failed: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to generate token".to_string(),
                }),
            ));
        }
    };

    info!("[LOGIN] ✅ User authenticated successfully!");
    info!("   User ID: {}", user.id);
    info!("   Username: {}", user.username);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            user: UserInfo {
                id: user.id.to_string(),
                username: user.username,
                email: user.email,
                created_at: user.created_at.to_string(),
            },
            token,
            message: "Login successful".to_string(),
        }),
    ))
}
