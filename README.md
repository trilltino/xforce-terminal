# XForce Terminal

A comprehensive non-custodial Solana DeFi trading platform featuring a desktop terminal application, web wallet interface, and robust backend API. Built with Rust, React, and Tauri.

## Overview

XForce Terminal provides traders with professional-grade tools for interacting with the Solana DeFi ecosystem. The platform combines a Bloomberg-style desktop terminal, a web-based wallet interface, and a high-performance backend API to deliver real-time market data, swap execution, and portfolio management.

## Project Structure

```
xforce-terminal/
├── backend/              # Axum REST API server
├── crates/               # Internal Rust library crates
│   ├── libs/
│   │   ├── lib-auth/     # Authentication and JWT handling
│   │   ├── lib-core/     # Core models, errors, and configuration
│   │   ├── lib-solana/   # Solana blockchain integration
│   │   ├── lib-utils/    # Utility functions and helpers
│   │   └── lib-web/      # Web server handlers and middleware
│   └── utils/
│       └── clear-users/  # User management utility
├── terminal-tauri/       # Desktop terminal application (Tauri + React)
├── wallet-react/         # Web wallet interface (React)
├── docs/                 # HTML documentation
├── idl/                  # Solana program IDL files
└── migrations/           # Database migration scripts
```

## Key Features

### Desktop Terminal
- Real-time price charts with lightweight-charts
- Multi-wallet support (Phantom, Solflare, Backpack)
- Jupiter aggregator integration for optimal swap routing
- Resizable Bloomberg-style panel layout
- WebSocket price feeds for live market data

### Web Wallet
- Connect any Solana wallet adapter
- View token balances and transaction history
- Interactive starfield animation background
- Responsive design for mobile and desktop

### Backend API
- JWT-based authentication with Argon2 password hashing
- Market data from Pyth Network and Jupiter
- Swap quote generation and execution
- WebSocket real-time price feeds
- RESTful API design with comprehensive error handling

## Technology Stack

### Backend
- **Framework**: Axum (Rust)
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT tokens with Argon2
- **Solana Integration**: solana-client, Jupiter API, Pyth Network
- **WebSocket**: tokio-tungstenite

### Desktop Terminal
- **Framework**: Tauri v2
- **Frontend**: React 18 with TypeScript
- **Styling**: Tailwind CSS
- **Charts**: lightweight-charts
- **State Management**: Zustand

### Web Wallet
- **Framework**: React 18 with TypeScript
- **Wallet Adapter**: Solana Wallet Adapter
- **Styling**: Tailwind CSS
- **Animations**: Custom canvas-based starfield

## Quick Start

### Prerequisites
- Rust 1.70+
- Node.js 18+
- PostgreSQL 14+
- Solana CLI (optional, for local testing)

### Backend

```bash
cd backend
cargo run
```

Server starts at `http://localhost:8080`

### Desktop Terminal

```bash
cd terminal-tauri/src-ui
npm install
cd ..
cargo tauri dev
```

### Web Wallet

```bash
cd wallet-react
npm install
npm run dev
```

Access at `http://localhost:5173`

## API Endpoints

### Authentication
- `POST /api/auth/login` - User login
- `POST /api/auth/signup` - User registration
- `POST /api/auth/logout` - User logout

### Market Data
- `GET /api/market/prices` - Get token prices
- `GET /api/market/candles` - Get OHLCV candle data
- `WS /api/ws/prices` - WebSocket price feed

### Swaps
- `POST /api/swap/quote` - Get swap quote
- `POST /api/swap/execute` - Execute swap transaction

### Wallet
- `GET /api/wallet/balance` - Get wallet balances
- `GET /api/wallet/transactions` - Get transaction history

### Social
- `GET /api/friends` - List friends
- `POST /api/friends/add` - Add friend
- `GET /api/chat/history` - Chat history
- `WS /api/chat` - Real-time chat

## Library Crates

### lib-auth
Authentication primitives including JWT token generation/validation and Argon2 password hashing.

### lib-core
Core domain models, database configurations, error types, and DTOs for auth, market, and messaging.

### lib-solana
Solana blockchain integration including:
- RPC client management
- Jupiter aggregator API
- Pyth price feeds
- SPL token operations
- Transaction building and caching
- Candle aggregation

### lib-utils
Utility functions for base64 encoding/decoding, environment variable loading, time formatting, and input validation.

### lib-web
Web server components including HTTP handlers, middleware (auth, logging, request stamping), routing, and services for market data, swaps, and transactions.

## Configuration

Environment variables (see `.env.example`):

```env
DATABASE_URL=postgres://user:pass@localhost/xforce
JWT_SECRET=your-secret-key
RPC_URL=https://api.devnet.solana.com
JUPITER_API_URL=https://quote-api.jup.ag/v6
PYTH_BENCHMARKS_URL=https://benchmarks.pyth.network
```

## Development

### Building

```bash
# Build all crates
cargo build --release

# Build specific crate
cargo build -p lib-solana --release
```

### Testing

```bash
# Run all tests
cargo test

# Run with logging
cargo test -- --nocapture
```

### Database Migrations

```bash
# Apply migrations
sqlx migrate run

# Create new migration
sqlx migrate add <name>
```

## Scripts

- `start.bat` - Start production backend
- `start-web.bat` - Start web wallet
- `start-debug-viewer.bat` - Start with debug logging
- `build.bat` - Build all components

## License

Apache-2.0 - See [LICENSE](LICENSE) for details.

## Security

This project handles cryptocurrency transactions. Always:
- Review transaction details before signing
- Use hardware wallets when possible
- Test on devnet before mainnet usage
- Keep private keys secure and never commit them

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request
