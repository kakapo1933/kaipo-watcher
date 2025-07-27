pub mod bandwidth;
pub mod bandwidth_collector;
pub mod packet_collector;
pub mod platform;

// The new bandwidth module structure is ready to be used
// For now, continue using the original bandwidth_collector to maintain compatibility
// This will be switched in task 10 when the original file is replaced
pub use bandwidth_collector::BandwidthCollector;
pub use packet_collector::PacketCollector;

// The new bandwidth module can be used like this:
// pub use bandwidth::BandwidthCollector;
