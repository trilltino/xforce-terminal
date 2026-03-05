# lib-solana

Solana blockchain integration library for XForce Terminal. Provides RPC client management, Jupiter aggregator integration, Pyth price feeds, and SPL token operations.

## Features

- **RPC Client**: Connection pooling and request management
- **Jupiter Integration**: Swap quotes and execution via Jupiter aggregator
- **Pyth Price Feeds**: Real-time price data from Pyth Network
- **SPL Tokens**: Token account management and transfers
- **Transaction Building**: Construction and signing of transactions
- **Caching**: Price and account data caching for performance
- **Candle Aggregation**: OHLCV data generation from trades

## Modules

### client.rs
Solana RPC client wrapper with connection management.

### jupiter/
Jupiter aggregator integration:
- `client.rs` - HTTP client for Jupiter API
- `price.rs` - Price fetching and caching
- `quote.rs` - Swap quote generation
- `swap.rs` - Swap execution
- `types.rs` - Jupiter API types

### contracts/
Smart contract interaction:
- `idl_handler.rs` - IDL parsing and account decoding
- `loader.rs` - Program account loading
- `plugin.rs` - Contract plugin system
- `registry.rs` - Program registry
- `transaction_builder.rs` - Transaction construction
- `batch_swap/` - Batch swap contract integration

### pyth.rs
Pyth Network price feed integration.

### spl_token.rs
SPL token program interactions.

### cache.rs
Price and data caching with TTL.

### candle_aggregator.rs
Trade aggregation into OHLCV candles.

## Usage

```rust
use lib_solana::client::SolanaClient;
use lib_solana::jupiter::JupiterClient;

let solana = SolanaClient::new("https://api.devnet.solana.com");
let jupiter = JupiterClient::new();

// Get price
let price = jupiter.get_price("SOL").await?;

// Get swap quote
let quote = jupiter.get_quote(input_mint, output_mint, amount).await?;
```
