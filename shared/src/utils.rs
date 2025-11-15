//! # Shared Utility Functions
//!
//! Common utility functions used across backend, terminal, and wallet-web applications.
//!
//! ## Address Formatting
//!
//! Functions for formatting Solana wallet addresses for display:
//! - [`format_address`] - Format address with ellipsis (first N and last M characters)
//! - [`truncate_address`] - Alias for `format_address` with default parameters
//!
//! ## Usage
//!
//! ```rust
//! use shared::utils::format_address;
//!
//! let address = "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL";
//! let formatted = format_address(address, 4, 4);
//! assert_eq!(formatted, "8W6Q...JKAL");
//! ```

/// Format a wallet address by showing the first `prefix_len` and last `suffix_len` characters.
///
/// If the address is shorter than `prefix_len + suffix_len`, it is returned as-is.
///
/// # Arguments
///
/// * `address` - The wallet address to format
/// * `prefix_len` - Number of characters to show at the start (default: 4)
/// * `suffix_len` - Number of characters to show at the end (default: 4)
///
/// # Examples
///
/// ```rust
/// use shared::utils::format_address;
///
/// let addr = "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL";
/// assert_eq!(format_address(addr, 4, 4), "8W6Q...JKAL");
/// assert_eq!(format_address(addr, 6, 6), "8W6Qgi...uJKAL");
/// assert_eq!(format_address("short", 4, 4), "short");
/// ```
pub fn format_address(address: &str, prefix_len: usize, suffix_len: usize) -> String {
    let address_len = address.len();
    
    // Return early if address is too short to truncate meaningfully
    // Also guard against individual lengths exceeding address length to prevent panics
    if address_len <= prefix_len + suffix_len
        || prefix_len >= address_len
        || suffix_len >= address_len
    {
        return address.to_string();
    }
    
    // Safe to slice: we've verified prefix_len and suffix_len are within bounds
    // For Solana addresses (base58), we can safely use byte indexing as they're ASCII-only
    let prefix = &address[..prefix_len];
    let suffix = &address[address_len - suffix_len..];
    
    format!("{}...{}", prefix, suffix)
}

/// Format a wallet address with default 4-character prefix and suffix.
///
/// This is a convenience function that calls [`format_address`] with `prefix_len=4` and `suffix_len=4`.
///
/// # Examples
///
/// ```rust
/// use shared::utils::truncate_address;
///
/// let addr = "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL";
/// assert_eq!(truncate_address(addr), "8W6Q...JKAL");
/// ```
pub fn truncate_address(address: &str) -> String {
    format_address(address, 4, 4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_address() {
        let addr = "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL";
        assert_eq!(format_address(addr, 4, 4), "8W6Q...JKAL");
        assert_eq!(format_address(addr, 6, 6), "8W6Qgi...uJKAL");
        assert_eq!(format_address(addr, 2, 2), "8W...AL");
    }

    #[test]
    fn test_format_address_short() {
        assert_eq!(format_address("short", 4, 4), "short");
        assert_eq!(format_address("abc", 4, 4), "abc");
    }

    #[test]
    fn test_truncate_address() {
        let addr = "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL";
        assert_eq!(truncate_address(addr), "8W6Q...JKAL");
    }
}

