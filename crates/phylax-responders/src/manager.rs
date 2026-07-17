use phylax_common::{Error, Result};
use phylax_engine::{Decision, Action};
use crate::{Responder, ResponderConfig, ResponseResult};
use std::sync::Arc;
use std::collections::HashMap;

pub struct ResponderManager {
    responders: Vec<Box<dyn Responder + Send + Sync>>,
    config: ResponderConfig,
}

impl ResponderManager {
    pub fn new(config: ResponderConfig) -> Self {
        let mut responders: Vec<Box<dyn Responder + Send + Sync>> = Vec::new();

        if config.allow_ip_blocking {
            responders.push(Box::new(
                crate::IpBlockingResponder::new(config.require_approval, None)
            ));
        }

        if config.allow_process_termination {
            responders.push(Box::new(
                crate::ProcessTerminationResponder::new(config.require_approval, false)
            ));
        }

        responders.push(Box::new(
            crate::FileQuarantineResponder::new(config.require_approval, std::path::PathBuf::from("/var/lib/phylax/quarantine"), true)
        ));

        if config.allow_account_disable {
            responders.push(Box::new(
                crate::AccountDisableResponder::new(config.require_approval)
            ));
        }

        ResponderManager {
            responders,
            config,
        }
    }

    pub fn execute_decision(&self, decision: &Decision) -> Result<HashMap<String, ResponseResult>> {
        let mut results = HashMap::new();

        for action in &decision.actions {
            for responder in &self.responders {
                if self.should_execute(responder, action) {
                    let result = responder.execute(decision, action)?;
                    results.insert(responder.name().to_string(), result);
                }
            }
        }

        Ok(results)
    }

    fn should_execute(&self, responder: &Box<dyn Responder + Send + Sync>, action: &Action) -> bool {
        match action.action_type {
            phylax_engine::ActionType::BlockIp => responder.name() == "ip_blocking",
            phylax_engine::ActionType::TerminateProcess => responder.name() == "process_termination",
            phylax_engine::ActionType::QuarantineFile => responder.name() == "file_quarantine",
            phylax_engine::ActionType::DisableAccount => responder.name() == "account_disable",
            _ => false,
        }
    }

    pub fn add_responder(&mut self, responder: Box<dyn Responder + Send + Sync>) {
        self.responders.push(responder);
    }

    pub fn responder_count(&self) -> usize {
        self.responders.len()
    }
}
