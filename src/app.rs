use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    System,
    Network,
    Personalization,
    Apps,
    Accounts,
    TimeLanguage,
    Gaming,
    Accessibility,
    Privacy,
    Update,
    ControlPanel,
}

impl Category {
    pub fn display_name(&self) -> &'static str {
        match self {
            Category::System => "System & Display",
            Category::Network => "Network & Internet",
            Category::Personalization => "Personalization",
            Category::Apps => "Apps & Features",
            Category::Accounts => "Accounts",
            Category::TimeLanguage => "Time & Language",
            Category::Gaming => "Gaming",
            Category::Accessibility => "Accessibility",
            Category::Privacy => "Privacy & Security",
            Category::Update => "Windows Update",
            Category::ControlPanel => "Control Panel (Classic)",
        }
    }
    
    pub fn all() -> Vec<Category> {
        vec![
            Category::System,
            Category::Network,
            Category::Personalization,
            Category::Apps,
            Category::Accounts,
            Category::TimeLanguage,
            Category::Gaming,
            Category::Accessibility,
            Category::Privacy,
            Category::Update,
            Category::ControlPanel,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LaunchType {
    MsSettings(String),
    ControlPanel(String),
    RunDll32(String),
    PowerShell(String),
    Command(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsItem {
    pub name: String,
    pub description: Option<String>,
    pub category: Category,
    pub launch_command: LaunchType,
    pub icon: Option<char>,
    pub requires_admin: bool,
    pub keywords: Vec<String>,
}

impl SettingsItem {
    pub fn new(
        name: impl Into<String>,
        category: Category,
        launch_command: LaunchType,
    ) -> Self {
        Self {
            name: name.into(),
            description: None,
            category,
            launch_command,
            icon: None,
            requires_admin: false,
            keywords: vec![],
        }
    }
    
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
    
    pub fn with_icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }
    
    pub fn with_admin(mut self) -> Self {
        self.requires_admin = true;
        self
    }
    
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }
}

pub static SETTINGS_ITEMS: Lazy<Vec<SettingsItem>> = Lazy::new(|| {
    vec![
        // System & Display
        SettingsItem::new("Display Settings", Category::System, LaunchType::MsSettings("display".into()))
            .with_description("Configure display resolution, scale, and multiple monitors")
            .with_icon('🖥'),
        SettingsItem::new("Sound Settings", Category::System, LaunchType::MsSettings("sound".into()))
            .with_description("Manage audio devices and sound preferences")
            .with_icon('🔊'),
        SettingsItem::new("Power & Battery", Category::System, LaunchType::MsSettings("powersleep".into()))
            .with_description("Power plans, battery settings, and sleep options")
            .with_icon('🔋'),
        SettingsItem::new("Storage", Category::System, LaunchType::MsSettings("storagesense".into()))
            .with_description("View storage usage and manage drives")
            .with_icon('💾'),
        SettingsItem::new("About This PC", Category::System, LaunchType::MsSettings("about".into()))
            .with_description("View PC specifications and Windows version")
            .with_icon('ℹ'),
        SettingsItem::new("System Properties", Category::System, LaunchType::ControlPanel("sysdm.cpl".into()))
            .with_description("Advanced system settings and computer name")
            .with_icon('⚙')
            .with_admin(),
            
        // Network & Internet
        SettingsItem::new("Wi-Fi", Category::Network, LaunchType::MsSettings("network-wifi".into()))
            .with_description("Wi-Fi settings and available networks")
            .with_icon('📶'),
        SettingsItem::new("Ethernet", Category::Network, LaunchType::MsSettings("network-ethernet".into()))
            .with_description("Wired network settings")
            .with_icon('🔌'),
        SettingsItem::new("VPN", Category::Network, LaunchType::MsSettings("network-vpn".into()))
            .with_description("Virtual Private Network connections")
            .with_icon('🔐'),
        SettingsItem::new("Network Status", Category::Network, LaunchType::MsSettings("network-status".into()))
            .with_description("View network status and properties")
            .with_icon('🌐'),
        SettingsItem::new("Network Connections", Category::Network, LaunchType::ControlPanel("ncpa.cpl".into()))
            .with_description("Classic network adapter settings")
            .with_icon('🖧'),
            
        // Personalization
        SettingsItem::new("Background", Category::Personalization, LaunchType::MsSettings("personalization-background".into()))
            .with_description("Desktop background and slideshow settings")
            .with_icon('🖼'),
        SettingsItem::new("Colors", Category::Personalization, LaunchType::MsSettings("personalization-colors".into()))
            .with_description("Windows colors and transparency effects")
            .with_icon('🎨'),
        SettingsItem::new("Themes", Category::Personalization, LaunchType::MsSettings("personalization-themes".into()))
            .with_description("Save and apply theme combinations")
            .with_icon('🎭'),
        SettingsItem::new("Lock Screen", Category::Personalization, LaunchType::MsSettings("lockscreen".into()))
            .with_description("Lock screen background and app settings")
            .with_icon('🔒'),
        SettingsItem::new("Taskbar", Category::Personalization, LaunchType::MsSettings("taskbar".into()))
            .with_description("Taskbar behavior and icon settings")
            .with_icon('📎'),
            
        // Apps & Features
        SettingsItem::new("Apps & Features", Category::Apps, LaunchType::MsSettings("appsfeatures".into()))
            .with_description("Uninstall, modify, or repair apps")
            .with_icon('📦'),
        SettingsItem::new("Default Apps", Category::Apps, LaunchType::MsSettings("defaultapps".into()))
            .with_description("Choose default apps for file types")
            .with_icon('🔧'),
        SettingsItem::new("Optional Features", Category::Apps, LaunchType::MsSettings("optionalfeatures".into()))
            .with_description("Add or remove Windows features")
            .with_icon('➕'),
        SettingsItem::new("Startup Apps", Category::Apps, LaunchType::MsSettings("startupapps".into()))
            .with_description("Control which apps run at startup")
            .with_icon('🚀'),
        SettingsItem::new("Programs and Features", Category::Apps, LaunchType::ControlPanel("appwiz.cpl".into()))
            .with_description("Classic uninstall or change programs")
            .with_icon('💿'),
            
        // Accounts
        SettingsItem::new("Your Info", Category::Accounts, LaunchType::MsSettings("yourinfo".into()))
            .with_description("Account picture and information")
            .with_icon('👤'),
        SettingsItem::new("Email & Accounts", Category::Accounts, LaunchType::MsSettings("emailandaccounts".into()))
            .with_description("Add email, calendar, and contact accounts")
            .with_icon('📧'),
        SettingsItem::new("Sign-in Options", Category::Accounts, LaunchType::MsSettings("signinoptions".into()))
            .with_description("PIN, password, and Windows Hello")
            .with_icon('🔑'),
        SettingsItem::new("Family & Other Users", Category::Accounts, LaunchType::MsSettings("otherusers".into()))
            .with_description("Add family members and other users")
            .with_icon('👨‍👩‍👧‍👦'),
            
        // Time & Language
        SettingsItem::new("Date & Time", Category::TimeLanguage, LaunchType::MsSettings("dateandtime".into()))
            .with_description("Time zone and date format settings")
            .with_icon('🕐'),
        SettingsItem::new("Region", Category::TimeLanguage, LaunchType::MsSettings("regionformatting".into()))
            .with_description("Regional formats for dates and numbers")
            .with_icon('🌍'),
        SettingsItem::new("Language", Category::TimeLanguage, LaunchType::MsSettings("language".into()))
            .with_description("Display language and keyboard settings")
            .with_icon('🔤'),
            
        // Gaming
        SettingsItem::new("Xbox Game Bar", Category::Gaming, LaunchType::MsSettings("gaming-gamebar".into()))
            .with_description("Game bar shortcuts and settings")
            .with_icon('🎮'),
        SettingsItem::new("Game Mode", Category::Gaming, LaunchType::MsSettings("gaming-gamemode".into()))
            .with_description("Optimize Windows for gaming")
            .with_icon('🏁'),
        SettingsItem::new("Captures", Category::Gaming, LaunchType::MsSettings("gaming-gamedvr".into()))
            .with_description("Screenshot and game recording settings")
            .with_icon('📸'),
            
        // Accessibility
        SettingsItem::new("Display", Category::Accessibility, LaunchType::MsSettings("easeofaccess-display".into()))
            .with_description("Text size and display preferences")
            .with_icon('👁'),
        SettingsItem::new("Mouse Pointer", Category::Accessibility, LaunchType::MsSettings("easeofaccess-mousepointer".into()))
            .with_description("Pointer size and color")
            .with_icon('🖱'),
        SettingsItem::new("Narrator", Category::Accessibility, LaunchType::MsSettings("easeofaccess-narrator".into()))
            .with_description("Screen reader settings")
            .with_icon('🗣'),
        SettingsItem::new("Magnifier", Category::Accessibility, LaunchType::MsSettings("easeofaccess-magnifier".into()))
            .with_description("Screen magnification settings")
            .with_icon('🔍'),
            
        // Privacy & Security
        SettingsItem::new("Windows Security", Category::Privacy, LaunchType::MsSettings("windowsdefender".into()))
            .with_description("Antivirus and threat protection")
            .with_icon('🛡'),
        SettingsItem::new("Camera Privacy", Category::Privacy, LaunchType::MsSettings("privacy-webcam".into()))
            .with_description("Control app access to camera")
            .with_icon('📷'),
        SettingsItem::new("Microphone Privacy", Category::Privacy, LaunchType::MsSettings("privacy-microphone".into()))
            .with_description("Control app access to microphone")
            .with_icon('🎤'),
        SettingsItem::new("Location Privacy", Category::Privacy, LaunchType::MsSettings("privacy-location".into()))
            .with_description("Control location access")
            .with_icon('📍'),
            
        // Windows Update
        SettingsItem::new("Check for Updates", Category::Update, LaunchType::MsSettings("windowsupdate-action".into()))
            .with_description("Check and install Windows updates")
            .with_icon('🔄'),
        SettingsItem::new("Update History", Category::Update, LaunchType::MsSettings("windowsupdate-history".into()))
            .with_description("View installed updates")
            .with_icon('📜'),
        SettingsItem::new("Advanced Options", Category::Update, LaunchType::MsSettings("windowsupdate-options".into()))
            .with_description("Update delivery and installation options")
            .with_icon('⚡'),
            
        // Control Panel (Classic)
        SettingsItem::new("Device Manager", Category::ControlPanel, LaunchType::RunDll32("devmgr.dll DeviceManager_Execute".into()))
            .with_description("Manage hardware devices and drivers")
            .with_icon('🔧')
            .with_admin(),
        SettingsItem::new("Administrative Tools", Category::ControlPanel, LaunchType::Command("control admintools".into()))
            .with_description("Advanced system administration tools")
            .with_icon('🔨')
            .with_admin(),
        SettingsItem::new("Power Options", Category::ControlPanel, LaunchType::ControlPanel("powercfg.cpl".into()))
            .with_description("Classic power plan settings")
            .with_icon('⚡'),
        SettingsItem::new("User Accounts", Category::ControlPanel, LaunchType::ControlPanel("nusrmgr.cpl".into()))
            .with_description("Classic user account control")
            .with_icon('👥'),
    ]
});