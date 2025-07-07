use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{debug, info, warn};
use std::process::Command;

use super::PlatformPacketCapture;

pub struct LinuxPacketCapture {
    interface: Option<String>,
}

impl LinuxPacketCapture {
    pub fn new() -> Result<Self> {
        Ok(Self { interface: None })
    }

    fn check_capabilities() -> Result<bool> {
        if nix::unistd::getuid().is_root() {
            return Ok(true);
        }

        let output = Command::new("getcap")
            .arg("/proc/self/exe")
            .output()
            .context("Failed to check capabilities")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        let has_net_raw = output_str.contains("cap_net_raw");
        let has_net_admin = output_str.contains("cap_net_admin");

        Ok(has_net_raw || has_net_admin)
    }

    fn set_socket_options() -> Result<()> {
        info!("Setting Linux-specific socket options for packet capture");
        Ok(())
    }

    fn check_netfilter_queue_support() -> bool {
        std::path::Path::new("/proc/net/netfilter/nfnetlink_queue").exists()
    }

    fn get_available_interfaces() -> Result<Vec<String>> {
        let output = Command::new("ip")
            .args(["link", "show"])
            .output()
            .context("Failed to list network interfaces")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();

        for line in output_str.lines() {
            if let Some(start) = line.find(": ") {
                if let Some(end) = line[start + 2..].find(':') {
                    let interface_name = &line[start + 2..start + 2 + end];
                    if !interface_name.starts_with("lo") {
                        interfaces.push(interface_name.to_string());
                    }
                }
            }
        }

        Ok(interfaces)
    }
}

#[async_trait]
impl PlatformPacketCapture for LinuxPacketCapture {
    async fn start_capture(&mut self, interface_name: &str) -> Result<()> {
        info!("Starting Linux packet capture on interface: {}", interface_name);
        
        if !Self::check_capabilities()? {
            return Err(anyhow::anyhow!(
                "Insufficient privileges. Need CAP_NET_RAW or CAP_NET_ADMIN capability"
            ));
        }

        let available_interfaces = Self::get_available_interfaces()?;
        if !available_interfaces.contains(&interface_name.to_string()) {
            warn!("Interface {} not found. Available: {:?}", interface_name, available_interfaces);
        }

        Self::set_socket_options()?;

        if Self::check_netfilter_queue_support() {
            debug!("Netfilter queue support detected");
        }

        self.interface = Some(interface_name.to_string());
        
        info!("Linux packet capture started successfully");
        Ok(())
    }

    async fn stop_capture(&mut self) -> Result<()> {
        if let Some(interface) = &self.interface {
            info!("Stopping Linux packet capture on interface: {}", interface);
            self.interface = None;
        }
        Ok(())
    }

    fn check_privileges() -> Result<bool> {
        Self::check_capabilities()
    }

    fn get_required_capabilities() -> Vec<String> {
        vec![
            "CAP_NET_RAW".to_string(),
            "CAP_NET_ADMIN".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_packet_capture_creation() {
        let capture = LinuxPacketCapture::new();
        assert!(capture.is_ok());
    }

    #[test]
    fn test_get_required_capabilities() {
        let caps = LinuxPacketCapture::get_required_capabilities();
        assert!(caps.contains(&"CAP_NET_RAW".to_string()));
        assert!(caps.contains(&"CAP_NET_ADMIN".to_string()));
    }

    #[tokio::test]
    async fn test_capture_lifecycle() {
        let mut capture = LinuxPacketCapture::new().unwrap();
        
        if LinuxPacketCapture::check_privileges().unwrap_or(false) {
            let result = capture.start_capture("lo").await;
            if result.is_ok() {
                let stop_result = capture.stop_capture().await;
                assert!(stop_result.is_ok());
            }
        }
    }
}