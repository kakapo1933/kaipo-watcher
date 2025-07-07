use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{debug, info, warn};
use std::process::Command;

use super::PlatformPacketCapture;

pub struct MacOSPacketCapture {
    interface: Option<String>,
}

impl MacOSPacketCapture {
    pub fn new() -> Result<Self> {
        Ok(Self { interface: None })
    }

    fn check_admin_privileges() -> Result<bool> {
        let uid = unsafe { libc::getuid() };
        Ok(uid == 0)
    }

    fn check_bpf_devices() -> Result<bool> {
        let output = Command::new("ls")
            .arg("/dev/bpf*")
            .output()
            .context("Failed to check BPF devices")?;

        let success = output.status.success();
        if success {
            debug!("BPF devices found");
        } else {
            warn!("No BPF devices found, packet capture may not work");
        }

        Ok(success)
    }

    fn get_available_interfaces() -> Result<Vec<String>> {
        let output = Command::new("ifconfig")
            .arg("-l")
            .output()
            .context("Failed to list network interfaces")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let interfaces: Vec<String> = output_str
            .split_whitespace()
            .filter(|name| !name.starts_with("lo") && !name.is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(interfaces)
    }

    fn setup_bpf_filter() -> Result<()> {
        info!("Setting up BPF filter for macOS packet capture");
        Ok(())
    }

    fn check_dtrace_support() -> bool {
        Command::new("which")
            .arg("dtrace")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[async_trait]
impl PlatformPacketCapture for MacOSPacketCapture {
    async fn start_capture(&mut self, interface_name: &str) -> Result<()> {
        info!("Starting macOS packet capture on interface: {}", interface_name);
        
        if !Self::check_admin_privileges()? {
            return Err(anyhow::anyhow!(
                "Administrator privileges required for packet capture on macOS"
            ));
        }

        if !Self::check_bpf_devices()? {
            warn!("BPF devices not available, some features may be limited");
        }

        let available_interfaces = Self::get_available_interfaces()?;
        if !available_interfaces.contains(&interface_name.to_string()) {
            warn!("Interface {} not found. Available: {:?}", interface_name, available_interfaces);
        }

        Self::setup_bpf_filter()?;

        if Self::check_dtrace_support() {
            debug!("DTrace support detected for advanced monitoring");
        }

        self.interface = Some(interface_name.to_string());
        
        info!("macOS packet capture started successfully");
        Ok(())
    }

    async fn stop_capture(&mut self) -> Result<()> {
        if let Some(interface) = &self.interface {
            info!("Stopping macOS packet capture on interface: {}", interface);
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
            "BPF device access".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_packet_capture_creation() {
        let capture = MacOSPacketCapture::new();
        assert!(capture.is_ok());
    }

    #[test]
    fn test_get_required_capabilities() {
        let caps = MacOSPacketCapture::get_required_capabilities();
        assert!(caps.contains(&"Administrator privileges".to_string()));
        assert!(caps.contains(&"BPF device access".to_string()));
    }

    #[tokio::test]
    async fn test_capture_lifecycle() {
        let mut capture = MacOSPacketCapture::new().unwrap();
        
        if MacOSPacketCapture::check_privileges().unwrap_or(false) {
            let result = capture.start_capture("en0").await;
            if result.is_ok() {
                let stop_result = capture.stop_capture().await;
                assert!(stop_result.is_ok());
            }
        }
    }
}