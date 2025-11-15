//! Application constants

pub const API_BASE: &str = "http://127.0.0.1:3001";

// Token mints (Solana mainnet/devnet)
pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

// Default slippage options (in basis points)
pub const SLIPPAGE_OPTIONS: &[(u16, &str)] = &[
    (10, "0.1%"),
    (50, "0.5%"),
    (100, "1.0%"),
    (300, "3.0%"),
];

pub const DEFAULT_SLIPPAGE_BPS: u16 = 50; // 0.5%

// UI constants
pub const QUOTE_DEBOUNCE_MS: u32 = 500;
pub const PRICE_UPDATE_INTERVAL_MS: u32 = 5000;
