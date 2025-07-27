//! Bandwidth collection module
//!
//! This module provides comprehensive bandwidth monitoring and collection functionality
//! organized into focused sub-modules for maintainability. The original monolithic
//! bandwidth_collector.rs file (1,881 lines) has been successfully refactored into
//! this modular structure while maintaining complete backward compatibility.
//!
//! ## Module Organization
//!
//! - `collector`: Core BandwidthCollector implementation and collection logic
//! - `errors`: Error types, system impact assessment, and error handling
//! - `stats`: BandwidthStats and related data structures
//! - `validation`: Data validation logic and speed calculation validation
//! - `reporting`: Troubleshooting reports and diagnostic information
//! - `formatting`: Utility functions for formatting bandwidth data
//!
//! ## Usage
//!
//! This module maintains the exact same public API as the original bandwidth_collector.rs:
//!
//! ```rust
//! use crate::collectors::bandwidth::BandwidthCollector;
//!
//! let mut collector = BandwidthCollector::new();
//! let stats = collector.collect()?;
//! ```
//!
//! ## Refactoring Benefits
//!
//! - **Maintainability**: Code is now organized into focused modules with clear responsibilities
//! - **Testability**: Each module has its own comprehensive test suite
//! - **Readability**: Developers can easily locate specific functionality
//! - **Extensibility**: New features can be added to appropriate modules without affecting others
//! - **Backward Compatibility**: All existing code continues to work unchanged

// Module declarations
pub mod collector;
pub mod errors;
pub mod formatting;
pub mod reporting;
pub mod stats;
pub mod validation;

// Re-export the main collector - primary public interface
pub use collector::BandwidthCollector;

// Re-export core data structures and types
pub use stats::{BandwidthStats, CalculationConfidence, InterfaceState, InterfaceType};

// Re-export error handling types
pub use errors::{BandwidthError, SystemImpact};

// Re-export formatting utilities
pub use formatting::{format_bytes, format_speed};

// Note: Additional types and functions are available but not re-exported by default
// to keep the public API clean. They can be accessed directly from their respective modules:
// - reporting::* for diagnostic and troubleshooting functionality
// - validation::* for advanced validation functions
// - errors::* for error logging functions

#[cfg(test)]
pub mod tests;
