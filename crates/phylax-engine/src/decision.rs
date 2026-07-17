use phylax_rules::Action;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_id: Uuid,
    pub rule_id: Uuid,
    pub rule_name: String,
    pub threat_level: ThreatLevel,
    pub actions: Vec<Action>,
    pub status: DecisionStatus,
}

impl Decision {
    pub fn new(event_id: Uuid, rule_id: Uuid, rule_name: String, threat_level: ThreatLevel, actions: Vec<Action>) -> Self {
        Decision {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_id,
            rule_id,
            rule_name,
            threat_level,
            actions,
            status: DecisionStatus::Pending,
        }
    }

    pub fn requires_approval(&self) -> bool {
        matches!(self.threat_level, ThreatLevel::High | ThreatLevel::Critical)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DecisionStatus {
    Pending,
    Approved,
    Rejected,
    Executing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThreatLevel {
    Minimal,
    Low,
    Medium,
    High,
    Critical,
}

impl ThreatLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThreatLevel::Minimal => "minimal",
            ThreatLevel::Low => "low",
            ThreatLevel::Medium => "medium",
            ThreatLevel::High => "high",
            ThreatLevel::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "minimal" => Some(ThreatLevel::Minimal),
            "low" => Some(ThreatLevel::Low),
            "medium" => Some(ThreatLevel::Medium),
            "high" => Some(ThreatLevel::High),
            "critical" => Some(ThreatLevel::Critical),
            _ => None,
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            ThreatLevel::Minimal => 1,
            ThreatLevel::Low => 2,
            ThreatLevel::Medium => 3,
            ThreatLevel::High => 4,
            ThreatLevel::Critical => 5,
        }
    }
}
