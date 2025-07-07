use anyhow::Result;
use async_trait::async_trait;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[async_trait]
pub trait PlatformPacketCapture: Send + Sync {
    async fn start_capture(&mut self, interface_name: &str) -> Result<()>;
    async fn stop_capture(&mut self) -> Result<()>;
    fn check_privileges() -> Result<bool>;
    fn get_required_capabilities() -> Vec<String>;
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
    if !PlatformPacketCapture::check_privileges()? {
        let capabilities = PlatformPacketCapture::get_required_capabilities();
        return Err(anyhow::anyhow!(
            "Insufficient privileges for packet capture. Required: {}",
            capabilities.join(", ")
        ));
    }
    Ok(())
}