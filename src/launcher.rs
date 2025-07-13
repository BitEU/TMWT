use crate::settings::{LaunchType, SettingsItem};
use anyhow::{Context, Result};
use std::process::Command;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HWND,
        UI::Shell::ShellExecuteW,
        UI::WindowsAndMessaging::SW_SHOWNORMAL,
    },
};

pub fn launch_setting(item: &SettingsItem) -> Result<()> {
    match &item.launch_command {
        LaunchType::MsSettings(uri) => launch_ms_settings(uri, item.requires_admin),
        LaunchType::ControlPanel(cpl) => launch_control_panel(cpl, item.requires_admin),
        LaunchType::RunDll32(cmd) => launch_rundll32(cmd, item.requires_admin),
        LaunchType::PowerShell(cmd) => launch_powershell(cmd, item.requires_admin),
        LaunchType::Command(cmd) => launch_command(cmd, item.requires_admin),
    }
}

fn launch_ms_settings(uri: &str, requires_admin: bool) -> Result<()> {
    let full_uri = format!("ms-settings:{}", uri);
    
    if requires_admin {
        // Use ShellExecute for elevation
        unsafe {
            let uri_wide = to_wide_string(&full_uri);
            let verb_wide = to_wide_string("runas");
            
            let result = ShellExecuteW(
                HWND(0),
                PCWSTR(verb_wide.as_ptr()),
                PCWSTR(uri_wide.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
            
            if result.0 as usize <= 32 {
                anyhow::bail!("Failed to launch settings with admin privileges");
            }
        }
    } else {
        Command::new("cmd")
            .args(&["/c", "start", &full_uri])
            .spawn()
            .context("Failed to launch Settings app")?;
    }
    
    Ok(())
}

fn launch_control_panel(cpl: &str, requires_admin: bool) -> Result<()> {
    if requires_admin {
        // Use ShellExecute for elevation
        unsafe {
            let control_wide = to_wide_string("control.exe");
            let cpl_wide = to_wide_string(cpl);
            let verb_wide = to_wide_string("runas");
            
            let result = ShellExecuteW(
                HWND(0),
                PCWSTR(verb_wide.as_ptr()),
                PCWSTR(control_wide.as_ptr()),
                PCWSTR(cpl_wide.as_ptr()),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
            
            if result.0 as usize <= 32 {
                anyhow::bail!("Failed to launch Control Panel with admin privileges");
            }
        }
    } else {
        Command::new("control")
            .arg(cpl)
            .spawn()
            .context("Failed to launch Control Panel")?;
    }
    
    Ok(())
}

fn launch_rundll32(cmd: &str, requires_admin: bool) -> Result<()> {
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    if parts.is_empty() {
        anyhow::bail!("Invalid rundll32 command");
    }
    
    if requires_admin {
        // Use ShellExecute for elevation
        unsafe {
            let rundll_wide = to_wide_string("rundll32.exe");
            let cmd_wide = to_wide_string(cmd);
            let verb_wide = to_wide_string("runas");
            
            let result = ShellExecuteW(
                HWND(0),
                PCWSTR(verb_wide.as_ptr()),
                PCWSTR(rundll_wide.as_ptr()),
                PCWSTR(cmd_wide.as_ptr()),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
            
            if result.0 as usize <= 32 {
                anyhow::bail!("Failed to launch rundll32 with admin privileges");
            }
        }
    } else {
        let mut command = Command::new("rundll32.exe");
        command.args(parts);
        command.spawn()
            .context("Failed to launch rundll32 command")?;
    }
    
    Ok(())
}

fn launch_powershell(cmd: &str, requires_admin: bool) -> Result<()> {
    if requires_admin {
        // Use ShellExecute for elevation
        unsafe {
            let ps_wide = to_wide_string("powershell.exe");
            let args_wide = to_wide_string(&format!("-Command \"{}\"", cmd));
            let verb_wide = to_wide_string("runas");
            
            let result = ShellExecuteW(
                HWND(0),
                PCWSTR(verb_wide.as_ptr()),
                PCWSTR(ps_wide.as_ptr()),
                PCWSTR(args_wide.as_ptr()),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
            
            if result.0 as usize <= 32 {
                anyhow::bail!("Failed to launch PowerShell with admin privileges");
            }
        }
    } else {
        Command::new("powershell")
            .args(&["-Command", cmd])
            .spawn()
            .context("Failed to launch PowerShell command")?;
    }
    
    Ok(())
}

fn launch_command(cmd: &str, requires_admin: bool) -> Result<()> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Invalid command");
    }
    
    if requires_admin {
        // Use ShellExecute for elevation
        unsafe {
            let exe_wide = to_wide_string(parts[0]);
            let args = if parts.len() > 1 {
                parts[1..].join(" ")
            } else {
                String::new()
            };
            let args_wide = to_wide_string(&args);
            let verb_wide = to_wide_string("runas");
            
            let result = ShellExecuteW(
                HWND(0),
                PCWSTR(verb_wide.as_ptr()),
                PCWSTR(exe_wide.as_ptr()),
                if args.is_empty() { PCWSTR::null() } else { PCWSTR(args_wide.as_ptr()) },
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
            
            if result.0 as usize <= 32 {
                anyhow::bail!("Failed to launch command with admin privileges");
            }
        }
    } else {
        Command::new(parts[0])
            .args(&parts[1..])
            .spawn()
            .context("Failed to launch command")?;
    }
    
    Ok(())
}

fn to_wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}