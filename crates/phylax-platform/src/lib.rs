use phylax_common::{Error, Result, Platform};
use std::path::PathBuf;

pub mod process;
pub mod filesystem;
pub mod network;
pub mod service;

pub use process::*;
pub use filesystem::*;
pub use network::*;
pub use service::*;

pub trait PlatformInfo {
    fn hostname(&self) -> Result<String>;
    fn username(&self) -> Result<String>;
    fn current_user_id(&self) -> Result<u32>;
    fn process_list(&self) -> Result<Vec<ProcessInfo>>;
}

pub struct PlatformDetector;

impl PlatformDetector {
    pub fn current_platform() -> Platform {
        Platform::current()
    }

    pub fn is_privileged() -> Result<bool> {
        #[cfg(target_os = "linux")]
        {
            Ok(nix::unistd::Uid::effective().is_root())
        }
        #[cfg(target_os = "macos")]
        {
            Ok(unsafe { libc::geteuid() } == 0)
        }
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::HANDLE;
            use windows::Win32::Security::GetTokenInformation;
            use windows::Win32::Security::TOKEN_ELEVATION;
            use windows::Win32::System::Threading::GetCurrentProcess;
            use windows::Win32::System::Threading::GetCurrentProcessToken;
            
            let token = unsafe { GetCurrentProcessToken() };
            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut return_length = 0;
            
            let result = unsafe {
                GetTokenInformation(
                    token,
                    windows::Win32::Security::TokenElevation,
                    Some(&mut elevation as *mut _ as *mut _),
                    std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                    &mut return_length,
                )
            };
            
            Ok(result.as_bool() && elevation.TokenIsElevated != 0)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(Error::PlatformError("Unsupported platform".to_string()))
        }
    }

    pub fn system_paths() -> SystemPaths {
        match Platform::current() {
            Platform::Linux => SystemPaths {
                config: PathBuf::from("/etc"),
                log: PathBuf::from("/var/log"),
                temp: PathBuf::from("/tmp"),
                run: PathBuf::from("/var/run"),
            },
            Platform::MacOS => SystemPaths {
                config: PathBuf::from("/Library/Application Support"),
                log: PathBuf::from("/var/log"),
                temp: PathBuf::from("/tmp"),
                run: PathBuf::from("/var/run"),
            },
            Platform::Windows => SystemPaths {
                config: PathBuf::from(std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string())),
                log: PathBuf::from(std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string())),
                temp: PathBuf::from(std::env::var("TEMP").unwrap_or_else(|_| "C:\\Temp".to_string())),
                run: PathBuf::from(std::env::var("TEMP").unwrap_or_else(|_| "C:\\Temp".to_string())),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemPaths {
    pub config: PathBuf,
    pub log: PathBuf,
    pub temp: PathBuf,
    pub run: PathBuf,
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use nix::unistd::{Uid, Gid};
    
    pub struct LinuxPlatformInfo;
    
    impl PlatformInfo for LinuxPlatformInfo {
        fn hostname(&self) -> Result<String> {
            std::fs::read_to_string("/etc/hostname")
                .map(|s| s.trim().to_string())
                .map_err(|e| Error::PlatformError(format!("Failed to read hostname: {}", e)))
        }
        
        fn username(&self) -> Result<String> {
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .ok_or_else(|| Error::PlatformError("Failed to get username".to_string()))
        }
        
        fn current_user_id(&self) -> Result<u32> {
            Ok(Uid::effective().as_raw())
        }
        
        fn process_list(&self) -> Result<Vec<ProcessInfo>> {
            let mut processes = Vec::new();
            
            for proc_entry in std::fs::read_dir("/proc")
                .map_err(|e| Error::PlatformError(format!("Failed to read /proc: {}", e)))?
            {
                let entry = proc_entry
                    .map_err(|e| Error::PlatformError(format!("Failed to read proc entry: {}", e)))?;
                
                if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
                    if let Ok(info) = self.get_process_info(pid) {
                        processes.push(info);
                    }
                }
            }
            
            Ok(processes)
        }
    }
    
    impl LinuxPlatformInfo {
        fn get_process_info(&self, pid: u32) -> Result<ProcessInfo> {
            let cmdline_path = format!("/proc/{}/cmdline", pid);
            let stat_path = format!("/proc/{}/stat", pid);
            
            let cmdline = std::fs::read_to_string(&cmdline_path)
                .unwrap_or_else(|_| String::new())
                .replace('\0', " ");
            
            let stat = std::fs::read_to_string(&stat_path)
                .unwrap_or_else(|_| String::new());
            
            let parts: Vec<&str> = stat.split_whitespace().collect();
            let comm = parts.get(1).unwrap_or(&&"unknown").to_string();
            let ppid = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
            
            Ok(ProcessInfo {
                pid,
                ppid: Some(ppid),
                name: comm,
                command: cmdline,
                user: None,
            })
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    
    pub struct MacosPlatformInfo;
    
    impl PlatformInfo for MacosPlatformInfo {
        fn hostname(&self) -> Result<String> {
            let output = std::process::Command::new("hostname")
                .output()
                .map_err(|e| Error::PlatformError(format!("Failed to get hostname: {}", e)))?;
            
            String::from_utf8(output.stdout)
                .map(|s| s.trim().to_string())
                .map_err(|e| Error::PlatformError(format!("Invalid hostname: {}", e)))
        }
        
        fn username(&self) -> Result<String> {
            std::env::var("USER")
                .ok_or_else(|| Error::PlatformError("Failed to get username".to_string()))
        }
        
        fn current_user_id(&self) -> Result<u32> {
            Ok(unsafe { libc::geteuid() } as u32)
        }
        
        fn process_list(&self) -> Result<Vec<ProcessInfo>> {
            let output = std::process::Command::new("ps")
                .args(["-axo", "pid,ppid,comm,args"])
                .output()
                .map_err(|e| Error::PlatformError(format!("Failed to list processes: {}", e)))?;
            
            let mut processes = Vec::new();
            let lines = String::from_utf8_lossy(&output.stdout);
            
            for line in lines.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let pid = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let ppid = parts.get(1).and_then(|s| s.parse().ok());
                    let name = parts.get(2).unwrap_or(&"unknown").to_string();
                    let command = parts[3..].join(" ");
                    
                    processes.push(ProcessInfo {
                        pid,
                        ppid,
                        name,
                        command,
                        user: None,
                    });
                }
            }
            
            Ok(processes)
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use windows::Win32::System::ProcessStatus::{GetProcessImageFileNameW, K32GetModuleBaseNameW};
    use windows::Win32::System::Threading::{GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
    use windows::Win32::Foundation::{HANDLE, CloseHandle};
    
    pub struct WindowsPlatformInfo;
    
    impl PlatformInfo for WindowsPlatformInfo {
        fn hostname(&self) -> Result<String> {
            std::env::var("COMPUTERNAME")
                .ok_or_else(|| Error::PlatformError("Failed to get hostname".to_string()))
        }
        
        fn username(&self) -> Result<String> {
            std::env::var("USERNAME")
                .ok_or_else(|| Error::PlatformError("Failed to get username".to_string()))
        }
        
        fn current_user_id(&self) -> Result<u32> {
            Ok(unsafe { GetCurrentProcessId() })
        }
        
        fn process_list(&self) -> Result<Vec<ProcessInfo>> {
            let output = std::process::Command::new("tasklist")
                .args(["/FO", "CSV", "/NH"])
                .output()
                .map_err(|e| Error::PlatformError(format!("Failed to list processes: {}", e)))?;
            
            let mut processes = Vec::new();
            let lines = String::from_utf8_lossy(&output.stdout);
            
            for line in lines.lines() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    let name = parts.get(0).map(|s| s.trim_matches('"')).unwrap_or("unknown");
                    let pid = parts.get(1).and_then(|s| s.trim_matches('"').parse().ok()).unwrap_or(0);
                    
                    processes.push(ProcessInfo {
                        pid,
                        ppid: None,
                        name: name.to_string(),
                        command: name.to_string(),
                        user: None,
                    });
                }
            }
            
            Ok(processes)
        }
    }
}