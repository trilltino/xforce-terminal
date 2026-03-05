# Backend

Axum-based REST API server for XForce Terminal. Provides authentication, market data, swap execution, and real-time WebSocket feeds.

## Overview

The backend serves as the central API hub for XForce Terminal, handling:
- User authentication and session management
- Market data aggregation from multiple sources
- Swap quote generation and execution
- Real-time price updates via WebSocket
- Social features (friends, chat)

## Architecture

```
backend/
├── Cargo.toml          # Dependencies
├── src/
│   └── main.rs        # Application entry point
```

## Dependencies

- **axum**: Web framework
- **tokio**: Async runtime
- **sqlx**: Database access
- **lib-core**: Core models and configuration
- **lib-web**: Web handlers and middleware

## Running

```bash
cargo run
```

Server starts at `http://localhost:8080`

## Environment Variables

```env
DATABASE_URL=postgres://user:pass@localhost/xforce
JWT_SECRET=your-secret-key
RPC_URL=https://api.devnet.solana.com
PORT=8080
```
