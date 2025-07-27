use chrono::Utc;
use kaipo_watcher::collectors::bandwidth_collector::{BandwidthCollector, CalculationConfidence, BandwidthError, InterfaceType, InterfaceState};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::process::Command;

/// Advanced integration tests for bandwidth monitoring
/// These tests focus on edge cases, platform-specific behavior, and advanced scenarios

#[tokio::test]
async fn test_bandwidth_calculation_precision() {
    // Test precision of bandwidth calculations under various conditions
    let mut collector = BandwidthCollector::new();
    
    // Test with very short intervals
    let baseline = collector.collect().expect("Baseline should work");
    
    // Test multiple short intervals
    let short_intervals = vec![100, 200, 500, 1000]; // milliseconds
    
    for interval_ms in short_intervals {
        tokio::time::sleep(Duration::from_millis(interval_ms)).await;
        
        let stats = collector.collect().expect("Short interval collection should work");
        
        for stat in &stats {
            // Verify precision handling for short intervals
            if stat.calculation_confidence != CalculationConfidence::None {
                assert!(stat.time_since_last_update > 0.0, "Time should be positive");
                assert!(stat.time_since_last_update < 2.0, "Time should be reasonable for interval {}ms", interval_ms);
                
                // Speed calculations should handle precision correctly
                assert!(stat.download_speed_bps.is_finite(), "Download speed should be finite");
                assert!(stat.upload_speed_bps.is_finite(), "Upload speed should be finite");
                assert!(stat.download_speed_bps >= 0.0, "Download speed should be non-negative");
                assert!(stat.upload_speed_bps >= 0.0, "Upload speed should be non-negative");
            }
        }
    }
    
    // Test with longer intervals for stability
    tokio::time::sleep(Duration::from_secs(2)).await;
    let stable_stats = collector.collect().expect("Stable collection should work");
    
    // Longer intervals should generally have higher confidence
    let high_confidence_count = stable_stats.iter()
        .filter(|s| matches!(s.calculation_confidence, CalculationConfidence::High | CalculationConfidence::Medium))
        .count();
    
    println!("Precision test: {}/{} interfaces with high/medium confidence after 2s interval", 
             high_confidence_count, stable_stats.len());
}

#[tokio::test]
async fn test_interface_state_transitions() {
    // Test handling of interface state changes
    let mut collector = BandwidthCollector::new();
    
    // Get initial interface states
    let initial_stats = collector.collect().expect("Initial collection should work");
    let initial_interfaces: HashMap<String, InterfaceState> = initial_stats.iter()
        .map(|s| (s.interface_name.clone(), s.interface_state.clone()))
        .collect();
    
    println!("Initial interface states:");
    for (name, state) in &initial_interfaces {
        println!("  {}: {:?}", name, state);
    }
    
    // Monitor for state changes over time
    let monitoring_duration = Duration::from_secs(3);
    let check_interval = Duration::from_millis(500);
    let start_time = Instant::now();
    let mut state_changes = Vec::new();
    
    while start_time.elapsed() < monitoring_duration {
        tokio::time::sleep(check_interval).await;
        
        let current_stats = collector.collect().expect("State monitoring should work");
        
        for stat in &current_stats {
            if let Some(initial_state) = initial_interfaces.get(&stat.interface_name) {
                if &stat.interface_state != initial_state {
                    state_changes.push((
                        stat.interface_name.clone(),
                        initial_state.clone(),
                        stat.interface_state.clone(),
                        start_time.elapsed()
                    ));
                }
            }
        }
    }
    
    if !state_changes.is_empty() {
        println!("Interface state changes detected:");
        for (name, old_state, new_state, when) in &state_changes {
            println!("  {} changed from {:?} to {:?} at {:.1}s", name, old_state, new_state, when.as_secs_f64());
        }
    } else {
        println!("No interface state changes detected during monitoring period");
    }
    
    // Verify that state changes are handled gracefully
    let final_stats = collector.collect().expect("Final collection should work");
    for stat in &final_stats {
        // All interfaces should have valid states
        match stat.interface_state {
            InterfaceState::Up | InterfaceState::Down | InterfaceState::Unknown => {
                // All valid states
            }
        }
        
        // Speed calculations should still work after state changes
        assert!(stat.download_speed_bps >= 0.0, "Speed should be non-negative after state changes");
        assert!(stat.upload_speed_bps >= 0.0, "Speed should be non-negative after state changes");
    }
}

#[tokio::test]
async fn test_counter_overflow_handling() {
    // Test handling of counter overflow scenarios (simulated)
    let mut collector = BandwidthCollector::new();
    
    // Get baseline
    let baseline = collector.collect().expect("Baseline should work");
    
    // Test with rapid collections to potentially trigger edge cases
    let rapid_collections = 50;
    let mut all_collections = Vec::new();
    
    for i in 0..rapid_collections {
        let stats = collector.collect().expect("Rapid collection should work");
        all_collections.push((i, stats));
        
        // Very short delay to stress the system
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    
    // Analyze for counter consistency
    let mut counter_anomalies = 0;
    let mut confidence_distribution = HashMap::new();
    
    for (collection_id, stats) in &all_collections {
        for stat in stats {
            // Track confidence distribution
            *confidence_distribution.entry(stat.calculation_confidence.clone()).or_insert(0) += 1;
            
            // Check for counter anomalies
            if let Some((_, baseline_stats)) = all_collections.first() {
                if let Some(baseline_stat) = baseline_stats.iter().find(|s| s.interface_name == stat.interface_name) {
                    // Counters should generally be monotonic
                    if stat.bytes_received < baseline_stat.bytes_received || stat.bytes_sent < baseline_stat.bytes_sent {
                        // This could indicate counter reset or overflow
                        counter_anomalies += 1;
                        println!("Counter anomaly detected in collection {} for {}: RX {} -> {}, TX {} -> {}", 
                                collection_id, stat.interface_name,
                                baseline_stat.bytes_received, stat.bytes_received,
                                baseline_stat.bytes_sent, stat.bytes_sent);
                    }
                }
            }
            
            // Verify that speed calculations handle edge cases
            if stat.calculation_confidence != CalculationConfidence::None {
                assert!(stat.download_speed_bps.is_finite(), "Speed should be finite even with rapid collections");
                assert!(stat.upload_speed_bps.is_finite(), "Speed should be finite even with rapid collections");
                assert!(stat.download_speed_bps >= 0.0, "Speed should be non-negative");
                assert!(stat.upload_speed_bps >= 0.0, "Speed should be non-negative");
            }
        }
    }
    
    println!("Counter overflow test results:");
    println!("  Collections: {}", rapid_collections);
    println!("  Counter anomalies: {}", counter_anomalies);
    println!("  Confidence distribution: {:?}", confidence_distribution);
    
    // Some counter anomalies might be expected in rapid collection scenarios
    assert!(counter_anomalies < rapid_collections / 5, "Too many counter anomalies: {}", counter_anomalies);
}

#[tokio::test]
async fn test_platform_specific_interfaces() {
    // Test platform-specific interface handling
    let mut collector = BandwidthCollector::new();
    let platform = std::env::consts::OS;
    
    let stats = collector.collect().expect("Platform test should work");
    let interface_info = collector.get_all_interface_info().expect("Interface info should work");
    
    println!("Platform-specific interface test on {}:", platform);
    
    // Create a map for easy lookup
    let info_map: HashMap<String, _> = interface_info.iter()
        .map(|info| (info.name.clone(), info))
        .collect();
    
    for stat in &stats {
        let info = info_map.get(&stat.interface_name).expect("Should have interface info");
        
        println!("  {}: type={:?}, state={:?}, physical={}, wifi={}, loopback={}, virtual={}", 
                stat.interface_name, stat.interface_type, stat.interface_state,
                info.is_physical, info.is_wifi, info.is_loopback, info.is_virtual);
        
        // Platform-specific validation
        match platform {
            "macos" => {
                // macOS specific interface patterns
                if stat.interface_name.starts_with("en") {
                    // Ethernet interfaces should be detected correctly
                    assert!(info.is_physical || info.is_wifi, 
                           "en* interface should be physical or wifi: {}", stat.interface_name);
                } else if stat.interface_name == "lo0" {
                    assert!(info.is_loopback, "lo0 should be loopback");
                    assert_eq!(stat.interface_type, InterfaceType::Loopback, "lo0 should have loopback type");
                } else if stat.interface_name.starts_with("utun") {
                    assert!(info.is_virtual, "utun* should be virtual");
                    assert_eq!(stat.interface_type, InterfaceType::Virtual, "utun* should have virtual type");
                }
            }
            "linux" => {
                // Linux specific interface patterns
                if stat.interface_name.starts_with("eth") {
                    assert!(info.is_physical, "eth* should be physical");
                    assert_eq!(stat.interface_type, InterfaceType::Ethernet, "eth* should be ethernet type");
                } else if stat.interface_name.starts_with("wlan") {
                    assert!(info.is_wifi, "wlan* should be wifi");
                    assert_eq!(stat.interface_type, InterfaceType::WiFi, "wlan* should be wifi type");
                } else if stat.interface_name == "lo" {
                    assert!(info.is_loopback, "lo should be loopback");
                    assert_eq!(stat.interface_type, InterfaceType::Loopback, "lo should have loopback type");
                }
            }
            "windows" => {
                // Windows interface validation is more complex due to naming
                // Just ensure basic functionality
                assert!(!stat.interface_name.is_empty(), "Windows interface should have name");
            }
            _ => {
                println!("Unknown platform: {}", platform);
            }
        }
        
        // Universal validations
        assert!(!stat.interface_name.is_empty(), "Interface name should not be empty");
        assert!(stat.bytes_received < u64::MAX, "Byte counters should be reasonable");
        assert!(stat.bytes_sent < u64::MAX, "Byte counters should be reasonable");
    }
    
    // Test filtering on this platform
    let filtered = collector.collect_filtered().expect("Filtered should work");
    let default = collector.collect_default().expect("Default should work");
    let important = collector.collect_important().expect("Important should work");
    
    println!("  Filtering results: all={}, filtered={}, default={}, important={}", 
             stats.len(), filtered.len(), default.len(), important.len());
    
    // Verify filtering makes sense for the platform
    assert!(important.len() <= default.len(), "Important should be subset of default");
    assert!(default.len() <= filtered.len(), "Default should be subset of filtered");
    assert!(filtered.len() <= stats.len(), "Filtered should be subset of all");
}

#[tokio::test]
async fn test_high_frequency_monitoring() {
    // Test high-frequency monitoring scenarios
    let mut collector = BandwidthCollector::new();
    
    // High frequency collection test
    let collection_frequency = Duration::from_millis(50); // 20 Hz
    let test_duration = Duration::from_secs(2);
    let start_time = Instant::now();
    
    let mut collections = Vec::new();
    let mut collection_times = Vec::new();
    let mut error_count = 0;
    
    while start_time.elapsed() < test_duration {
        let collect_start = Instant::now();
        
        match collector.collect() {
            Ok(stats) => {
                let collect_time = collect_start.elapsed();
                collections.push(stats);
                collection_times.push(collect_time);
            }
            Err(e) => {
                error_count += 1;
                println!("High frequency collection error: {}", e);
            }
        }
        
        tokio::time::sleep(collection_frequency).await;
    }
    
    let total_attempts = collections.len() + error_count;
    let success_rate = collections.len() as f64 / total_attempts as f64;
    let avg_collection_time = collection_times.iter().sum::<Duration>() / collection_times.len() as u32;
    let max_collection_time = collection_times.iter().max().unwrap_or(&Duration::ZERO);
    
    println!("High frequency monitoring results:");
    println!("  Frequency: {:.1} Hz", 1000.0 / collection_frequency.as_millis() as f64);
    println!("  Success rate: {:.1}% ({}/{})", success_rate * 100.0, collections.len(), total_attempts);
    println!("  Average collection time: {:.3}ms", avg_collection_time.as_secs_f64() * 1000.0);
    println!("  Maximum collection time: {:.3}ms", max_collection_time.as_secs_f64() * 1000.0);
    
    // High frequency monitoring should maintain reasonable performance
    assert!(success_rate >= 0.9, "High frequency success rate should be high: {:.1}%", success_rate * 100.0);
    assert!(avg_collection_time < Duration::from_millis(20), 
           "Average collection time should be fast: {:.3}ms", avg_collection_time.as_secs_f64() * 1000.0);
    assert!(max_collection_time < Duration::from_millis(100), 
           "Maximum collection time should be reasonable: {:.3}ms", max_collection_time.as_secs_f64() * 1000.0);
    
    // Verify data quality under high frequency
    if let Some(final_collection) = collections.last() {
        for stat in final_collection {
            assert!(stat.download_speed_bps >= 0.0, "Speed should be non-negative under high frequency");
            assert!(stat.upload_speed_bps >= 0.0, "Speed should be non-negative under high frequency");
            assert!(stat.download_speed_bps.is_finite(), "Speed should be finite under high frequency");
            assert!(stat.upload_speed_bps.is_finite(), "Speed should be finite under high frequency");
        }
    }
}

#[tokio::test]
async fn test_system_resource_usage() {
    // Test system resource usage during bandwidth monitoring
    let mut collector = BandwidthCollector::new();
    
    // Measure resource usage during normal operation
    let monitoring_duration = Duration::from_secs(5);
    let collection_interval = Duration::from_millis(100);
    let start_time = Instant::now();
    
    let mut cpu_samples = Vec::new();
    let mut memory_samples = Vec::new();
    let mut collection_count = 0;
    
    while start_time.elapsed() < monitoring_duration {
        let before_collection = Instant::now();
        
        // Perform collection
        match collector.collect() {
            Ok(_) => {
                collection_count += 1;
                let collection_time = before_collection.elapsed();
                cpu_samples.push(collection_time);
                
                // Estimate memory usage (simplified)
                let (cache_size, _) = collector.get_interface_manager_stats();
                memory_samples.push(cache_size);
            }
            Err(e) => {
                println!("Resource test collection error: {}", e);
            }
        }
        
        tokio::time::sleep(collection_interval).await;
    }
    
    // Analyze resource usage
    let avg_cpu_time = cpu_samples.iter().sum::<Duration>() / cpu_samples.len() as u32;
    let max_cpu_time = cpu_samples.iter().max().unwrap_or(&Duration::ZERO);
    let avg_cache_size = memory_samples.iter().sum::<usize>() / memory_samples.len();
    let max_cache_size = memory_samples.iter().max().unwrap_or(&0);
    
    println!("System resource usage analysis:");
    println!("  Collections: {}", collection_count);
    println!("  Average CPU time per collection: {:.3}ms", avg_cpu_time.as_secs_f64() * 1000.0);
    println!("  Maximum CPU time per collection: {:.3}ms", max_cpu_time.as_secs_f64() * 1000.0);
    println!("  Average cache size: {} entries", avg_cache_size);
    println!("  Maximum cache size: {} entries", max_cache_size);
    
    // Resource usage should be reasonable
    assert!(avg_cpu_time < Duration::from_millis(10), 
           "Average CPU time should be low: {:.3}ms", avg_cpu_time.as_secs_f64() * 1000.0);
    assert!(max_cpu_time < Duration::from_millis(50), 
           "Maximum CPU time should be reasonable: {:.3}ms", max_cpu_time.as_secs_f64() * 1000.0);
    assert!(avg_cache_size < 50, "Average cache size should be reasonable: {}", avg_cache_size);
    assert!(max_cache_size < 100, "Maximum cache size should be reasonable: {}", max_cache_size);
    
    // Test resource cleanup
    collector.clear_interface_cache();
    let (post_clear_cache_size, _) = collector.get_interface_manager_stats();
    assert!(post_clear_cache_size <= avg_cache_size, 
           "Cache should be cleared: {} <= {}", post_clear_cache_size, avg_cache_size);
}

#[tokio::test]
async fn test_edge_case_scenarios() {
    // Test various edge case scenarios
    let mut collector = BandwidthCollector::new();
    
    // Test with minimal retry configuration
    let mut minimal_collector = BandwidthCollector::with_retry_config(0, 1);
    match minimal_collector.collect() {
        Ok(stats) => {
            assert!(!stats.is_empty(), "Minimal retry collector should work");
        }
        Err(e) => {
            println!("Minimal retry collector failed (acceptable): {}", e);
        }
    }
    
    // Test with maximum retry configuration
    let mut max_collector = BandwidthCollector::with_retry_config(10, 1000);
    let max_stats = max_collector.collect().expect("Max retry collector should work");
    assert!(!max_stats.is_empty(), "Max retry collector should find interfaces");
    
    // Test rapid cache clearing
    for i in 0..10 {
        collector.clear_interface_cache();
        let stats = collector.collect().expect("Collection after cache clear should work");
        assert!(!stats.is_empty(), "Should work after cache clear #{}", i);
        
        if i % 3 == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    
    // Test interface info retrieval for non-existent interface
    let fake_interface_info = collector.get_interface_info("non_existent_interface_12345");
    assert_eq!(fake_interface_info.name, "non_existent_interface_12345");
    assert!(!fake_interface_info.is_physical, "Non-existent interface should not be physical");
    assert!(!fake_interface_info.is_wifi, "Non-existent interface should not be wifi");
    assert!(!fake_interface_info.is_loopback, "Non-existent interface should not be loopback");
    
    // Test total bandwidth calculation edge cases
    let (total_down, total_up) = collector.get_total_bandwidth();
    assert!(total_down >= 0.0, "Total download should be non-negative");
    assert!(total_up >= 0.0, "Total upload should be non-negative");
    assert!(total_down.is_finite(), "Total download should be finite");
    assert!(total_up.is_finite(), "Total upload should be finite");
    
    // Test with empty previous stats (fresh collector)
    let mut fresh_collector = BandwidthCollector::new();
    let fresh_stats = fresh_collector.collect().expect("Fresh collector should work");
    
    // First collection should have None confidence for speed calculations
    for stat in &fresh_stats {
        if stat.calculation_confidence == CalculationConfidence::None {
            assert_eq!(stat.download_speed_bps, 0.0, "First collection should have 0 speed");
            assert_eq!(stat.upload_speed_bps, 0.0, "First collection should have 0 speed");
        }
    }
    
    println!("Edge case scenarios completed successfully");
}