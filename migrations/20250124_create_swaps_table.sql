-- Create swaps table for tracking swap transactions
CREATE TABLE IF NOT EXISTS swaps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    signature TEXT NOT NULL UNIQUE,
    input_mint TEXT NOT NULL,
    output_mint TEXT NOT NULL,
    input_amount BIGINT NOT NULL,
    output_amount BIGINT NOT NULL,
    price_impact REAL,
    slippage_bps INTEGER,
    status TEXT NOT NULL CHECK(status IN ('pending', 'confirmed', 'failed')),
    error_message TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    confirmed_at DATETIME,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_swaps_user_id ON swaps(user_id);
CREATE INDEX IF NOT EXISTS idx_swaps_signature ON swaps(signature);
CREATE INDEX IF NOT EXISTS idx_swaps_status ON swaps(status);
CREATE INDEX IF NOT EXISTS idx_swaps_created_at ON swaps(created_at DESC);
