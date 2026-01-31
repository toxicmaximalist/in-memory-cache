//! Command-line interface definitions.
//!
//! This module defines the CLI structure for the cache client using clap.

use clap::{Parser, Subcommand};

/// In-memory cache client.
///
/// A CLI tool for interacting with the in-memory cache server.
#[derive(Parser, Debug)]
#[command(name = "cache-client")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The command to execute.
    #[clap(subcommand)]
    pub command: ClientCommand,
}

/// Available client commands.
#[derive(Subcommand, Debug)]
pub enum ClientCommand {
    /// Get a value by key.
    ///
    /// Retrieves the value stored at the given key.
    /// Returns nothing if the key doesn't exist.
    Get {
        /// The key to look up.
        key: String,
    },

    /// Set a key-value pair.
    ///
    /// Stores the value at the given key. If the key already
    /// exists, its value is updated.
    Set {
        /// The key to store the value under.
        key: String,
        /// The value to store.
        value: String,
    },

    /// Delete a key.
    ///
    /// Removes the key and its value from the cache.
    Delete {
        /// The key to delete.
        key: String,
    },

    /// Ping the server.
    ///
    /// Checks if the server is running and responsive.
    Ping,

    /// Get server statistics.
    ///
    /// Shows cache hits, misses, size, and hit rate.
    Stats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get() {
        let cli = Cli::parse_from(["test", "get", "mykey"]);
        match cli.command {
            ClientCommand::Get { key } => assert_eq!(key, "mykey"),
            _ => panic!("Expected Get command"),
        }
    }

    #[test]
    fn test_parse_set() {
        let cli = Cli::parse_from(["test", "set", "mykey", "myvalue"]);
        match cli.command {
            ClientCommand::Set { key, value } => {
                assert_eq!(key, "mykey");
                assert_eq!(value, "myvalue");
            }
            _ => panic!("Expected Set command"),
        }
    }

    #[test]
    fn test_parse_delete() {
        let cli = Cli::parse_from(["test", "delete", "mykey"]);
        match cli.command {
            ClientCommand::Delete { key } => assert_eq!(key, "mykey"),
            _ => panic!("Expected Delete command"),
        }
    }

    #[test]
    fn test_parse_ping() {
        let cli = Cli::parse_from(["test", "ping"]);
        assert!(matches!(cli.command, ClientCommand::Ping));
    }

    #[test]
    fn test_parse_stats() {
        let cli = Cli::parse_from(["test", "stats"]);
        assert!(matches!(cli.command, ClientCommand::Stats));
    }
}