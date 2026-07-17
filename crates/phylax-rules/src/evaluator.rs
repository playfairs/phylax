use phylax_events::Event;
use phylax_rules::{Rule, Condition};

pub struct RuleEvaluator;

impl RuleEvaluator {
    pub fn evaluate_rule(rule: &Rule, event: &Event) -> bool {
        if !rule.matches_event(event) {
            return false;
        }

        rule.conditions.iter().all(|cond| cond.evaluate(event))
    }

    pub fn evaluate_rules(rules: &[Rule], event: &Event) -> Vec<&Rule> {
        rules
            .iter()
            .filter(|rule| Self::evaluate_rule(rule, event))
            .collect()
    }
}
