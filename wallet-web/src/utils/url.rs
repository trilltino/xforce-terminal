//! URL utility functions for reading query parameters

use web_sys::window;

/// Get a query parameter from the current URL
/// This is a fallback method that reads directly from window.location.search
/// Use this when the router's query map might not be initialized yet
pub fn get_query_param(key: &str) -> Option<String> {
    let window = window()?;
    let location = window.location();
    let search = location.search().ok()?;
    
    if search.is_empty() {
        return None;
    }
    
    // Remove leading '?' if present
    let query_string = search.strip_prefix('?').unwrap_or(&search);
    
    // Parse query parameters
    for pair in query_string.split('&') {
        if let Some(equal_pos) = pair.find('=') {
            let param_key = &pair[..equal_pos];
            let param_value = &pair[equal_pos + 1..];
            if param_key == key {
                // URL decode the value
                return Some(urlencoding::decode(param_value)
                    .unwrap_or_else(|_| param_value.into())
                    .into_owned());
            }
        } else {
            // Handle case where parameter has no value (just the key)
            if pair == key {
                return Some(String::new());
            }
        }
    }
    
    None
}

/// Get multiple query parameters from the current URL
pub fn get_query_params() -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
    
    if let Some(window) = window() {
        if let Ok(search) = window.location().search() {
            if !search.is_empty() {
                let query_string = search.strip_prefix('?').unwrap_or(&search);
                for pair in query_string.split('&') {
                    let mut parts = pair.splitn(2, '=');
                    if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                        let decoded_value = urlencoding::decode(value)
                            .unwrap_or_else(|_| value.into())
                            .into_owned();
                        params.insert(key.to_string(), decoded_value);
                    }
                }
            }
        }
    }
    
    params
}

