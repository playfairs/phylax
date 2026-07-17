use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub log_path: Option<PathBuf>,
    pub console_output: bool,
    pub max_size_mb: Option<u64>,
    pub max_files: Option<u32>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
            log_path: None,
            console_output: false,
            max_size_mb: Some(100),
            max_files: Some(10),
        }
    }
}
