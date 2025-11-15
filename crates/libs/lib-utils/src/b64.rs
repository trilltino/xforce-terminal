//! # Base64 Encoding/Decoding
//!
//! Utilities for base64 encoding and decoding.

use base64::{Engine as _, engine::general_purpose};

/// Encode bytes to base64 URL-safe string (no padding).
pub fn b64u_encode(content: impl AsRef<[u8]>) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(content)
}

/// Decode base64 URL-safe string to bytes.
pub fn b64u_decode(b64u: &str) -> Result<Vec<u8>, Error> {
    general_purpose::URL_SAFE_NO_PAD
        .decode(b64u)
        .map_err(|_| Error::FailToB64uDecode)
}

/// Decode base64 URL-safe string to UTF-8 string.
pub fn b64u_decode_to_string(b64u: &str) -> Result<String, Error> {
    b64u_decode(b64u)
        .and_then(|bytes| String::from_utf8(bytes).map_err(|_| Error::FailToB64uDecode))
}

// region:    --- Error
#[derive(Debug)]
pub enum Error {
    FailToB64uDecode,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error

