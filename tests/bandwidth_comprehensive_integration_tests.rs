use kaipo_watcher::collectors::bandwidth_collector::{BandwidthCollector, CalculationConfidence, InterfaceType, InterfaceState};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Comprehensive integration tests for bandwidth monitoring functionality
/// These tests verify the bandwidth collector works correctly in real-world scenarios
/// covering speed calculations, cross-platform compatibility, long-running scenarios, and performance

#[tokio::test]
async fn test_speed_calculations_against_known_network_activity() {
    // Test that speed calculations work correctly with known network activity patterns
    let mut collector = BandwidthCollector::new();
    
    // Take initial baseline reading to establish interface state
    let initial_stats = collector.collect().expect("Failed to collect initial stats");
    assert!(!initial_stats.is_empty(), "Should find at least one network interface");
    
    println!("Speed calculation test: Found {} interfaces", initial_stats.len());
    for stat in &initial_stats {
        println!("  {}: type={:?}, state={:?}", stat.interface_name, stat.interface_type, stat.interface_state);
    }
    
    // Generate controlled network activity to test speed calculations
    let network_activity = tokio::spawn(async {
        // Create multiple HTTP requests to generate measurable network activity
        let client = reqwest::Client::new();
        for i in 0..10 {
            match tokio::time::timeout(
                Duration::from_millis(200),
                client.get("http://httpbin.org/bytes/2048").send()
            ).await {
                Ok(Ok(response)) => {
                    if let Ok(bytes) = response.bytes().await {
                        println!("Network activity {}: Downloaded {} bytes", i + 1, bytes.len());
                    }
                }
                Ok(Err(e)) => println!("Network request {} failed: {}", i + 1, e),
                Err(_) => println!("Network request {} timed out", i + 1),
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    
    // Wait for network activity to generate traffic
    tokio::time::sleep(Duration::from_millis(1500)).await;
    
    // Take second reading for speed calculation
    let second_stats = collector.collect().expect("Failed to collect second stats");
    assert_eq!(initial_stats.len(), second_stats.len(), "Interface count should remain consistent");
    
    // Verify that speed calculations are mathematically sound
    let mut interfaces_with_activity = 0;
    let mut total_download_speed = 0.0;
    let mut total_upload_speed = 0.0;
    
    for (initial_stat, second_stat) in initial_stats.iter().zip(second_stats.iter()) {
        assert_eq!(initial_stat.interface_name, second_stat.interface_name, "Interface order should be consistent");
        
        // Verify speed calculation properties
        assert!(second_stat.download_speed_bps >= 0.0, 
               "Download speed should be non-negative for {}: {:.2} B/s", 
               second_stat.interface_name, second_stat.download_speed_bps);
        assert!(second_stat.upload_speed_bps >= 0.0, 
               "Upload speed should be non-negative for {}: {:.2} B/s", 
               second_stat.interface_name, second_stat.upload_speed_bps);
        
        // Speed should not be impossibly high (> 1 Gbps for test environment)
        assert!(second_stat.download_speed_bps < 1_000_000_000.0, 
               "Download speed should be realistic for {}: {:.2} B/s", 
               second_stat.interface_name, second_stat.download_speed_bps);
        assert!(second_stat.upload_speed_bps < 1_000_000_000.0, 
               "Upload speed should be realistic for {}: {:.2} B/s", 
               second_stat.interface_name, second_stat.upload_speed_bps);
        
        // Verify finite values
        assert!(second_stat.download_speed_bps.is_finite(), 
               "Download speed should be finite for {}", second_stat.interface_name);
        assert!(second_stat.upload_speed_bps.is_finite(), 
               "Upload speed should be finite for {}", second_stat.interface_name);
        
        // Check for network activity
        if second_stat.download_speed_bps > 0.0 || second_stat.upload_speed_bps > 0.0 {
            interfaces_with_activity += 1;
            total_download_speed += second_stat.download_speed_bps;
            total_upload_speed += second_stat.upload_speed_bps;
            
            println!("Interface {} shows activity: down={:.2} B/s, up={:.2} B/s, confidence={:?}", 
                    second_stat.interface_name, second_stat.download_speed_bps, 
                    second_stat.upload_speed_bps, second_stat.calculation_confidence);
        }
        
        // Verify confidence levels are appropriate
        match second_stat.calculation_confidence {
            CalculationConfidence::None => {
                // This is acceptable for interfaces with no previous data or no activity
                assert_eq!(second_stat.download_speed_bps, 0.0, "None confidence should have 0 speed");
                assert_eq!(second_stat.upload_speed_bps, 0.0, "None confidence should have 0 speed");
            }
            CalculationConfidence::Low | CalculationConfidence::Medium | CalculationConfidence::High => {
                // These are all valid confidence levels for active calculations
            }
        }
        
        // Time since last update should be reasonable
        assert!(second_stat.time_since_last_update >= 0.0, 
               "Time since last update should be non-negative for {}", second_stat.interface_name);
        assert!(second_stat.time_since_last_update < 10.0, 
               "Time since last update should be recent for {}: {:.3}s", 
               second_stat.interface_name, second_stat.time_since_last_update);
        
        // Verify byte counters are monotonic (increasing or same)
        assert!(second_stat.bytes_received >= initial_stat.bytes_received,
               "RX bytes should be monotonic for {}: {} -> {}", 
               second_stat.interface_name, initial_stat.bytes_received, second_stat.bytes_received);
        assert!(second_stat.bytes_sent >= initial_stat.bytes_sent,
               "TX bytes should be monotonic for {}: {} -> {}", 
               second_stat.interface_name, initial_stat.bytes_sent, second_stat.bytes_sent);
    }
    
    // Wait for network activity task to complete
    let _ = network_activity.await;
    
    println!("Speed calculation results: {}/{} interfaces with activity, total_down={:.2} B/s, total_up={:.2} B/s", 
             interfaces_with_activity, second_stats.len(), total_download_speed, total_upload_speed);
    
    // Test precision with controlled timing
    let mut precision_collector = BandwidthCollector::new();
    let baseline = precision_collector.collect().expect("Precision baseline should work");
    
    // Wait exactly 1 second for precise timing validation
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    let measurement = precision_collector.collect().expect("Precision measurement should work");
    
    // Verify timing accuracy for precision test
    for (baseline_stat, measurement_stat) in baseline.iter().zip(measurement.iter()) {
        assert_eq!(baseline_stat.interface_name, measurement_stat.interface_name);
        
        // Time since last update should be approximately 1 second (allow for variance)
        if measurement_stat.calculation_confidence != CalculationConfidence::None {
            assert!(measurement_stat.time_since_last_update >= 0.9, 
                   "Time measurement should be accurate for {}: {:.3}s", 
                   measurement_stat.interface_name, measurement_stat.time_since_last_update);
            assert!(measurement_stat.time_since_last_update <= 1.2, 
                   "Time measurement should be accurate for {}: {:.3}s", 
                   measurement_stat.interface_name, measurement_stat.time_since_last_update);
        }
    }
    
    println!("Speed calculation test completed successfully");
}

#[tokio::test]
async fn test_cross_platform_compatibility() {
    // Test that bandwidth collection works across different platforms
    let mut collector = BandwidthCollector::new();
    let platform = std::env::consts::OS;
    
    println!("Cross-platform compatibility test on: {}", platform);
    
    // Test basic collection functionality
    let stats = collector.collect().expect("Cross-platform collection should work");
    assert!(!stats.is_empty(), "Should find network interfaces on any platform");
    
    println!("Found {} interfaces on {}", stats.len(), platform);
    
    // Test platform-specific interface naming conventions and validation
    for stat in &stats {
        assert!(!stat.interface_name.is_empty(), "Interface name should not be empty");
        assert!(stat.interface_name.len() <= 64, "Interface name should be reasonable length: {}", stat.interface_name.len());
        
        // Interface name should contain valid characters
        assert!(stat.interface_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':'), 
                "Interface name should contain valid characters: {}", stat.interface_name);
        
        println!("  {}: type={:?}, state={:?}, rx={} bytes, tx={} bytes", 
                stat.interface_name, stat.interface_type, stat.interface_state, 
                stat.bytes_received, stat.bytes_sent);
        
        // Platform-specific interface name validation
        match platform {
            "macos" => {
                // macOS interfaces typically start with en, lo, utun, anpi, etc.
                let valid_prefixes = ["en", "lo", "utun", "anpi", "awdl", "llw", "bridge", "gif", "stf", "XHC"];
                let has_valid_prefix = valid_prefixes.iter().any(|prefix| stat.interface_name.starts_with(prefix));
                if !has_valid_prefix {
                    println!("    Warning: Unexpected macOS interface name: {}", stat.interface_name);
                }
                
                // Validate macOS-specific interface types
                if stat.interface_name.starts_with("en") {
                    assert!(matches!(stat.interface_type, InterfaceType::Ethernet | InterfaceType::WiFi | InterfaceType::Unknown), 
                           "en* interfaces should be ethernet/wifi on macOS: {}", stat.interface_name);
                } else if stat.interface_name == "lo0" {
                    assert_eq!(stat.interface_type, InterfaceType::Loopback, "lo0 should be loopback on macOS");
                }
            }
            "linux" => {
                // Linux interfaces typically start with eth, wlan, lo, docker, veth, etc.
                let valid_prefixes = ["eth", "wlan", "lo", "docker", "veth", "br-", "virbr", "tun", "tap", "wg", "enp", "wlp"];
                let has_valid_prefix = valid_prefixes.iter().any(|prefix| stat.interface_name.starts_with(prefix));
                if !has_valid_prefix {
                    println!("    Warning: Unexpected Linux interface name: {}", stat.interface_name);
                }
                
                // Validate Linux-specific interface types
                if stat.interface_name.starts_with("eth") || stat.interface_name.starts_with("enp") {
                    assert!(matches!(stat.interface_type, InterfaceType::Ethernet | InterfaceType::Unknown), 
                           "eth*/enp* interfaces should be ethernet on Linux: {}", stat.interface_name);
                } else if stat.interface_name.starts_with("wlan") || stat.interface_name.starts_with("wlp") {
                    assert!(matches!(stat.interface_type, InterfaceType::WiFi | InterfaceType::Unknown), 
                           "wlan*/wlp* interfaces should be wifi on Linux: {}", stat.interface_name);
                } else if stat.interface_name == "lo" {
                    assert_eq!(stat.interface_type, InterfaceType::Loopback, "lo should be loopback on Linux");
                }
            }
            "windows" => {
                // Windows interfaces have various naming patterns
                assert!(stat.interface_name.len() >= 1, "Windows interface name should not be empty");
                // Windows interface validation is more complex due to varied naming
            }
            _ => {
                println!("    Testing on unknown platform: {}", platform);
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
    
    // Test platform-specific filtering methods
    let filtered_stats = collector.collect_filtered().expect("Filtered collection should work");
    let default_stats = collector.collect_default().expect("Default collection should work");
    let important_stats = collector.collect_important().expect("Important collection should work");
    
    println!("Platform filtering results: all={}, filtered={}, default={}, important={}", 
             stats.len(), filtered_stats.len(), default_stats.len(), important_stats.len());
    
    // Verify filtering relationships
    assert!(important_stats.len() <= default_stats.len(), "Important interfaces should be subset of default");
    assert!(default_stats.len() <= filtered_stats.len(), "Default interfaces should be subset of filtered");
    assert!(filtered_stats.len() <= stats.len(), "Filtered interfaces should be subset of all");
    
    // Test interface information retrieval
    let interface_info = collector.get_all_interface_info().expect("Should get interface info");
    assert!(!interface_info.is_empty(), "Should have interface information");
    
    println!("Interface analysis results:");
    for info in &interface_info {
        assert!(!info.name.is_empty(), "Interface info should have valid name");
        
        println!("  {}: type={:?}, relevance={}, should_filter={}", 
                info.name, info.interface_type, info.relevance.score, info.should_filter);
        
        // Verify platform-specific interface analysis
        match platform {
            "macos" => {
                if info.name.starts_with("en") {
                    // en* interfaces should be ethernet or wifi on macOS
                    assert!(matches!(info.interface_type, 
                                   kaipo_watcher::collectors::platform::interface_manager::EnhancedInterfaceType::Ethernet { .. } |
                                   kaipo_watcher::collectors::platform::interface_manager::EnhancedInterfaceType::WiFi { .. }), 
                           "en* interfaces should be ethernet or wifi on macOS");
                } else if info.name == "lo0" {
                    assert!(matches!(info.interface_type, 
                                   kaipo_watcher::collectors::platform::interface_manager::EnhancedInterfaceType::Loopback), 
                           "lo0 should be loopback on macOS");
                }
            }
            "linux" => {
                if info.name.starts_with("eth") || info.name.starts_with("enp") {
                    assert!(matches!(info.interface_type, 
                                   kaipo_watcher::collectors::platform::interface_manager::EnhancedInterfaceType::Ethernet { .. }), 
                           "eth*/enp* interfaces should be ethernet on Linux");
                } else if info.name.starts_with("wlan") || info.name.starts_with("wlp") {
                    assert!(matches!(info.interface_type, 
                                   kaipo_watcher::collectors::platform::interface_manager::EnhancedInterfaceType::WiFi { .. }), 
                           "wlan*/wlp* interfaces should be wifi on Linux");
                } else if info.name == "lo" {
                    assert!(matches!(info.interface_type, 
                                   kaipo_watcher::collectors::platform::interface_manager::EnhancedInterfaceType::Loopback), 
                           "lo should be loopback on Linux");
                }
            }
            _ => {
                // For other platforms, just ensure basic functionality
            }
        }
    }
    
    // Test platform-specific error handling
    let mut error_collector = BandwidthCollector::with_retry_config(1, 10);
    match error_collector.collect() {
        Ok(error_stats) => {
            assert!(!error_stats.is_empty(), "Error collector should still work");
            println!("Platform error handling test: {} interfaces collected", error_stats.len());
        }
        Err(e) => {
            println!("Platform-specific error handling test result: {}", e);
        }
    }
    
    println!("Cross-platform compatibility test completed successfully");
}

#[tokio::test]
async fn test_long_running_collection_scenarios() {
    // Test that bandwidth collection works correctly over extended periods
    let mut collector = BandwidthCollector::new();
    let test_duration = Duration::from_secs(15); // Extended test duration
    let collection_interval = Duration::from_millis(250); // Collect every 250ms
    
    println!("Long-running collection test: {}s duration, {}ms intervals", 
             test_duration.as_secs(), collection_interval.as_millis());
    
    let start_time = Instant::now();
    let mut collection_count = 0;
    let mut all_stats = Vec::new();
    let mut collection_times = Vec::new();
    let mut error_count = 0;
    let mut confidence_evolution = HashMap::new();
    
    // Collect data over the test period
    while start_time.elapsed() < test_duration {
        let collection_start = Instant::now();
        
        match collector.collect() {
            Ok(stats) => {
                let collection_duration = collection_start.elapsed();
                all_stats.push((Instant::now(), stats.clone()));
                collection_times.push(collection_duration);
                collection_count += 1;
                
                // Track confidence evolution
                for stat in &stats {
                    confidence_evolution.entry(stat.interface_name.clone())
                        .or_insert_with(Vec::new)
                        .push((collection_count, format!("{:?}", stat.calculation_confidence)));
                }
                
                if collection_count % 20 == 0 {
                    println!("  Collection #{}: {} interfaces, avg_time={:.3}ms", 
                            collection_count, stats.len(), 
                            collection_times.iter().sum::<Duration>().as_secs_f64() * 1000.0 / collection_times.len() as f64);
                }
            }
            Err(e) => {
                error_count += 1;
                println!("  Collection error #{}: {}", error_count, e);
                
                // Allow some errors but not excessive
                assert!(error_count < collection_count / 20, "Too many collection errors: {}/{}", error_count, collection_count);
            }
        }
        
        tokio::time::sleep(collection_interval).await;
    }
    
    assert!(collection_count >= 50, "Should have collected data many times: {}", collection_count);
    println!("Long-running test completed: {} collections, {} errors over {:.1}s", 
             collection_count, error_count, test_duration.as_secs_f64());
    
    // Analyze collection performance over time
    let avg_collection_time = collection_times.iter().sum::<Duration>() / collection_times.len() as u32;
    let max_collection_time = collection_times.iter().max().unwrap();
    let min_collection_time = collection_times.iter().min().unwrap();
    
    println!("Collection performance analysis:");
    println!("  Average: {:.3}ms", avg_collection_time.as_secs_f64() * 1000.0);
    println!("  Minimum: {:.3}ms", min_collection_time.as_secs_f64() * 1000.0);
    println!("  Maximum: {:.3}ms", max_collection_time.as_secs_f64() * 1000.0);
    
    // Collection times should be reasonable and not degrade significantly
    assert!(avg_collection_time < Duration::from_millis(200), 
           "Average collection time should be reasonable: {:?}", avg_collection_time);
    assert!(*max_collection_time < Duration::from_millis(1000), 
           "Maximum collection time should be reasonable: {:?}", max_collection_time);
    
    // Verify consistency across collections
    let first_interfaces: Vec<String> = all_stats[0].1.iter().map(|s| s.interface_name.clone()).collect();
    let mut interface_stability_violations = 0;
    let mut speed_anomalies = 0;
    
    for (_timestamp, stats) in &all_stats {
        let current_interfaces: Vec<String> = stats.iter().map(|s| s.interface_name.clone()).collect();
        
        // Allow for some interface changes but not dramatic ones
        let interface_diff = first_interfaces.len().abs_diff(current_interfaces.len());
        if interface_diff > 3 {
            interface_stability_violations += 1;
        }
        
        // Verify data quality for each interface
        for stat in stats {
            assert!(stat.bytes_received < u64::MAX / 2, "Byte counters should not overflow");
            assert!(stat.bytes_sent < u64::MAX / 2, "Byte counters should not overflow");
            
            // Speed calculations should be stable (not wildly fluctuating)
            if stat.calculation_confidence != CalculationConfidence::None {
                assert!(stat.download_speed_bps < 2_000_000_000.0, "Speed should be reasonable"); // < 2 Gbps
                assert!(stat.upload_speed_bps < 2_000_000_000.0, "Speed should be reasonable");
                assert!(stat.download_speed_bps.is_finite(), "Download speed should be finite");
                assert!(stat.upload_speed_bps.is_finite(), "Upload speed should be finite");
                
                // Check for speed anomalies (impossibly high speeds)
                if stat.download_speed_bps > 100_000_000.0 || stat.upload_speed_bps > 100_000_000.0 {
                    speed_anomalies += 1;
                }
            }
            
            // Timestamps should be reasonable
            assert!(stat.timestamp <= chrono::Utc::now(), "Timestamp should not be in future");
            assert!(stat.time_since_last_update >= 0.0, "Time since last update should be non-negative");
        }
    }
    
    // Allow some instability but not excessive
    assert!(interface_stability_violations < all_stats.len() / 20, 
           "Interface stability violations should be minimal: {}/{}", 
           interface_stability_violations, all_stats.len());
    assert!(speed_anomalies < all_stats.len() / 50, 
           "Speed anomalies should be rare: {}/{}", 
           speed_anomalies, all_stats.len());
    
    // Analyze confidence evolution
    println!("Confidence evolution analysis:");
    for (interface_name, confidence_history) in &confidence_evolution {
        let final_confidence = &confidence_history.last().unwrap().1;
        let confidence_changes = confidence_history.windows(2)
            .filter(|window| window[0].1 != window[1].1)
            .count();
        
        println!("  {}: final={:?}, changes={}", interface_name, final_confidence, confidence_changes);
    }
    
    // Test that confidence improves over time for active interfaces
    let final_stats = &all_stats.last().unwrap().1;
    let mut confidence_distribution = HashMap::new();
    
    for stat in final_stats {
        let confidence_str = format!("{:?}", stat.calculation_confidence);
        *confidence_distribution.entry(confidence_str).or_insert(0) += 1;
    }
    
    println!("Final confidence distribution: {:?}", confidence_distribution);
    
    // At least some interfaces should have good confidence after extended collection
    let high_confidence_count = confidence_distribution.get("High").unwrap_or(&0) +
                               confidence_distribution.get("Medium").unwrap_or(&0);
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
    
    println!("Long-running collection test completed successfully");
}

#[tokio::test]
async fn test_performance_impact() {
    // Test that bandwidth collection doesn't significantly impact system performance
    let mut collector = BandwidthCollector::new();
    
    println!("Performance impact test starting...");
    
    // Measure baseline system performance
    let baseline_start = Instant::now();
    let baseline_iterations = 5000; // Reduced for faster test execution
    let mut baseline_results = Vec::new();
    
    for i in 0..baseline_iterations {
        let work_start = Instant::now();
        // Simulate CPU work
        let _result: u64 = (0..200).map(|x| x * x).sum();
        baseline_results.push(work_start.elapsed());
        
        // Periodic yield to prevent test timeout
        if i % 500 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    let baseline_duration = baseline_start.elapsed();
    let baseline_avg = baseline_results.iter().sum::<Duration>() / baseline_results.len() as u32;
    
    println!("Baseline performance: {:.3}ms total, {:.3}μs per work unit", 
             baseline_duration.as_secs_f64() * 1000.0,
             baseline_avg.as_nanos() as f64 / 1000.0);
    
    // Measure performance with bandwidth collection
    let collection_start = Instant::now();
    let collection_iterations = 50; // Reduced for faster execution
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
            let _result: u64 = (0..200).map(|x| x * x).sum();
            collection_results.push(work_start.elapsed());
        }
        
        // Periodic yield
        if i % 5 == 0 {
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
    assert!(performance_ratio < 15.0, "Overall performance impact should be reasonable: {:.2}x", performance_ratio);
    assert!(work_slowdown < 10.0, "Individual work units should not be significantly slower: {:.2}x", work_slowdown);
    
    // Collection time assertions
    assert!(avg_collection_time < Duration::from_millis(200), 
            "Average collection time should be fast: {:.3}ms", avg_collection_time.as_secs_f64() * 1000.0);
    assert!(*max_collection_time < Duration::from_millis(1000), 
            "Maximum collection time should be reasonable: {:.3}ms", max_collection_time.as_secs_f64() * 1000.0);
    
    // Test memory usage during performance test
    let memory_test_start = Instant::now();
    let memory_iterations = 200; // Reduced for faster execution
    
    for i in 0..memory_iterations {
        let _stats = collector.collect().expect("Memory performance test should work");
        
        // Periodic cache clearing to test cleanup performance
        if i % 20 == 0 {
            collector.clear_interface_cache();
        }
        
        // Yield periodically
        if i % 10 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    let memory_test_duration = memory_test_start.elapsed();
    let avg_memory_collection = memory_test_duration / memory_iterations;
    
    println!("  Memory test: {} collections in {:.3}ms, avg={:.3}ms per collection", 
             memory_iterations, 
             memory_test_duration.as_secs_f64() * 1000.0,
             avg_memory_collection.as_secs_f64() * 1000.0);
    
    assert!(avg_memory_collection < Duration::from_millis(50), 
            "Memory test collections should be fast: {:.3}ms", avg_memory_collection.as_secs_f64() * 1000.0);
    
    // Test concurrent performance impact
    let concurrent_start = Instant::now();
    let concurrent_tasks = 3; // Reduced for faster execution
    let collections_per_task = 10; // Reduced for faster execution
    
    let mut handles = Vec::new();
    for task_id in 0..concurrent_tasks {
        let handle = tokio::spawn(async move {
            let mut task_collector = BandwidthCollector::new();
            let mut task_times = Vec::new();
            
            for _ in 0..collections_per_task {
                let start = Instant::now();
                let _stats = task_collector.collect().expect("Concurrent collection should work");
                task_times.push(start.elapsed());
                
                tokio::time::sleep(Duration::from_millis(20)).await;
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
    
    assert!(avg_concurrent_time < Duration::from_millis(500), 
            "Concurrent collections should not be significantly slower: {:.3}ms", 
            avg_concurrent_time.as_secs_f64() * 1000.0);
    
    println!("Performance impact test completed successfully");
}