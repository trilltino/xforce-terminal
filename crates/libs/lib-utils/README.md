# lib-utils

Utility functions and helpers for XForce Terminal.

## Modules

### b64.rs
Base64 encoding and decoding utilities.

### envs.rs
Environment variable loading with defaults.

### time.rs
Time formatting and conversion helpers.

### validation.rs
Input validation functions.

## Usage

```rust
use lib_utils::b64::{encode, decode};
use lib_utils::envs::get_env_or_default;
use lib_utils::time::format_timestamp;
use lib_utils::validation::is_valid_pubkey;

// Base64
let encoded = encode(b"data");
let decoded = decode(&encoded)?;

// Environment
let port = get_env_or_default("PORT", "8080");

// Validation
let valid = is_valid_pubkey("11111111111111111111111111111111");
```
