use phylax_common::{Error, Result};
use phylax_engine::{Decision, Action};
use crate::{Responder, ResponseResult};

pub struct IpBlockingResponder {
    require_approval: bool,
    firewall_command: Option<String>,
}

impl IpBlockingResponder {
    pub fn new(require_approval: bool, firewall_command: Option<String>) -> Self {
        IpBlockingResponder {
            require_approval,
            firewall_command,
        }
    }

    fn block_ip(&self, ip: &str) -> Result<ResponseResult> {
        if let Some(command) = &self.firewall_command {
            let full_command = command.replace("{IP}", ip);
            
            let output = std::process::Command::new("sh")
                .args(["-c", &full_command])
                .output()
                .map_err(|e| Error::ResponderError(format!("Failed to execute firewall command: {}", e)))?;

            if output.status.success() {
                Ok(ResponseResult::success(format!("Blocked IP: {}", ip)))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Ok(ResponseResult::failure(format!("Failed to block IP: {}", error)))
            }
        } else {
            Ok(ResponseResult::failure("No firewall command configured".to_string()))
        }
    }
}

impl Responder for IpBlockingResponder {
    fn name(&self) -> &'static str {
        "ip_blocking"
    }

    fn execute(&self, _decision: &Decision, action: &Action) -> Result<ResponseResult> {
        if let Some(ip) = action.parameters.get("ip_address") {
            if let Some(ip_str) = ip.as_str() {
                return self.block_ip(ip_str);
            }
        }
        Ok(ResponseResult::failure("No IP address provided".to_string()))
    }

    fn requires_approval(&self) -> bool {
        self.require_approval
    }
}
