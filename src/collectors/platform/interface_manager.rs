use log::{debug, trace};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::collectors::bandwidth_collector::InterfaceType;

/// Enhanced interface type with platform-specific subtypes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnhancedInterfaceType {
    /// Physical Ethernet connection
    Ethernet {
        /// Specific ethernet subtype (e.g., gigabit, 10G)
        subtype: EthernetSubtype,
    },
    /// Wireless network interface
    WiFi {
        /// WiFi standard (e.g., 802.11ac, 802.11ax)
        standard: Option<String>,
    },
    /// Loopback interface (localhost)
    Loopback,
    /// Virtual interface with specific purpose
    Virtual {
        /// Type of virtual interface
        virtual_type: VirtualInterfaceType,
    },
    /// Unknown or unclassified interface type
    Unknown,
}

/// Ethernet interface subtypes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EthernetSubtype {
    Standard,
    Gigabit,
    TenGigabit,
    Thunderbolt,
    USB,
}

/// Virtual interface types for better categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VirtualInterfaceType {
    /// VPN tunnel interface
    VPN,
    /// Container network interface
    Container,
    /// Virtual machine interface
    VM,
    /// Bridge interface
    Bridge,
    /// Tunnel interface
    Tunnel,
    /// Apple-specific virtual interfaces
    AppleVirtual,
    /// Other virtual interface
    Other,
}

/// Interface relevance score for prioritization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct InterfaceRelevance {
    /// Primary score (0-100, higher is more relevant)
    pub score: u8,
    /// Reason for the score
    pub reason: String,
    /// Whether this interface should be shown by default
    pub show_by_default: bool,
    /// Whether this interface is considered "important"
    pub is_important: bool,
}

/// Platform-specific interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInterfaceInfo {
    /// Interface name
    pub name: String,
    /// Enhanced interface type with platform-specific details
    pub interface_type: EnhancedInterfaceType,
    /// Relevance score for prioritization
    pub relevance: InterfaceRelevance,
    /// Platform-specific metadata
    pub platform_metadata: HashMap<String, String>,
    /// Whether this interface should be filtered out
    pub should_filter: bool,
}

/// Cross-platform interface manager
#[derive(Debug)]
pub struct InterfaceManager {
    /// Current platform
    platform: Platform,
    /// Cached interface information
    interface_cache: HashMap<String, PlatformInterfaceInfo>,
}

/// Supported platforms
#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    MacOS,
    Linux,
    Windows,
    Unknown,
}

impl InterfaceManager {
    /// Create a new interface manager for the current platform
    pub fn new() -> Self {
        let platform = Self::detect_platform();
        debug!("Initializing InterfaceManager for platform: {:?}", platform);
        
        Self {
            platform,
            interface_cache: HashMap::new(),
        }
    }

    /// Detect the current platform
    fn detect_platform() -> Platform {
        match std::env::consts::OS {
            "macos" => Platform::MacOS,
            "linux" => Platform::Linux,
            "windows" => Platform::Windows,
            _ => Platform::Unknown,
        }
    }

    /// Analyze and categorize a network interface
    pub fn analyze_interface(&mut self, interface_name: &str) -> PlatformInterfaceInfo {
        // Check cache first
        if let Some(cached_info) = self.interface_cache.get(interface_name) {
            trace!("Using cached interface info for: {}", interface_name);
            return cached_info.clone();
        }

        let info = self.perform_interface_analysis(interface_name);
        
        // Cache the result
        self.interface_cache.insert(interface_name.to_string(), info.clone());
        
        debug!("Analyzed interface '{}': type={:?}, relevance_score={}, should_filter={}", 
               interface_name, info.interface_type, info.relevance.score, info.should_filter);
        
        info
    }

    /// Perform the actual interface analysis
    fn perform_interface_analysis(&self, interface_name: &str) -> PlatformInterfaceInfo {
        let interface_type = self.determine_enhanced_interface_type(interface_name);
        let relevance = self.calculate_interface_relevance(interface_name, &interface_type);
        let platform_metadata = self.gather_platform_metadata(interface_name);
        let should_filter = self.should_filter_interface(interface_name, &interface_type);

        PlatformInterfaceInfo {
            name: interface_name.to_string(),
            interface_type,
            relevance,
            platform_metadata,
            should_filter,
        }
    }

    /// Determine enhanced interface type with platform-specific logic
    fn determine_enhanced_interface_type(&self, interface_name: &str) -> EnhancedInterfaceType {
        let name_lower = interface_name.to_lowercase();
        
        match self.platform {
            Platform::MacOS => self.analyze_macos_interface(&name_lower),
            Platform::Linux => self.analyze_linux_interface(&name_lower),
            Platform::Windows => self.analyze_windows_interface(&name_lower),
            Platform::Unknown => self.analyze_generic_interface(&name_lower),
        }
    }

    /// Analyze macOS-specific interface patterns
    fn analyze_macos_interface(&self, name_lower: &str) -> EnhancedInterfaceType {
        // macOS-specific interface patterns
        match name_lower {
            // Loopback
            name if name.starts_with("lo") => EnhancedInterfaceType::Loopback,
            
            // Ethernet interfaces
            name if name.starts_with("en") => {
                // en0, en1 are typically built-in ethernet
                // en2+ might be USB/Thunderbolt adapters
                if name == "en0" || name == "en1" {
                    EnhancedInterfaceType::Ethernet { subtype: EthernetSubtype::Standard }
                } else if name.contains("thunderbolt") || name.starts_with("en") && name.len() > 3 {
                    EnhancedInterfaceType::Ethernet { subtype: EthernetSubtype::Thunderbolt }
                } else {
                    EnhancedInterfaceType::Ethernet { subtype: EthernetSubtype::Standard }
                }
            },
            
            // WiFi interfaces
            name if name.starts_with("en") && (name.contains("wifi") || name.contains("wlan")) => {
                EnhancedInterfaceType::WiFi { standard: None }
            },
            
            // Apple-specific virtual interfaces
            name if name.starts_with("utun") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::VPN }
            },
            name if name.starts_with("anpi") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::AppleVirtual }
            },
            name if name.starts_with("ipsec") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::VPN }
            },
            name if name.starts_with("ppp") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::VPN }
            },
            name if name.starts_with("awdl") => {
                // Apple Wireless Direct Link (AirDrop, etc.)
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::AppleVirtual }
            },
            name if name.starts_with("llw") => {
                // Low Latency WLAN (Apple-specific)
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::AppleVirtual }
            },
            name if name.starts_with("ap") => {
                // Access Point mode interfaces
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::AppleVirtual }
            },
            
            // Bridge interfaces
            name if name.starts_with("bridge") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::Bridge }
            },
            
            // VM interfaces
            name if name.starts_with("vmnet") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::VM }
            },
            
            // Default fallback
            _ => self.analyze_generic_interface(name_lower),
        }
    }

    /// Analyze Linux-specific interface patterns
    fn analyze_linux_interface(&self, name_lower: &str) -> EnhancedInterfaceType {
        match name_lower {
            // Loopback
            name if name.starts_with("lo") => EnhancedInterfaceType::Loopback,
            
            // Ethernet interfaces
            name if name.starts_with("eth") => {
                EnhancedInterfaceType::Ethernet { subtype: EthernetSubtype::Standard }
            },
            name if name.starts_with("en") => {
                // Predictable network interface names
                if name.starts_with("eno") {
                    EnhancedInterfaceType::Ethernet { subtype: EthernetSubtype::Standard }
                } else if name.starts_with("enp") {
                    EnhancedInterfaceType::Ethernet { subtype: EthernetSubtype::Standard }
                } else if name.starts_with("ens") {
                    EnhancedInterfaceType::Ethernet { subtype: EthernetSubtype::Standard }
                } else {
                    EnhancedInterfaceType::Ethernet { subtype: EthernetSubtype::Standard }
                }
            },
            name if name.starts_with("em") => {
                EnhancedInterfaceType::Ethernet { subtype: EthernetSubtype::Standard }
            },
            
            // WiFi interfaces
            name if name.starts_with("wl") => {
                EnhancedInterfaceType::WiFi { standard: None }
            },
            name if name.starts_with("wlan") => {
                EnhancedInterfaceType::WiFi { standard: None }
            },
            
            // Virtual interfaces
            name if name.starts_with("tun") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::Tunnel }
            },
            name if name.starts_with("tap") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::Tunnel }
            },
            name if name.starts_with("veth") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::Container }
            },
            name if name.starts_with("docker") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::Container }
            },
            name if name.starts_with("br-") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::Bridge }
            },
            name if name.starts_with("virbr") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::VM }
            },
            
            // Default fallback
            _ => self.analyze_generic_interface(name_lower),
        }
    }

    /// Analyze Windows-specific interface patterns
    fn analyze_windows_interface(&self, name_lower: &str) -> EnhancedInterfaceType {
        match name_lower {
            // Windows interface patterns are typically more descriptive
            name if name.contains("loopback") => EnhancedInterfaceType::Loopback,
            name if name.contains("ethernet") => {
                EnhancedInterfaceType::Ethernet { subtype: EthernetSubtype::Standard }
            },
            name if name.contains("wifi") || name.contains("wireless") => {
                EnhancedInterfaceType::WiFi { standard: None }
            },
            name if name.contains("vpn") || name.contains("tunnel") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::VPN }
            },
            name if name.contains("hyper-v") || name.contains("vmware") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::VM }
            },
            
            // Default fallback
            _ => self.analyze_generic_interface(name_lower),
        }
    }

    /// Generic interface analysis for unknown platforms or fallback
    fn analyze_generic_interface(&self, name_lower: &str) -> EnhancedInterfaceType {
        match name_lower {
            name if name.starts_with("lo") => EnhancedInterfaceType::Loopback,
            name if name.starts_with("eth") || name.starts_with("en") => {
                EnhancedInterfaceType::Ethernet { subtype: EthernetSubtype::Standard }
            },
            name if name.starts_with("wl") || name.contains("wifi") => {
                EnhancedInterfaceType::WiFi { standard: None }
            },
            name if name.starts_with("tun") || name.starts_with("tap") => {
                EnhancedInterfaceType::Virtual { virtual_type: VirtualInterfaceType::Tunnel }
            },
            _ => EnhancedInterfaceType::Unknown,
        }
    }

    /// Calculate interface relevance score for prioritization
    fn calculate_interface_relevance(&self, interface_name: &str, interface_type: &EnhancedInterfaceType) -> InterfaceRelevance {
        let (base_score, reason, show_by_default, is_important) = match interface_type {
            EnhancedInterfaceType::Ethernet { subtype } => {
                let score = match subtype {
                    EthernetSubtype::Standard => 90,
                    EthernetSubtype::Gigabit => 95,
                    EthernetSubtype::TenGigabit => 98,
                    EthernetSubtype::Thunderbolt => 85,
                    EthernetSubtype::USB => 80,
                };
                (score, "Physical ethernet connection".to_string(), true, true)
            },
            
            EnhancedInterfaceType::WiFi { .. } => {
                (85, "Wireless network connection".to_string(), true, true)
            },
            
            EnhancedInterfaceType::Loopback => {
                (10, "Loopback interface (localhost only)".to_string(), false, false)
            },
            
            EnhancedInterfaceType::Virtual { virtual_type } => {
                let (score, reason) = match virtual_type {
                    VirtualInterfaceType::VPN => (70, "VPN connection"),
                    VirtualInterfaceType::Container => (30, "Container network interface"),
                    VirtualInterfaceType::VM => (40, "Virtual machine interface"),
                    VirtualInterfaceType::Bridge => (25, "Network bridge interface"),
                    VirtualInterfaceType::Tunnel => (60, "Network tunnel interface"),
                    VirtualInterfaceType::AppleVirtual => (20, "Apple system virtual interface"),
                    VirtualInterfaceType::Other => (15, "Other virtual interface"),
                };
                (score, reason.to_string(), score >= 60, score >= 70)
            },
            
            EnhancedInterfaceType::Unknown => {
                (50, "Unknown interface type".to_string(), true, false)
            },
        };

        // Apply platform-specific adjustments
        let adjusted_score = self.apply_platform_score_adjustments(interface_name, base_score);
        
        InterfaceRelevance {
            score: adjusted_score,
            reason,
            show_by_default,
            is_important,
        }
    }

    /// Apply platform-specific score adjustments
    fn apply_platform_score_adjustments(&self, interface_name: &str, base_score: u8) -> u8 {
        let mut score = base_score;
        
        match self.platform {
            Platform::MacOS => {
                // Prioritize en0 and en1 on macOS as they're typically the main interfaces
                if interface_name == "en0" {
                    score = std::cmp::min(100, score + 10);
                } else if interface_name == "en1" {
                    score = std::cmp::min(100, score + 5);
                }
                
                // Deprioritize Apple-specific virtual interfaces
                if interface_name.starts_with("anpi") || 
                   interface_name.starts_with("awdl") || 
                   interface_name.starts_with("llw") {
                    score = std::cmp::max(5, score.saturating_sub(15));
                }
            },
            
            Platform::Linux => {
                // Prioritize predictable interface names
                if interface_name.starts_with("eno") || interface_name == "eth0" {
                    score = std::cmp::min(100, score + 5);
                }
                
                // Deprioritize container interfaces
                if interface_name.starts_with("docker") || 
                   interface_name.starts_with("veth") ||
                   interface_name.starts_with("br-") {
                    score = std::cmp::max(5, score.saturating_sub(10));
                }
            },
            
            Platform::Windows => {
                // Windows interface prioritization would go here
                // Currently no specific adjustments
            },
            
            Platform::Unknown => {
                // No platform-specific adjustments
            },
        }
        
        score
    }

    /// Gather platform-specific metadata for an interface
    fn gather_platform_metadata(&self, interface_name: &str) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        
        metadata.insert("platform".to_string(), format!("{:?}", self.platform));
        metadata.insert("interface_name".to_string(), interface_name.to_string());
        
        match self.platform {
            Platform::MacOS => {
                self.gather_macos_metadata(interface_name, &mut metadata);
            },
            Platform::Linux => {
                self.gather_linux_metadata(interface_name, &mut metadata);
            },
            Platform::Windows => {
                self.gather_windows_metadata(interface_name, &mut metadata);
            },
            Platform::Unknown => {
                metadata.insert("note".to_string(), "Unknown platform - limited metadata".to_string());
            },
        }
        
        metadata
    }

    /// Gather macOS-specific metadata
    fn gather_macos_metadata(&self, interface_name: &str, metadata: &mut HashMap<String, String>) {
        // Add macOS-specific interface information
        if interface_name.starts_with("en") {
            metadata.insert("macos_type".to_string(), "built_in_ethernet".to_string());
        } else if interface_name.starts_with("utun") {
            metadata.insert("macos_type".to_string(), "vpn_tunnel".to_string());
        } else if interface_name.starts_with("anpi") {
            metadata.insert("macos_type".to_string(), "apple_network_interface".to_string());
        } else if interface_name.starts_with("awdl") {
            metadata.insert("macos_type".to_string(), "apple_wireless_direct_link".to_string());
            metadata.insert("purpose".to_string(), "airdrop_handoff".to_string());
        }
    }

    /// Gather Linux-specific metadata
    fn gather_linux_metadata(&self, interface_name: &str, metadata: &mut HashMap<String, String>) {
        // Add Linux-specific interface information
        if interface_name.starts_with("eno") {
            metadata.insert("linux_naming".to_string(), "predictable_onboard".to_string());
        } else if interface_name.starts_with("enp") {
            metadata.insert("linux_naming".to_string(), "predictable_pci".to_string());
        } else if interface_name.starts_with("ens") {
            metadata.insert("linux_naming".to_string(), "predictable_slot".to_string());
        } else if interface_name.starts_with("veth") {
            metadata.insert("linux_type".to_string(), "virtual_ethernet_pair".to_string());
        } else if interface_name.starts_with("docker") {
            metadata.insert("linux_type".to_string(), "docker_bridge".to_string());
        }
    }

    /// Gather Windows-specific metadata
    fn gather_windows_metadata(&self, _interface_name: &str, metadata: &mut HashMap<String, String>) {
        // Add Windows-specific interface information
        metadata.insert("windows_note".to_string(), "Windows interface metadata not yet implemented".to_string());
    }

    /// Determine if an interface should be filtered out
    fn should_filter_interface(&self, interface_name: &str, interface_type: &EnhancedInterfaceType) -> bool {
        match interface_type {
            EnhancedInterfaceType::Loopback => true, // Usually filter loopback
            
            EnhancedInterfaceType::Virtual { virtual_type } => {
                match virtual_type {
                    VirtualInterfaceType::AppleVirtual => {
                        // Filter most Apple virtual interfaces except VPN
                        !interface_name.starts_with("utun")
                    },
                    VirtualInterfaceType::Container => true, // Filter container interfaces by default
                    VirtualInterfaceType::Bridge => true,   // Filter bridge interfaces by default
                    VirtualInterfaceType::VPN => false,     // Don't filter VPN interfaces
                    VirtualInterfaceType::VM => true,       // Filter VM interfaces by default
                    VirtualInterfaceType::Tunnel => false,  // Don't filter tunnel interfaces
                    VirtualInterfaceType::Other => true,    // Filter unknown virtual interfaces
                }
            },
            
            _ => false, // Don't filter physical interfaces
        }
    }

    /// Get filtered and sorted interfaces based on relevance
    pub fn get_relevant_interfaces(&mut self, interface_names: &[String]) -> Vec<PlatformInterfaceInfo> {
        let mut interfaces: Vec<PlatformInterfaceInfo> = interface_names
            .iter()
            .map(|name| self.analyze_interface(name))
            .filter(|info| !info.should_filter)
            .collect();

        // Sort by relevance score (highest first)
        interfaces.sort_by(|a, b| b.relevance.score.cmp(&a.relevance.score));

        debug!("Filtered and sorted {} interfaces from {} total", 
               interfaces.len(), interface_names.len());

        interfaces
    }

    /// Get interfaces that should be shown by default
    pub fn get_default_interfaces(&mut self, interface_names: &[String]) -> Vec<PlatformInterfaceInfo> {
        self.get_relevant_interfaces(interface_names)
            .into_iter()
            .filter(|info| info.relevance.show_by_default)
            .collect()
    }

    /// Get only important interfaces (high priority)
    pub fn get_important_interfaces(&mut self, interface_names: &[String]) -> Vec<PlatformInterfaceInfo> {
        self.get_relevant_interfaces(interface_names)
            .into_iter()
            .filter(|info| info.relevance.is_important)
            .collect()
    }

    /// Clear the interface cache (useful when interfaces change)
    pub fn clear_cache(&mut self) {
        debug!("Clearing interface cache ({} entries)", self.interface_cache.len());
        self.interface_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, Platform) {
        (self.interface_cache.len(), self.platform.clone())
    }
}

impl Default for InterfaceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert enhanced interface type to basic interface type for compatibility
impl From<EnhancedInterfaceType> for InterfaceType {
    fn from(enhanced: EnhancedInterfaceType) -> Self {
        match enhanced {
            EnhancedInterfaceType::Ethernet { .. } => InterfaceType::Ethernet,
            EnhancedInterfaceType::WiFi { .. } => InterfaceType::WiFi,
            EnhancedInterfaceType::Loopback => InterfaceType::Loopback,
            EnhancedInterfaceType::Virtual { .. } => InterfaceType::Virtual,
            EnhancedInterfaceType::Unknown => InterfaceType::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interface_manager_creation() {
        let manager = InterfaceManager::new();
        assert!(!matches!(manager.platform, Platform::Unknown));
    }

    #[test]
    fn test_macos_interface_analysis() {
        let mut manager = InterfaceManager::new();
        
        // Test various macOS interface patterns
        let test_cases = vec![
            ("en0", true, 90), // Should be high priority, not filtered
            ("en1", true, 85), // Should be high priority, not filtered
            ("utun0", false, 70), // VPN interface, should not be filtered
            ("anpi0", true, 5), // Apple virtual, should be filtered
            ("awdl0", true, 5), // Apple wireless direct link, should be filtered
            ("lo0", true, 10), // Loopback, should be filtered
        ];

        for (interface_name, should_filter, min_score) in test_cases {
            let info = manager.analyze_interface(interface_name);
            assert_eq!(info.should_filter, should_filter, 
                      "Interface {} filter status mismatch", interface_name);
            assert!(info.relevance.score >= min_score, 
                   "Interface {} score {} below minimum {}", 
                   interface_name, info.relevance.score, min_score);
        }
    }

    #[test]
    fn test_linux_interface_analysis() {
        let mut manager = InterfaceManager::new();
        
        let test_cases = vec![
            ("eth0", false, 90),
            ("eno1", false, 95),
            ("wlan0", false, 85),
            ("docker0", true, 20),
            ("veth123", true, 20),
            ("lo", true, 10),
        ];

        for (interface_name, should_filter, min_score) in test_cases {
            let info = manager.analyze_interface(interface_name);
            assert_eq!(info.should_filter, should_filter, 
                      "Interface {} filter status mismatch", interface_name);
            assert!(info.relevance.score >= min_score, 
                   "Interface {} score {} below minimum {}", 
                   interface_name, info.relevance.score, min_score);
        }
    }

    #[test]
    fn test_interface_filtering_and_sorting() {
        let mut manager = InterfaceManager::new();
        
        let interfaces = vec![
            "lo0".to_string(),
            "en0".to_string(),
            "utun0".to_string(),
            "anpi0".to_string(),
            "en1".to_string(),
        ];

        let relevant = manager.get_relevant_interfaces(&interfaces);
        
        // Should filter out lo0 and anpi0
        assert!(relevant.len() <= 3);
        
        // Should be sorted by relevance (en0 should be first if on macOS)
        if !relevant.is_empty() {
            assert!(relevant[0].relevance.score >= relevant.last().unwrap().relevance.score);
        }
    }

    #[test]
    fn test_cache_functionality() {
        let mut manager = InterfaceManager::new();
        
        // Analyze an interface
        let info1 = manager.analyze_interface("en0");
        let (cache_size, _) = manager.get_cache_stats();
        assert_eq!(cache_size, 1);
        
        // Analyze the same interface again (should use cache)
        let info2 = manager.analyze_interface("en0");
        assert_eq!(info1.name, info2.name);
        assert_eq!(info1.relevance.score, info2.relevance.score);
        
        // Clear cache
        manager.clear_cache();
        let (cache_size, _) = manager.get_cache_stats();
        assert_eq!(cache_size, 0);
    }
}