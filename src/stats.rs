//! Statistics and metrics for the cache.
//!
//! This module provides atomic counters for tracking cache operations,
//! enabling observability without impacting performance.

use std::sync::atomic::{AtomicU64, Ordering};

/// Statistics for cache operations.
///
/// All counters are atomic and can be safely accessed from multiple threads.
/// Use `Cache::stats()` to get a snapshot of the current statistics.
///
/// # Example
/// ```ignore
/// let cache = Cache::new(CacheConfig::default());
/// cache.set("key", "value");
/// let stats = cache.stats();
/// println!("Cache size: {}", stats.size());
/// ```
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Number of successful get operations (key found).
    hits: AtomicU64,

    /// Number of failed get operations (key not found or expired).
    misses: AtomicU64,

    /// Number of entries evicted due to capacity limits.
    evictions: AtomicU64,

    /// Number of entries removed due to TTL expiration.
    expirations: AtomicU64,

    /// Current number of entries in the cache.
    size: AtomicU64,

    /// Total number of set operations performed.
    sets: AtomicU64,

    /// Total number of delete operations performed.
    deletes: AtomicU64,
}

impl CacheStats {
    /// Create a new stats instance with all counters at zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cache hit.
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss.
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an eviction (due to capacity).
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an expiration (due to TTL).
    pub fn record_expiration(&self) {
        self.expirations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a set operation.
    pub fn record_set(&self) {
        self.sets.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a delete operation.
    pub fn record_delete(&self) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the size counter.
    pub fn increment_size(&self) {
        self.size.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the size counter.
    pub fn decrement_size(&self) {
        self.size.fetch_sub(1, Ordering::Relaxed);
    }

    /// Set the size to a specific value.
    pub fn set_size(&self, size: u64) {
        self.size.store(size, Ordering::Relaxed);
    }

    // Getters for reading statistics

    /// Get the number of cache hits.
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Get the number of cache misses.
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Get the number of evictions.
    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }

    /// Get the number of expirations.
    pub fn expirations(&self) -> u64 {
        self.expirations.load(Ordering::Relaxed)
    }

    /// Get the current cache size.
    pub fn size(&self) -> u64 {
        self.size.load(Ordering::Relaxed)
    }

    /// Get the total number of set operations.
    pub fn sets(&self) -> u64 {
        self.sets.load(Ordering::Relaxed)
    }

    /// Get the total number of delete operations.
    pub fn deletes(&self) -> u64 {
        self.deletes.load(Ordering::Relaxed)
    }

    /// Calculate the hit rate as a percentage (0.0 to 100.0).
    /// Returns 0.0 if no operations have been performed.
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits();
        let misses = self.misses();
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }

    /// Create a snapshot of the current statistics.
    /// This is useful for serialization or logging.
    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            hits: self.hits(),
            misses: self.misses(),
            evictions: self.evictions(),
            expirations: self.expirations(),
            size: self.size(),
            sets: self.sets(),
            deletes: self.deletes(),
            hit_rate: self.hit_rate(),
        }
    }
}

/// A point-in-time snapshot of cache statistics.
///
/// Unlike `CacheStats`, this struct contains plain values (not atomics)
/// and can be easily serialized or logged.
#[derive(Debug, Clone, PartialEq)]
pub struct StatsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub expirations: u64,
    pub size: u64,
    pub sets: u64,
    pub deletes: u64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_stats() {
        let stats = CacheStats::new();
        assert_eq!(stats.hits(), 0);
        assert_eq!(stats.misses(), 0);
        assert_eq!(stats.size(), 0);
    }

    #[test]
    fn test_record_operations() {
        let stats = CacheStats::new();

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.hits(), 2);
        assert_eq!(stats.misses(), 1);
    }

    #[test]
    fn test_hit_rate() {
        let stats = CacheStats::new();

        // No operations = 0% hit rate
        assert_eq!(stats.hit_rate(), 0.0);

        // 3 hits, 1 miss = 75% hit rate
        stats.record_hit();
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert!((stats.hit_rate() - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_size_tracking() {
        let stats = CacheStats::new();

        stats.increment_size();
        stats.increment_size();
        assert_eq!(stats.size(), 2);

        stats.decrement_size();
        assert_eq!(stats.size(), 1);
    }

    #[test]
    fn test_snapshot() {
        let stats = CacheStats::new();
        stats.record_hit();
        stats.record_set();
        stats.increment_size();

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.hits, 1);
        assert_eq!(snapshot.sets, 1);
        assert_eq!(snapshot.size, 1);
    }
}
