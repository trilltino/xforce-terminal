//! # Environment Variables
//!
//! Utilities for reading and parsing environment variables.

use std::env;
use std::str::FromStr;

/// Get an environment variable by name.
pub fn get_env(name: &'static str) -> Result<String, Error> {
    env::var(name).map_err(|_| Error::MissingEnv(name))
}

/// Get and parse an environment variable.
pub fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T, Error> {
    let val = get_env(name)?;
    val.parse::<T>().map_err(|_| Error::WrongFormat(name))
}

// region:    --- Error
#[derive(Debug)]
pub enum Error {
    MissingEnv(&'static str),
    WrongFormat(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error

