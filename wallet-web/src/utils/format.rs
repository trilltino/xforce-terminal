//! # Formatting Utilities for Wallet Web
//!
//! Number and value formatting utilities specific to the wallet-web application.
//! For address formatting, use [`shared::utils::format_address`] or [`shared::utils::truncate_address`].
//!
//! ## Functions
//!
//! - [`format_number`] - Format numbers with comma separators
//! - [`format_lamports_to_sol`] - Convert lamports to SOL with formatting
//! - [`format_percentage`] - Format percentage changes with sign
//! - [`format_price_impact`] - Format price impact as percentage

/// Format a number with commas (e.g., 1234567.89 -> "1,234,567.89")
///
/// # Arguments
///
/// * `value` - The number to format
/// * `decimals` - Number of decimal places to show
///
/// # Examples
///
/// ```rust
/// use wallet_web::utils::format::format_number;
///
/// assert_eq!(format_number(1234567.89, 2), "1,234,567.89");
/// assert_eq!(format_number(100.0, 2), "100.00");
/// ```
pub fn format_number(value: f64, decimals: usize) -> String {
    let formatted = format!("{:.prec$}", value, prec = decimals);
    let parts: Vec<&str> = formatted.split('.').collect();

    let integer_part = parts[0];
    let decimal_part = if parts.len() > 1 { parts[1] } else { "" };

    // Add commas to integer part
    let mut result = String::new();
    for (i, ch) in integer_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }

    let integer_with_commas: String = result.chars().rev().collect();

    if decimal_part.is_empty() {
        integer_with_commas
    } else {
        format!("{}.{}", integer_with_commas, decimal_part)
    }
}

/// Format lamports to SOL (9 decimals)
///
/// Converts lamports (smallest unit of SOL) to SOL with 4 decimal places.
///
/// # Arguments
///
/// * `lamports` - Amount in lamports (1 SOL = 1,000,000,000 lamports)
///
/// # Examples
///
/// ```rust
/// use wallet_web::utils::format::format_lamports_to_sol;
///
/// assert_eq!(format_lamports_to_sol(1_000_000_000), "1.0000");
/// assert_eq!(format_lamports_to_sol(500_000_000), "0.5000");
/// ```
pub fn format_lamports_to_sol(lamports: u64) -> String {
    let sol = lamports as f64 / 1_000_000_000.0;
    format_number(sol, 4)
}

/// Format a wallet address (show first 4 and last 4 characters)
///
/// **Deprecated**: Use [`shared::utils::format_address`] or [`shared::utils::truncate_address`] instead.
///
/// This function is kept for backward compatibility but delegates to the shared implementation.
#[deprecated(note = "Use shared::utils::format_address or shared::utils::truncate_address instead")]
pub fn format_address(address: &str) -> String {
    shared::utils::truncate_address(address)
}

/// Format percentage change with sign and color indicator
pub fn format_percentage(pct: f64) -> String {
    if pct >= 0.0 {
        format!("+{:.2}%", pct)
    } else {
        format!("{:.2}%", pct)
    }
}

/// Format price impact (always show as percentage)
pub fn format_price_impact(impact: f64) -> String {
    format!("{:.2}%", impact)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234567.89, 2), "1,234,567.89");
        assert_eq!(format_number(100.0, 2), "100.00");
    }

    #[test]
    fn test_format_address() {
        let addr = "8W6QginkhTTxoP2deQjq7rZ9YMwN5FH9JYuLfSKuJKAL";
        #[allow(deprecated)]
        assert_eq!(format_address(addr), "8W6Q...JKAL");
    }
}
