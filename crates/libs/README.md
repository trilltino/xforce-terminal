# Library Crates

Core library crates providing shared functionality across XForce Terminal applications.

## Available Libraries

### lib-auth
Authentication primitives including JWT handling and password hashing.

### lib-core
Domain models, database types, error handling, and configuration.

### lib-solana
Solana blockchain integration - RPC, Jupiter, Pyth, SPL tokens.

### lib-utils
Common utilities - base64, environment, time, validation.

### lib-web
Web server abstractions - handlers, middleware, routing.

## Adding a New Library

1. Create directory: `crates/libs/lib-<name>/`
2. Add `Cargo.toml` with workspace metadata
3. Create `src/lib.rs` with public API
4. Add to workspace `Cargo.toml`
5. Document in respective README
