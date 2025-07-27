use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kaipo_watcher::collectors::bandwidth_collector::BandwidthCollector;
use std::time::Duration;

/// Benchmark bandwidth collection performance
fn benchmark_bandwidth_collection(c: &mut Criterion) {
    let mut group = c.benchmark_group("bandwidth_collection");
    
    // Set measurement time to get more stable results
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("single_collection", |b| {
        let mut collector = BandwidthCollector::new();
        b.iter(|| {
            let stats = collector.collect().expect("Collection should work");
            black_box(stats);
        });
    });
    
    group.bench_function("filtered_collection", |b| {
        let mut collector = BandwidthCollector::new();
        b.iter(|| {
            let stats = collector.collect_filtered().expect("Filtered collection should work");
            black_box(stats);
        });
    });
    
    group.bench_function("default_collection", |b| {
        let mut collector = BandwidthCollector::new();
        b.iter(|| {
            let stats = collector.collect_default().expect("Default collection should work");
            black_box(stats);
        });
    });
    
    group.bench_function("important_collection", |b| {
        let mut collector = BandwidthCollector::new();
        b.iter(|| {
            let stats = collector.collect_important().expect("Important collection should work");
            black_box(stats);
        });
    });
    
    group.finish();
}

/// Benchmark collector initialization
fn benchmark_collector_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("collector_initialization");
    
    group.bench_function("new_collector", |b| {
        b.iter(|| {
            let collector = BandwidthCollector::new();
            black_box(collector);
        });
    });
    
    group.bench_function("new_collector_with_retry_config", |b| {
        b.iter(|| {
            let collector = BandwidthCollector::with_retry_config(3, 100);
            black_box(collector);
        });
    });
    
    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    group.bench_function("repeated_collections", |b| {
        let mut collector = BandwidthCollector::new();
        b.iter(|| {
            // Perform multiple collections to test memory usage
            for _ in 0..10 {
                let stats = collector.collect().expect("Collection should work");
                black_box(stats);
            }
        });
    });
    
    group.bench_function("cache_operations", |b| {
        let mut collector = BandwidthCollector::new();
        b.iter(|| {
            // Test cache clearing performance
            collector.clear_interface_cache();
            let stats = collector.collect().expect("Collection should work");
            black_box(stats);
        });
    });
    
    group.finish();
}

/// Benchmark concurrent access patterns
fn benchmark_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_access");
    
    group.bench_function("multiple_collectors", |b| {
        b.iter(|| {
            // Create multiple collectors to simulate concurrent usage
            let mut collectors = Vec::new();
            for _ in 0..3 {
                collectors.push(BandwidthCollector::new());
            }
            
            // Collect from all simultaneously
            let mut all_stats = Vec::new();
            for collector in &mut collectors {
                let stats = collector.collect().expect("Collection should work");
                all_stats.push(stats);
            }
            
            black_box(all_stats);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_bandwidth_collection,
    benchmark_collector_initialization,
    benchmark_memory_usage,
    benchmark_concurrent_access
);
criterion_main!(benches);