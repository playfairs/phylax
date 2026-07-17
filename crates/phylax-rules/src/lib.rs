use phylax_common::{Error, Result, Severity};
use phylax_events::{Event, EventType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

pub mod loader;
pub mod evaluator;

pub use loader::*;
pub use evaluator::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub event: String,
    pub severity: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub enabled: bool,
}

impl Rule {
    pub fn new(name: String, event: String, severity: String) -> Self {
        Rule {
            id: Uuid::new_v4(),
            name,
            event,
            severity,
            conditions: Vec::new(),
            actions: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn with_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    pub fn matches_event(&self, event: &Event) -> bool {
        if !self.enabled {
            return false;
        }

        if self.event != event.event_type.as_str() {
            return false;
        }

        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    Matches,
    Exists,
}

impl Condition {
    pub fn evaluate(&self, event: &Event) -> bool {
        let event_value = self.get_field_value(event);
        match self.operator {
            ConditionOperator::Equals => self.equals(&event_value, &self.value),
            ConditionOperator::NotEquals => !self.equals(&event_value, &self.value),
            ConditionOperator::Contains => self.contains(&event_value, &self.value),
            ConditionOperator::NotContains => !self.contains(&event_value, &self.value),
            ConditionOperator::GreaterThan => self.greater_than(&event_value, &self.value),
            ConditionOperator::LessThan => self.less_than(&event_value, &self.value),
            ConditionOperator::Matches => self.matches(&event_value, &self.value),
            ConditionOperator::Exists => event_value.is_some(),
        }
    }

    fn get_field_value(&self, event: &Event) -> Option<serde_json::Value> {
        match self.field.as_str() {
            "severity" => Some(serde_json::Value::String(event.severity.as_str().to_string())),
            "source.hostname" => Some(serde_json::Value::String(event.source.hostname.clone())),
            "source.user" => event.source.user.as_ref().map(|u| serde_json::Value::String(u.clone())),
            "source.process_name" => event.source.process_name.as_ref().map(|p| serde_json::Value::String(p.clone())),
            "source.ip_address" => event.source.ip_address.as_ref().map(|i| serde_json::Value::String(i.clone())),
            _ => event.metadata.get(&self.field).cloned(),
        }
    }

    fn equals(&self, a: &Option<serde_json::Value>, b: &serde_json::Value) -> bool {
        match a {
            Some(av) => av == b,
            None => false,
        }
    }

    fn contains(&self, a: &Option<serde_json::Value>, b: &serde_json::Value) -> bool {
        match (a, b) {
            (Some(serde_json::Value::String(av)), serde_json::Value::String(bv)) => av.contains(bv),
            _ => false,
        }
    }

    fn greater_than(&self, a: &Option<serde_json::Value>, b: &serde_json::Value) -> bool {
        match (a, b) {
            (Some(serde_json::Value::Number(av)), serde_json::Value::Number(bv)) => {
                av.as_i64().map(|ai| bv.as_i64().map(|bi| ai > bi).unwrap_or(false)).unwrap_or(false)
            }
            _ => false,
        }
    }

    fn less_than(&self, a: &Option<serde_json::Value>, b: &serde_json::Value) -> bool {
        match (a, b) {
            (Some(serde_json::Value::Number(av)), serde_json::Value::Number(bv)) => {
                av.as_i64().map(|ai| bv.as_i64().map(|bi| ai < bi).unwrap_or(false)).unwrap_or(false)
            }
            _ => false,
        }
    }

    fn matches(&self, a: &Option<serde_json::Value>, b: &serde_json::Value) -> bool {
        match (a, b) {
            (Some(serde_json::Value::String(av)), serde_json::Value::String(bv)) => {
                regex::Regex::new(bv).map(|r| r.is_match(av)).unwrap_or(false)
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Alert,
    Log,
    BlockIp,
    TerminateProcess,
    QuarantineFile,
    DisableAccount,
}

impl Action {
    pub fn new(action_type: ActionType) -> Self {
        Action {
            action_type,
            parameters: HashMap::new(),
        }
    }

    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.parameters.insert(key, value);
        self
    }
}

pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        RuleEngine {
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn remove_rule(&mut self, id: &Uuid) -> Result<()> {
        self.rules.retain(|r| r.id != *id);
        Ok(())
    }

    pub fn get_rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn evaluate_event(&self, event: &Event) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|rule| rule.matches_event(event))
            .filter(|rule| rule.conditions.iter().all(|cond| cond.evaluate(event)))
            .collect()
    }

    pub fn load_rules_from_dir(&mut self, dir: &Path) -> Result<usize> {
        let pattern = dir.join("*.toml");
        let paths = glob::glob(pattern.to_str().unwrap())
            .map_err(|e| Error::RuleError(format!("Failed to read rule directory: {}", e)))?;

        let mut count = 0;
        for path in paths {
            let path = path.map_err(|e| Error::RuleError(format!("Invalid rule path: {}", e)))?;
            let rule = RuleLoader::load_from_file(&path)?;
            self.add_rule(rule);
            count += 1;
        }

        Ok(count)
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}