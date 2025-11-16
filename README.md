# XForce Terminal

![XForce Terminal](xforce-terminal.png)

Non-custodial Solana DeFi trading terminal with Bloomberg-style interface.

## Architecture

- **Native Desktop GUI**: Rust + egui framework for cross-platform terminal application
- **Web Wallet Interface**: Leptos-based WASM application for wallet connection
- **Backend Services**: Rust async backend with PostgreSQL database
- **Modular Library Design**: Shared crates for core, auth, web, Solana, and utilities

## Technology Stack

- **Language**: Rust (edition 2021)
- **Desktop GUI**: egui + eframe for native window rendering
- **Web Framework**: Leptos for reactive WASM frontend
- **Blockchain**: Solana SDK integration
- **DEX Integration**: Jupiter aggregator for swap routing
- **Price Feeds**: Pyth oracle integration
- **Database**: PostgreSQL with SQL migrations
- **Async Runtime**: Tokio for concurrent operations

## Features

- Real-time market data via Jupiter aggregator and Pyth oracles
- Non-custodial wallet management with local keypair signing
- DEX swap execution with optimal route finding
- Transaction history tracking and monitoring
- Bloomberg-style terminal interface for professional trading
- Multi-wallet support (Phantom, Solflare, Backpack)

## Project Structure

- `terminal/` - Native desktop GUI application
- `wallet-web/` - Web-based wallet connection interface
- `backend/` - API server and database management
- `crates/libs/` - Shared library modules
- `shared/` - Common DTOs and utilities
- `migrations/` - Database schema migrations

## License

Apache-2.0

