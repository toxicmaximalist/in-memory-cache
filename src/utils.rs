//! Utility functions for buffer parsing and manipulation.

use bytes::{Buf, BytesMut};

use crate::error::{CacheError, CacheResult};

/// Receives buffer and converts it to vector of strings.
///
/// Splits the buffer on space characters. Note that this simple
/// implementation doesn't handle quoted strings or escaping.
///
/// # Arguments
/// * `buf` - The buffer to parse. Will be consumed.
///
/// # Returns
/// A vector of strings, split by spaces.
///
/// # Example
/// ```ignore
/// let mut buf = BytesMut::from("set key value");
/// let parts = buffer_to_array(&mut buf);
/// assert_eq!(parts, vec!["set", "key", "value"]);
/// ```
pub fn buffer_to_array(buf: &mut BytesMut) -> Vec<String> {
    let mut vec = vec![];
    let length = buf.len();

    if length == 0 {
        return vec;
    }

    let mut word = String::new();

    for i in 0..length {
        match buf.get_u8() {
            // Space indicates end of word
            b' ' => {
                if !word.is_empty() {
                    vec.push(word);
                    word = String::new();
                }
            }
            // Collect character into current word
            other => {
                word.push(other as char);
                if i == length - 1 && !word.is_empty() {
                    vec.push(word.clone());
                }
            }
        }
    }
    vec
}

/// Parse a buffer into command parts with validation.
///
/// Returns an error if the buffer is empty or malformed.
///
/// # Arguments
/// * `buf` - The buffer to parse. Will be consumed.
///
/// # Returns
/// A vector of at least one string, or an error.
pub fn parse_command(buf: &mut BytesMut) -> CacheResult<Vec<String>> {
    let parts = buffer_to_array(buf);

    if parts.is_empty() {
        return Err(CacheError::ParseError("empty command".to_string()));
    }

    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_to_array_basic() {
        let mut buf = BytesMut::from("set key value");
        let result = buffer_to_array(&mut buf);
        assert_eq!(result, vec!["set", "key", "value"]);
    }

    #[test]
    fn test_buffer_to_array_empty() {
        let mut buf = BytesMut::new();
        let result = buffer_to_array(&mut buf);
        assert!(result.is_empty());
    }

    #[test]
    fn test_buffer_to_array_single_word() {
        let mut buf = BytesMut::from("ping");
        let result = buffer_to_array(&mut buf);
        assert_eq!(result, vec!["ping"]);
    }

    #[test]
    fn test_buffer_to_array_multiple_spaces() {
        let mut buf = BytesMut::from("set  key   value");
        let result = buffer_to_array(&mut buf);
        // Multiple spaces are treated as single separator
        assert_eq!(result, vec!["set", "key", "value"]);
    }

    #[test]
    fn test_parse_command_empty() {
        let mut buf = BytesMut::new();
        let result = parse_command(&mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_valid() {
        let mut buf = BytesMut::from("get mykey");
        let result = parse_command(&mut buf);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["get", "mykey"]);
    }
}
