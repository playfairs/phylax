use phylax_common::{Error, Result};
use std::path::PathBuf;

pub struct FilesystemInfo;

impl FilesystemInfo {
    pub fn file_exists(path: &PathBuf) -> bool {
        path.exists()
    }

    pub fn is_directory(path: &PathBuf) -> Result<bool> {
        Ok(path.is_dir())
    }

    pub fn file_size(path: &PathBuf) -> Result<u64> {
        std::fs::metadata(path)
            .map(|m| m.len())
            .map_err(|e| Error::IoError(format!("Failed to get file size: {}", e)))
    }

    pub fn read_file(path: &PathBuf) -> Result<String> {
        std::fs::read_to_string(path)
            .map_err(|e| Error::IoError(format!("Failed to read file: {}", e)))
    }
}
