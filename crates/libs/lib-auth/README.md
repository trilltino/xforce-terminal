# lib-auth

Authentication primitives for XForce Terminal. Provides JWT token management and secure password hashing using Argon2.

## Features

- JWT token generation and validation
- Argon2 password hashing
- Token expiration handling
- Secure random generation

## API

### Password Hashing

```rust
use lib_auth::pwd::{hash_pwd, verify_pwd};

// Hash password
let hash = hash_pwd("user_password").await?;

// Verify password
let valid = verify_pwd("user_password", &hash).await?;
```

### JWT Tokens

```rust
use lib_auth::token::{generate_token, validate_token};

// Generate token
let token = generate_token(user_id, "secret_key", 3600)?;

// Validate token
let claims = validate_token(&token, "secret_key")?;
```

## Security

- Argon2id with recommended parameters
- JWT HS256 signing
- Constant-time comparison for passwords
