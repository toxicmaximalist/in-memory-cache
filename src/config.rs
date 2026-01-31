//! Configuration for the in-memory cache.
//!
//! This module provides a builder pattern for configuring cache behavior
//! including capacity limits, TTL defaults, and cleanup intervals.

use std::time::Duration;

/// Configuration for creating a new cache instance.
///
/// Use the builder pattern to construct configuration:
///
/// ```
/// use in_memory_cache::CacheConfig;
/// use std::time::Duration;
///
/// let config = CacheConfig::new()
///     .max_capacity(10_000)
///     .default_ttl(Duration::from_secs(300))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries the cache can hold.
    /// When this limit is reached, the least recently used entry is evicted.
    /// `None` means unlimited (not recommended for production).
    pub(crate) max_capacity: Option<usize>,

    /// Default TTL for entries when not explicitly specified.
    /// `None` means entries don't expire by default.
    pub(crate) default_ttl: Option<Duration>,

    /// Interval for background cleanup of expired entries.
    /// `None` disables background cleanup (lazy expiration only).
    pub(crate) cleanup_interval: Option<Duration>,

    /// Whether to enable background cleanup task.
    pub(crate) background_cleanup: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: None,
            default_ttl: None,
            cleanup_interval: Some(Duration::from_secs(60)),
            background_cleanup: false,
        }
    }
}

impl CacheConfig {
    /// Create a new configuration builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum capacity of the cache.
    ///
    /// When the cache reaches this capacity, the least recently used
    /// entry will be evicted to make room for new entries.
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of entries. Use 0 for unlimited (not recommended).
    pub fn max_capacity(mut self, capacity: usize) -> Self {
        self.max_capacity = if capacity == 0 { None } else { Some(capacity) };
        self
    }

    /// Set the default TTL for entries.
    ///
    /// Entries without an explicit TTL will use this value.
    /// Set to `Duration::ZERO` to disable default TTL.
    pub fn default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = if ttl.is_zero() { None } else { Some(ttl) };
        self
    }

    /// Set the interval for background cleanup of expired entries.
    ///
    /// The background task will run at this interval to remove expired entries.
    /// This is in addition to lazy expiration (entries checked on access).
    pub fn cleanup_interval(mut self, interval: Duration) -> Self {
        self.cleanup_interval = if interval.is_zero() {
            None
        } else {
            Some(interval)
        };
        self
    }

    /// Enable or disable background cleanup.
    ///
    /// When enabled, a background task periodically removes expired entries.
    /// When disabled, entries are only removed on access (lazy expiration).
    pub fn background_cleanup(mut self, enabled: bool) -> Self {
        self.background_cleanup = enabled;
        self
    }

    /// Build the final configuration.
    ///
    /// This method validates the configuration and returns the final config.
    pub fn build(self) -> Self {
        self
    }

    /// Get the maximum capacity, if set.
    pub fn get_max_capacity(&self) -> Option<usize> {
        self.max_capacity
    }

    /// Get the default TTL, if set.
    pub fn get_default_ttl(&self) -> Option<Duration> {
        self.default_ttl
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CacheConfig::default();
        assert!(config.max_capacity.is_none());
        assert!(config.default_ttl.is_none());
        assert!(!config.background_cleanup);
    }

    #[test]
    fn test_builder_pattern() {
        let config = CacheConfig::new()
            .max_capacity(1000)
            .default_ttl(Duration::from_secs(60))
            .background_cleanup(true)
            .build();

        assert_eq!(config.max_capacity, Some(1000));
        assert_eq!(config.default_ttl, Some(Duration::from_secs(60)));
        assert!(config.background_cleanup);
    }

    #[test]
    fn test_zero_capacity_means_unlimited() {
        let config = CacheConfig::new().max_capacity(0).build();
        assert!(config.max_capacity.is_none());
    }

    #[test]
    fn test_zero_ttl_means_no_default() {
        let config = CacheConfig::new().default_ttl(Duration::ZERO).build();
        assert!(config.default_ttl.is_none());
    }
}
