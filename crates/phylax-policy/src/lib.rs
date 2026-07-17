use phylax_common::{Error, Result, Severity};
use phylax_engine::{Decision, ThreatLevel, DecisionStatus};
use phylax_events::Event;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

pub mod config;

pub use config::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<PolicyRule>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub event_type: String,
    pub min_threat_level: ThreatLevel,
    pub action: PolicyAction,
    pub require_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyAction {
    Ignore,
    LogOnly,
    Alert,
    Respond,
}

pub struct PolicyEngine {
    policies: Vec<Policy>,
    config: PolicyConfig,
}

impl PolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        PolicyEngine {
            policies: Vec::new(),
            config,
        }
    }

    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
    }

    pub fn remove_policy(&mut self, id: &Uuid) -> Result<()> {
        self.policies.retain(|p| p.id != *id);
        Ok(())
    }

    pub fn evaluate_decision(&self, decision: &Decision) -> PolicyResult {
        let applicable_policies: Vec<&Policy> = self.policies
            .iter()
            .filter(|p| p.enabled)
            .collect();

        for policy in &applicable_policies {
            for rule in &policy.rules {
                if self.rule_matches(rule, decision) {
                    return PolicyResult {
                        action: rule.action.clone(),
                        require_approval: rule.require_approval || self.config.require_approval_for_high_threat && decision.threat_level == ThreatLevel::Critical,
                        policy_id: policy.id,
                        policy_name: policy.name.clone(),
                    };
                }
            }
        }

        PolicyResult {
            action: PolicyAction::LogOnly,
            require_approval: false,
            policy_id: Uuid::nil(),
            policy_name: "default".to_string(),
        }
    }

    pub fn evaluate_event(&self, event: &Event) -> PolicyAction {
        let severity_action = match event.severity {
            Severity::Info => PolicyAction::LogOnly,
            Severity::Low => PolicyAction::LogOnly,
            Severity::Medium => PolicyAction::Alert,
            Severity::High => PolicyAction::Alert,
            Severity::Critical => PolicyAction::Respond,
        };

        if self.config.auto_respond_critical && event.severity == Severity::Critical {
            return PolicyAction::Respond;
        }

        severity_action
    }

    fn rule_matches(&self, rule: &PolicyRule, decision: &Decision) -> bool {
        decision.threat_level.score() >= rule.min_threat_level.score()
    }

    pub fn should_execute_action(&self, decision: &Decision) -> bool {
        let policy_result = self.evaluate_decision(decision);
        
        match policy_result.action {
            PolicyAction::Ignore => false,
            PolicyAction::LogOnly => false,
            PolicyAction::Alert => false,
            PolicyAction::Respond => {
                if policy_result.require_approval {
                    decision.status == DecisionStatus::Approved
                } else {
                    true
                }
            }
        }
    }

    pub fn should_alert(&self, decision: &Decision) -> bool {
        let policy_result = self.evaluate_decision(decision);
        matches!(policy_result.action, PolicyAction::Alert | PolicyAction::Respond)
    }

    pub fn should_log(&self, _decision: &Decision) -> bool {
        true
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new(PolicyConfig::default())
    }
}

#[derive(Debug, Clone)]
pub struct PolicyResult {
    pub action: PolicyAction,
    pub require_approval: bool,
    pub policy_id: Uuid,
    pub policy_name: String,
}