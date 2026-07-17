use phylax_common::Severity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_id: Uuid,
    pub severity: Severity,
    pub title: String,
    pub description: Option<String>,
    pub status: AlertStatus,
    pub acknowledged: bool,
}

impl Alert {
    pub fn new(event_id: Uuid, severity: Severity, title: String) -> Self {
        Alert {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_id,
            severity,
            title,
            description: None,
            status: AlertStatus::Open,
            acknowledged: false,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    Open,
    Investigating,
    Resolved,
    Closed,
}

impl AlertStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertStatus::Open => "open",
            AlertStatus::Investigating => "investigating",
            AlertStatus::Resolved => "resolved",
            AlertStatus::Closed => "closed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "open" => Some(AlertStatus::Open),
            "investigating" => Some(AlertStatus::Investigating),
            "resolved" => Some(AlertStatus::Resolved),
            "closed" => Some(AlertStatus::Closed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub title: String,
    pub description: Option<String>,
    pub severity: Severity,
    pub status: IncidentStatus,
    pub event_ids: Vec<Uuid>,
}

impl Incident {
    pub fn new(title: String, severity: Severity) -> Self {
        Incident {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            title,
            description: None,
            severity,
            status: IncidentStatus::Open,
            event_ids: Vec::new(),
        }
    }

    pub fn with_events(mut self, event_ids: Vec<Uuid>) -> Self {
        self.event_ids = event_ids;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IncidentStatus {
    Open,
    Active,
    Contained,
    Eradicated,
    Resolved,
}

impl IncidentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            IncidentStatus::Open => "open",
            IncidentStatus::Active => "active",
            IncidentStatus::Contained => "contained",
            IncidentStatus::Eradicated => "eradicated",
            IncidentStatus::Resolved => "resolved",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "open" => Some(IncidentStatus::Open),
            "active" => Some(IncidentStatus::Active),
            "contained" => Some(IncidentStatus::Contained),
            "eradicated" => Some(IncidentStatus::Eradicated),
            "resolved" => Some(IncidentStatus::Resolved),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub incident_id: Uuid,
    pub responder_type: String,
    pub action: String,
    pub status: ResponseStatus,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Pending,
    Executing,
    Success,
    Failed,
    Cancelled,
}

impl ResponseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResponseStatus::Pending => "pending",
            ResponseStatus::Executing => "executing",
            ResponseStatus::Success => "success",
            ResponseStatus::Failed => "failed",
            ResponseStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Some(ResponseStatus::Pending),
            "executing" => Some(ResponseStatus::Executing),
            "success" => Some(ResponseStatus::Success),
            "failed" => Some(ResponseStatus::Failed),
            "cancelled" => Some(ResponseStatus::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub incident_id: Uuid,
    pub evidence_type: String,
    pub path: Option<String>,
    pub hash: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
