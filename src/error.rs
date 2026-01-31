//! Error types for the in-memory cache library.
//!
//! This module defines a comprehensive error type that covers all failure modes
//! of the cache operations, avoiding panics in favor of explicit error handling.

use std::fmt;
use std::io;

/// The main error type for cache operations.
///
/// This enum covers all possible error conditions that can occur when
/// interacting with the cache, from key-not-found conditions to I/O errors.
#[derive(Debug)]
pub enum CacheError {
    /// The requested key was not found in the cache.
    KeyNotFound(String),

    /// The command received was invalid or malformed.
    InvalidCommand(String),

    /// Failed to parse the input buffer or protocol message.
    ParseError(String),

    /// An I/O error occurred (network, file, etc.).
    IoError(io::Error),

    /// The cache has reached its maximum capacity.
    CapacityExceeded { current: usize, max: usize },

    /// The provided key is invalid (empty, too long, etc.).
    InvalidKey(String),

    /// The provided value is invalid (too large, etc.).
    InvalidValue(String),

    /// A lock could not be acquired (poisoned mutex).
    LockError(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::KeyNotFound(key) => write!(f, "key not found: '{}'", key),
            CacheError::InvalidCommand(cmd) => write!(f, "invalid command: '{}'", cmd),
            CacheError::ParseError(msg) => write!(f, "parse error: {}", msg),
            CacheError::IoError(err) => write!(f, "I/O error: {}", err),
            CacheError::CapacityExceeded { current, max } => {
                write!(f, "capacity exceeded: {} items (max: {})", current, max)
            }
            CacheError::InvalidKey(reason) => write!(f, "invalid key: {}", reason),
            CacheError::InvalidValue(reason) => write!(f, "invalid value: {}", reason),
            CacheError::LockError(msg) => write!(f, "lock error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CacheError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for CacheError {
    fn from(err: io::Error) -> Self {
        CacheError::IoError(err)
    }
}

/// A specialized Result type for cache operations.
pub type CacheResult<T> = Result<T, CacheError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CacheError::KeyNotFound("mykey".to_string());
        assert_eq!(format!("{}", err), "key not found: 'mykey'");

        let err = CacheError::InvalidCommand("foo".to_string());
        assert_eq!(format!("{}", err), "invalid command: 'foo'");

        let err = CacheError::CapacityExceeded {
            current: 100,
            max: 100,
        };
        assert_eq!(
            format!("{}", err),
            "capacity exceeded: 100 items (max: 100)"
        );
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
        let cache_err: CacheError = io_err.into();
        assert!(matches!(cache_err, CacheError::IoError(_)));
    }
}
