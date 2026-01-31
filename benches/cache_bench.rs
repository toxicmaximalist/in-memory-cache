//! Benchmarks for the in-memory cache.
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use in_memory_cache::{Cache, CacheConfig};
use std::time::Duration;

/// Benchmark single-threaded get/set operations.
fn bench_single_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_threaded");

    // Create a cache with enough capacity
    let config = CacheConfig::new().max_capacity(100_000).build();
    let cache = Cache::new(config);

    // Pre-populate some keys
    for i in 0..10_000 {
        cache.set(format!("key_{}", i), format!("value_{}", i));
    }

    group.bench_function("get_existing", |b| {
        let mut i = 0;
        b.iter(|| {
            let key = format!("key_{}", i % 10_000);
            black_box(cache.get(&key));
            i += 1;
        });
    });

    group.bench_function("get_missing", |b| {
        let mut i = 0;
        b.iter(|| {
            let key = format!("missing_{}", i);
            black_box(cache.get(&key));
            i += 1;
        });
    });

    group.bench_function("set_new", |b| {
        let cache = Cache::new(CacheConfig::new().max_capacity(1_000_000).build());
        let mut i = 0;
        b.iter(|| {
            cache.set(format!("new_key_{}", i), "value");
            i += 1;
        });
    });

    group.bench_function("set_existing", |b| {
        let mut i = 0;
        b.iter(|| {
            let key = format!("key_{}", i % 10_000);
            cache.set(key, "updated_value");
            i += 1;
        });
    });

    group.finish();
}

/// Benchmark concurrent operations.
fn bench_concurrent(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent");

    for num_threads in [2, 4, 8].iter() {
        let config = CacheConfig::new().max_capacity(100_000).build();
        let cache = Cache::new(config);

        // Pre-populate
        for i in 0..10_000 {
            cache.set(format!("key_{}", i), format!("value_{}", i));
        }

        group.throughput(Throughput::Elements(1000));
        group.bench_with_input(
            BenchmarkId::new("mixed_ops", num_threads),
            num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|t| {
                            let cache = cache.clone();
                            std::thread::spawn(move || {
                                for i in 0..1000 {
                                    let key = format!("key_{}", (t * 1000 + i) % 10_000);
                                    if i % 5 == 0 {
                                        cache.set(key, "value");
                                    } else {
                                        black_box(cache.get(&key));
                                    }
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark TTL operations.
fn bench_ttl(c: &mut Criterion) {
    let mut group = c.benchmark_group("ttl");

    let cache = Cache::new(CacheConfig::default());

    group.bench_function("set_with_ttl", |b| {
        let mut i = 0;
        b.iter(|| {
            cache.set_with_ttl(format!("ttl_key_{}", i), "value", Duration::from_secs(300));
            i += 1;
        });
    });

    group.finish();
}

/// Benchmark eviction under pressure.
fn bench_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("eviction");

    // Small cache that will constantly evict
    let config = CacheConfig::new().max_capacity(1000).build();
    let cache = Cache::new(config);

    // Fill the cache
    for i in 0..1000 {
        cache.set(format!("key_{}", i), "value");
    }

    group.bench_function("set_with_eviction", |b| {
        let mut i = 1000;
        b.iter(|| {
            cache.set(format!("key_{}", i), "value");
            i += 1;
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_threaded,
    bench_concurrent,
    bench_ttl,
    bench_eviction,
);
criterion_main!(benches);
