# Crates

Internal Rust library crates for XForce Terminal. Organized into libraries (`libs/`) and utilities (`utils/`).

## Structure

```
crates/
├── libs/              # Reusable library crates
│   ├── lib-auth/      # Authentication primitives
│   ├── lib-core/      # Core domain models
│   ├── lib-solana/    # Solana integration
│   ├── lib-utils/     # Utility functions
│   └── lib-web/       # Web server components
└── utils/             # Standalone utilities
    └── clear-users/   # User management tool
```

## Usage

These crates are workspace members and can be imported as dependencies:

```toml
[dependencies]
lib-core = { path = "../crates/libs/lib-core" }
lib-solana = { path = "../crates/libs/lib-solana" }
```

## Design Principles

- **Single Responsibility**: Each crate has a focused purpose
- **Minimal Dependencies**: Avoid unnecessary external crates
- **Well-Documented**: Comprehensive rustdocs
- **Tested**: Unit tests for critical logic
