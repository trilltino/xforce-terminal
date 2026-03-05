# Utils

Standalone utility applications for XForce Terminal.

## Available Utilities

### clear-users
Database utility for clearing user data.

```bash
cd crates/utils/clear-users
cargo run
```

## Creating New Utilities

1. Create directory: `crates/utils/<utility-name>/`
2. Add `Cargo.toml` with binary target
3. Create `src/main.rs`
4. Add to workspace members
