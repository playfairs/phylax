use phylax_common::{Error, Result, Severity};
use phylax_events::Event;
use phylax_rules::{Rule, RuleEngine, Action, ActionType};
use phylax_storage::Storage;
use std::sync::Arc;
use crossbeam_channel::{Sender, Receiver, unbounded};
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub mod decision;
pub mod threat;

pub use decision::*;
pub use threat::*;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub max_queue_size: usize,
    pub worker_threads: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            max_queue_size: 10000,
            worker_threads: 4,
        }
    }
}

pub struct Engine {
    rule_engine: RuleEngine,
    storage: Arc<Storage>,
    event_sender: Sender<Event>,
    event_receiver: Receiver<Event>,
    decision_sender: Sender<Decision>,
    decision_receiver: Receiver<Decision>,
}

impl Engine {
    pub fn new(storage: Arc<Storage>, config: EngineConfig) -> Self {
        let (event_sender, event_receiver) = unbounded();
        let (decision_sender, decision_receiver) = unbounded();

        Engine {
            rule_engine: RuleEngine::new(),
            storage,
            event_sender,
            event_receiver,
            decision_sender,
            decision_receiver,
        }
    }

    pub fn set_rule_engine(&mut self, rule_engine: RuleEngine) {
        self.rule_engine = rule_engine;
    }

    pub fn get_event_sender(&self) -> Sender<Event> {
        self.event_sender.clone()
    }

    pub fn get_decision_receiver(&self) -> Receiver<Decision> {
        self.decision_receiver.clone()
    }

    pub fn process_event(&mut self, event: Event) -> Result<Vec<Decision>> {
        self.storage.store_event(&event)?;

        let matching_rules = self.rule_engine.evaluate_event(&event);
        let mut decisions = Vec::new();

        for rule in matching_rules {
            let threat_level = self.calculate_threat_level(&event, rule);
            let decision = self.create_decision(&event, rule, threat_level);
            decisions.push(decision);
        }

        for decision in &decisions {
            let _ = self.decision_sender.send(decision.clone());
        }

        Ok(decisions)
    }

    fn calculate_threat_level(&self, event: &Event, rule: &Rule) -> ThreatLevel {
        let base_score = match rule.severity.as_str() {
            "info" => 10,
            "low" => 25,
            "medium" => 50,
            "high" => 75,
            "critical" => 100,
            _ => 50,
        };

        let event_severity_score = match event.severity {
            Severity::Info => 10,
            Severity::Low => 25,
            Severity::Medium => 50,
            Severity::High => 75,
            Severity::Critical => 100,
        };

        let combined_score = (base_score + event_severity_score) / 2;

        match combined_score {
            0..=20 => ThreatLevel::Minimal,
            21..=40 => ThreatLevel::Low,
            41..=60 => ThreatLevel::Medium,
            61..=80 => ThreatLevel::High,
            81..=100 => ThreatLevel::Critical,
            _ => ThreatLevel::Critical,
        }
    }

    fn create_decision(&self, event: &Event, rule: &Rule, threat_level: ThreatLevel) -> Decision {
        let actions: Vec<Action> = rule.actions.clone();

        Decision {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_id: event.id,
            rule_id: rule.id,
            rule_name: rule.name.clone(),
            threat_level,
            actions,
            status: DecisionStatus::Pending,
        }
    }

    pub fn start_processing(&mut self) -> Result<()> {
        while let Ok(event) = self.event_receiver.recv() {
            if let Err(e) = self.process_event(event) {
                eprintln!("Failed to process event: {}", e);
            }
        }
        Ok(())
    }

    pub fn get_statistics(&self) -> EngineStatistics {
        EngineStatistics {
            total_events_processed: 0,
            total_decisions_made: 0,
            active_rules: self.rule_engine.get_rules().len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineStatistics {
    pub total_events_processed: u64,
    pub total_decisions_made: u64,
    pub active_rules: usize,
}