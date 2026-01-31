//! Command types for the cache protocol.
//!
//! This module defines the commands that can be sent to the cache server.

use crate::error::{CacheError, CacheResult};

/// Types of commands supported by the cache server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Get a value by key.
    Get,
    /// Set a key-value pair.
    Set,
    /// Delete a key.
    Delete,
    /// Ping the server (health check).
    Ping,
    /// Get server statistics.
    Stats,
    /// Invalid or unknown command.
    Invalid,
}

impl Command {
    /// Parse a command from a string.
    ///
    /// # Arguments
    /// * `s` - The command string (case-insensitive).
    ///
    /// # Returns
    /// The parsed command, or `Command::Invalid` for unknown commands.
    pub fn get(s: &str) -> Command {
        match s.to_lowercase().as_str() {
            "set" => Command::Set,
            "get" => Command::Get,
            "delete" | "del" => Command::Delete,
            "ping" => Command::Ping,
            "stats" | "info" => Command::Stats,
            _ => Command::Invalid,
        }
    }

    /// Parse a command from a string, returning an error for invalid commands.
    ///
    /// # Arguments
    /// * `s` - The command string (case-insensitive).
    ///
    /// # Returns
    /// The parsed command, or an error for unknown commands.
    pub fn parse(s: &str) -> CacheResult<Command> {
        let cmd = Self::get(s);
        if cmd == Command::Invalid {
            Err(CacheError::InvalidCommand(s.to_string()))
        } else {
            Ok(cmd)
        }
    }

    /// Get the string representation of this command.
    pub fn as_str(&self) -> &'static str {
        match self {
            Command::Get => "get",
            Command::Set => "set",
            Command::Delete => "delete",
            Command::Ping => "ping",
            Command::Stats => "stats",
            Command::Invalid => "invalid",
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commands() {
        assert_eq!(Command::get("get"), Command::Get);
        assert_eq!(Command::get("GET"), Command::Get);
        assert_eq!(Command::get("set"), Command::Set);
        assert_eq!(Command::get("SET"), Command::Set);
        assert_eq!(Command::get("delete"), Command::Delete);
        assert_eq!(Command::get("del"), Command::Delete);
        assert_eq!(Command::get("ping"), Command::Ping);
        assert_eq!(Command::get("stats"), Command::Stats);
        assert_eq!(Command::get("unknown"), Command::Invalid);
    }

    #[test]
    fn test_parse_with_error() {
        assert!(Command::parse("get").is_ok());
        assert!(Command::parse("unknown").is_err());
    }

    #[test]
    fn test_as_str() {
        assert_eq!(Command::Get.as_str(), "get");
        assert_eq!(Command::Set.as_str(), "set");
    }
}