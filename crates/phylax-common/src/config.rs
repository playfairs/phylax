use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub logging: LoggingConfig,
    pub storage: StorageConfig,
    pub collectors: CollectorsConfig,
    pub responders: RespondersConfig,
    pub alerts: AlertsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub bind_address: String,
    pub bind_port: u16,
    pub worker_threads: Option<usize>,
    pub max_events_per_second: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_path: Option<String>,
    pub max_size_mb: Option<u64>,
    pub max_files: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_path: String,
    pub max_size_mb: Option<u64>,
    pub retention_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorsConfig {
    pub enabled: Vec<String>,
    pub ssh: Option<SshCollectorConfig>,
    pub process: Option<ProcessCollectorConfig>,
    pub filesystem: Option<FilesystemCollectorConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshCollectorConfig {
    pub monitor_log_file: Option<String>,
    pub log_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCollectorConfig {
    pub monitor_interval_ms: u64,
    pub track_children: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemCollectorConfig {
    pub watch_paths: Vec<String>,
    pub ignore_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespondersConfig {
    pub enabled: Vec<String>,
    pub require_approval: bool,
    pub ip_blocking: Option<IpBlockingConfig>,
    pub process_termination: Option<ProcessTerminationConfig>,
    pub file_quarantine: Option<FileQuarantineConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpBlockingConfig {
    pub firewall_command: Option<String>,
    pub block_duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTerminationConfig {
    pub allow_termination: bool,
    pub require_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileQuarantineConfig {
    pub quarantine_path: String,
    pub allow_restore: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsConfig {
    pub enabled: Vec<String>,
    pub email: Option<EmailAlertConfig>,
    pub webhook: Option<WebhookAlertConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAlertConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookAlertConfig {
    pub url: String,
    pub method: String,
    pub headers: Option<Vec<(String, String)>>,
    pub timeout_seconds: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            daemon: DaemonConfig {
                bind_address: "127.0.0.1".to_string(),
                bind_port: 8080,
                worker_threads: None,
                max_events_per_second: Some(1000),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                file_path: None,
                max_size_mb: Some(100),
                max_files: Some(10),
            },
            storage: StorageConfig {
                database_path: "/var/lib/phylax/phylax.db".to_string(),
                max_size_mb: Some(1000),
                retention_days: Some(90),
            },
            collectors: CollectorsConfig {
                enabled: vec!["ssh".to_string(), "process".to_string()],
                ssh: Some(SshCollectorConfig {
                    monitor_log_file: None,
                    log_paths: vec![
                        "/var/log/auth.log".to_string(),
                        "/var/log/secure".to_string(),
                    ],
                }),
                process: Some(ProcessCollectorConfig {
                    monitor_interval_ms: 1000,
                    track_children: true,
                }),
                filesystem: Some(FilesystemCollectorConfig {
                    watch_paths: vec!["/etc".to_string(), "/home".to_string()],
                    ignore_paths: vec![],
                }),
            },
            responders: RespondersConfig {
                enabled: vec![],
                require_approval: true,
                ip_blocking: Some(IpBlockingConfig {
                    firewall_command: None,
                    block_duration_seconds: Some(86400),
                }),
                process_termination: Some(ProcessTerminationConfig {
                    allow_termination: false,
                    require_confirmation: true,
                }),
                file_quarantine: Some(FileQuarantineConfig {
                    quarantine_path: "/var/lib/phylax/quarantine".to_string(),
                    allow_restore: true,
                }),
            },
            alerts: AlertsConfig {
                enabled: vec![],
                email: None,
                webhook: None,
            },
        }
    }
}

pub fn load_config(path: &PathBuf) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::ConfigError(format!("Failed to read config file: {}", e)))?;
    
    let config: Config = toml::from_str(&content)
        .map_err(|e| Error::ConfigError(format!("Failed to parse config: {}", e)))?;
    
    Ok(config)
}

pub fn save_config(config: &Config, path: &PathBuf) -> Result<()> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| Error::ConfigError(format!("Failed to serialize config: {}", e)))?;
    
    std::fs::write(path, content)
        .map_err(|e| Error::ConfigError(format!("Failed to write config file: {}", e)))?;
    
    Ok(())
}
