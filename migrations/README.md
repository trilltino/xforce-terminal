# Database Migrations

SQL migration files for XForce Terminal database schema.

## Files

| File | Description |
|------|-------------|
| `20250106_create_users.sql` | Initial users table |
| `20250107_add_wallet_to_users.sql` | Add wallet address column |
| `20250113_create_messaging_tables.sql` | Chat and messaging tables |
| `20250124_create_swaps_table.sql` | Swap history table |

## Running Migrations

Using sqlx CLI:

```bash
# Install sqlx
sqlx migrate run

# Create new migration
sqlx migrate add <description>
```

## Schema Overview

### Users
- id, username, email, password_hash, wallet_address, created_at

### Messages
- id, sender_id, receiver_id, content, created_at

### Swaps
- id, user_id, input_mint, output_mint, amount, signature, created_at
