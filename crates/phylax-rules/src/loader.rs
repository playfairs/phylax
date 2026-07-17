use phylax_common::{Error, Result};
use phylax_rules::Rule;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
struct RuleFile {
    name: String,
    event: String,
    severity: String,
    #[serde(default)]
    conditions: Vec<RuleCondition>,
    #[serde(default)]
    actions: Vec<RuleAction>,
    #[serde(default = "default_enabled")]
    enabled: bool,
}

#[derive(Debug, serde::Deserialize)]
struct RuleCondition {
    field: String,
    operator: String,
    value: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
struct RuleAction {
    action_type: String,
    #[serde(default)]
    parameters: std::collections::HashMap<String, serde_json::Value>,
}

fn default_enabled() -> bool {
    true
}

pub struct RuleLoader;

impl RuleLoader {
    pub fn load_from_file(path: &Path) -> Result<Rule> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::RuleError(format!("Failed to read rule file: {}", e)))?;

        let rule_file: RuleFile = toml::from_str(&content)
            .map_err(|e| Error::RuleError(format!("Failed to parse rule file: {}", e)))?;

        let mut rule = Rule::new(rule_file.name, rule_file.event, rule_file.severity);
        rule.enabled = rule_file.enabled;

        for cond in rule_file.conditions {
            let operator = match cond.operator.as_str() {
                "equals" => phylax_rules::ConditionOperator::Equals,
                "not_equals" => phylax_rules::ConditionOperator::NotEquals,
                "contains" => phylax_rules::ConditionOperator::Contains,
                "not_contains" => phylax_rules::ConditionOperator::NotContains,
                "greater_than" => phylax_rules::ConditionOperator::GreaterThan,
                "less_than" => phylax_rules::ConditionOperator::LessThan,
                "matches" => phylax_rules::ConditionOperator::Matches,
                "exists" => phylax_rules::ConditionOperator::Exists,
                _ => return Err(Error::RuleError(format!("Unknown operator: {}", cond.operator))),
            };

            let condition = phylax_rules::Condition {
                field: cond.field,
                operator,
                value: cond.value,
            };

            rule = rule.with_condition(condition);
        }

        for action in rule_file.actions {
            let action_type = match action.action_type.as_str() {
                "alert" => phylax_rules::ActionType::Alert,
                "log" => phylax_rules::ActionType::Log,
                "block_ip" => phylax_rules::ActionType::BlockIp,
                "terminate_process" => phylax_rules::ActionType::TerminateProcess,
                "quarantine_file" => phylax_rules::ActionType::QuarantineFile,
                "disable_account" => phylax_rules::ActionType::DisableAccount,
                _ => return Err(Error::RuleError(format!("Unknown action type: {}", action.action_type))),
            };

            let mut action_obj = phylax_rules::Action::new(action_type);
            for (key, value) in action.parameters {
                action_obj = action_obj.with_parameter(key, value);
            }

            rule = rule.with_action(action_obj);
        }

        Ok(rule)
    }
}
