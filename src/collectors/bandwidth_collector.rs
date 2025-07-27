//! Bandwidth collector re-export module
//!
//! This module provides backward compatibility by re-exporting all public types
//! from the new modular bandwidth collection system. The original monolithic
//! bandwidth_collector.rs file has been refactored into focused sub-modules
//! for better maintainability while preserving the exact same public API.
//!
//! ## Usage
//!
//! This module maintains complete backward compatibility:
//!
//! ```rust
//! use crate::collectors::bandwidth_collector::{BandwidthCollector, BandwidthStats};
//!
//! let mut collector = BandwidthCollector::new();
//! let stats = collector.collect()?;
//! ```
//!
//! ## Migration
//!
//! Existing code using this module will continue to work unchanged.
//! New code can optionally use the new modular structure:
//!
//! ```rust
//! use crate::collectors::bandwidth::{BandwidthCollector, BandwidthStats};
//! ```

// Re-export all public types from the new bandwidth module
// This maintains complete backward compatibility with the original API
pub use crate::collectors::bandwidth::{
    BandwidthCollector, BandwidthError, BandwidthStats, CalculationConfidence, InterfaceState,
    InterfaceType, SystemImpact, format_bytes, format_speed,
};
