# lib-web

Web server components for XForce Terminal. Provides HTTP handlers, middleware, routing, and services.

## Modules

### server.rs
Axum server setup and configuration.

### types.rs
Common web types and extractors.

### handlers/
HTTP request handlers:
- `mod.rs` - Handler registration
- `auth/` - Authentication handlers
- `wallet_auth/` - Wallet-based authentication
- `market.rs` - Market data endpoints
- `swap.rs` - Swap execution
- `wallet.rs` - Wallet operations
- `transaction.rs` - Transaction management
- `friends.rs` - Social features
- `websocket.rs` - WebSocket handlers
- `contracts.rs` - Contract interactions

### services/
Business logic services:
- `mod.rs` - Service registration
- `market.rs` - Market data service
- `swap.rs` - Swap service
- `transaction.rs` - Transaction service
- `wallet.rs` - Wallet service
- `staking.rs` - Staking operations

### middleware/
HTTP middleware:
- `mw_auth.rs` - Authentication middleware
- `mw_logging.rs` - Request/response logging
- `mw_req_stamp.rs` - Request timestamping
- `mw_res_map.rs` - Response mapping

### chat/
Real-time chat system:
- `mod.rs` - Chat module
- `state.rs` - Chat state management
- `db.rs` - Chat database operations
- `ai_bot.rs` - AI chatbot integration
- `handlers/` - Chat message handlers

### routes/mod.rs
Route definitions.

## Usage

```rust
use lib_web::server::create_server;

let app = create_server(config).await?;
```
