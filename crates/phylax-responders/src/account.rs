use phylax_common::{Error, Result};
use phylax_engine::{Decision, Action};
use crate::{Responder, ResponseResult};

pub struct AccountDisableResponder {
    require_approval: bool,
}

impl AccountDisableResponder {
    pub fn new(require_approval: bool) -> Self {
        AccountDisableResponder {
            require_approval,
        }
    }

    fn disable_account(&self, username: &str) -> Result<ResponseResult> {
        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("usermod")
                .args(["--lock", username])
                .output()
                .map_err(|e| Error::ResponderError(format!("Failed to disable account: {}", e)))?;

            if output.status.success() {
                Ok(ResponseResult::success(format!("Disabled account: {}", username)))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Ok(ResponseResult::failure(format!("Failed to disable account: {}", error)))
            }
        }

        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("pwpolicy")
                .args(["-u", username, "disableuser"])
                .output()
                .map_err(|e| Error::ResponderError(format!("Failed to disable account: {}", e)))?;

            if output.status.success() {
                Ok(ResponseResult::success(format!("Disabled account: {}", username)))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Ok(ResponseResult::failure(format!("Failed to disable account: {}", error)))
            }
        }

        #[cfg(target_os = "windows")]
        {
            let output = std::process::Command::new("net")
                .args(["user", username, "/active:no"])
                .output()
                .map_err(|e| Error::ResponderError(format!("Failed to disable account: {}", e)))?;

            if output.status.success() {
                Ok(ResponseResult::success(format!("Disabled account: {}", username)))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Ok(ResponseResult::failure(format!("Failed to disable account: {}", error)))
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Ok(ResponseResult::failure("Account disable not supported on this platform".to_string()))
        }
    }
}

impl Responder for AccountDisableResponder {
    fn name(&self) -> &'static str {
        "account_disable"
    }

    fn execute(&self, _decision: &Decision, action: &Action) -> Result<ResponseResult> {
        if let Some(username) = action.parameters.get("username") {
            if let Some(user_str) = username.as_str() {
                return self.disable_account(user_str);
            }
        }
        Ok(ResponseResult::failure("No username provided".to_string()))
    }

    fn requires_approval(&self) -> bool {
        self.require_approval
    }
}
