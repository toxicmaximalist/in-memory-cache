//! # In-Memory Cache
//!
//! A fast, thread-safe, in-memory cache library for Rust with TTL support
//! and LRU eviction.
//!
//! ## Features
//!
//! - **Thread-safe**: Share across threads with `Clone` (uses `Arc` internally)
//! - **TTL support**: Entries can expire after a configurable duration
//! - **LRU eviction**: Automatic eviction of least-recently-used entries when at capacity
//! - **Statistics**: Track cache hits, misses, evictions, and more
//! - **Zero unsafe code**: Built entirely with safe Rust
//!
//! ## Quick Start
//!
//! ```rust
//! use in_memory_cache::{Cache, CacheConfig};
//! use std::time::Duration;
//!
//! // Create a cache with configuration
//! let config = CacheConfig::new()
//!     .max_capacity(10_000)
//!     .default_ttl(Duration::from_secs(300))
//!     .build();
//!
//! let cache = Cache::new(config);
//!
//! // Store and retrieve values
//! cache.set("user:123", "Alice");
//!
//! if let Some(value) = cache.get("user:123") {
//!     println!("Found: {:?}", value);
//! }
//!
//! // Set with custom TTL
//! cache.set_with_ttl("session:abc", "session_data", Duration::from_secs(60));
//!
//! // Check statistics
//! let stats = cache.stats();
//! println!("Hit rate: {:.1}%", stats.hit_rate);
//! ```
//!
//! ## Thread Safety
//!
//! The cache is safe to share across threads. Cloning a `Cache` creates a new
//! handle to the same underlying data:
//!
//! ```rust
//! use in_memory_cache::Cache;
//! use std::thread;
//!
//! let cache = Cache::default();
//!
//! let handles: Vec<_> = (0..4).map(|i| {
//!     let cache = cache.clone();
//!     thread::spawn(move || {
//!         cache.set(format!("key_{}", i), format!("value_{}", i));
//!     })
//! }).collect();
//!
//! for handle in handles {
//!     handle.join().unwrap();
//! }
//! ```

// Public API - stable in v1.0.0
pub mod cache;
pub mod config;
pub mod error;
pub mod stats;

pub use cache::Cache;
pub use config::CacheConfig;
pub use error::{CacheError, CacheResult};
pub use stats::{CacheStats, StatsSnapshot};

// Internal modules - not part of public API
pub(crate) mod entry;
pub(crate) mod storage;

// Legacy modules - preserved for backward compatibility with server/client binaries
pub mod utils;
pub use utils::buffer_to_array;

pub mod command;
pub use command::Command;

// Re-export Db for backward compatibility, but mark as deprecated
#[doc(hidden)]
pub mod database {
    //! Legacy database module - use `Cache` instead.
    pub use crate::storage::Db;
}
#[doc(hidden)]
pub use storage::Db;

pub mod cli;
pub use cli::{Cli, ClientCommand};
