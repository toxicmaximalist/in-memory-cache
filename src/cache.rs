//! The main cache interface.
//!
//! This module provides the primary `Cache` type that users interact with.
//! It wraps the internal storage and provides a clean, thread-safe API.

use bytes::Bytes;
use std::sync::Arc;
use std::time::Duration;

use crate::config::CacheConfig;
use crate::stats::{CacheStats, StatsSnapshot};
use crate::storage::Db;

/// A thread-safe, in-memory cache with optional TTL and LRU eviction.
///
/// # Features
/// - **Thread-safe**: Can be safely shared across threads using `Arc<Cache>` or cloning.
/// - **TTL support**: Entries can have optional time-to-live.
/// - **LRU eviction**: When capacity is reached, least recently used entries are evicted.
/// - **Statistics**: Track hits, misses, evictions, and more.
///
/// # Example
/// ```
/// use in_memory_cache::{Cache, CacheConfig};
/// use std::time::Duration;
///
/// // Create a cache with max 1000 entries and 5 minute default TTL
/// let config = CacheConfig::new()
///     .max_capacity(1000)
///     .default_ttl(Duration::from_secs(300))
///     .build();
///
/// let cache = Cache::new(config);
///
/// // Basic operations
/// cache.set("user:123", "Alice");
/// if let Some(value) = cache.get("user:123") {
///     println!("Found: {:?}", value);
/// }
///
/// // With explicit TTL
/// cache.set_with_ttl("session:abc", "data", Duration::from_secs(60));
///
/// // Check statistics
/// let stats = cache.stats();
/// println!("Hit rate: {:.1}%", stats.hit_rate);
/// ```
#[derive(Debug, Clone)]
pub struct Cache {
    /// Internal storage.
    db: Arc<Db>,
}

impl Cache {
    /// Create a new cache with the given configuration.
    ///
    /// # Arguments
    /// * `config` - Configuration options for the cache.
    ///
    /// # Example
    /// ```
    /// use in_memory_cache::{Cache, CacheConfig};
    ///
    /// let cache = Cache::new(CacheConfig::default());
    /// ```
    pub fn new(config: CacheConfig) -> Self {
        Self {
            db: Arc::new(Db::new(config)),
        }
    }

    /// Get a value from the cache.
    ///
    /// Returns `None` if the key doesn't exist or has expired.
    /// Accessing a key updates its last-accessed time for LRU tracking.
    ///
    /// # Arguments
    /// * `key` - The key to look up.
    ///
    /// # Example
    /// ```
    /// use in_memory_cache::{Cache, CacheConfig};
    ///
    /// let cache = Cache::new(CacheConfig::default());
    /// cache.set("key", "value");
    ///
    /// match cache.get("key") {
    ///     Some(value) => println!("Found: {:?}", value),
    ///     None => println!("Not found"),
    /// }
    /// ```
    pub fn get(&self, key: &str) -> Option<Bytes> {
        self.db.get(key)
    }

    /// Set a value in the cache.
    ///
    /// If a `default_ttl` is configured, entries will use that TTL.
    /// Otherwise, entries will not expire.
    ///
    /// # Arguments
    /// * `key` - The key to store the value under.
    /// * `value` - The value to store (anything that can be converted to `Bytes`).
    ///
    /// # Example
    /// ```
    /// use in_memory_cache::{Cache, CacheConfig};
    ///
    /// let cache = Cache::new(CacheConfig::default());
    /// cache.set("string_key", "string value");
    /// cache.set("bytes_key", vec![1, 2, 3, 4]);
    /// ```
    pub fn set(&self, key: impl Into<String>, value: impl Into<Bytes>) {
        self.db.set(key, value);
    }

    /// Set a value in the cache with a specific TTL.
    ///
    /// The entry will be removed after the specified duration.
    ///
    /// # Arguments
    /// * `key` - The key to store the value under.
    /// * `value` - The value to store.
    /// * `ttl` - How long the entry should live.
    ///
    /// # Example
    /// ```
    /// use in_memory_cache::{Cache, CacheConfig};
    /// use std::time::Duration;
    ///
    /// let cache = Cache::new(CacheConfig::default());
    /// cache.set_with_ttl("session", "data", Duration::from_secs(3600));
    /// ```
    pub fn set_with_ttl(&self, key: impl Into<String>, value: impl Into<Bytes>, ttl: Duration) {
        self.db.set_with_ttl(key, value, ttl);
    }

    /// Delete a key from the cache.
    ///
    /// Returns `true` if the key existed and was removed.
    ///
    /// # Arguments
    /// * `key` - The key to delete.
    ///
    /// # Example
    /// ```
    /// use in_memory_cache::{Cache, CacheConfig};
    ///
    /// let cache = Cache::new(CacheConfig::default());
    /// cache.set("key", "value");
    /// assert!(cache.delete("key"));
    /// assert!(!cache.delete("key")); // Already deleted
    /// ```
    pub fn delete(&self, key: &str) -> bool {
        self.db.delete(key)
    }

    /// Check if a key exists in the cache.
    ///
    /// Returns `false` if the key doesn't exist or has expired.
    /// Note: This does NOT update the LRU access time.
    ///
    /// # Arguments
    /// * `key` - The key to check.
    ///
    /// # Example
    /// ```
    /// use in_memory_cache::{Cache, CacheConfig};
    ///
    /// let cache = Cache::new(CacheConfig::default());
    /// assert!(!cache.contains("key"));
    /// cache.set("key", "value");
    /// assert!(cache.contains("key"));
    /// ```
    pub fn contains(&self, key: &str) -> bool {
        self.db.contains(key)
    }

    /// Get the number of entries in the cache.
    ///
    /// Note: This may include expired entries that haven't been
    /// cleaned up yet by lazy expiration or background cleanup.
    ///
    /// # Example
    /// ```
    /// use in_memory_cache::{Cache, CacheConfig};
    ///
    /// let cache = Cache::new(CacheConfig::default());
    /// assert_eq!(cache.len(), 0);
    /// cache.set("key", "value");
    /// assert_eq!(cache.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.db.len()
    }

    /// Check if the cache is empty.
    ///
    /// # Example
    /// ```
    /// use in_memory_cache::{Cache, CacheConfig};
    ///
    /// let cache = Cache::new(CacheConfig::default());
    /// assert!(cache.is_empty());
    /// cache.set("key", "value");
    /// assert!(!cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.db.is_empty()
    }

    /// Remove all entries from the cache.
    ///
    /// # Example
    /// ```
    /// use in_memory_cache::{Cache, CacheConfig};
    ///
    /// let cache = Cache::new(CacheConfig::default());
    /// cache.set("key1", "value1");
    /// cache.set("key2", "value2");
    /// cache.clear();
    /// assert!(cache.is_empty());
    /// ```
    pub fn clear(&self) {
        self.db.clear();
    }

    /// Get a snapshot of the cache statistics.
    ///
    /// Returns a point-in-time snapshot of hits, misses, evictions, etc.
    ///
    /// # Example
    /// ```
    /// use in_memory_cache::{Cache, CacheConfig};
    ///
    /// let cache = Cache::new(CacheConfig::default());
    /// cache.set("key", "value");
    /// let _ = cache.get("key");        // Hit
    /// let _ = cache.get("missing");    // Miss
    ///
    /// let stats = cache.stats();
    /// println!("Hits: {}, Misses: {}", stats.hits, stats.misses);
    /// ```
    pub fn stats(&self) -> StatsSnapshot {
        self.db.stats().snapshot()
    }

    /// Manually trigger cleanup of expired entries.
    ///
    /// Returns the number of entries that were removed.
    /// This is useful if you want to control when cleanup happens
    /// instead of relying on lazy expiration or background cleanup.
    ///
    /// # Example
    /// ```
    /// use in_memory_cache::{Cache, CacheConfig};
    /// use std::time::Duration;
    ///
    /// let cache = Cache::new(CacheConfig::default());
    /// cache.set_with_ttl("key", "value", Duration::from_millis(1));
    /// std::thread::sleep(Duration::from_millis(10));
    /// let removed = cache.cleanup_expired();
    /// println!("Removed {} expired entries", removed);
    /// ```
    pub fn cleanup_expired(&self) -> usize {
        self.db.cleanup_expired()
    }

    /// Get a reference to the internal statistics counter.
    ///
    /// This is useful for integrating with external metrics systems.
    pub fn stats_ref(&self) -> Arc<CacheStats> {
        self.db.stats()
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = Cache::default();

        cache.set("key", "value");
        assert_eq!(cache.get("key"), Some(Bytes::from("value")));
        assert!(cache.contains("key"));

        cache.delete("key");
        assert!(!cache.contains("key"));
    }

    #[test]
    fn test_cache_is_clone() {
        let cache1 = Cache::default();
        cache1.set("key", "value");

        let cache2 = cache1.clone();

        // Both point to the same underlying data
        assert_eq!(cache2.get("key"), Some(Bytes::from("value")));

        cache2.set("key2", "value2");
        assert_eq!(cache1.get("key2"), Some(Bytes::from("value2")));
    }

    #[test]
    fn test_cache_stats() {
        let cache = Cache::default();

        cache.set("key", "value");
        let _ = cache.get("key");
        let _ = cache.get("missing");

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_thread_safety() {
        use std::thread;

        let cache = Cache::default();
        let mut handles = vec![];

        // Spawn multiple threads that read/write concurrently
        for i in 0..10 {
            let cache = cache.clone();
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key_{}", j);
                    cache.set(key.clone(), format!("value_{}_{}", i, j));
                    let _ = cache.get(&key);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should have completed without panics
        assert!(!cache.is_empty());
    }
}
