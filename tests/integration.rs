//! Integration tests for the cache library.

use in_memory_cache::{Cache, CacheConfig};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_basic_workflow() {
    let cache = Cache::default();

    // Initially empty
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);

    // Set a value
    cache.set("key1", "value1");
    assert_eq!(cache.len(), 1);
    assert!(!cache.is_empty());

    // Get the value back
    let value = cache.get("key1");
    assert!(value.is_some());
    assert_eq!(&value.unwrap()[..], b"value1");

    // Check contains
    assert!(cache.contains("key1"));
    assert!(!cache.contains("nonexistent"));

    // Delete
    assert!(cache.delete("key1"));
    assert!(!cache.contains("key1"));
    assert!(!cache.delete("key1")); // Already deleted

    // Clear
    cache.set("a", "1");
    cache.set("b", "2");
    cache.set("c", "3");
    assert_eq!(cache.len(), 3);
    cache.clear();
    assert!(cache.is_empty());
}

#[test]
fn test_ttl_expiration() {
    let cache = Cache::default();

    // Set with short TTL
    cache.set_with_ttl("expiring", "value", Duration::from_millis(50));

    // Should exist immediately
    assert!(cache.get("expiring").is_some());

    // Wait for expiration
    thread::sleep(Duration::from_millis(100));

    // Should be expired
    assert!(cache.get("expiring").is_none());
}

#[test]
fn test_lru_eviction() {
    let config = CacheConfig::new().max_capacity(3).build();
    let cache = Cache::new(config);

    // Fill to capacity
    cache.set("a", "1");
    cache.set("b", "2");
    cache.set("c", "3");
    assert_eq!(cache.len(), 3);

    // Access 'a' to make it recently used
    let _ = cache.get("a");

    // Add new key, should evict 'b' (LRU)
    cache.set("d", "4");
    assert_eq!(cache.len(), 3);

    assert!(cache.contains("a")); // Was accessed, not evicted
    assert!(!cache.contains("b")); // Was LRU, evicted
    assert!(cache.contains("c"));
    assert!(cache.contains("d"));
}

#[test]
fn test_concurrent_reads() {
    let config = CacheConfig::new().max_capacity(1000).build();
    let cache = Arc::new(Cache::new(config));

    // Pre-populate
    for i in 0..100 {
        cache.set(format!("key_{}", i), format!("value_{}", i));
    }

    // Spawn multiple reader threads
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let cache = Arc::clone(&cache);
            thread::spawn(move || {
                for _ in 0..1000 {
                    for i in 0..100 {
                        let _ = cache.get(&format!("key_{}", i));
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // All keys should still exist
    for i in 0..100 {
        assert!(cache.contains(&format!("key_{}", i)));
    }
}

#[test]
fn test_concurrent_writes() {
    let config = CacheConfig::new().max_capacity(10_000).build();
    let cache = Arc::new(Cache::new(config));

    // Spawn multiple writer threads
    let handles: Vec<_> = (0..8)
        .map(|t| {
            let cache = Arc::clone(&cache);
            thread::spawn(move || {
                for i in 0..1000 {
                    let key = format!("thread_{}_key_{}", t, i);
                    cache.set(key.clone(), format!("value_{}", i));
                    let _ = cache.get(&key);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Should have 8000 keys (8 threads × 1000 keys each)
    assert_eq!(cache.len(), 8000);
}

#[test]
fn test_stats_accuracy() {
    let cache = Cache::default();

    // Perform operations
    cache.set("key1", "value1");
    cache.set("key2", "value2");
    let _ = cache.get("key1"); // Hit
    let _ = cache.get("key2"); // Hit
    let _ = cache.get("missing"); // Miss
    cache.delete("key1");

    let stats = cache.stats();
    assert_eq!(stats.sets, 2);
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.deletes, 1);
    assert_eq!(stats.size, 1); // key1 deleted, key2 remains
}

#[test]
fn test_config_builder() {
    let config = CacheConfig::new()
        .max_capacity(5000)
        .default_ttl(Duration::from_secs(60))
        .background_cleanup(false)
        .build();

    assert_eq!(config.get_max_capacity(), Some(5000));
    assert_eq!(config.get_default_ttl(), Some(Duration::from_secs(60)));
}

#[test]
fn test_cache_clone_shares_data() {
    let cache1 = Cache::default();
    cache1.set("key", "value1");

    let cache2 = cache1.clone();

    // Both see the same data
    assert_eq!(cache2.get("key"), cache1.get("key"));

    // Modification through one is visible to the other
    cache2.set("key", "value2");
    assert_eq!(
        cache1
            .get("key")
            .map(|b| String::from_utf8_lossy(&b).to_string()),
        Some("value2".to_string())
    );
}

#[test]
fn test_overwrite_preserves_capacity() {
    let config = CacheConfig::new().max_capacity(3).build();
    let cache = Cache::new(config);

    cache.set("a", "1");
    cache.set("b", "2");
    cache.set("c", "3");
    assert_eq!(cache.len(), 3);

    // Overwrite existing key should not trigger eviction
    cache.set("a", "updated");
    assert_eq!(cache.len(), 3);
    assert!(cache.contains("b"));
    assert!(cache.contains("c"));
}

#[test]
fn test_binary_values() {
    let cache = Cache::default();

    // Store binary data
    let binary_data: Vec<u8> = vec![0, 1, 2, 255, 254, 253];
    cache.set("binary", binary_data.clone());

    let retrieved = cache.get("binary");
    assert!(retrieved.is_some());
    assert_eq!(&retrieved.unwrap()[..], &binary_data[..]);
}
