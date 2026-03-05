# Wallet React

Web wallet interface for XForce Terminal. React application for Solana wallet management.

## Overview

A web-based wallet interface allowing users to connect their Solana wallets, view balances, and sign transactions. Features an interactive starfield background animation.

## Features

- Connect any Solana wallet (Phantom, Solflare, etc.)
- View token balances and transaction history
- Sign and send transactions
- Responsive design
- Starfield animation background

## Structure

```
wallet-react/
├── src/
│   ├── components/
│   │   └── Starfield.tsx    # Background animation
│   ├── pages/
│   │   ├── Connect.tsx      # Wallet connection
│   │   ├── WalletSetup.tsx  # Initial setup
│   │   ├── SignTransaction.tsx # Transaction signing
│   │   └── Status.tsx       # Status display
│   ├── stores/
│   │   └── walletStore.ts   # Wallet state
│   ├── App.tsx
│   ├── index.css
│   └── main.tsx
├── index.html
├── package.json
└── postcss.config.js
```

## Development

### Setup

```bash
npm install
```

### Run

```bash
npm run dev
```

Access at `http://localhost:5173`

### Build

```bash
npm run build
```

## Technologies

- **React 18**: UI framework
- **TypeScript**: Type safety
- **Tailwind CSS**: Styling
- **Solana Wallet Adapter**: Wallet connection
- **Zustand**: State management
