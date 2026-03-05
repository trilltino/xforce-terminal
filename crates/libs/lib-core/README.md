# lib-core

Core domain models, error types, database configuration, and DTOs for XForce Terminal.

## Modules

### config.rs
Application configuration loaded from environment variables.

### error.rs
Centralized error types and handling for the application.

### dto/
Data transfer objects for API requests and responses:
- `auth.rs` - Authentication DTOs
- `market.rs` - Market data DTOs
- `messaging.rs` - Chat/message DTOs

### model/
Domain models and database types:
- `store/models.rs` - Database entities
- `store/users.rs` - User repository
- `store/swap_repository.rs` - Swap data access

## Usage

```rust
use lib_core::config::AppConfig;
use lib_core::error::Result;
use lib_core::dto::auth::LoginRequest;

let config = AppConfig::from_env()?;
```

## Database Schema

Core entities include:
- Users
- Wallets
- Swaps
- Messages
- Friends
