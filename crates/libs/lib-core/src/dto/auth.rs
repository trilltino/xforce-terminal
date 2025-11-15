//! # Authentication Data Transfer Objects
//!
//! Defines request and response structures for authentication endpoints.
//!
//! ## Overview
//!
//! This module contains all DTOs for:
//! - **Traditional auth**: Login/signup with email/password
//! - **Wallet authentication**: Phantom/Solana wallet integration
//! - **User information**: Public user data exchange
//! - **Error handling**: Standard error responses
//!
//! ## Endpoints Using These DTOs
//!
//! ### Traditional Authentication
//! - `POST /api/auth/signup` - [`SignupRequest`] -> [`AuthResponse`]
//! - `POST /api/auth/login` - [`LoginRequest`] -> [`AuthResponse`]
//!
//! ### Wallet Authentication Flow
//! 1. `GET /api/wallet/setup/validate?token=...` - [`WalletSetupValidateRequest`] (query) -> [`WalletSetupValidateResponse`]
//! 2. `POST /api/wallet/setup/complete` - [`WalletSetupCompleteRequest`] -> [`WalletSetupCompleteResponse`]
//! 3. `POST /api/wallet/login` - [`WalletLoginRequest`] -> [`AuthResponse`]
//!
//! ## Wire Format
//!
//! All DTOs use **snake_case** field names in JSON (default serde behavior).
//! Optional fields are omitted when `None` using `#[serde(skip_serializing_if = "Option::is_none")]`.
//!
//! ## Authentication Flow Examples
//!
//! ### Traditional Login Flow
//!
//! ```text
//! POST /api/auth/login
//! Content-Type: application/json
//!
//! {
//!   "email_or_username": "alice",
//!   "password": "MyPassword123!"
//! }
//! ```
//!
//! Response:
//! ```text
//! {
//!   "user": {
//!     "id": "1",
//!     "username": "alice",
//!     "email": "alice@example.com",
//!     "created_at": "2024-01-01T00:00:00Z"
//!   },
//!   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
//!   "message": "Login successful"
//! }
//! ```
//!
//! ### Wallet Authentication Flow
//!
//! Step 1: Validate setup token
//! ```text
//! GET /api/wallet/setup/validate?token=abc123
//! ```
//!
//! Response:
//! ```text
//! {
//!   "valid": true,
//!   "username": "alice",
//!   "challenge": "Sign this message: 8f3d9a2b..."
//! }
//! ```
//!
//! Step 2: Complete wallet setup with signature
//! ```text
//! POST /api/wallet/setup/complete
//! Content-Type: application/json
//!
//! {
//!   "setup_token": "abc123",
//!   "wallet_address": "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde",
//!   "signature": "base58_encoded_signature",
//!   "challenge": "Sign this message: 8f3d9a2b..."
//! }
//! ```
//!
//! Response:
//! ```text
//! {
//!   "success": true,
//!   "message": "Wallet connected successfully"
//! }
//! ```
//!
//! ## Example Usage
//!
//! ```rust
//! use shared::dto::auth::{LoginRequest, AuthResponse};
//! use serde_json;
//!
//! // Serialize a login request
//! let request = LoginRequest {
//!     email_or_username: "alice@example.com".to_string(),
//!     password: "SecurePassword123!".to_string(),
//! };
//! let json = serde_json::to_string(&request).unwrap();
//!
//! // Deserialize an auth response
//! # let response_json = r#"{"user":{"id":"1","username":"alice","email":"alice@example.com","created_at":"2024-01-01"},"token":"jwt_token","message":"Login successful"}"#;
//! let response: AuthResponse = serde_json::from_str(response_json).unwrap();
//! assert_eq!(response.user.username, "alice");
//! ```

use serde::{Deserialize, Serialize};

/// Login request with email or username.
///
/// Supports login with either email address or username for flexibility.
///
/// # Fields
///
/// * `email_or_username` - Can be either email (e.g., "alice@example.com") or username (e.g., "alice")
/// * `password` - Plaintext password (will be hashed server-side)
///
/// # Example
///
/// ```rust
/// use shared::dto::auth::LoginRequest;
///
/// let request = LoginRequest {
///     email_or_username: "alice".to_string(),
///     password: "MyPassword123!".to_string(),
/// };
/// ```
///
/// # Security Note
///
/// Password is sent in plaintext over HTTPS. Server immediately hashes it using Argon2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginRequest {
    pub email_or_username: String,
    pub password: String,
}

/// Signup request for new user registration.
///
/// Used by the `POST /api/auth/signup` endpoint to create a new user account.
///
/// # Fields
///
/// * `username` - Unique username (3-20 chars, alphanumeric + underscore)
/// * `email` - Valid email address (must be unique)
/// * `password` - Plaintext password (min 8 chars, will be hashed server-side with Argon2)
///
/// # Validation Rules (Server-Side)
///
/// - Username must be 3-20 characters, alphanumeric or underscore
/// - Email must be valid format and not already registered
/// - Password must be at least 8 characters
/// - Both username and email must be unique in the database
///
/// # JSON Example
///
/// ```json
/// {
///   "username": "alice",
///   "email": "alice@example.com",
///   "password": "SecurePassword123!"
/// }
/// ```
///
/// # Response
///
/// On success, returns [`AuthResponse`] with user info and JWT token.
/// On failure, returns [`ErrorResponse`] with validation errors.
///
/// # Security Note
///
/// Password is sent in plaintext over HTTPS. Server immediately hashes it using Argon2
/// before storing. Never log or store passwords in plaintext.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Authentication response returned on successful login or signup.
///
/// Used by:
/// - `POST /api/auth/login` - Traditional login
/// - `POST /api/auth/signup` - New user registration
/// - `POST /api/wallet/login` - Wallet authentication
///
/// # Fields
///
/// * `user` - Public user information (see [`UserInfo`])
/// * `token` - JWT authentication token for subsequent API requests
/// * `message` - Human-readable success message
/// * `wallet_setup_required` - Optional flag indicating if wallet setup is needed
/// * `wallet_setup_token` - Optional token for wallet setup flow
///
/// # Serialization
///
/// - Optional fields (`wallet_setup_required`, `wallet_setup_token`) are **omitted from JSON** when `None`
/// - Uses `#[serde(skip_serializing_if = "Option::is_none")]` for cleaner responses
///
/// # JSON Example (Basic Login)
///
/// ```json
/// {
///   "user": {
///     "id": "1",
///     "username": "alice",
///     "email": "alice@example.com",
///     "created_at": "2024-01-01T00:00:00Z"
///   },
///   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwiZXhwIjoxNjQwOTk1MjAwfQ.xyz",
///   "message": "Login successful"
/// }
/// ```
///
/// # JSON Example (With Wallet Setup Required)
///
/// ```json
/// {
///   "user": {
///     "id": "2",
///     "username": "bob",
///     "email": "bob@example.com",
///     "created_at": "2024-01-02T00:00:00Z"
///   },
///   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.xyz",
///   "message": "Login successful",
///   "wallet_setup_required": true,
///   "wallet_setup_token": "ws_abc123def456"
/// }
/// ```
///
/// # Usage
///
/// The `token` field should be included in subsequent API requests as:
/// ```text
/// Authorization: Bearer <token>
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthResponse {
    pub user: UserInfo,
    pub token: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_setup_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_setup_token: Option<String>,
}

/// User information (public, safe to send to client).
///
/// Contains public user data that can be safely transmitted to the frontend.
/// Never includes sensitive data like password hashes or private keys.
///
/// # Fields
///
/// * `id` - Unique user ID (database primary key as string)
/// * `username` - User's display name (unique)
/// * `email` - User's email address
/// * `created_at` - Account creation timestamp (ISO 8601 format)
/// * `wallet_address` - Optional Solana wallet address (Phantom wallet)
///
/// # Serialization
///
/// - `wallet_address` is **omitted from JSON** when `None`
/// - Uses `#[serde(skip_serializing_if = "Option::is_none")]`
///
/// # JSON Example (Without Wallet)
///
/// ```json
/// {
///   "id": "1",
///   "username": "alice",
///   "email": "alice@example.com",
///   "created_at": "2024-01-01T00:00:00Z"
/// }
/// ```
///
/// # JSON Example (With Wallet)
///
/// ```json
/// {
///   "id": "2",
///   "username": "bob",
///   "email": "bob@example.com",
///   "created_at": "2024-01-02T00:00:00Z",
///   "wallet_address": "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde"
/// }
/// ```
///
/// # Security
///
/// This struct intentionally excludes:
/// - Password hashes
/// - Private keys or seeds
/// - Internal database IDs (other than the public user ID)
/// - Session tokens (sent separately in [`AuthResponse`])
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<String>,
}

/// Standard error response for all API endpoints.
///
/// Returned when any API request fails due to:
/// - Validation errors (invalid input)
/// - Authentication failures (invalid credentials, expired token)
/// - Authorization failures (insufficient permissions)
/// - Server errors (database errors, internal errors)
///
/// # Fields
///
/// * `error` - Human-readable error message describing what went wrong
///
/// # HTTP Status Codes
///
/// This DTO is returned with various HTTP status codes:
/// - `400 Bad Request` - Validation error, malformed request
/// - `401 Unauthorized` - Authentication required or failed
/// - `403 Forbidden` - Authenticated but not authorized
/// - `404 Not Found` - Resource doesn't exist
/// - `409 Conflict` - Duplicate username/email
/// - `500 Internal Server Error` - Server-side error
///
/// # JSON Example
///
/// ```json
/// {
///   "error": "Invalid email or password"
/// }
/// ```
///
/// # Common Error Messages
///
/// - "Invalid email or password" - Login failed
/// - "Username already exists" - Signup conflict
/// - "Email already exists" - Signup conflict
/// - "Invalid token" - JWT validation failed
/// - "Token expired" - JWT expired, need to login again
/// - "Wallet signature verification failed" - Invalid wallet signature
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorResponse {
    pub error: String,
}

/// Price data for charts.
///
/// **DEPRECATED**: This struct is in the wrong module. Use [`crate::dto::market::OHLC`] instead.
///
/// # JSON Example
///
/// ```json
/// {
///   "timestamp": 1704067200,
///   "price": 42150.50
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub timestamp: i64,
    pub price: f64,
}

/// Market data response.
///
/// **DEPRECATED**: This struct is in the wrong module. Use [`crate::dto::market::OHLCResponse`] instead.
///
/// # JSON Example
///
/// ```json
/// {
///   "asset": "SOL/USD",
///   "prices": [
///     {"timestamp": 1704067200, "price": 100.50},
///     {"timestamp": 1704067260, "price": 100.75}
///   ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataResponse {
    pub asset: String,
    pub prices: Vec<PriceData>,
}

// ============================================================================
// WALLET AUTHENTICATION
// ============================================================================

/// Wallet setup validation request (Step 1 of wallet setup).
///
/// Used by `GET /api/wallet/setup/validate?token=<token>` to validate a setup token.
/// This is typically sent as a query parameter, not in the request body.
///
/// # Fields
///
/// * `token` - Temporary setup token from [`AuthResponse::wallet_setup_token`]
///
/// # Flow
///
/// 1. User logs in with email/password
/// 2. Server returns `wallet_setup_token` in [`AuthResponse`]
/// 3. Frontend sends this token to validate and get a challenge
/// 4. Server responds with [`WalletSetupValidateResponse`]
///
/// # JSON Example (Query Parameter)
///
/// ```text
/// GET /api/wallet/setup/validate?token=ws_abc123def456
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletSetupValidateRequest {
    pub token: String,
}

/// Wallet setup validation response (Step 1 response).
///
/// Returned by `GET /api/wallet/setup/validate` with the challenge to sign.
///
/// # Fields
///
/// * `valid` - Whether the setup token is valid and not expired
/// * `username` - Username associated with the setup token
/// * `challenge` - Random challenge string that must be signed by the wallet
///
/// # Security
///
/// The `challenge` is a unique random string generated per request.
/// The user must sign this challenge with their Phantom wallet private key
/// to prove ownership of the wallet address.
///
/// # JSON Example
///
/// ```json
/// {
///   "valid": true,
///   "username": "alice",
///   "challenge": "Sign this message to verify wallet ownership: 8f3d9a2b5c7e1f4d"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletSetupValidateResponse {
    pub valid: bool,
    pub username: String,
    pub challenge: String, // Random challenge for signing
}

/// Wallet setup complete request (Step 2 of wallet setup).
///
/// Used by `POST /api/wallet/setup/complete` to link a Phantom wallet to a user account.
///
/// # Fields
///
/// * `setup_token` - The same token used in Step 1
/// * `wallet_address` - Solana wallet public key (base58 encoded)
/// * `signature` - Ed25519 signature of the challenge (base58 encoded)
/// * `challenge` - The challenge from [`WalletSetupValidateResponse`]
///
/// # Signature Verification
///
/// Server verifies:
/// 1. Setup token is still valid
/// 2. Challenge matches the one generated in Step 1
/// 3. Signature is valid for the wallet address and challenge
/// 4. Wallet address is not already linked to another account
///
/// # JSON Example
///
/// ```json
/// {
///   "setup_token": "ws_abc123def456",
///   "wallet_address": "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde",
///   "signature": "3Bv7wVP9qZ8K1xJ5Y2hN6mR4tL8vS9nC7fD3gE5hK2jM...",
///   "challenge": "Sign this message to verify wallet ownership: 8f3d9a2b5c7e1f4d"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletSetupCompleteRequest {
    pub setup_token: String,
    pub wallet_address: String,
    pub signature: String,
    pub challenge: String,
}

/// Wallet setup complete response (Step 2 response).
///
/// Returned by `POST /api/wallet/setup/complete` after successful wallet linking.
///
/// # Fields
///
/// * `success` - Whether the wallet was successfully linked
/// * `message` - Human-readable success or error message
///
/// # JSON Example (Success)
///
/// ```json
/// {
///   "success": true,
///   "message": "Wallet connected successfully"
/// }
/// ```
///
/// # JSON Example (Failure)
///
/// ```json
/// {
///   "success": false,
///   "message": "Invalid signature"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletSetupCompleteResponse {
    pub success: bool,
    pub message: String,
}

/// Wallet login request (sign challenge to prove wallet ownership).
///
/// Used by `POST /api/wallet/login` to authenticate using a Phantom wallet.
/// No email/password required - authentication is purely cryptographic.
///
/// # Fields
///
/// * `wallet_address` - Solana wallet public key (base58 encoded)
/// * `signature` - Ed25519 signature of the challenge (base58 encoded)
/// * `challenge` - Challenge string that was signed (from separate challenge endpoint)
///
/// # Authentication Flow
///
/// 1. Frontend requests a challenge (separate endpoint, not shown here)
/// 2. User signs the challenge with Phantom wallet
/// 3. Frontend sends this request with wallet address, signature, and challenge
/// 4. Server verifies signature and returns [`AuthResponse`] with JWT token
///
/// # Security
///
/// - No passwords involved - uses Ed25519 public key cryptography
/// - Challenge prevents replay attacks
/// - Server verifies the signature using the wallet's public key
///
/// # JSON Example
///
/// ```json
/// {
///   "wallet_address": "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde",
///   "signature": "2Kv8xYQ7pN9L2mJ4Z1hR5tB3vC8nD6fE4gH2kM9jL3nP...",
///   "challenge": "Login challenge: 9b4e7f2a8c5d1e3f6a9b2c5d8e1f4a7b"
/// }
/// ```
///
/// # Response
///
/// On success, returns [`AuthResponse`] with user info and JWT token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletLoginRequest {
    pub wallet_address: String,
    pub signature: String,
    pub challenge: String,
}

/// Transaction submission request.
///
/// Used by `POST /api/transaction/submit` to submit a signed Solana transaction.
///
/// # Fields
///
/// * `transaction` - Base64-encoded signed transaction bytes
/// * `wallet_address` - Solana wallet address that signed the transaction
/// * `transaction_type` - Type of transaction (e.g., "swap", "transfer", "stake")
///
/// # JSON Example
///
/// ```json
/// {
///   "transaction": "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDArczNgag3jGgUOGF4R8d4p4k3gV4qJ3p5k2jL3nP...",
///   "wallet_address": "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde",
///   "transaction_type": "swap"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubmitTransactionRequest {
    pub transaction: String, // Base64-encoded signed transaction
    pub wallet_address: String,
    pub transaction_type: String,
}

/// Transaction submission response.
///
/// Returned by `POST /api/transaction/submit` after submitting a transaction.
///
/// # Fields
///
/// * `success` - Whether the transaction was successfully submitted
/// * `signature` - Transaction signature if successful
/// * `message` - Human-readable status message
///
/// # JSON Example (Success)
///
/// ```json
/// {
///   "success": true,
///   "signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW",
///   "message": "Transaction submitted successfully"
/// }
/// ```
///
/// # JSON Example (Failure)
///
/// ```json
/// {
///   "success": false,
///   "signature": null,
///   "message": "Transaction failed: insufficient funds"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubmitTransactionResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // ========== LoginRequest Tests ==========

    #[test]
    fn test_login_request_serialize() {
        let request = LoginRequest {
            email_or_username: "alice@example.com".to_string(),
            password: "MyPassword123!".to_string(),
        };

        let json = serde_json::to_string(&request)
            .expect("LoginRequest should serialize to JSON");
        assert!(json.contains("alice@example.com"));
        assert!(json.contains("MyPassword123!"));
    }

    #[test]
    fn test_login_request_deserialize() {
        let json = r#"{"email_or_username":"bob","password":"SecurePass456!"}"#;
        let request: LoginRequest = serde_json::from_str(json)
            .expect("Valid JSON should deserialize to LoginRequest");

        assert_eq!(request.email_or_username, "bob");
        assert_eq!(request.password, "SecurePass456!");
    }

    #[test]
    fn test_login_request_roundtrip() {
        let original = LoginRequest {
            email_or_username: "test@example.com".to_string(),
            password: "TestPassword789!".to_string(),
        };

        let json = serde_json::to_string(&original)
            .expect("LoginRequest should serialize to JSON");
        let deserialized: LoginRequest = serde_json::from_str(&json)
            .expect("Round-trip serialization should succeed");

        assert_eq!(original, deserialized);
    }

    // ========== SignupRequest Tests ==========

    #[test]
    fn test_signup_request_serialize() {
        let request = SignupRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "AlicePassword123!".to_string(),
        };

        let json = serde_json::to_string(&request)
            .expect("SignupRequest should serialize to JSON");
        assert!(json.contains("alice"));
        assert!(json.contains("alice@example.com"));
    }

    #[test]
    fn test_signup_request_deserialize() {
        let json = r#"{"username":"bob","email":"bob@example.com","password":"BobPass456!"}"#;
        let request: SignupRequest = serde_json::from_str(json)
            .expect("Valid JSON should deserialize to SignupRequest");

        assert_eq!(request.username, "bob");
        assert_eq!(request.email, "bob@example.com");
        assert_eq!(request.password, "BobPass456!");
    }

    #[test]
    fn test_signup_request_roundtrip() {
        let original = SignupRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "TestPassword789!".to_string(),
        };

        let json = serde_json::to_string(&original)
            .expect("SignupRequest should serialize to JSON");
        let deserialized: SignupRequest = serde_json::from_str(&json)
            .expect("Round-trip serialization should succeed");

        assert_eq!(original, deserialized);
    }

    // ========== AuthResponse Tests ==========

    #[test]
    fn test_auth_response_serialize() {
        let response = AuthResponse {
            user: UserInfo {
                id: "1".to_string(),
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                wallet_address: None,
            },
            token: "jwt_token_here".to_string(),
            message: "Login successful".to_string(),
            wallet_setup_required: None,
            wallet_setup_token: None,
        };

        let json = serde_json::to_string(&response)
            .expect("AuthResponse should serialize to JSON");
        assert!(json.contains("alice"));
        assert!(json.contains("jwt_token_here"));
    }

    #[test]
    fn test_auth_response_deserialize() {
        let json = r#"{
            "user": {
                "id": "42",
                "username": "bob",
                "email": "bob@example.com",
                "created_at": "2024-01-02T00:00:00Z"
            },
            "token": "token_123",
            "message": "Signup successful"
        }"#;

        let response: AuthResponse = serde_json::from_str(json)
            .expect("Valid JSON should deserialize to AuthResponse");

        assert_eq!(response.user.id, "42");
        assert_eq!(response.user.username, "bob");
        assert_eq!(response.token, "token_123");
        assert_eq!(response.message, "Signup successful");
    }

    #[test]
    fn test_auth_response_with_optional_fields() {
        let response = AuthResponse {
            user: UserInfo {
                id: "1".to_string(),
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                wallet_address: Some("WalletAddress123".to_string()),
            },
            token: "jwt_token".to_string(),
            message: "Login successful".to_string(),
            wallet_setup_required: Some(true),
            wallet_setup_token: Some("setup_token_456".to_string()),
        };

        let json = serde_json::to_string(&response)
            .expect("AuthResponse should serialize to JSON");
        let deserialized: AuthResponse = serde_json::from_str(&json)
            .expect("Round-trip serialization should succeed");

        assert_eq!(response, deserialized);
        assert_eq!(deserialized.wallet_setup_required, Some(true));
        assert_eq!(deserialized.wallet_setup_token, Some("setup_token_456".to_string()));
    }

    #[test]
    fn test_auth_response_optional_fields_omitted() {
        // When optional fields are None, they should be omitted from JSON
        let response = AuthResponse {
            user: UserInfo {
                id: "1".to_string(),
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                wallet_address: None,
            },
            token: "jwt_token".to_string(),
            message: "Login successful".to_string(),
            wallet_setup_required: None,
            wallet_setup_token: None,
        };

        let json = serde_json::to_string(&response)
            .expect("AuthResponse should serialize to JSON");
        assert!(!json.contains("wallet_setup_required"));
        assert!(!json.contains("wallet_setup_token"));
    }

    // ========== UserInfo Tests ==========

    #[test]
    fn test_user_info_serialize() {
        let user = UserInfo {
            id: "123".to_string(),
            username: "charlie".to_string(),
            email: "charlie@example.com".to_string(),
            created_at: "2024-01-03T00:00:00Z".to_string(),
            wallet_address: Some("WalletABC".to_string()),
        };

        let json = serde_json::to_string(&user)
            .expect("UserInfo should serialize to JSON");
        assert!(json.contains("charlie"));
        assert!(json.contains("WalletABC"));
    }

    #[test]
    fn test_user_info_deserialize() {
        let json = r#"{
            "id": "456",
            "username": "dave",
            "email": "dave@example.com",
            "created_at": "2024-01-04T00:00:00Z"
        }"#;

        let user: UserInfo = serde_json::from_str(json)
            .expect("Valid JSON should deserialize to UserInfo");

        assert_eq!(user.id, "456");
        assert_eq!(user.username, "dave");
        assert_eq!(user.email, "dave@example.com");
        assert_eq!(user.wallet_address, None);
    }

    // ========== ErrorResponse Tests ==========

    #[test]
    fn test_error_response_serialize() {
        let error = ErrorResponse {
            error: "Invalid credentials".to_string(),
        };

        let json = serde_json::to_string(&error)
            .expect("ErrorResponse should serialize to JSON");
        assert!(json.contains("Invalid credentials"));
    }

    #[test]
    fn test_error_response_deserialize() {
        let json = r#"{"error":"Database error"}"#;
        let error: ErrorResponse = serde_json::from_str(json)
            .expect("Valid JSON should deserialize to ErrorResponse");

        assert_eq!(error.error, "Database error");
    }

    // ========== WalletSetupValidateRequest Tests ==========

    #[test]
    fn test_wallet_setup_validate_request_roundtrip() {
        let request = WalletSetupValidateRequest {
            token: "setup_token_abc123".to_string(),
        };

        let json = serde_json::to_string(&request)
            .expect("WalletSetupValidateRequest should serialize to JSON");
        let deserialized: WalletSetupValidateRequest = serde_json::from_str(&json)
            .expect("Round-trip serialization should succeed");

        assert_eq!(request, deserialized);
    }

    // ========== WalletSetupValidateResponse Tests ==========

    #[test]
    fn test_wallet_setup_validate_response_roundtrip() {
        let response = WalletSetupValidateResponse {
            valid: true,
            username: "alice".to_string(),
            challenge: "challenge_xyz789".to_string(),
        };

        let json = serde_json::to_string(&response)
            .expect("WalletSetupValidateResponse should serialize to JSON");
        let deserialized: WalletSetupValidateResponse = serde_json::from_str(&json)
            .expect("Round-trip serialization should succeed");

        assert_eq!(response, deserialized);
        assert!(deserialized.valid);
    }

    // ========== WalletSetupCompleteRequest Tests ==========

    #[test]
    fn test_wallet_setup_complete_request_roundtrip() {
        let request = WalletSetupCompleteRequest {
            setup_token: "setup_token_123".to_string(),
            wallet_address: "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde".to_string(),
            signature: "signature_base58".to_string(),
            challenge: "challenge_xyz".to_string(),
        };

        let json = serde_json::to_string(&request)
            .expect("WalletSetupCompleteRequest should serialize to JSON");
        let deserialized: WalletSetupCompleteRequest = serde_json::from_str(&json)
            .expect("Round-trip serialization should succeed");

        assert_eq!(request, deserialized);
    }

    // ========== WalletSetupCompleteResponse Tests ==========

    #[test]
    fn test_wallet_setup_complete_response_roundtrip() {
        let response = WalletSetupCompleteResponse {
            success: true,
            message: "Wallet connected successfully".to_string(),
        };

        let json = serde_json::to_string(&response)
            .expect("WalletSetupCompleteResponse should serialize to JSON");
        let deserialized: WalletSetupCompleteResponse = serde_json::from_str(&json)
            .expect("Round-trip serialization should succeed");

        assert_eq!(response, deserialized);
        assert!(deserialized.success);
    }

    // ========== WalletLoginRequest Tests ==========

    #[test]
    fn test_wallet_login_request_roundtrip() {
        let request = WalletLoginRequest {
            wallet_address: "9aE476sH92Vz7DMPyq5WLPkrKWivxeuTKEFKd2sZZcde".to_string(),
            signature: "signature_base58_encoded".to_string(),
            challenge: "challenge_string".to_string(),
        };

        let json = serde_json::to_string(&request)
            .expect("WalletLoginRequest should serialize to JSON");
        let deserialized: WalletLoginRequest = serde_json::from_str(&json)
            .expect("Round-trip serialization should succeed");

        assert_eq!(request, deserialized);
    }

    // ========== Special Characters and Edge Cases ==========

    #[test]
    fn test_login_request_with_special_chars() {
        let request = LoginRequest {
            email_or_username: "user+tag@example.com".to_string(),
            password: "P@ssw0rd!#$%".to_string(),
        };

        let json = serde_json::to_string(&request)
            .expect("LoginRequest should serialize to JSON");
        let deserialized: LoginRequest = serde_json::from_str(&json)
            .expect("Round-trip serialization should succeed");

        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_signup_request_with_unicode() {
        let request = SignupRequest {
            username: "ユーザー".to_string(),
            email: "user@example.com".to_string(),
            password: "パスワード123".to_string(),
        };

        let json = serde_json::to_string(&request)
            .expect("SignupRequest should serialize to JSON");
        let deserialized: SignupRequest = serde_json::from_str(&json)
            .expect("Round-trip serialization should succeed");

        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_error_response_with_multiline_error() {
        let error = ErrorResponse {
            error: "Line 1\nLine 2\nLine 3".to_string(),
        };

        let json = serde_json::to_string(&error)
            .expect("ErrorResponse should serialize to JSON");
        let deserialized: ErrorResponse = serde_json::from_str(&json)
            .expect("Round-trip serialization should succeed");

        assert_eq!(error, deserialized);
    }
}
