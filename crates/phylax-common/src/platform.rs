use crate::{Error, Platform, Result};

pub trait PlatformDetection {
    fn detect_platform() -> Platform;
    fn is_linux() -> bool;
    fn is_macos() -> bool;
    fn is_windows() -> bool;
}

pub struct PlatformDetector;

impl PlatformDetection for PlatformDetector {
    fn detect_platform() -> Platform {
        Platform::current()
    }

    fn is_linux() -> bool {
        cfg!(target_os = "linux")
    }

    fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }

    fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }
}

pub fn get_system_info() -> Result<SystemInfo> {
    Ok(SystemInfo {
        platform: Platform::current(),
        hostname: std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string()),
        username: std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string()),
    })
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub platform: Platform,
    pub hostname: String,
    pub username: String,
}
