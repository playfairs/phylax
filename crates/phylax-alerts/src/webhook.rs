use phylax_common::{Error, Result};
use crate::{AlertProvider, AlertContext};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

pub struct WebhookProvider {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    timeout: Duration,
}

impl WebhookProvider {
    pub fn new(url: String, method: String, headers: Vec<(String, String)>, timeout_seconds: u64) -> Self {
        WebhookProvider {
            url,
            method,
            headers,
            timeout: Duration::from_secs(timeout_seconds),
        }
    }

    fn build_payload(&self, context: &AlertContext) -> serde_json::Value {
        json!({
            "alert_id": context.decision.id,
            "event_id": context.event_id,
            "title": context.title,
            "description": context.description,
            "severity": context.severity.as_str(),
            "threat_level": context.decision.threat_level.as_str(),
            "rule_name": context.decision.rule_name,
            "timestamp": context.decision.timestamp.to_rfc3339(),
            "actions": context.decision.actions,
        })
    }
}

#[async_trait::async_trait]
impl AlertProvider for WebhookProvider {
    async fn send_alert(&self, context: &AlertContext) -> Result<()> {
        let client = Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| Error::AlertError(format!("Failed to create HTTP client: {}", e)))?;

        let payload = self.build_payload(context);

        let mut request = match self.method.to_lowercase().as_str() {
            "post" => client.post(&self.url),
            "put" => client.put(&self.url),
            "patch" => client.patch(&self.url),
            _ => client.post(&self.url),
        };

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let response = request
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::AlertError(format!("Failed to send webhook: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            return Err(Error::AlertError(format!(
                "Webhook returned error status {}: {}",
                status, body
            )));
        }

        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "webhook"
    }
}
