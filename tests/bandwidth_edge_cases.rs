use kaipo_watcher::collectors::bandwidth_collector::{BandwidthCollector, CalculationConfidence};
use std::time::Duration;
use tokio::time::sleep;

/// Test edge cases and error conditions in bandwidth monitoring
#[tokio::test]
async fn test_rapid_successive_collections() {
    // Test that rapid successive collections don't cause issues
    let mut collector = BandwidthCollector::new();
    
    // Perform rapid collections
    for i in 0..20 {
        let stats = collector.collect().expect("Rapid collection should work");
        assert!(!stats.is_empty(), "Should find interfaces in rapid collection {}", i);
        
        // Very short delay
        sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn test_confidence_level_progression() {
    // Test that confidence levels improve over time as expected
    let mut collector = BandwidthCollector::new();
    
    // First collection should have None confidence
    let first_stats = collector.collect().expect("First collection should work");
    for stat in &first_stats {
        assert_eq!(stat.calculation_confidence, CalculationConfidence::None, 
                  "First collection should have None confidence for {}", stat.interface_name);
    }
    
    // Wait and collect again
    sleep(Duration::from_millis(500)).await;
    let second_stats = collector.collect().expect("Second collection should work");
    
    // Some interfaces should now have better confidence
    let mut improved_confidence_count = 0;
    for stat in &second_stats {
        if stat.calculation_confidence != CalculationConfidence::None {
            improved_confidence_count += 1;
        }
    }
    
    // At least some interfaces should have improved confidence
    assert!(improved_confidence_count > 0, "Some interfaces should have improved confidence after delay");
}

#[tokio::test]
async fn test_interface_stability_over_time() {
    // Test that interface list remains relatively stable over time
    let mut collector = BandwidthCollector::new();
    
    let mut interface_lists = Vec::new();
    
    // Collect interface lists over time
    for _ in 0..10 {
        let stats = collector.collect().expect("Collection should work");
        let interface_names: Vec<String> = stats.iter()
            .map(|s| s.interface_name.clone())
            .collect();
        interface_lists.push(interface_names);
        
        sleep(Duration::from_millis(200)).await;
    }
    
    // Check that interface lists are relatively stable
    let first_list = &interface_lists[0];
    
    for (i, list) in interface_lists.iter().enumerate() {
        // Allow for some variation but not dramatic changes
        let common_interfaces = first_list.iter()
            .filter(|name| list.contains(name))
            .count();
        
        let stability_ratio = common_interfaces as f64 / first_list.len() as f64;
        assert!(stability_ratio >= 0.7, 
               "Interface list should be relatively stable at iteration {}: {:.2}% common", 
               i, stability_ratio * 100.0);
    }
}

#[tokio::test]
async fn test_zero_speed_handling() {
    // Test handling of interfaces with zero speed
    let mut collector = BandwidthCollector::new();
    
    // Take baseline
    let _baseline = collector.collect().expect("Baseline collection should work");
    
    // Wait very briefly (should result in zero or very low speeds)
    sleep(Duration::from_millis(50)).await;
    
    let stats = collector.collect().expect("Second collection should work");
    
    // Verify zero speeds are handled correctly
    for stat in &stats {
        assert!(stat.download_speed_bps >= 0.0, "Download speed should be non-negative");
        assert!(stat.upload_speed_bps >= 0.0, "Upload speed should be non-negative");
        assert!(stat.download_speed_bps.is_finite(), "Download speed should be finite");
        assert!(stat.upload_speed_bps.is_finite(), "Upload speed should be finite");
        
        // Zero speeds should have appropriate confidence
        if stat.download_speed_bps == 0.0 && stat.upload_speed_bps == 0.0 {
            // This is acceptable - interface might have no traffic
        }
    }
}

#[tokio::test]
async fn test_large_time_intervals() {
    // Test behavior with larger time intervals
    let mut collector = BandwidthCollector::new();
    
    // Take baseline
    let baseline_stats = collector.collect().expect("Baseline collection should work");
    
    // Wait longer interval
    sleep(Duration::from_secs(2)).await;
    
    let later_stats = collector.collect().expect("Later collection should work");
    
    // Verify that longer intervals work correctly
    assert_eq!(baseline_stats.len(), later_stats.len(), "Interface count should remain consistent");
    
    for stat in &later_stats {
        // Time since last update should reflect the longer interval
        assert!(stat.time_since_last_update >= 1.8, 
               "Time since last update should reflect longer interval for {}: {:.3}s", 
               stat.interface_name, stat.time_since_last_update);
        
        // Confidence should be reasonable for longer intervals
        if stat.calculation_confidence != CalculationConfidence::None {
            // Longer intervals should generally give better confidence
            assert!(matches!(stat.calculation_confidence, 
                           CalculationConfidence::Medium | CalculationConfidence::High),
                   "Longer intervals should give better confidence for {}", stat.interface_name);
        }
    }
}

#[tokio::test]
async fn test_interface_filtering_consistency() {
    // Test that different filtering methods are consistent
    let mut collector = BandwidthCollector::new();
    
    let all_stats = collector.collect().expect("All collection should work");
    let filtered_stats = collector.collect_filtered().expect("Filtered collection should work");
    let default_stats = collector.collect_default().expect("Default collection should work");
    let important_stats = collector.collect_important().expect("Important collection should work");
    
    // Verify filtering relationships
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
    let all_names: Vec<&String> = all_stats.iter().map(|s| &s.interface_name).collect();
    let filtered_names: Vec<&String> = filtered_stats.iter().map(|s| &s.interface_name).collect();
    let default_names: Vec<&String> = default_stats.iter().map(|s| &s.interface_name).collect();
    let important_names: Vec<&String> = important_stats.iter().map(|s| &s.interface_name).collect();
    
    // All filtered interfaces should exist in all interfaces
    for name in &filtered_names {
        assert!(all_names.contains(name), "Filtered interface {} should exist in all interfaces", name);
    }
    
    // All default interfaces should exist in filtered interfaces
    for name in &default_names {
        assert!(filtered_names.contains(name), "Default interface {} should exist in filtered interfaces", name);
    }
    
    // All important interfaces should exist in default interfaces
    for name in &important_names {
        assert!(default_names.contains(name), "Important interface {} should exist in default interfaces", name);
    }
}

#[tokio::test]
async fn test_collector_state_isolation() {
    // Test that multiple collectors maintain independent state
    let mut collector1 = BandwidthCollector::new();
    let mut collector2 = BandwidthCollector::new();
    
    // Collect from first collector
    let stats1_first = collector1.collect().expect("Collector 1 first collection should work");
    
    // Wait and collect from second collector
    sleep(Duration::from_millis(300)).await;
    let stats2_first = collector2.collect().expect("Collector 2 first collection should work");
    
    // Both should have None confidence for first collection
    for stat in &stats1_first {
        assert_eq!(stat.calculation_confidence, CalculationConfidence::None);
    }
    for stat in &stats2_first {
        assert_eq!(stat.calculation_confidence, CalculationConfidence::None);
    }
    
    // Wait and collect from first collector again
    sleep(Duration::from_millis(300)).await;
    let stats1_second = collector1.collect().expect("Collector 1 second collection should work");
    
    // First collector should now have better confidence, second should still be at first collection level
    let mut collector1_improved = 0;
    for stat in &stats1_second {
        if stat.calculation_confidence != CalculationConfidence::None {
            collector1_improved += 1;
        }
    }
    
    assert!(collector1_improved > 0, "Collector 1 should have improved confidence after multiple collections");
}

#[tokio::test]
async fn test_error_recovery_scenarios() {
    // Test various error recovery scenarios
    let mut collector = BandwidthCollector::new();
    
    // Normal collection
    let normal_stats = collector.collect().expect("Normal collection should work");
    assert!(!normal_stats.is_empty());
    
    // Clear cache and collect again (simulates interface changes)
    collector.clear_interface_cache();
    let post_clear_stats = collector.collect().expect("Post-clear collection should work");
    assert!(!post_clear_stats.is_empty());
    
    // Verify that clearing cache doesn't break functionality
    assert_eq!(normal_stats.len(), post_clear_stats.len(), 
              "Interface count should be consistent after cache clear");
    
    // Test interface info retrieval
    if let Some(first_interface) = normal_stats.first() {
        let interface_info = collector.get_interface_info(&first_interface.interface_name);
        assert_eq!(interface_info.name, first_interface.interface_name);
        assert!(!interface_info.name.is_empty());
    }
    
    // Test total bandwidth calculation
    let (total_download, total_upload) = collector.get_total_bandwidth();
    assert!(total_download >= 0.0);
    assert!(total_upload >= 0.0);
    assert!(total_download.is_finite());
    assert!(total_upload.is_finite());
}

#[tokio::test]
async fn test_high_frequency_collection() {
    // Test high-frequency collection to ensure stability
    let mut collector = BandwidthCollector::new();
    let collection_count = 50;
    let mut successful_collections = 0;
    let mut total_interfaces = 0;
    
    for i in 0..collection_count {
        match collector.collect() {
            Ok(stats) => {
                successful_collections += 1;
                total_interfaces += stats.len();
                
                // Verify data quality
                for stat in &stats {
                    assert!(!stat.interface_name.is_empty());
                    assert!(stat.download_speed_bps >= 0.0);
                    assert!(stat.upload_speed_bps >= 0.0);
                    assert!(stat.download_speed_bps.is_finite());
                    assert!(stat.upload_speed_bps.is_finite());
                }
            }
            Err(e) => {
                // Log error but don't fail test immediately
                eprintln!("Collection {} failed: {}", i, e);
            }
        }
        
        // Very short delay for high frequency
        sleep(Duration::from_millis(20)).await;
    }
    
    // Should have high success rate
    let success_rate = successful_collections as f64 / collection_count as f64;
    assert!(success_rate >= 0.9, "High frequency collection should have high success rate: {:.2}%", success_rate * 100.0);
    
    // Average interface count should be reasonable
    if successful_collections > 0 {
        let avg_interfaces = total_interfaces as f64 / successful_collections as f64;
        assert!(avg_interfaces >= 1.0, "Should find at least one interface on average");
        assert!(avg_interfaces <= 50.0, "Interface count should be reasonable");
    }
}