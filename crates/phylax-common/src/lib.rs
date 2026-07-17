use std::path::PathBuf;
use thiserror::Error;

pub mod error;
pub mod result;
pub mod config;
pub mod platform;

pub use error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "info" => Some(Severity::Info),
            "low" => Some(Severity::Low),
            "medium" => Some(Severity::Medium),
            "high" => Some(Severity::High),
            "critical" => Some(Severity::Critical),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        return Platform::Linux;
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        panic!("Unsupported platform");
    }
}

pub fn get_config_path() -> Result<PathBuf> {
    let mut path = match Platform::current() {
        Platform::Linux => {
            let mut p = PathBuf::from("/etc");
            p.push("phylax");
            p
        }
        Platform::MacOS => {
            let mut p = PathBuf::from("/Library");
            p.push("Application Support");
            p.push("phylax");
            p
        }
        Platform::Windows => {
            let mut p = std::env::var("PROGRAMDATA")
                .map_err(|e| Error::ConfigError(format!("Failed to get PROGRAMDATA: {}", e)))?;
            let mut pb = PathBuf::from(p);
            pb.push("phylax");
            pb
        }
    };
    path.push("phylax.toml");
    Ok(path)
}

pub fn get_data_path() -> Result<PathBuf> {
    let mut path = match Platform::current() {
        Platform::Linux => {
            let mut p = PathBuf::from("/var");
            p.push("lib");
            p.push("phylax");
            p
        }
        Platform::MacOS => {
            let mut p = PathBuf::from("/Library");
            p.push("Application Support");
            p.push("phylax");
            p
        }
        Platform::Windows => {
            let mut p = std::env::var("PROGRAMDATA")
                .map_err(|e| Error::ConfigError(format!("Failed to get PROGRAMDATA: {}", e)))?;
            let mut pb = PathBuf::from(p);
            pb.push("phylax");
            pb
        }
    };
    Ok(path)
}

pub fn ensure_dir_exists(path: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(path)
        .map_err(|e| Error::IoError(format!("Failed to create directory {}: {}", path.display(), e)))?;
    Ok(())
}