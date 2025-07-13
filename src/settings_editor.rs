use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::process::Command;
use windows::{
    core::PCWSTR,
    Win32::Graphics::Gdi::*,
};
use crate::network_editor::*;

/// Represents different types of setting values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingValue {
    Bool(bool),
    String(String),
    Integer(i64),
    Float(f64),
    Selection(String), // For dropdown selections
    Resolution { width: u32, height: u32 },
    Custom(serde_json::Value),
}

impl fmt::Display for SettingValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingValue::Bool(b) => write!(f, "{}", if *b { "Enabled" } else { "Disabled" }),
            SettingValue::String(s) => write!(f, "{}", s),
            SettingValue::Integer(i) => write!(f, "{}", i),
            SettingValue::Float(fl) => write!(f, "{:.2}", fl),
            SettingValue::Selection(s) => write!(f, "{}", s),
            SettingValue::Resolution { width, height } => write!(f, "{}x{}", width, height),
            SettingValue::Custom(v) => write!(f, "{}", v),
        }
    }
}

/// Represents an option for a setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingOption {
    pub label: String,
    pub value: SettingValue,
    pub description: Option<String>,
}

/// Types of UI editors for different settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorType {
    Toggle,
    Dropdown,
    Slider { min: f64, max: f64, step: f64 },
    TextInput { multiline: bool },
    NumberInput { min: Option<i64>, max: Option<i64> },
    ResolutionPicker,
    Custom,
}

/// Trait for implementing setting editors
pub trait SettingEditor: Send + Sync + fmt::Debug {
    /// Create a clone of the trait object
    fn clone_box(&self) -> Box<dyn SettingEditor>;

    /// Get the current value of the setting
    fn get_current_value(&self) -> Result<SettingValue>;
    
    /// Get available options for this setting
    fn get_available_options(&self) -> Result<Vec<SettingOption>>;
    
    /// Set a new value for the setting
    fn set_value(&self, value: SettingValue) -> Result<()>;
    
    /// Validate if a value is acceptable
    fn validate_value(&self, value: &SettingValue) -> Result<bool>;
    
    /// Get the appropriate editor type for UI
    fn get_editor_type(&self) -> EditorType;
    
    /// Check if setting requires admin privileges
    fn requires_admin(&self) -> bool;
}

impl Clone for Box<dyn SettingEditor> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Display settings editor implementation
#[derive(Debug, Clone)]
pub struct DisplaySettingsEditor;

impl DisplaySettingsEditor {
    pub fn new() -> Self {
        Self
    }
    
    fn get_display_modes(&self) -> Result<Vec<(u32, u32, u32)>> {
        let mut modes = Vec::new();
        let mut dev_mode = DEVMODEW::default();
        dev_mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
        
        let mut i = 0;
        unsafe {
            loop {
                let result = EnumDisplaySettingsW(PCWSTR::null(), ENUM_DISPLAY_SETTINGS_MODE(i), &mut dev_mode);
                if !result.as_bool() {
                    break;
                }
                
                // Only add unique resolutions with common refresh rates
                let resolution = (dev_mode.dmPelsWidth, dev_mode.dmPelsHeight, dev_mode.dmDisplayFrequency);
                if !modes.contains(&resolution) && dev_mode.dmDisplayFrequency >= 59 {
                    modes.push(resolution);
                }
                i += 1;
            }
        }
        
        // Sort by resolution (width * height) descending
        modes.sort_by(|a, b| {
            let area_a = a.0 * a.1;
            let area_b = b.0 * b.1;
            area_b.cmp(&area_a)
        });
        
        Ok(modes)
    }
    
    fn get_current_display_mode(&self) -> Result<(u32, u32, u32)> {
        let mut dev_mode = DEVMODEW::default();
        dev_mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
        
        unsafe {
            let result = EnumDisplaySettingsW(PCWSTR::null(), ENUM_CURRENT_SETTINGS, &mut dev_mode);
            if !result.as_bool() {
                anyhow::bail!("Failed to get current display settings");
            }
        }
        
        Ok((dev_mode.dmPelsWidth, dev_mode.dmPelsHeight, dev_mode.dmDisplayFrequency))
    }
}

impl SettingEditor for DisplaySettingsEditor {
    fn clone_box(&self) -> Box<dyn SettingEditor> {
        Box::new(self.clone())
    }

    fn get_current_value(&self) -> Result<SettingValue> {
        let (width, height, _) = self.get_current_display_mode()?;
        Ok(SettingValue::Resolution { width, height })
    }
    
    fn get_available_options(&self) -> Result<Vec<SettingOption>> {
        let modes = self.get_display_modes()?;
        let mut options = Vec::new();
        
        // Group by resolution, showing refresh rates
        let mut seen_resolutions = std::collections::HashSet::new();
        
        for (width, height, refresh) in modes {
            let res_key = (width, height);
            if seen_resolutions.insert(res_key) {
                options.push(SettingOption {
                    label: format!("{} × {}", width, height),
                    value: SettingValue::Resolution { width, height },
                    description: Some(format!("{}Hz available", refresh)),
                });
            }
        }
        
        Ok(options)
    }
    
    fn set_value(&self, value: SettingValue) -> Result<()> {
        if let SettingValue::Resolution { width, height } = value {
            let mut dev_mode = DEVMODEW::default();
            dev_mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
            dev_mode.dmPelsWidth = width;
            dev_mode.dmPelsHeight = height;
            dev_mode.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT;
            
            unsafe {
                let result = ChangeDisplaySettingsW(Some(&dev_mode), CDS_TEST);
                if result != DISP_CHANGE_SUCCESSFUL {
                    anyhow::bail!("Display mode test failed: {:?}", result);
                }
                
                let result = ChangeDisplaySettingsW(Some(&dev_mode), CDS_TYPE(0));
                if result != DISP_CHANGE_SUCCESSFUL {
                    anyhow::bail!("Failed to change display settings: {:?}", result);
                }
            }
            Ok(())
        } else {
            anyhow::bail!("Invalid value type for display settings")
        }
    }
    
    fn validate_value(&self, value: &SettingValue) -> Result<bool> {
        if let SettingValue::Resolution { width, height } = value {
            let modes = self.get_display_modes()?;
            Ok(modes.iter().any(|(w, h, _)| w == width && h == height))
        } else {
            Ok(false)
        }
    }
    
    fn get_editor_type(&self) -> EditorType {
        EditorType::ResolutionPicker
    }
    
    fn requires_admin(&self) -> bool {
        false
    }
}

/// Power plan settings editor
#[derive(Debug, Clone)]
pub struct PowerPlanEditor;

impl PowerPlanEditor {
    pub fn new() -> Self {
        Self
    }
    
    fn get_power_plans(&self) -> Result<Vec<(String, String)>> {
        let output = Command::new("powercfg")
            .args(&["/list"])
            .output()
            .context("Failed to execute powercfg")?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut plans = Vec::new();
        
        for line in output_str.lines() {
            if line.contains("GUID:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(guid_pos) = parts.iter().position(|&x| x == "GUID:") {
                    if let Some(guid) = parts.get(guid_pos + 1) {
                        let name = parts[guid_pos + 2..].join(" ")
                            .trim_start_matches('(')
                            .trim_end_matches(')')
                            .trim_end_matches('*')
                            .trim()
                            .to_string();
                        plans.push((guid.to_string(), name));
                    }
                }
            }
        }
        
        Ok(plans)
    }
    
    fn get_active_plan(&self) -> Result<String> {
        let output = Command::new("powercfg")
            .args(&["/getactivescheme"])
            .output()
            .context("Failed to get active power scheme")?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("GUID:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(guid_pos) = parts.iter().position(|&x| x == "GUID:") {
                    if let Some(guid) = parts.get(guid_pos + 1) {
                        return Ok(guid.to_string());
                    }
                }
            }
        }
        
        anyhow::bail!("Could not determine active power plan")
    }
}

impl SettingEditor for PowerPlanEditor {
    fn clone_box(&self) -> Box<dyn SettingEditor> {
        Box::new(self.clone())
    }

    fn get_current_value(&self) -> Result<SettingValue> {
        let active_guid = self.get_active_plan()?;
        let plans = self.get_power_plans()?;
        
        if let Some((_, name)) = plans.iter().find(|(guid, _)| guid == &active_guid) {
            Ok(SettingValue::Selection(name.clone()))
        } else {
            Ok(SettingValue::Selection("Unknown".to_string()))
        }
    }
    
    fn get_available_options(&self) -> Result<Vec<SettingOption>> {
        let plans = self.get_power_plans()?;
        Ok(plans.into_iter().map(|(guid, name)| {
            SettingOption {
                label: name.clone(),
                value: SettingValue::Selection(guid),
                description: None,
            }
        }).collect())
    }
    
    fn set_value(&self, value: SettingValue) -> Result<()> {
        if let SettingValue::Selection(guid) = value {
            Command::new("powercfg")
                .args(&["/setactive", &guid])
                .output()
                .context("Failed to set active power scheme")?;
            Ok(())
        } else {
            anyhow::bail!("Invalid value type for power plan")
        }
    }
    
    fn validate_value(&self, value: &SettingValue) -> Result<bool> {
        if let SettingValue::Selection(guid) = value {
            let plans = self.get_power_plans()?;
            Ok(plans.iter().any(|(g, _)| g == guid))
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

/// Default audio device editor
#[derive(Debug, Clone)]
pub struct AudioDeviceEditor;

impl AudioDeviceEditor {
    pub fn new() -> Self {
        Self
    }
    
    fn get_audio_devices(&self) -> Result<Vec<(String, String)>> {
        // This is a simplified version - in practice, you'd use Windows Core Audio APIs
        // For now, we'll use PowerShell
        let script = r#"
            Get-CimInstance Win32_SoundDevice | Select-Object Name, DeviceID | ConvertTo-Json
        "#;
        
        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", script])
            .output()
            .context("Failed to get audio devices")?;
        
        let devices: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
            .context("Failed to parse audio devices")?;
        
        Ok(devices.iter()
            .filter_map(|d| {
                let name = d["Name"].as_str()?;
                let id = d["DeviceID"].as_str()?;
                Some((id.to_string(), name.to_string()))
            })
            .collect())
    }
}

impl SettingEditor for AudioDeviceEditor {
    fn clone_box(&self) -> Box<dyn SettingEditor> {
        Box::new(self.clone())
    }

    fn get_current_value(&self) -> Result<SettingValue> {
        // Simplified - would need Core Audio API for actual implementation
        Ok(SettingValue::Selection("Default Device".to_string()))
    }
    
    fn get_available_options(&self) -> Result<Vec<SettingOption>> {
        let devices = self.get_audio_devices()?;
        Ok(devices.into_iter().map(|(id, name)| {
            SettingOption {
                label: name,
                value: SettingValue::Selection(id),
                description: None,
            }
        }).collect())
    }
    
    fn set_value(&self, _value: SettingValue) -> Result<()> {
        // Would require Core Audio API implementation
        anyhow::bail!("Audio device switching not yet implemented")
    }
    
    fn validate_value(&self, value: &SettingValue) -> Result<bool> {
        if let SettingValue::Selection(_) = value {
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    fn get_editor_type(&self) -> EditorType {
        EditorType::Dropdown
    }
    
    fn requires_admin(&self) -> bool {
        false
    }
}

/// Factory function to create appropriate editor for a setting
pub fn create_editor(setting_type: &str) -> Option<Box<dyn SettingEditor>> {
    match setting_type {
        "display_resolution" => Some(Box::new(DisplaySettingsEditor::new())),
        "power_plan" => Some(Box::new(PowerPlanEditor::new())),
        "audio_device" => Some(Box::new(AudioDeviceEditor::new())),
        "wifi_adapter_toggle" => Some(Box::new(NetworkAdapterToggleEditor::new("Wi-Fi".to_string()))),
        "ethernet_adapter_toggle" => Some(Box::new(NetworkAdapterToggleEditor::new("Ethernet".to_string()))),
        "wifi_dns" => Some(Box::new(DNSServerEditor::new("Wi-Fi".to_string()))),
        "ethernet_dns" => Some(Box::new(DNSServerEditor::new("Ethernet".to_string()))),
        "wifi_power_mode" => Some(Box::new(WiFiPowerEditor::new())),
        _ => None,
    }
}
