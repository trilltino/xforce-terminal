//! # Time Utilities
//!
//! Utilities for time formatting and manipulation using chrono.

use chrono::{DateTime, Utc};

/// Get current UTC time.
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Format time as RFC3339 string.
pub fn format_time(time: DateTime<Utc>) -> String {
    time.to_rfc3339()
}

/// Parse RFC3339 string to UTC DateTime.
pub fn parse_utc(moment: &str) -> Result<DateTime<Utc>, Error> {
    DateTime::parse_from_rfc3339(moment)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| Error::FailToDateParse(moment.to_string()))
}

// region:    --- Error
#[derive(Debug)]
pub enum Error {
    FailToDateParse(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error

