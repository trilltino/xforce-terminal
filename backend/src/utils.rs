/// Truncate a Stellar address for display purposes
pub fn truncate_address(address: &str) -> String {
    if address.len() <= 12 {
        return address.to_string();
    }
    format!("{}...{}", &address[..6], &address[address.len()-6..])
}
