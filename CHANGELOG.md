# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-31

### Added

- **Thread-safe cache implementation** using `RwLock<IndexMap>` for concurrent access
- **`Cache` struct** as the main public API with clean, ergonomic methods:
  - `get()`, `set()`, `set_with_ttl()`, `delete()`, `contains()`, `len()`, `clear()`
- **TTL (Time-To-Live) support** for cache entries
  - Lazy expiration: expired entries removed on access
  - Manual cleanup via `cleanup_expired()`
- **LRU eviction** when `max_capacity` is reached
  - Least recently used entries evicted first
  - Access via `get()` updates LRU ordering
- **`CacheConfig` builder** for configuration:
  - `max_capacity`: Maximum number of entries
  - `default_ttl`: Default TTL for entries without explicit TTL
  - `cleanup_interval`: Interval for background cleanup
  - `background_cleanup`: Enable/disable background task
- **`CacheStats`** for monitoring cache performance:
  - Hits, misses, evictions, expirations, size
  - Hit rate calculation
  - Thread-safe atomic counters
- **`CacheError` enum** for proper error handling
- **Comprehensive test suite**:
  - 45+ unit tests
  - 10 integration tests
  - Doc tests for all public methods
- **CLI tools**:
  - `server`: TCP-based cache server with concurrent connection handling
  - `client`: CLI client with `get`, `set`, `delete`, `ping`, `stats` commands
- **Benchmarks** using Criterion
- **CI/CD** with GitHub Actions

### Changed

- **BREAKING**: Renamed internal `Db` struct to internal implementation detail
- **BREAKING**: Changed error types from `&'static str` to `CacheError` enum
- **BREAKING**: Server now handles concurrent connections (was sequential)
- Updated `clap` from 3.x to 4.x
- Added `indexmap` dependency for LRU implementation

### Fixed

- Server no longer blocks on single connections
- Input validation prevents panics on malformed commands
- Empty buffers handled gracefully

### Security

- No more panics (`expect`/`unwrap`) in production code paths
- Input validation on all user-provided data

## [0.1.0] - Previous

Initial development version (not production-ready).
