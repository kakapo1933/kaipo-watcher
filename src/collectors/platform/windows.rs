use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{debug, info, warn};
use std::process::Command;

use super::PlatformPacketCapture;

pub struct WindowsPacketCapture {
    interface: Option<String>,
}

impl WindowsPacketCapture {
    pub fn new() -> Result<Self> {
        Ok(Self { interface: None })
    }

    fn check_admin_privileges() -> Result<bool> {
        let output = Command::new("net")
            .args(["session"])
            .output()
            .context("Failed to check administrator privileges")?;

        Ok(output.status.success())
    }

    fn check_npcap_installation() -> Result<bool> {
        let npcap_paths = [
            r"C:\Windows\System32\Npcap\",
            r"C:\Windows\SysWOW64\Npcap\",
            r"C:\Program Files\Npcap\",
        ];

        for path in &npcap_paths {
            if std::path::Path::new(path).exists() {
                debug!("Npcap installation found at: {}", path);
                return Ok(true);
            }
        }

        warn!("Npcap installation not found");
        Ok(false)
    }

    fn check_winpcap_installation() -> Result<bool> {
        let winpcap_paths = [
            r"C:\Windows\System32\wpcap.dll",
            r"C:\Windows\SysWOW64\wpcap.dll",
        ];

        for path in &winpcap_paths {
            if std::path::Path::new(path).exists() {
                debug!("WinPcap installation found at: {}", path);
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn get_available_interfaces() -> Result<Vec<String>> {
        let output = Command::new("netsh")
            .args(["interface", "show", "interface"])
            .output()
            .context("Failed to list network interfaces")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();

        for line in output_str.lines() {
            if line.contains("Connected") && !line.contains("Loopback") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let interface_name = parts[3..].join(" ");
                    interfaces.push(interface_name);
                }
            }
        }

        Ok(interfaces)
    }

    fn setup_windows_capture() -> Result<()> {
        info!("Setting up Windows packet capture");
        
        if Self::check_npcap_installation()? {
            info!("Using Npcap for packet capture");
        } else if Self::check_winpcap_installation()? {
            info!("Using WinPcap for packet capture");
        } else {
            return Err(anyhow::anyhow!(
                "Neither Npcap nor WinPcap found. Please install Npcap from https://npcap.com/"
            ));
        }

        Ok(())
    }

    fn check_firewall_rules() -> Result<()> {
        debug!("Checking Windows Firewall rules");
        
        let output = Command::new("netsh")
            .args(["advfirewall", "show", "allprofiles", "state"])
            .output()
            .context("Failed to check firewall status")?;

        if output.status.success() {
            debug!("Firewall status checked successfully");
        }

        Ok(())
    }
}

#[async_trait]
impl PlatformPacketCapture for WindowsPacketCapture {
    async fn start_capture(&mut self, interface_name: &str) -> Result<()> {
        info!("Starting Windows packet capture on interface: {}", interface_name);
        
        if !Self::check_admin_privileges()? {
            return Err(anyhow::anyhow!(
                "Administrator privileges required for packet capture on Windows"
            ));
        }

        Self::setup_windows_capture()?;

        let available_interfaces = Self::get_available_interfaces()?;
        if !available_interfaces.iter().any(|iface| iface.contains(interface_name)) {
            warn!("Interface '{}' not found. Available: {:?}", interface_name, available_interfaces);
        }

        Self::check_firewall_rules()?;

        self.interface = Some(interface_name.to_string());
        
        info!("Windows packet capture started successfully");
        Ok(())
    }

    async fn stop_capture(&mut self) -> Result<()> {
        if let Some(interface) = &self.interface {
            info!("Stopping Windows packet capture on interface: {}", interface);
            self.interface = None;
        }
        Ok(())
    }

    fn check_privileges() -> Result<bool> {
        Self::check_admin_privileges()
    }

    fn get_required_capabilities() -> Vec<String> {
        vec![
            "Administrator privileges".to_string(),
            "Npcap or WinPcap installation".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_packet_capture_creation() {
        let capture = WindowsPacketCapture::new();
        assert!(capture.is_ok());
    }

    #[test]
    fn test_get_required_capabilities() {
        let caps = WindowsPacketCapture::get_required_capabilities();
        assert!(caps.contains(&"Administrator privileges".to_string()));
        assert!(caps.contains(&"Npcap or WinPcap installation".to_string()));
    }

    #[tokio::test]
    async fn test_capture_lifecycle() {
        let mut capture = WindowsPacketCapture::new().unwrap();
        
        if WindowsPacketCapture::check_privileges().unwrap_or(false) {
            let result = capture.start_capture("Ethernet").await;
            if result.is_ok() {
                let stop_result = capture.stop_capture().await;
                assert!(stop_result.is_ok());
            }
        }
    }
}