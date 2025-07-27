use chrono::Utc;
use kaipo_watcher::collectors::bandwidth_collector::{BandwidthCollector, CalculationConfidence, BandwidthError, InterfaceType, InterfaceState};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::process::Command;

/// Integration tests for bandwidth monitoring functionality
/// These tests verify the bandwidth collector works correctly in real-world scenarios

#[tokio::test]
async fn test_speed_calculations_against_known_activity() {
    // Test that speed calculations work correctly with known network activity patterns
    let mut collector = BandwidthCollector::new();
    
    // Take initial baseline reading
    let initial_stats = collector.collect().expect("Failed to collect initial stats");
    assert!(!initial_stats.is_empty(), "Should find at least one network interface");
    
    // Generate some network activity to test speed calculations
    let _network_activity = tokio::spawn(async {
        // Create some network activity by making HTTP requests
        for _ in 0..5 {
            let _ = tokio::time::timeout(
                Duration::from_millis(100),
                reqwest::get("http://httpbin.org/bytes/1024")
            ).await;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
    
    // Wait for network activity to occur
    tokio::time::sleep(Duration::from_millis(800)).await;
    
    // Take second reading for speed calculation
    let second_stats = collector.collect().expect("Failed to collect second stats");
    assert_eq!(initial_stats.len(), second_stats.len(), "Interface count should remain consistent");
    
    // Verify that speed calculations are reasonable
    for stat in &second_stats {
        // Speed should be non-negative
        assert!(stat.download_speed_bps >= 0.0, "Download speed should be non-negative for {}", stat.interface_name);
        assert!(stat.upload_speed_bps >= 0.0, "Upload speed should be non-negative for {}", stat.interface_name);
        
        // Speed should not be impossibly high (> 10 Gbps)
        assert!(stat.download_speed_bps < 10_000_000_000.0, "Download speed should be realistic for {}", stat.interface_name);
        assert!(stat.upload_speed_bps < 10_000_000_000.0, "Upload speed should be realistic for {}", stat.interface_name);
        
        // Confidence should be appropriate for second reading
        match stat.calculation_confidence {
            CalculationConfidence::None => {
                // This is acceptable for interfaces with no activity
            }
            CalculationConfidence::Low | CalculationConfidence::Medium | CalculationConfidence::High => {
                // These are all valid confidence levels
            }
        }
        
        // Time since last update should be reasonable
        assert!(stat.time_since_last_update >= 0.0, "Time since last update should be non-negative");
        assert!(stat.time_since_last_update < 10.0, "Time since last update should be recent");
    }
    
    // Test speed calculation accuracy with controlled intervals
    let mut accuracy_collector = BandwidthCollector::new();
    let baseline = accuracy_collector.collect().expect("Baseline collection should work");
    
    // Wait exactly 1 second for precise timing
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    let measurement = accuracy_collector.collect().expect("Measurement collection should work");
    
    // Verify timing accuracy
    for (baseline_stat, measurement_stat) in baseline.iter().zip(measurement.iter()) {
        assert_eq!(baseline_stat.interface_name, measurement_stat.interface_name);
        
        // Time since last update should be approximately 1 second (allow for some variance)
        if measurement_stat.calculation_confidence != CalculationConfidence::None {
            assert!(measurement_stat.time_since_last_update >= 0.9, 
                   "Time measurement should be accurate for {}: {:.3}s", 
                   measurement_stat.interface_name, measurement_stat.time_since_last_update);
            assert!(measurement_stat.time_since_last_update <= 1.2, 
                   "Time measurement should be accurate for {}: {:.3}s", 
                   measurement_stat.interface_name, measurement_stat.time_since_last_update);
        }
        
        // Byte counters should be monotonic (increasing or same)
        assert!(measurement_stat.bytes_received >= baseline_stat.bytes_received,
               "RX bytes should be monotonic for {}: {} -> {}", 
               measurement_stat.interface_name, baseline_stat.bytes_received, measurement_stat.bytes_received);
        assert!(measurement_stat.bytes_sent >= baseline_stat.bytes_sent,
               "TX bytes should be monotonic for {}: {} -> {}", 
               measurement_stat.interface_name, baseline_stat.bytes_sent, measurement_stat.bytes_sent);
    }
}

#[tokio::test]
async fn test_cross_platform_compatibility() {
    // Test that bandwidth collection works across different platforms
    let mut collector = BandwidthCollector::new();
    
    // Test basic collection functionality
    let stats = collector.collect().expect("Cross-platform collection should work");
    assert!(!stats.is_empty(), "Should find network interfaces on any platform");
    
    // Test platform-specific interface naming conventions
    let current_platform = std::env::consts::OS;
    println!("Testing on platform: {}", current_platform);
    
    for stat in &stats {
        assert!(!stat.interface_name.is_empty(), "Interface name should not be empty");
        assert!(stat.interface_name.len() <= 64, "Interface name should be reasonable length");
        
        // Interface name should contain valid characters
        assert!(stat.interface_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':'), 
                "Interface name should contain valid characters: {}", stat.interface_name);
        
        // Platform-specific interface name validation
        match current_platform {
            "macos" => {
                // macOS interfaces typically start with en, lo, utun, anpi, etc.
                let valid_prefixes = ["en", "lo", "utun", "anpi", "awdl", "llw", "bridge", "gif", "stf", "XHC"];
                let has_valid_prefix = valid_prefixes.iter().any(|prefix| stat.interface_name.starts_with(prefix));
                if !has_valid_prefix {
                    println!("Warning: Unexpected macOS interface name: {}", stat.interface_name);
                }
            }
            "linux" => {
                // Linux interfaces typically start with eth, wlan, lo, docker, veth, etc.
                let valid_prefixes = ["eth", "wlan", "lo", "docker", "veth", "br-", "virbr", "tun", "tap", "wg"];
                let has_valid_prefix = valid_prefixes.iter().any(|prefix| stat.interface_name.starts_with(prefix));
                if !has_valid_prefix {
                    println!("Warning: Unexpected Linux interface name: {}", stat.interface_name);
                }
            }
            "windows" => {
                // Windows interfaces have various naming patterns
                // Just ensure they're not empty and reasonable length
                assert!(stat.interface_name.len() >= 1, "Windows interface name should not be empty");
            }
            _ => {
                println!("Testing on unknown platform: {}", current_platform);
            }
        }
        
        // Verify interface type detection works across platforms
        match stat.interface_type {
            InterfaceType::Ethernet | InterfaceType::WiFi | InterfaceType::Loopback | 
            InterfaceType::Virtual | InterfaceType::Unknown => {
                // All valid interface types
            }
        }
        
        // Verify interface state detection
        match stat.interface_state {
            InterfaceState::Up | InterfaceState::Down | InterfaceState::Unknown => {
                // All valid interface states
            }
        }
    }
    
    // Test filtered collection methods
    let filtered_stats = collector.collect_filtered().expect("Filtered collection should work");
    let default_stats = collector.collect_default().expect("Default collection should work");
    let important_stats = collector.collect_important().expect("Important collection should work");
    
    // Verify filtering relationships
    assert!(important_stats.len() <= default_stats.len(), "Important interfaces should be subset of default");
    assert!(default_stats.len() <= filtered_stats.len(), "Default interfaces should be subset of filtered");
    assert!(filtered_stats.len() <= stats.len(), "Filtered interfaces should be subset of all");
    
    // Test interface information retrieval
    let interface_info = collector.get_all_interface_info().expect("Should get interface info");
    assert!(!interface_info.is_empty(), "Should have interface information");
    
    // Verify interface info matches collected stats
    for info in &interface_info {
        assert!(!info.name.is_empty(), "Interface info should have valid name");
        
        // Verify platform-specific interface analysis
        match current_platform {
            "macos" => {
                // macOS should detect interface types correctly
                if info.name.starts_with("en") {
                    // Ethernet or WiFi interface
                    assert!(info.is_physical || info.is_wifi, "en* interfaces should be physical or wifi on macOS");
                } else if info.name == "lo0" {
                    assert!(info.is_loopback, "lo0 should be detected as loopback on macOS");
                }
            }
            "linux" => {
                // Linux should detect interface types correctly
                if info.name.starts_with("eth") {
                    assert!(info.is_physical, "eth* interfaces should be physical on Linux");
                } else if info.name.starts_with("wlan") {
                    assert!(info.is_wifi, "wlan* interfaces should be wifi on Linux");
                } else if info.name == "lo" {
                    assert!(info.is_loopback, "lo should be detected as loopback on Linux");
                }
            }
            _ => {
                // For other platforms, just ensure basic functionality
                println!("Interface analysis on {}: {} -> physical={}, wifi={}, loopback={}, virtual={}", 
                        current_platform, info.name, info.is_physical, info.is_wifi, info.is_loopback, info.is_virtual);
            }
        }
    }
    
    // Test platform-specific error handling
    let mut error_collector = BandwidthCollector::with_retry_config(1, 10);
    match error_collector.collect() {
        Ok(stats) => {
            assert!(!stats.is_empty(), "Error collector should still work");
        }
        Err(e) => {
            // Verify error types are appropriate for the platform
            println!("Platform-specific error handling test: {}", e);
        }
    }
}

#[tokio::test]
async fn test_long_running_collection_scenarios() {
    // Test that bandwidth collection works correctly over extended periods
    let mut collector = BandwidthCollector::new();
    let test_duration = Duration::from_secs(10); // Extended to 10 seconds for more thorough testing
    let collection_interval = Duration::from_millis(200); // Collect every 200ms
    
    let start_time = Instant::now();
    let mut collection_count = 0;
    let mut all_stats = Vec::new();
    let mut collection_times = Vec::new();
    let mut error_count = 0;
    
    // Collect data over the test period
    while start_time.elapsed() < test_duration {
        let collection_start = Instant::now();
        
        match collector.collect() {
            Ok(stats) => {
                let collection_duration = collection_start.elapsed();
                all_stats.push((Instant::now(), stats));
                collection_times.push(collection_duration);
                collection_count += 1;
            }
            Err(e) => {
                error_count += 1;
                println!("Collection error #{}: {}", error_count, e);
                
                // Allow some errors but not too many
                assert!(error_count < collection_count / 10, "Too many collection errors: {}/{}", error_count, collection_count);
            }
        }
        
        tokio::time::sleep(collection_interval).await;
    }
    
    assert!(collection_count >= 40, "Should have collected data multiple times: {}", collection_count);
    println!("Long-running test completed: {} collections, {} errors over {:.1}s", 
             collection_count, error_count, test_duration.as_secs_f64());
    
    // Analyze collection performance over time
    let avg_collection_time = collection_times.iter().sum::<Duration>() / collection_times.len() as u32;
    let max_collection_time = collection_times.iter().max().unwrap();
    let min_collection_time = collection_times.iter().min().unwrap();
    
    println!("Collection performance: avg={:.3}ms, min={:.3}ms, max={:.3}ms", 
             avg_collection_time.as_secs_f64() * 1000.0,
             min_collection_time.as_secs_f64() * 1000.0,
             max_collection_time.as_secs_f64() * 1000.0);
    
    // Collection times should be reasonable and not degrade significantly
    assert!(avg_collection_time < Duration::from_millis(100), 
           "Average collection time should be reasonable: {:?}", avg_collection_time);
    assert!(max_collection_time < Duration::from_millis(500), 
           "Maximum collection time should be reasonable: {:?}", max_collection_time);
    
    // Verify consistency across collections
    let first_interfaces: Vec<String> = all_stats[0].1.iter().map(|s| s.interface_name.clone()).collect();
    let mut interface_stability_violations = 0;
    
    for (timestamp, stats) in &all_stats {
        // Interface count should remain relatively stable
        let current_interfaces: Vec<String> = stats.iter().map(|s| s.interface_name.clone()).collect();
        
        // Allow for some interface changes but not dramatic ones
        let interface_diff = first_interfaces.len().abs_diff(current_interfaces.len());
        if interface_diff > 2 {
            interface_stability_violations += 1;
        }
        
        // Verify data quality for each interface
        for stat in stats {
            assert!(stat.bytes_received < u64::MAX / 2, "Byte counters should not overflow");
            assert!(stat.bytes_sent < u64::MAX / 2, "Byte counters should not overflow");
            
            // Speed calculations should be stable (not wildly fluctuating)
            if stat.calculation_confidence != CalculationConfidence::None {
                assert!(stat.download_speed_bps < 1_000_000_000.0, "Speed should be reasonable"); // < 1 Gbps
                assert!(stat.upload_speed_bps < 1_000_000_000.0, "Speed should be reasonable");
                assert!(stat.download_speed_bps.is_finite(), "Download speed should be finite");
                assert!(stat.upload_speed_bps.is_finite(), "Upload speed should be finite");
            }
            
            // Timestamps should be reasonable
            assert!(stat.timestamp <= chrono::Utc::now(), "Timestamp should not be in future");
            assert!(stat.time_since_last_update >= 0.0, "Time since last update should be non-negative");
        }
    }
    
    // Allow some interface instability but not excessive
    assert!(interface_stability_violations < all_stats.len() / 10, 
           "Interface stability violations should be minimal: {}/{}", 
           interface_stability_violations, all_stats.len());
    
    // Test that confidence improves over time for active interfaces
    let final_stats = &all_stats.last().unwrap().1;
    let mut confidence_distribution = HashMap::new();
    
    for stat in final_stats {
        *confidence_distribution.entry(stat.calculation_confidence.clone()).or_insert(0) += 1;
    }
    
    println!("Final confidence distribution: {:?}", confidence_distribution);
    
    // At least some interfaces should have good confidence after extended collection
    let high_confidence_count = confidence_distribution.get(&CalculationConfidence::High).unwrap_or(&0) +
                               confidence_distribution.get(&CalculationConfidence::Medium).unwrap_or(&0);
    assert!(high_confidence_count > 0, "Some interfaces should have good confidence after extended collection");
    
    // Test memory stability - collector should not accumulate excessive state
    let (cache_size, platform) = collector.get_interface_manager_stats();
    assert!(cache_size < 1000, "Interface cache should not grow excessively: {} entries", cache_size);
    println!("Interface manager stats: {} cached entries on {}", cache_size, platform);
    
    // Test total bandwidth calculation stability
    let (total_download, total_upload) = collector.get_total_bandwidth();
    assert!(total_download >= 0.0, "Total download should be non-negative");
    assert!(total_upload >= 0.0, "Total upload should be non-negative");
    assert!(total_download.is_finite(), "Total download should be finite");
    assert!(total_upload.is_finite(), "Total upload should be finite");
}

#[tokio::test]
async fn test_performance_impact() {
    // Test that bandwidth collection doesn't significantly impact system performance
    let mut collector = BandwidthCollector::new();
    
    // Measure baseline system performance
    let baseline_start = Instant::now();
    let baseline_iterations = 10000;
    let mut baseline_results = Vec::new();
    
    for i in 0..baseline_iterations {
        let work_start = Instant::now();
        // Simulate some CPU work
        let _result: u64 = (0..100).map(|x| x * x).sum();
        baseline_results.push(work_start.elapsed());
        
        // Periodic yield to prevent test timeout
        if i % 1000 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    let baseline_duration = baseline_start.elapsed();
    let baseline_avg = baseline_results.iter().sum::<Duration>() / baseline_results.len() as u32;
    
    // Measure performance with bandwidth collection
    let collection_start = Instant::now();
    let collection_iterations = 100;
    let mut collection_results = Vec::new();
    let mut collection_times = Vec::new();
    
    for i in 0..collection_iterations {
        let collect_start = Instant::now();
        let _stats = collector.collect().expect("Performance test collection should work");
        let collect_time = collect_start.elapsed();
        collection_times.push(collect_time);
        
        // Do the same CPU work as baseline
        let work_iterations = baseline_iterations / collection_iterations;
        for _ in 0..work_iterations {
            let work_start = Instant::now();
            let _result: u64 = (0..100).map(|x| x * x).sum();
            collection_results.push(work_start.elapsed());
        }
        
        // Periodic yield
        if i % 10 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    let collection_duration = collection_start.elapsed();
    let collection_avg = collection_results.iter().sum::<Duration>() / collection_results.len() as u32;
    
    // Analyze collection performance
    let avg_collection_time = collection_times.iter().sum::<Duration>() / collection_times.len() as u32;
    let max_collection_time = collection_times.iter().max().unwrap();
    let min_collection_time = collection_times.iter().min().unwrap();
    
    // Collection should not significantly slow down the system
    let performance_ratio = collection_duration.as_secs_f64() / baseline_duration.as_secs_f64();
    let work_slowdown = collection_avg.as_nanos() as f64 / baseline_avg.as_nanos() as f64;
    
    // Comprehensive performance logging
    println!("Performance Analysis:");
    println!("  Baseline: {:.3}ms total, {:.3}μs per work unit", 
             baseline_duration.as_secs_f64() * 1000.0,
             baseline_avg.as_nanos() as f64 / 1000.0);
    println!("  With collection: {:.3}ms total, {:.3}μs per work unit", 
             collection_duration.as_secs_f64() * 1000.0,
             collection_avg.as_nanos() as f64 / 1000.0);
    println!("  Performance ratio: {:.2}x", performance_ratio);
    println!("  Work slowdown: {:.2}x", work_slowdown);
    println!("  Collection times: avg={:.3}ms, min={:.3}ms, max={:.3}ms", 
             avg_collection_time.as_secs_f64() * 1000.0,
             min_collection_time.as_secs_f64() * 1000.0,
             max_collection_time.as_secs_f64() * 1000.0);
    
    // Performance assertions with reasonable thresholds for test environment
    assert!(performance_ratio < 10.0, "Overall performance impact should be reasonable: {:.2}x", performance_ratio);
    assert!(work_slowdown < 5.0, "Individual work units should not be significantly slower: {:.2}x", work_slowdown);
    
    // Collection time assertions
    assert!(avg_collection_time < Duration::from_millis(100), 
            "Average collection time should be fast: {:.3}ms", avg_collection_time.as_secs_f64() * 1000.0);
    assert!(max_collection_time < Duration::from_millis(500), 
            "Maximum collection time should be reasonable: {:.3}ms", max_collection_time.as_secs_f64() * 1000.0);
    
    // Test memory usage during performance test
    let memory_test_start = Instant::now();
    let memory_iterations = 1000;
    
    for i in 0..memory_iterations {
        let _stats = collector.collect().expect("Memory performance test should work");
        
        // Periodic cache clearing to test cleanup performance
        if i % 100 == 0 {
            collector.clear_interface_cache();
        }
        
        // Yield periodically
        if i % 50 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    let memory_test_duration = memory_test_start.elapsed();
    let avg_memory_collection = memory_test_duration / memory_iterations;
    
    println!("  Memory test: {} collections in {:.3}ms, avg={:.3}ms per collection", 
             memory_iterations, 
             memory_test_duration.as_secs_f64() * 1000.0,
             avg_memory_collection.as_secs_f64() * 1000.0);
    
    assert!(avg_memory_collection < Duration::from_millis(10), 
            "Memory test collections should be fast: {:.3}ms", avg_memory_collection.as_secs_f64() * 1000.0);
    
    // Test concurrent performance impact
    let concurrent_start = Instant::now();
    let concurrent_tasks = 5;
    let collections_per_task = 20;
    
    let mut handles = Vec::new();
    for task_id in 0..concurrent_tasks {
        let handle = tokio::spawn(async move {
            let mut task_collector = BandwidthCollector::new();
            let mut task_times = Vec::new();
            
            for _ in 0..collections_per_task {
                let start = Instant::now();
                let _stats = task_collector.collect().expect("Concurrent collection should work");
                task_times.push(start.elapsed());
                
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            
            (task_id, task_times)
        });
        handles.push(handle);
    }
    
    let mut all_concurrent_times = Vec::new();
    for handle in handles {
        let (task_id, times) = handle.await.expect("Concurrent task should complete");
        println!("  Concurrent task {}: avg={:.3}ms", 
                task_id, 
                times.iter().sum::<Duration>().as_secs_f64() * 1000.0 / times.len() as f64);
        all_concurrent_times.extend(times);
    }
    
    let concurrent_duration = concurrent_start.elapsed();
    let avg_concurrent_time = all_concurrent_times.iter().sum::<Duration>() / all_concurrent_times.len() as u32;
    
    println!("  Concurrent test: {:.3}ms total, avg={:.3}ms per collection", 
             concurrent_duration.as_secs_f64() * 1000.0,
             avg_concurrent_time.as_secs_f64() * 1000.0);
    
    assert!(avg_concurrent_time < Duration::from_millis(200), 
            "Concurrent collections should not be significantly slower: {:.3}ms", 
            avg_concurrent_time.as_secs_f64() * 1000.0);
}

#[tokio::test]
async fn test_concurrent_collection_safety() {
    // Test that multiple collectors can work simultaneously without interference
    let collector_count = 3;
    let collections_per_collector = 5;
    
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    
    // Spawn multiple concurrent collectors
    for collector_id in 0..collector_count {
        let _results_clone = Arc::clone(&results);
        
        let handle = tokio::spawn(async move {
            let mut collector = BandwidthCollector::new();
            let mut collector_results = Vec::new();
            
            for collection_id in 0..collections_per_collector {
                let start_time = Instant::now();
                
                match collector.collect() {
                    Ok(stats) => {
                        let duration = start_time.elapsed();
                        collector_results.push((collector_id, collection_id, stats.len(), duration, None));
                    }
                    Err(e) => {
                        let duration = start_time.elapsed();
                        collector_results.push((collector_id, collection_id, 0, duration, Some(e.to_string())));
                    }
                }
                
                // Small delay between collections
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            
            collector_results
        });
        
        handles.push(handle);
    }
    
    // Wait for all collectors to complete
    for handle in handles {
        let collector_results = handle.await.expect("Collector task should complete");
        results.lock().unwrap().extend(collector_results);
    }
    
    let all_results = results.lock().unwrap();
    
    // Verify all collections completed
    assert_eq!(all_results.len(), collector_count * collections_per_collector);
    
    // Verify most collections succeeded
    let successful_collections = all_results.iter().filter(|(_, _, _, _, error)| error.is_none()).count();
    let success_rate = successful_collections as f64 / all_results.len() as f64;
    
    assert!(success_rate >= 0.8, "At least 80% of concurrent collections should succeed: {:.2}%", success_rate * 100.0);
    
    // Verify performance is reasonable under concurrent load
    let avg_duration: Duration = all_results.iter()
        .map(|(_, _, _, duration, _)| *duration)
        .sum::<Duration>() / all_results.len() as u32;
    
    assert!(avg_duration < Duration::from_secs(1), 
            "Average collection time should be reasonable under concurrent load: {:?}", avg_duration);
}

#[tokio::test]
async fn test_error_recovery_and_resilience() {
    // Test that the collector can recover from various error conditions
    let mut collector = BandwidthCollector::new();
    
    // Test normal operation first
    let initial_stats = collector.collect().expect("Initial collection should work");
    assert!(!initial_stats.is_empty());
    
    // Test collection with custom retry configuration
    let mut retry_collector = BandwidthCollector::with_retry_config(5, 50);
    let retry_stats = retry_collector.collect().expect("Retry collector should work");
    assert!(!retry_stats.is_empty());
    
    // Test interface cache management
    collector.clear_interface_cache();
    let post_clear_stats = collector.collect().expect("Collection after cache clear should work");
    assert!(!post_clear_stats.is_empty());
    
    // Test interface manager statistics
    let (_cache_size, platform) = collector.get_interface_manager_stats();
    assert!(!platform.is_empty(), "Platform should be detected");
    
    // Test total bandwidth calculation
    let (total_download, total_upload) = collector.get_total_bandwidth();
    assert!(total_download >= 0.0, "Total download should be non-negative");
    assert!(total_upload >= 0.0, "Total upload should be non-negative");
    
    // Test interface information retrieval
    if let Some(first_interface) = initial_stats.first() {
        let interface_info = collector.get_interface_info(&first_interface.interface_name);
        assert_eq!(interface_info.name, first_interface.interface_name);
    }
}

#[tokio::test]
async fn test_data_consistency_and_validation() {
    // Test that collected data is consistent and properly validated
    let mut collector = BandwidthCollector::new();
    
    // Collect multiple samples
    let mut samples = Vec::new();
    for _ in 0..5 {
        let stats = collector.collect().expect("Data consistency test should work");
        samples.push(stats);
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    // Verify data consistency across samples
    for (sample_idx, sample) in samples.iter().enumerate() {
        for stat in sample {
            // Basic data validation
            assert!(!stat.interface_name.is_empty(), "Interface name should not be empty in sample {}", sample_idx);
            assert!(stat.timestamp <= Utc::now(), "Timestamp should not be in the future in sample {}", sample_idx);
            
            // Counter validation
            assert!(stat.bytes_received < u64::MAX, "Bytes received should not overflow in sample {}", sample_idx);
            assert!(stat.bytes_sent < u64::MAX, "Bytes sent should not overflow in sample {}", sample_idx);
            assert!(stat.packets_received < u64::MAX, "Packets received should not overflow in sample {}", sample_idx);
            assert!(stat.packets_sent < u64::MAX, "Packets sent should not overflow in sample {}", sample_idx);
            
            // Speed validation
            assert!(stat.download_speed_bps >= 0.0, "Download speed should be non-negative in sample {}", sample_idx);
            assert!(stat.upload_speed_bps >= 0.0, "Upload speed should be non-negative in sample {}", sample_idx);
            assert!(stat.download_speed_bps.is_finite(), "Download speed should be finite in sample {}", sample_idx);
            assert!(stat.upload_speed_bps.is_finite(), "Upload speed should be finite in sample {}", sample_idx);
            
            // Time validation
            assert!(stat.time_since_last_update >= 0.0, "Time since last update should be non-negative in sample {}", sample_idx);
            assert!(stat.time_since_last_update.is_finite(), "Time since last update should be finite in sample {}", sample_idx);
        }
    }
    
    // Verify counter monotonicity (counters should generally increase or stay the same)
    if samples.len() >= 2 {
        let first_sample = &samples[0];
        let last_sample = &samples[samples.len() - 1];
        
        // Find matching interfaces between first and last sample
        for first_stat in first_sample {
            if let Some(last_stat) = last_sample.iter().find(|s| s.interface_name == first_stat.interface_name) {
                // Counters should not decrease significantly (allowing for small variations due to counter resets)
                let rx_diff = last_stat.bytes_received as i64 - first_stat.bytes_received as i64;
                let tx_diff = last_stat.bytes_sent as i64 - first_stat.bytes_sent as i64;
                
                // Allow for counter resets but not impossible negative values
                if rx_diff < 0 {
                    assert!(rx_diff > -1_000_000, "RX counter should not decrease dramatically for {}: {} -> {}", 
                            first_stat.interface_name, first_stat.bytes_received, last_stat.bytes_received);
                }
                
                if tx_diff < 0 {
                    assert!(tx_diff > -1_000_000, "TX counter should not decrease dramatically for {}: {} -> {}", 
                            first_stat.interface_name, first_stat.bytes_sent, last_stat.bytes_sent);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_memory_usage_and_cleanup() {
    // Test that the collector doesn't leak memory during extended operation
    let mut collector = BandwidthCollector::new();
    
    // Perform many collections to test for memory leaks
    let collection_count = 100;
    let mut interface_counts = Vec::new();
    
    for i in 0..collection_count {
        let stats = collector.collect().expect("Memory test collection should work");
        interface_counts.push(stats.len());
        
        // Periodically clear cache to test cleanup
        if i % 20 == 0 {
            collector.clear_interface_cache();
        }
        
        // Small delay to prevent overwhelming the system
        if i % 10 == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    
    // Verify that interface counts remain stable (no memory corruption)
    let avg_interface_count = interface_counts.iter().sum::<usize>() as f64 / interface_counts.len() as f64;
    
    for (i, &count) in interface_counts.iter().enumerate() {
        let deviation = (count as f64 - avg_interface_count).abs() / avg_interface_count;
        assert!(deviation < 0.5, "Interface count should remain stable at iteration {}: {} vs avg {:.1}", 
                i, count, avg_interface_count);
    }
    
    // Test final collection to ensure collector is still functional
    let final_stats = collector.collect().expect("Final collection should work");
    assert!(!final_stats.is_empty(), "Collector should still be functional after extended use");
}

#[tokio::test]
async fn test_network_activity_correlation() {
    // Test that bandwidth measurements correlate with actual network activity
    let mut collector = BandwidthCollector::new();
    
    // Take baseline measurement
    let baseline = collector.collect().expect("Baseline collection should work");
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Generate controlled network activity
    let activity_start = Instant::now();
    let mut activity_handles = Vec::new();
    
    // Create multiple concurrent network activities
    for i in 0..3 {
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::new();
            let mut total_bytes = 0u64;
            
            for _ in 0..5 {
                match client.get(&format!("http://httpbin.org/bytes/{}", 1024 * (i + 1))).send().await {
                    Ok(response) => {
                        if let Ok(bytes) = response.bytes().await {
                            total_bytes += bytes.len() as u64;
                        }
                    }
                    Err(_) => {
                        // Network activity failed, but that's okay for this test
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            
            total_bytes
        });
        activity_handles.push(handle);
    }
    
    // Wait for network activity to complete
    let mut total_generated_bytes = 0u64;
    for handle in activity_handles {
        if let Ok(bytes) = handle.await {
            total_generated_bytes += bytes;
        }
    }
    
    let activity_duration = activity_start.elapsed();
    tokio::time::sleep(Duration::from_millis(200)).await; // Allow for measurement settling
    
    // Take measurement after activity
    let after_activity = collector.collect().expect("After activity collection should work");
    
    println!("Network activity test: generated ~{} bytes over {:.1}s", 
             total_generated_bytes, activity_duration.as_secs_f64());
    
    // Verify that some interfaces show increased activity
    let mut activity_detected = false;
    for (baseline_stat, after_stat) in baseline.iter().zip(after_activity.iter()) {
        assert_eq!(baseline_stat.interface_name, after_stat.interface_name);
        
        let rx_increase = after_stat.bytes_received.saturating_sub(baseline_stat.bytes_received);
        let tx_increase = after_stat.bytes_sent.saturating_sub(baseline_stat.bytes_sent);
        
        if rx_increase > 0 || tx_increase > 0 {
            activity_detected = true;
            println!("Interface {} activity: +{} RX, +{} TX, speed: {:.1} bps down, {:.1} bps up", 
                    after_stat.interface_name, rx_increase, tx_increase,
                    after_stat.download_speed_bps, after_stat.upload_speed_bps);
        }
        
        // Speed calculations should be reasonable if there was activity
        if after_stat.calculation_confidence != CalculationConfidence::None {
            assert!(after_stat.download_speed_bps >= 0.0, "Download speed should be non-negative");
            assert!(after_stat.upload_speed_bps >= 0.0, "Upload speed should be non-negative");
            assert!(after_stat.download_speed_bps.is_finite(), "Download speed should be finite");
            assert!(after_stat.upload_speed_bps.is_finite(), "Upload speed should be finite");
        }
    }
    
    // Note: We don't assert activity_detected because network requests might fail in test environment
    if activity_detected {
        println!("Network activity successfully detected and measured");
    } else {
        println!("No network activity detected (possibly due to network issues in test environment)");
    }
}

#[tokio::test]
async fn test_interface_filtering_accuracy() {
    // Test that interface filtering works correctly across different collection methods
    let mut collector = BandwidthCollector::new();
    
    // Collect using all methods
    let all_stats = collector.collect().expect("All interfaces collection should work");
    let filtered_stats = collector.collect_filtered().expect("Filtered collection should work");
    let default_stats = collector.collect_default().expect("Default collection should work");
    let important_stats = collector.collect_important().expect("Important collection should work");
    
    println!("Interface filtering test:");
    println!("  All interfaces: {}", all_stats.len());
    println!("  Filtered interfaces: {}", filtered_stats.len());
    println!("  Default interfaces: {}", default_stats.len());
    println!("  Important interfaces: {}", important_stats.len());
    
    // Verify filtering hierarchy
    assert!(important_stats.len() <= default_stats.len(), 
           "Important ({}) should be subset of default ({})", 
           important_stats.len(), default_stats.len());
    assert!(default_stats.len() <= filtered_stats.len(), 
           "Default ({}) should be subset of filtered ({})", 
           default_stats.len(), filtered_stats.len());
    assert!(filtered_stats.len() <= all_stats.len(), 
           "Filtered ({}) should be subset of all ({})", 
           filtered_stats.len(), all_stats.len());
    
    // Verify that filtered interfaces are actually subsets
    let all_names: std::collections::HashSet<_> = all_stats.iter().map(|s| &s.interface_name).collect();
    let filtered_names: std::collections::HashSet<_> = filtered_stats.iter().map(|s| &s.interface_name).collect();
    let default_names: std::collections::HashSet<_> = default_stats.iter().map(|s| &s.interface_name).collect();
    let important_names: std::collections::HashSet<_> = important_stats.iter().map(|s| &s.interface_name).collect();
    
    assert!(important_names.is_subset(&default_names), "Important interfaces should be subset of default");
    assert!(default_names.is_subset(&filtered_names), "Default interfaces should be subset of filtered");
    assert!(filtered_names.is_subset(&all_names), "Filtered interfaces should be subset of all");
    
    // Test interface information consistency
    for stat in &important_stats {
        let info = collector.get_interface_info(&stat.interface_name);
        assert_eq!(info.name, stat.interface_name, "Interface info name should match");
        
        // Important interfaces should generally be physical, wifi, or important virtual interfaces
        assert!(info.is_physical || info.is_wifi || info.is_important_virtual, 
               "Important interface {} should be physical, wifi, or important virtual: physical={}, wifi={}, important_virtual={}", 
               stat.interface_name, info.is_physical, info.is_wifi, info.is_important_virtual);
    }
    
    // Test that loopback interfaces are handled correctly
    let loopback_in_all = all_stats.iter().any(|s| s.interface_type == InterfaceType::Loopback);
    let loopback_in_important = important_stats.iter().any(|s| s.interface_type == InterfaceType::Loopback);
    
    if loopback_in_all {
        println!("  Loopback interface found in all interfaces");
        // Loopback should generally not be in important interfaces unless specifically configured
        if loopback_in_important {
            println!("  Loopback interface included in important interfaces");
        }
    }
}

#[tokio::test]
async fn test_error_recovery_scenarios() {
    // Test various error recovery scenarios
    let mut collector = BandwidthCollector::new();
    
    // Test normal operation first
    let normal_stats = collector.collect().expect("Normal collection should work");
    assert!(!normal_stats.is_empty(), "Should have interfaces");
    
    // Test with different retry configurations
    let retry_configs = vec![
        (1, 10),   // Minimal retries
        (3, 50),   // Default-like
        (5, 100),  // Aggressive retries
    ];
    
    for (max_retries, delay_ms) in retry_configs {
        let mut retry_collector = BandwidthCollector::with_retry_config(max_retries, delay_ms);
        
        match retry_collector.collect() {
            Ok(stats) => {
                assert!(!stats.is_empty(), "Retry collector should work with config ({}, {})", max_retries, delay_ms);
                
                // Verify data quality
                for stat in &stats {
                    assert!(stat.download_speed_bps >= 0.0, "Speed should be non-negative");
                    assert!(stat.upload_speed_bps >= 0.0, "Speed should be non-negative");
                    assert!(stat.download_speed_bps.is_finite(), "Speed should be finite");
                    assert!(stat.upload_speed_bps.is_finite(), "Speed should be finite");
                }
            }
            Err(e) => {
                println!("Retry collector failed with config ({}, {}): {}", max_retries, delay_ms, e);
                // This is acceptable in some test environments
            }
        }
    }
    
    // Test cache clearing and recovery
    collector.clear_interface_cache();
    let post_clear_stats = collector.collect().expect("Collection after cache clear should work");
    assert!(!post_clear_stats.is_empty(), "Should work after cache clear");
    
    // Test multiple rapid collections (stress test)
    let rapid_collection_count = 20;
    let mut rapid_success_count = 0;
    let mut rapid_error_count = 0;
    
    for i in 0..rapid_collection_count {
        match collector.collect() {
            Ok(_) => rapid_success_count += 1,
            Err(e) => {
                rapid_error_count += 1;
                println!("Rapid collection #{} failed: {}", i, e);
            }
        }
        
        // Small delay to prevent overwhelming the system
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    let rapid_success_rate = rapid_success_count as f64 / rapid_collection_count as f64;
    println!("Rapid collection test: {}/{} successful ({:.1}%)", 
             rapid_success_count, rapid_collection_count, rapid_success_rate * 100.0);
    
    // Should have high success rate even under stress
    assert!(rapid_success_rate >= 0.8, "Rapid collection success rate should be high: {:.1}%", rapid_success_rate * 100.0);
    
    // Test interface manager statistics
    let (cache_size, platform) = collector.get_interface_manager_stats();
    println!("Interface manager: {} cached entries on {}", cache_size, platform);
    assert!(cache_size < 100, "Cache size should be reasonable: {}", cache_size);
    assert!(!platform.is_empty(), "Platform should be detected");
}