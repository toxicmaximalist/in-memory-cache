//! Internal storage implementation for the cache.
//!
//! This module provides the low-level storage using an `IndexMap` for
//! maintaining insertion order (used for LRU eviction).

use bytes::Bytes;
use indexmap::IndexMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

use crate::config::CacheConfig;
use crate::entry::Entry;
use crate::error::{CacheError, CacheResult};
use crate::stats::CacheStats;

/// Thread-safe wrapper around the internal database.
///
/// This is the internal implementation; users should use `Cache` instead.
#[derive(Debug)]
pub struct Db {
    /// The actual storage, protected by a read-write lock.
    /// IndexMap maintains insertion order, which we use for LRU eviction.
    entries: RwLock<IndexMap<String, Entry>>,

    /// Configuration for this cache instance.
    config: CacheConfig,

    /// Statistics for cache operations.
    stats: Arc<CacheStats>,
}

impl Db {
    /// Create a new database with the given configuration.
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: RwLock::new(IndexMap::new()),
            config,
            stats: Arc::new(CacheStats::new()),
        }
    }

    /// Create a new database with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Get a value from the cache.
    ///
    /// Returns `None` if the key doesn't exist or has expired.
    /// Updates the entry's last accessed time (LRU tracking).
    pub fn get(&self, key: &str) -> Option<Bytes> {
        // First, try to read with a read lock
        {
            let entries = self.read_lock()?;

            if let Some(entry) = entries.get(key) {
                if entry.is_expired() {
                    // Entry expired - need write lock to remove it
                    drop(entries);
                    self.remove_expired(key);
                    self.stats.record_miss();
                    self.stats.record_expiration();
                    return None;
                }

                // Clone the value before dropping the read lock
                let value = entry.value().clone();
                self.stats.record_hit();

                // Update access time (need write lock)
                drop(entries);
                if let Some(mut entries) = self.write_lock() {
                    if let Some(idx) = entries.get_index_of(key) {
                        if let Some(entry) = entries.get_index_mut(idx) {
                            entry.1.touch();
                        }
                        // Move to end for LRU (most recently used)
                        let new_idx = entries.len() - 1;
                        entries.move_index(idx, new_idx);
                    }
                }

                return Some(value);
            }
        }

        self.stats.record_miss();
        None
    }

    /// Set a value in the cache without TTL.
    pub fn set(&self, key: impl Into<String>, value: impl Into<Bytes>) {
        let key = key.into();
        let value = value.into();

        let ttl = self.config.default_ttl;
        self.set_internal(key, value, ttl);
    }

    /// Set a value in the cache with a specific TTL.
    pub fn set_with_ttl(&self, key: impl Into<String>, value: impl Into<Bytes>, ttl: Duration) {
        let key = key.into();
        let value = value.into();

        self.set_internal(key, value, Some(ttl));
    }

    /// Internal set implementation.
    fn set_internal(&self, key: String, value: Bytes, ttl: Option<Duration>) {
        let entry = match ttl {
            Some(duration) => Entry::with_expiration(value, Instant::now() + duration),
            None => Entry::new(value),
        };

        let mut entries = match self.write_lock() {
            Some(e) => e,
            None => return, // Lock poisoned, silently fail
        };

        // Check if we need to evict
        if let Some(max_capacity) = self.config.max_capacity {
            // If key already exists, we're replacing, not adding
            if !entries.contains_key(&key) {
                while entries.len() >= max_capacity {
                    self.evict_one(&mut entries);
                }
            }
        }

        let is_new = !entries.contains_key(&key);
        entries.insert(key, entry);

        if is_new {
            self.stats.increment_size();
        }
        self.stats.record_set();
    }

    /// Delete a key from the cache.
    ///
    /// Returns `true` if the key existed and was removed.
    pub fn delete(&self, key: &str) -> bool {
        let mut entries = match self.write_lock() {
            Some(e) => e,
            None => return false,
        };

        let existed = entries.shift_remove(key).is_some();
        if existed {
            self.stats.decrement_size();
            self.stats.record_delete();
        }
        existed
    }

    /// Check if a key exists in the cache (and is not expired).
    pub fn contains(&self, key: &str) -> bool {
        let entries = match self.read_lock() {
            Some(e) => e,
            None => return false,
        };

        match entries.get(key) {
            Some(entry) => {
                if entry.is_expired() {
                    drop(entries);
                    self.remove_expired(key);
                    false
                } else {
                    true
                }
            }
            None => false,
        }
    }

    /// Get the number of entries in the cache.
    ///
    /// Note: This may include expired entries that haven't been cleaned up yet.
    pub fn len(&self) -> usize {
        match self.read_lock() {
            Some(entries) => entries.len(),
            None => 0,
        }
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove all entries from the cache.
    pub fn clear(&self) {
        if let Some(mut entries) = self.write_lock() {
            entries.clear();
            self.stats.set_size(0);
        }
    }

    /// Get a reference to the statistics.
    pub fn stats(&self) -> Arc<CacheStats> {
        Arc::clone(&self.stats)
    }

    /// Remove all expired entries from the cache.
    ///
    /// This is called by the background cleanup task.
    pub fn cleanup_expired(&self) -> usize {
        let mut entries = match self.write_lock() {
            Some(e) => e,
            None => return 0,
        };

        let initial_len = entries.len();
        let now = Instant::now();

        entries.retain(|_, entry| {
            let expired = entry.is_expired_at(now);
            if expired {
                self.stats.record_expiration();
                self.stats.decrement_size();
            }
            !expired
        });

        initial_len - entries.len()
    }

    // Private helper methods

    /// Acquire a read lock, returning None if poisoned.
    fn read_lock(&self) -> Option<RwLockReadGuard<'_, IndexMap<String, Entry>>> {
        self.entries.read().ok()
    }

    /// Acquire a write lock, returning None if poisoned.
    fn write_lock(&self) -> Option<RwLockWriteGuard<'_, IndexMap<String, Entry>>> {
        self.entries.write().ok()
    }

    /// Remove a specific expired key.
    fn remove_expired(&self, key: &str) {
        if let Some(mut entries) = self.write_lock() {
            if let Some(entry) = entries.get(key) {
                if entry.is_expired() {
                    entries.shift_remove(key);
                    self.stats.decrement_size();
                }
            }
        }
    }

    /// Evict one entry (the least recently used).
    fn evict_one(&self, entries: &mut IndexMap<String, Entry>) {
        // IndexMap maintains insertion order; the first entry is the oldest
        // We move recently accessed entries to the end, so first = LRU
        if let Some((key, _)) = entries.first() {
            let key = key.clone();
            entries.shift_remove(&key);
            self.stats.record_eviction();
            self.stats.decrement_size();
        }
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// Implement Clone by creating a new Db with cloned data
impl Clone for Db {
    fn clone(&self) -> Self {
        let entries = self.read_lock().map(|e| e.clone()).unwrap_or_default();

        Self {
            entries: RwLock::new(entries),
            config: self.config.clone(),
            stats: Arc::new(CacheStats::new()), // New stats for cloned instance
        }
    }
}

/// Legacy API support for backward compatibility.
/// These methods match the original API signature.
impl Db {
    /// Legacy write method - parses key/value from array.
    ///
    /// # Deprecated
    /// Use `set(key, value)` instead.
    #[deprecated(since = "1.0.0", note = "Use set() instead")]
    pub fn write(&self, arr: &[String]) -> CacheResult<&'static str> {
        if arr.len() < 3 {
            return Err(CacheError::ParseError(
                "write requires at least 3 arguments: command key value".to_string(),
            ));
        }

        let key = &arr[1];
        let value = &arr[2];

        let existed = self.contains(key);
        self.set(key.clone(), value.clone());

        if existed {
            Ok("r Ok") // Replaced existing key
        } else {
            Ok("Ok") // New key
        }
    }

    /// Legacy read method - parses key from array.
    ///
    /// # Deprecated
    /// Use `get(key)` instead.
    #[deprecated(since = "1.0.0", note = "Use get() instead")]
    pub fn read(&self, arr: &[String]) -> CacheResult<Bytes> {
        if arr.len() < 2 {
            return Err(CacheError::ParseError(
                "read requires at least 2 arguments: command key".to_string(),
            ));
        }

        let key = &arr[1];
        self.get(key)
            .ok_or_else(|| CacheError::KeyNotFound(key.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_set_get() {
        let db = Db::with_defaults();

        db.set("key1", "value1");
        let result = db.get("key1");

        assert_eq!(result, Some(Bytes::from("value1")));
    }

    #[test]
    fn test_get_nonexistent() {
        let db = Db::with_defaults();

        let result = db.get("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_delete() {
        let db = Db::with_defaults();

        db.set("key1", "value1");
        assert!(db.contains("key1"));

        let deleted = db.delete("key1");
        assert!(deleted);
        assert!(!db.contains("key1"));
    }

    #[test]
    fn test_delete_nonexistent() {
        let db = Db::with_defaults();

        let deleted = db.delete("nonexistent");
        assert!(!deleted);
    }

    #[test]
    fn test_overwrite() {
        let db = Db::with_defaults();

        db.set("key1", "value1");
        db.set("key1", "value2");

        assert_eq!(db.get("key1"), Some(Bytes::from("value2")));
        assert_eq!(db.len(), 1);
    }

    #[test]
    fn test_clear() {
        let db = Db::with_defaults();

        db.set("key1", "value1");
        db.set("key2", "value2");
        assert_eq!(db.len(), 2);

        db.clear();
        assert!(db.is_empty());
    }

    #[test]
    fn test_capacity_eviction() {
        let config = CacheConfig::new().max_capacity(3).build();
        let db = Db::new(config);

        db.set("key1", "value1");
        db.set("key2", "value2");
        db.set("key3", "value3");
        assert_eq!(db.len(), 3);

        // This should evict key1 (oldest)
        db.set("key4", "value4");
        assert_eq!(db.len(), 3);
        assert!(!db.contains("key1"));
        assert!(db.contains("key4"));
    }

    #[test]
    fn test_lru_eviction_order() {
        let config = CacheConfig::new().max_capacity(3).build();
        let db = Db::new(config);

        db.set("key1", "value1");
        db.set("key2", "value2");
        db.set("key3", "value3");

        // Access key1, making it recently used
        let _ = db.get("key1");

        // Now key2 should be the LRU
        db.set("key4", "value4");

        assert!(db.contains("key1")); // Was accessed, not evicted
        assert!(!db.contains("key2")); // Was LRU, evicted
        assert!(db.contains("key3"));
        assert!(db.contains("key4"));
    }

    #[test]
    fn test_ttl_expiration() {
        let db = Db::with_defaults();

        // Set with very short TTL
        db.set_with_ttl("key1", "value1", Duration::from_millis(1));

        // Should exist immediately
        assert!(db.contains("key1"));

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(10));

        // Should be expired
        assert!(db.get("key1").is_none());
    }

    #[test]
    fn test_stats_tracking() {
        let db = Db::with_defaults();

        db.set("key1", "value1");
        let _ = db.get("key1"); // Hit
        let _ = db.get("nonexistent"); // Miss

        let stats = db.stats();
        assert_eq!(stats.hits(), 1);
        assert_eq!(stats.misses(), 1);
        assert_eq!(stats.sets(), 1);
    }

    #[test]
    #[allow(deprecated)]
    fn test_legacy_write_read() {
        let db = Db::with_defaults();

        let arr = vec!["set".to_string(), "key1".to_string(), "value1".to_string()];
        let result = db.write(&arr);
        assert!(result.is_ok());

        let arr = vec!["get".to_string(), "key1".to_string()];
        let result = db.read(&arr);
        assert_eq!(result.unwrap(), Bytes::from("value1"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_legacy_read_missing_args() {
        let db = Db::with_defaults();

        let arr = vec!["get".to_string()]; // Missing key
        let result = db.read(&arr);
        assert!(result.is_err());
    }
}
