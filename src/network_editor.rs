use crate::settings_editor::{EditorType, SettingEditor, SettingOption, SettingValue};
use anyhow::{Context, Result};
use std::process::Command;

/// Network adapter enable/disable editor
#[derive(Debug, Clone)]
pub struct NetworkAdapterToggleEditor {
    adapter_name: String,
}

impl NetworkAdapterToggleEditor {
    pub fn new(adapter_name: String) -> Self {
        Self { adapter_name }
    }
    
    fn is_adapter_enabled(&self) -> Result<bool> {
        let output = Command::new("netsh")
            .args(&["interface", "show", "interface", &self.adapter_name])
            .output()
            .context("Failed to query network adapter status")?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.contains("Connected") || output_str.contains("Enabled"))
    }
}

impl SettingEditor for NetworkAdapterToggleEditor {
    fn clone_box(&self) -> Box<dyn SettingEditor> {
        Box::new(self.clone())
    }

    fn get_current_value(&self) -> Result<SettingValue> {
        Ok(SettingValue::Bool(self.is_adapter_enabled()?))
    }
    
    fn get_available_options(&self) -> Result<Vec<SettingOption>> {
        Ok(vec![
            SettingOption {
                label: "Enabled".to_string(),
                value: SettingValue::Bool(true),
                description: Some("Enable the network adapter".to_string()),
            },
            SettingOption {
                label: "Disabled".to_string(),
                value: SettingValue::Bool(false),
                description: Some("Disable the network adapter".to_string()),
            },
        ])
    }
    
    fn set_value(&self, value: SettingValue) -> Result<()> {
        if let SettingValue::Bool(enable) = value {
            let action = if enable { "enable" } else { "disable" };
            
            Command::new("netsh")
                .args(&["interface", "set", "interface", &self.adapter_name, action])
                .output()
                .context("Failed to change network adapter state")?;
            
            Ok(())
        } else {
            anyhow::bail!("Invalid value type for network adapter toggle")
        }
    }
    
    fn validate_value(&self, value: &SettingValue) -> Result<bool> {
        Ok(matches!(value, SettingValue::Bool(_)))
    }
    
    fn get_editor_type(&self) -> EditorType {
        EditorType::Toggle
    }
    
    fn requires_admin(&self) -> bool {
        true
    }
}

/// DNS server configuration editor
#[derive(Debug, Clone)]
pub struct DNSServerEditor {
    adapter_name: String,
}

impl DNSServerEditor {
    pub fn new(adapter_name: String) -> Self {
        Self { adapter_name }
    }
    
    fn get_current_dns(&self) -> Result<String> {
        let output = Command::new("netsh")
            .args(&["interface", "ip", "show", "dns", &self.adapter_name])
            .output()
            .context("Failed to get DNS servers")?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse DNS servers from output
        for line in output_str.lines() {
            if line.contains("DNS Servers") || line.contains("Statically Configured DNS Servers") {
                if let Some(dns) = line.split(':').nth(1) {
                    return Ok(dns.trim().to_string());
                }
            }
        }
        
        Ok("Automatic (DHCP)".to_string())
    }
}

impl SettingEditor for DNSServerEditor {
    fn clone_box(&self) -> Box<dyn SettingEditor> {
        Box::new(self.clone())
    }

    fn get_current_value(&self) -> Result<SettingValue> {
        Ok(SettingValue::Selection(self.get_current_dns()?))
    }
    
    fn get_available_options(&self) -> Result<Vec<SettingOption>> {
        Ok(vec![
            SettingOption {
                label: "Automatic (DHCP)".to_string(),
                value: SettingValue::Selection("dhcp".to_string()),
                description: Some("Use DNS servers from DHCP".to_string()),
            },
            SettingOption {
                label: "Google DNS".to_string(),
                value: SettingValue::Selection("8.8.8.8,8.8.4.4".to_string()),
                description: Some("8.8.8.8, 8.8.4.4".to_string()),
            },
            SettingOption {
                label: "Cloudflare DNS".to_string(),
                value: SettingValue::Selection("1.1.1.1,1.0.0.1".to_string()),
                description: Some("1.1.1.1, 1.0.0.1 - Privacy focused".to_string()),
            },
            SettingOption {
                label: "OpenDNS".to_string(),
                value: SettingValue::Selection("208.67.222.222,208.67.220.220".to_string()),
                description: Some("208.67.222.222, 208.67.220.220".to_string()),
            },
            SettingOption {
                label: "Quad9 DNS".to_string(),
                value: SettingValue::Selection("9.9.9.9,149.112.112.112".to_string()),
                description: Some("9.9.9.9, 149.112.112.112 - Security focused".to_string()),
            },
        ])
    }
    
    fn set_value(&self, value: SettingValue) -> Result<()> {
        if let SettingValue::Selection(dns_config) = value {
            if dns_config == "dhcp" {
                // Set to automatic
                Command::new("netsh")
                    .args(&["interface", "ip", "set", "dns", &self.adapter_name, "dhcp"])
                    .output()
                    .context("Failed to set DNS to automatic")?;
            } else {
                // Set static DNS servers
                let servers: Vec<&str> = dns_config.split(',').collect();
                
                // Set primary DNS
                if let Some(primary) = servers.get(0) {
                    Command::new("netsh")
                        .args(&[
                            "interface", "ip", "set", "dns", 
                            &self.adapter_name, "static", primary.trim()
                        ])
                        .output()
                        .context("Failed to set primary DNS")?;
                }
                
                // Add secondary DNS
                if let Some(secondary) = servers.get(1) {
                    Command::new("netsh")
                        .args(&[
                            "interface", "ip", "add", "dns", 
                            &self.adapter_name, secondary.trim(), "index=2"
                        ])
                        .output()
                        .context("Failed to set secondary DNS")?;
                }
            }
            
            Ok(())
        } else {
            anyhow::bail!("Invalid value type for DNS configuration")
        }
    }
    
    fn validate_value(&self, value: &SettingValue) -> Result<bool> {
        Ok(matches!(value, SettingValue::Selection(_)))
    }
    
    fn get_editor_type(&self) -> EditorType {
        EditorType::Dropdown
    }
    
    fn requires_admin(&self) -> bool {
        true
    }
}

/// Wi-Fi power management editor
#[derive(Debug, Clone)]
pub struct WiFiPowerEditor;

impl WiFiPowerEditor {
    pub fn new() -> Self {
        Self
    }
    
    fn get_power_saving_mode(&self) -> Result<String> {
        let output = Command::new("powercfg")
            .args(&["/q", "SCHEME_CURRENT", "19cbb8fa-5279-450e-9fac-8a3d5fedd0c1"])
            .output()
            .context("Failed to get Wi-Fi power settings")?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse the power saving mode from output
        if output_str.contains("Power Saving Mode: Maximum Performance") {
            Ok("Maximum Performance".to_string())
        } else if output_str.contains("Power Saving Mode: Low Power Saving") {
            Ok("Low Power Saving".to_string())
        } else if output_str.contains("Power Saving Mode: Medium Power Saving") {
            Ok("Medium Power Saving".to_string())
        } else {
            Ok("Maximum Power Saving".to_string())
        }
    }
}

impl SettingEditor for WiFiPowerEditor {
    fn clone_box(&self) -> Box<dyn SettingEditor> {
        Box::new(self.clone())
    }

    fn get_current_value(&self) -> Result<SettingValue> {
        Ok(SettingValue::Selection(self.get_power_saving_mode()?))
    }
    
    fn get_available_options(&self) -> Result<Vec<SettingOption>> {
        Ok(vec![
            SettingOption {
                label: "Maximum Performance".to_string(),
                value: SettingValue::Selection("0".to_string()),
                description: Some("No power saving, best performance".to_string()),
            },
            SettingOption {
                label: "Low Power Saving".to_string(),
                value: SettingValue::Selection("1".to_string()),
                description: Some("Minimal power saving".to_string()),
            },
            SettingOption {
                label: "Medium Power Saving".to_string(),
                value: SettingValue::Selection("2".to_string()),
                description: Some("Balanced power saving".to_string()),
            },
            SettingOption {
                label: "Maximum Power Saving".to_string(),
                value: SettingValue::Selection("3".to_string()),
                description: Some("Maximum power saving, may affect performance".to_string()),
            },
        ])
    }
    
    fn set_value(&self, value: SettingValue) -> Result<()> {
        if let SettingValue::Selection(mode) = value {
            // Set for AC power
            Command::new("powercfg")
                .args(&[
                    "/setacvalueindex",
                    "SCHEME_CURRENT",
                    "19cbb8fa-5279-450e-9fac-8a3d5fedd0c1",
                    "12bbebe6-58d6-4636-95bb-3217ef867c1a",
                    &mode,
                ])
                .output()
                .context("Failed to set Wi-Fi power mode for AC")?;
            
            // Set for DC power (battery)
            Command::new("powercfg")
                .args(&[
                    "/setdcvalueindex",
                    "SCHEME_CURRENT",
                    "19cbb8fa-5279-450e-9fac-8a3d5fedd0c1",
                    "12bbebe6-58d6-4636-95bb-3217ef867c1a",
                    &mode,
                ])
                .output()
                .context("Failed to set Wi-Fi power mode for battery")?;
            
            // Apply the changes
            Command::new("powercfg")
                .args(&["/setactive", "SCHEME_CURRENT"])
                .output()
                .context("Failed to apply power settings")?;
            
            Ok(())
        } else {
            anyhow::bail!("Invalid value type for Wi-Fi power mode")
        }
    }
    
    fn validate_value(&self, value: &SettingValue) -> Result<bool> {
        if let SettingValue::Selection(mode) = value {
            Ok(["0", "1", "2", "3"].contains(&mode.as_str()))
        } else {
            Ok(false)
        }
    }
    
    fn get_editor_type(&self) -> EditorType {
        EditorType::Dropdown
    }
    
    fn requires_admin(&self) -> bool {
        true
    }
}