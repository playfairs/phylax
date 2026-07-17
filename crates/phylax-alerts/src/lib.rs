use phylax_common::{Error, Result, Severity};
use phylax_storage::{Alert, AlertStatus};
use phylax_engine::Decision;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

pub mod email;
pub mod webhook;
pub mod config;

pub use email::*;
pub use webhook::*;
pub use config::*;

#[derive(Debug, Clone)]
pub struct AlertContext {
    pub decision: Decision,
    pub event_id: Uuid,
    pub severity: Severity,
    pub title: String,
    pub description: Option<String>,
}

pub trait AlertProvider {
    async fn send_alert(&self, context: &AlertContext) -> Result<()>;
    fn provider_name(&self) -> &'static str;
}

pub struct AlertManager {
    providers: Vec<Box<dyn AlertProvider + Send + Sync>>,
}

impl AlertManager {
    pub fn new() -> Self {
        AlertManager {
            providers: Vec::new(),
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn AlertProvider + Send + Sync>) {
        self.providers.push(provider);
    }

    pub async fn send_alert(&self, context: &AlertContext) -> Result<()> {
        let mut errors = Vec::new();

        for provider in &self.providers {
            if let Err(e) = provider.send_alert(context).await {
                errors.push(format!("{}: {}", provider.provider_name(), e));
            }
        }

        if !errors.is_empty() {
            return Err(Error::AlertError(format!("Failed to send alerts: {}", errors.join(", "))));
        }

        Ok(())
    }

    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}