# GTK4 Terminal

**Native desktop DeFi trading terminal with GTK4 UI and real-time Stellar market data**

- GTK4 Rust bindings for native cross-platform desktop UI with login/signup forms, spinner states, and async event handling
- Soroban RPC integration: connection pooling, circuit breaker resilience, contract event streaming, state management, and simulation
- Horizon API client for account balances, transaction history, market data (Reflector Oracle prices), and payment operations
- Authentication system: JWT tokens, cookie sessions, PostgreSQL persistence, bcrypt password hashing, and user repository
- Workspace architecture with shared DTOs, Tokio async runtime bridge for glib main loop, and optimized compile profiles
