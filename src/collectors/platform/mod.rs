use anyhow::Result;
use async_trait::async_trait;

// Platform-specific packet capture implementations
// Provides optimized packet capture for different operating systems
// Handles platform-specific privilege requirements and performance optimizations

/// Cross-platform network interface management
/// Provides intelligent interface filtering, type detection, and relevance scoring
pub mod interface_manager;

/// Linux packet capture implementation
/// Uses AF_PACKET sockets with CAP_NET_RAW capability requirements
#[cfg(target_os = "linux")]
pub mod linux;

/// macOS packet capture implementation  
/// Uses Berkeley Packet Filter (BPF) devices requiring root privileges
#[cfg(target_os = "macos")]
pub mod macos;

/// Windows packet capture implementation
/// Uses Npcap driver with Administrator privilege requirements
#[cfg(target_os = "windows")]
pub mod windows;

#[async_trait]
pub trait PlatformPacketCapture: Send + Sync {
    async fn start_capture(&mut self, interface_name: &str) -> Result<()>;
    async fn stop_capture(&mut self) -> Result<()>;
    fn check_privileges() -> Result<bool> where Self: Sized;
    fn get_required_capabilities() -> Vec<String> where Self: Sized;
}

pub fn create_platform_capturer() -> Result<Box<dyn PlatformPacketCapture>> {
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::LinuxPacketCapture::new()?))
    }
    
    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(macos::MacOSPacketCapture::new()?))
    }
    
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(windows::WindowsPacketCapture::new()?))
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(anyhow::anyhow!("Unsupported platform for packet capture"))
    }
}

pub fn check_packet_capture_support() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        if !linux::LinuxPacketCapture::check_privileges()? {
            let capabilities = linux::LinuxPacketCapture::get_required_capabilities();
            return Err(anyhow::anyhow!(
                "Insufficient privileges for packet capture. Required: {}",
                capabilities.join(", ")
            ));
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if !macos::MacOSPacketCapture::check_privileges()? {
            let capabilities = macos::MacOSPacketCapture::get_required_capabilities();
            return Err(anyhow::anyhow!(
                "Insufficient privileges for packet capture. Required: {}",
                capabilities.join(", ")
            ));
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if !windows::WindowsPacketCapture::check_privileges()? {
            let capabilities = windows::WindowsPacketCapture::get_required_capabilities();
            return Err(anyhow::anyhow!(
                "Insufficient privileges for packet capture. Required: {}",
                capabilities.join(", ")
            ));
        }
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Err(anyhow::anyhow!("Unsupported platform for packet capture"));
    }
    
    Ok(())
}