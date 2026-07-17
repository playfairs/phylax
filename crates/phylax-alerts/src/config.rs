use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled_providers: Vec<String>,
    pub email: Option<EmailConfig>,
    pub webhook: Option<WebhookConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub method: String,
    pub headers: Option<Vec<(String, String)>>,
    pub timeout_seconds: Option<u64>,
}

impl Default for AlertConfig {
    fn default() -> Self {
        AlertConfig {
            enabled_providers: Vec::new(),
            email: None,
            webhook: None,
        }
    }
}
