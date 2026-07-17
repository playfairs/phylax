use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub require_approval_for_high_threat: bool,
    pub auto_respond_critical: bool,
    pub default_action: String,
    pub log_all_events: bool,
    pub alert_threshold: String,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        PolicyConfig {
            require_approval_for_high_threat: true,
            auto_respond_critical: false,
            default_action: "log_only".to_string(),
            log_all_events: true,
            alert_threshold: "medium".to_string(),
        }
    }
}
