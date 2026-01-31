//! Cache entry with metadata for TTL and LRU tracking.

use bytes::Bytes;
use std::time::Instant;

/// A single cache entry containing the value and metadata.
///
/// Each entry tracks:
/// - The stored value
/// - When the entry expires (if TTL is set)
/// - When the entry was last accessed (for LRU eviction)
#[derive(Debug, Clone)]
pub struct Entry {
    /// The stored value.
    pub(crate) value: Bytes,

    /// When this entry expires. `None` means no expiration.
    pub(crate) expires_at: Option<Instant>,

    /// When this entry was last accessed (for LRU tracking).
    pub(crate) last_accessed: Instant,
}

impl Entry {
    /// Create a new entry with no expiration.
    pub fn new(value: Bytes) -> Self {
        Self {
            value,
            expires_at: None,
            last_accessed: Instant::now(),
        }
    }

    /// Create a new entry with an expiration time.
    pub fn with_expiration(value: Bytes, expires_at: Instant) -> Self {
        Self {
            value,
            expires_at: Some(expires_at),
            last_accessed: Instant::now(),
        }
    }

    /// Check if this entry has expired.
    pub fn is_expired(&self) -> bool {
        self.is_expired_at(Instant::now())
    }

    /// Check if this entry has expired at a given time.
    /// This is useful for testing with a controlled clock.
    pub fn is_expired_at(&self, now: Instant) -> bool {
        match self.expires_at {
            Some(expires) => now >= expires,
            None => false,
        }
    }

    /// Update the last accessed time to now.
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }

    /// Update the last accessed time to a specific instant.
    /// This is useful for testing with a controlled clock.
    pub fn touch_at(&mut self, now: Instant) {
        self.last_accessed = now;
    }

    /// Get a reference to the value.
    pub fn value(&self) -> &Bytes {
        &self.value
    }

    /// Get the expiration time, if set.
    pub fn expires_at(&self) -> Option<Instant> {
        self.expires_at
    }

    /// Get the last accessed time.
    pub fn last_accessed(&self) -> Instant {
        self.last_accessed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_new_entry_not_expired() {
        let entry = Entry::new(Bytes::from("test"));
        assert!(!entry.is_expired());
        assert!(entry.expires_at.is_none());
    }

    #[test]
    fn test_entry_with_future_expiration() {
        let future = Instant::now() + Duration::from_secs(60);
        let entry = Entry::with_expiration(Bytes::from("test"), future);
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_entry_with_past_expiration() {
        // Create entry that expires in the past (already expired)
        let past = Instant::now() - Duration::from_secs(1);
        let entry = Entry::with_expiration(Bytes::from("test"), past);
        assert!(entry.is_expired());
    }

    #[test]
    fn test_touch_updates_access_time() {
        let mut entry = Entry::new(Bytes::from("test"));
        let initial = entry.last_accessed;
        
        // Small delay to ensure time advances
        std::thread::sleep(Duration::from_millis(1));
        entry.touch();
        
        assert!(entry.last_accessed > initial);
    }
}
