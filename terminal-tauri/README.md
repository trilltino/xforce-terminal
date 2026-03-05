# Terminal Tauri

Desktop terminal application for XForce Terminal. Built with Tauri v2 and React.

## Overview

A Bloomberg-style trading terminal providing real-time market data, charting, wallet integration, and swap execution. Combines the security of a desktop application with the flexibility of web technologies.

## Features

- Real-time price charts with lightweight-charts
- Multi-wallet support (Phantom, Solflare, Backpack)
- Jupiter aggregator for optimal swap routing
- Resizable panel layout
- WebSocket price feeds
- Professional trading interface

## Structure

```
terminal-tauri/
├── src-tauri/           # Rust backend (Tauri)
│   ├── src/
│   │   ├── lib.rs      # Main library
│   │   ├── main.rs     # Entry point
│   │   ├── commands/   # Tauri commands
│   │   └── services/   # Backend services
│   ├── Cargo.toml
│   └── tauri.conf.json
└── src-ui/              # React frontend
    ├── src/
    │   ├── components/ # React components
    │   ├── pages/      # Page components
    │   ├── hooks/      # Custom hooks
    │   └── stores/     # Zustand stores
    ├── package.json
    └── vite.config.ts
```

## Development

### Prerequisites
- Rust 1.70+
- Node.js 18+

### Setup

```bash
cd src-ui
npm install
cd ..
```

### Run Development

```bash
cargo tauri dev
```

### Build

```bash
cargo tauri build
```

## Commands

### Market Commands
- `get_price(token)` - Get current price
- `get_candles(token, timeframe)` - Get OHLCV data

### Wallet Commands
- `connect_wallet(adapter)` - Connect wallet
- `get_balance()` - Get wallet balance
- `sign_transaction(tx)` - Sign transaction

### Swap Commands
- `get_quote(params)` - Get swap quote
- `execute_swap(quote)` - Execute swap

## Technologies

- **Tauri v2**: Desktop framework
- **React 18**: UI framework
- **TypeScript**: Type safety
- **Tailwind CSS**: Styling
- **Zustand**: State management
- **lightweight-charts**: Charting
