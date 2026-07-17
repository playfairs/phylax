use phylax_common::{Error, Result};
use phylax_engine::{Decision, Action, ActionType};
use uuid::Uuid;
use std::collections::HashMap;

pub mod ip_blocking;
pub mod process_termination;
pub mod file_quarantine;
pub mod account;
pub mod manager;

pub use ip_blocking::*;
pub use process_termination::*;
pub use file_quarantine::*;
pub use account::*;
pub use manager::*;

pub trait Responder {
    fn name(&self) -> &'static str;
    fn execute(&self, decision: &Decision, action: &Action) -> Result<ResponseResult>;
    fn requires_approval(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct ResponseResult {
    pub success: bool,
    pub message: String,
    pub details: Option<HashMap<String, String>>,
}

impl ResponseResult {
    pub fn success(message: String) -> Self {
        ResponseResult {
            success: true,
            message,
            details: None,
        }
    }

    pub fn failure(message: String) -> Self {
        ResponseResult {
            success: false,
            message,
            details: None,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = Some(details);
        self
    }
}

pub struct ResponderConfig {
    pub require_approval: bool,
    pub allow_ip_blocking: bool,
    pub allow_process_termination: bool,
    pub allow_file_quarantine: bool,
    pub allow_account_disable: bool,
}

impl Default for ResponderConfig {
    fn default() -> Self {
        ResponderConfig {
            require_approval: true,
            allow_ip_blocking: false,
            allow_process_termination: false,
            allow_file_quarantine: true,
            allow_account_disable: false,
        }
    }
}