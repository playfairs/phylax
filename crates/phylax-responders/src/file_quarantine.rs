use phylax_common::{Error, Result};
use phylax_engine::{Decision, Action};
use crate::{Responder, ResponseResult};
use std::path::PathBuf;
use std::fs;

pub struct FileQuarantineResponder {
    require_approval: bool,
    quarantine_path: PathBuf,
    allow_restore: bool,
}

impl FileQuarantineResponder {
    pub fn new(require_approval: bool, quarantine_path: PathBuf, allow_restore: bool) -> Self {
        FileQuarantineResponder {
            require_approval,
            quarantine_path,
            allow_restore,
        }
    }

    fn quarantine_file(&self, file_path: &str) -> Result<ResponseResult> {
        let source_path = PathBuf::from(file_path);
        
        if !source_path.exists() {
            return Ok(ResponseResult::failure("File does not exist".to_string()));
        }

        fs::create_dir_all(&self.quarantine_path)
            .map_err(|e| Error::ResponderError(format!("Failed to create quarantine directory: {}", e)))?;

        let file_name = source_path.file_name()
            .ok_or_else(|| Error::ResponderError("Invalid file name".to_string()))?;

        let dest_path = self.quarantine_path.join(file_name);
        
        if dest_path.exists() {
            let timestamp = chrono::Utc::now().timestamp();
            let new_name = format!("{}_{}", file_name.to_string_lossy(), timestamp);
            let new_dest = self.quarantine_path.join(&new_name);
            fs::rename(&source_path, &new_dest)
                .map_err(|e| Error::ResponderError(format!("Failed to quarantine file: {}", e)))?;
        } else {
            fs::rename(&source_path, &dest_path)
                .map_err(|e| Error::ResponderError(format!("Failed to quarantine file: {}", e)))?;
        }

        Ok(ResponseResult::success(format!("Quarantined file: {}", file_path)))
    }

    fn restore_file(&self, file_path: &str, restore_path: &str) -> Result<ResponseResult> {
        if !self.allow_restore {
            return Ok(ResponseResult::failure("File restore is not allowed".to_string()));
        }

        let source_path = self.quarantine_path.join(file_path);
        let dest_path = PathBuf::from(restore_path);

        if !source_path.exists() {
            return Ok(ResponseResult::failure("Quarantined file does not exist".to_string()));
        }

        fs::rename(&source_path, &dest_path)
            .map_err(|e| Error::ResponderError(format!("Failed to restore file: {}", e)))?;

        Ok(ResponseResult::success(format!("Restored file: {}", file_path)))
    }
}

impl Responder for FileQuarantineResponder {
    fn name(&self) -> &'static str {
        "file_quarantine"
    }

    fn execute(&self, _decision: &Decision, action: &Action) -> Result<ResponseResult> {
        if let Some(path) = action.parameters.get("file_path") {
            if let Some(path_str) = path.as_str() {
                return self.quarantine_file(path_str);
            }
        }
        Ok(ResponseResult::failure("No file path provided".to_string()))
    }

    fn requires_approval(&self) -> bool {
        self.require_approval
    }
}
