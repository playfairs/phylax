use phylax_events::Event;
use phylax_rules::Rule;
use phylax_common::Severity;

pub struct ThreatCalculator;

impl ThreatCalculator {
    pub fn calculate_threat(event: &Event, rule: &Rule) -> u8 {
        let rule_severity = Self::parse_severity(&rule.severity);
        let event_severity = Self::severity_to_score(event.severity);
        
        let base_score = (rule_severity + event_severity) / 2;
        
        let condition_multiplier = if rule.conditions.is_empty() {
            1.0
        } else {
            1.0 + (rule.conditions.len() as f64 * 0.1)
        };
        
        let final_score = (base_score as f64 * condition_multiplier).min(100.0) as u8;
        
        final_score
    }

    fn parse_severity(severity: &str) -> u8 {
        match severity.to_lowercase().as_str() {
            "info" => 10,
            "low" => 25,
            "medium" => 50,
            "high" => 75,
            "critical" => 100,
            _ => 50,
        }
    }

    fn severity_to_score(severity: Severity) -> u8 {
        match severity {
            Severity::Info => 10,
            Severity::Low => 25,
            Severity::Medium => 50,
            Severity::High => 75,
            Severity::Critical => 100,
        }
    }

    pub fn score_to_threat_level(score: u8) -> super::decision::ThreatLevel {
        match score {
            0..=20 => super::decision::ThreatLevel::Minimal,
            21..=40 => super::decision::ThreatLevel::Low,
            41..=60 => super::decision::ThreatLevel::Medium,
            61..=80 => super::decision::ThreatLevel::High,
            81..=100 => super::decision::ThreatLevel::Critical,
            _ => super::decision::ThreatLevel::Critical,
        }
    }
}
