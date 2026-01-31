# In-Memory Cache

[![CI](https://github.com/yourusername/in-memory-cache/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/in-memory-cache/actions)
[![Crates.io](https://img.shields.io/crates/v/in-memory-cache.svg)](https://crates.io/crates/in-memory-cache)
[![Documentation](https://docs.rs/in-memory-cache/badge.svg)](https://docs.rs/in-memory-cache)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A fast, thread-safe, in-memory cache library for Rust with TTL support and LRU eviction.

## Features

- **Thread-safe**: Share across threads using `Clone` (uses `Arc` internally)
- **TTL Support**: Entries can expire after a configurable duration
- **LRU Eviction**: Automatic eviction of least-recently-used entries when at capacity
- **Statistics**: Track cache hits, misses, evictions, and more
- **Zero unsafe code**: Built entirely with safe Rust
- **Minimal dependencies**: Only essential crates used

## Quick Start

```rust
use in_memory_cache::{Cache, CacheConfig};
use std::time::Duration;

// Create a cache with configuration
let config = CacheConfig::new()
    .max_capacity(10_000)
    .default_ttl(Duration::from_secs(300))
    .build();

let cache = Cache::new(config);

// Store and retrieve values
cache.set("user:123", "Alice");

if let Some(value) = cache.get("user:123") {
    println!("Found: {:?}", value);
}

// Set with custom TTL
cache.set_with_ttl("session:abc", "session_data", Duration::from_secs(60));

// Delete a key
cache.delete("user:123");

// Check statistics
let stats = cache.stats();
println!("Hit rate: {:.1}%", stats.hit_rate);
```

## Configuration

```rust
use in_memory_cache::CacheConfig;
use std::time::Duration;

let config = CacheConfig::new()
    // Maximum number of entries (LRU eviction when exceeded)
    .max_capacity(10_000)
    // Default TTL for entries (None = no expiration)
    .default_ttl(Duration::from_secs(300))
    // Enable background cleanup task
    .background_cleanup(true)
    // Cleanup interval for background task
    .cleanup_interval(Duration::from_secs(60))
    .build();
```

## Thread Safety

The cache is safe to share across threads. Cloning creates a new handle to the same data:

```rust
use in_memory_cache::Cache;
use std::thread;

let cache = Cache::default();

let handles: Vec<_> = (0..4).map(|i| {
    let cache = cache.clone();
    thread::spawn(move || {
        cache.set(format!("key_{}", i), format!("value_{}", i));
    })
}).collect();

for handle in handles {
    handle.join().unwrap();
}
```

## TTL and Expiration

Entries can have time-to-live (TTL) values. Expired entries are removed:
- **On access** (lazy expiration): When you try to `get()` an expired key
- **Background cleanup** (if enabled): Periodic removal of expired entries

```rust
use in_memory_cache::Cache;
use std::time::Duration;

let cache = Cache::default();

// Entry expires in 60 seconds
cache.set_with_ttl("session", "data", Duration::from_secs(60));

// Manual cleanup of expired entries
let removed = cache.cleanup_expired();
println!("Removed {} expired entries", removed);
```

## LRU Eviction

When `max_capacity` is set and the cache is full, the least recently used entry is evicted:

```rust
use in_memory_cache::{Cache, CacheConfig};

let config = CacheConfig::new().max_capacity(3).build();
let cache = Cache::new(config);

cache.set("a", "1");
cache.set("b", "2");
cache.set("c", "3");

// Access 'a' to make it recently used
let _ = cache.get("a");

// Adding 'd' evicts 'b' (LRU)
cache.set("d", "4");

assert!(cache.contains("a"));  // Recently accessed
assert!(!cache.contains("b")); // Evicted (LRU)
assert!(cache.contains("c"));
assert!(cache.contains("d"));
```

## Statistics

Monitor cache performance with built-in statistics:

```rust
use in_memory_cache::Cache;

let cache = Cache::default();
// ... use the cache ...

let stats = cache.stats();
println!("Hits: {}", stats.hits);
println!("Misses: {}", stats.misses);
println!("Hit rate: {:.1}%", stats.hit_rate);
println!("Evictions: {}", stats.evictions);
println!("Size: {}", stats.size);
```

## CLI Tools

The crate includes server and client binaries for testing:

```bash
# Start the cache server
cargo run --bin server

# In another terminal, use the client
cargo run --bin client set mykey "my value"
cargo run --bin client get mykey
cargo run --bin client delete mykey
cargo run --bin client ping
cargo run --bin client stats
```

## Design Choices

### Why RwLock instead of sharding?

We use `RwLock<IndexMap>` for simplicity and correctness. Sharded locks add complexity and are only beneficial under very high contention (>32 cores). Profile before optimizing.

### Why IndexMap for LRU?

`IndexMap` provides O(1) access and maintains insertion order, which we use for LRU tracking. When an entry is accessed, it's moved to the end. Eviction removes from the front.

### Why lazy + eager expiration?

- **Lazy**: Simple, no background tasks, entries removed on access
- **Eager** (optional): Background cleanup prevents memory buildup from expired-but-unaccessed entries

Both strategies work together for best results.

## Limitations & Guarantees

### What this cache IS:
- Fast single-node in-memory storage
- Thread-safe for concurrent access
- Suitable for caching frequently accessed data

### What this cache is NOT:
- **Not persistent**: Data is lost on restart
- **Not distributed**: Single-node only
- **Not a database**: No transactions, queries, or durability
- **Not bounded by bytes**: Capacity is measured in entries, not memory

### Guarantees:
- No unsafe code
- No panics on normal operations
- Eventual expiration (expired entries may briefly remain until accessed or cleaned)
- LRU eviction is approximate (access time updates may race under high concurrency)

## Benchmarks

Run benchmarks with:

```bash
cargo bench
```

Typical performance on a modern laptop:
- Single-threaded: ~5-10M ops/sec
- Concurrent (8 threads): ~2-5M ops/sec total

## License

MIT License. See [LICENSE](LICENSE) for details.