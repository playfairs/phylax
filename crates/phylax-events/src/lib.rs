use phylax_common::{Severity, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub mod types;

pub use types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub severity: Severity,
    pub source: EventSource,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Event {
    pub fn new(event_type: EventType, severity: Severity, source: EventSource) -> Self {
        Event {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            severity,
            source,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn get_metadata<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T> {
        self.metadata
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| Error::ValidationError(format!("Missing or invalid metadata: {}", key)))
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize event: {}", e)))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| Error::SerializationError(format!("Failed to deserialize event: {}", e)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    AuthenticationLogin,
    AuthenticationLogout,
    AuthenticationFailure,
    SshLogin,
    SshLogout,
    ProcessCreated,
    ProcessTerminated,
    FilesystemCreate,
    FilesystemModify,
    FilesystemDelete,
    NetworkConnection,
    NetworkDisconnection,
    PrivilegeEscalation,
    PrivilegeDrop,
    PersistenceCreated,
    PersistenceRemoved,
    ConfigurationChange,
    ServiceStarted,
    ServiceStopped,
    ServiceModified,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::AuthenticationLogin => "authentication.login",
            EventType::AuthenticationLogout => "authentication.logout",
            EventType::AuthenticationFailure => "authentication.failure",
            EventType::SshLogin => "ssh.login",
            EventType::SshLogout => "ssh.logout",
            EventType::ProcessCreated => "process.created",
            EventType::ProcessTerminated => "process.terminated",
            EventType::FilesystemCreate => "filesystem.create",
            EventType::FilesystemModify => "filesystem.modify",
            EventType::FilesystemDelete => "filesystem.delete",
            EventType::NetworkConnection => "network.connection",
            EventType::NetworkDisconnection => "network.disconnection",
            EventType::PrivilegeEscalation => "privilege.escalation",
            EventType::PrivilegeDrop => "privilege.drop",
            EventType::PersistenceCreated => "persistence.created",
            EventType::PersistenceRemoved => "persistence.removed",
            EventType::ConfigurationChange => "configuration.change",
            EventType::ServiceStarted => "service.started",
            EventType::ServiceStopped => "service.stopped",
            EventType::ServiceModified => "service.modified",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "authentication.login" => Some(EventType::AuthenticationLogin),
            "authentication.logout" => Some(EventType::AuthenticationLogout),
            "authentication.failure" => Some(EventType::AuthenticationFailure),
            "ssh.login" => Some(EventType::SshLogin),
            "ssh.logout" => Some(EventType::SshLogout),
            "process.created" => Some(EventType::ProcessCreated),
            "process.terminated" => Some(EventType::ProcessTerminated),
            "filesystem.create" => Some(EventType::FilesystemCreate),
            "filesystem.modify" => Some(EventType::FilesystemModify),
            "filesystem.delete" => Some(EventType::FilesystemDelete),
            "network.connection" => Some(EventType::NetworkConnection),
            "network.disconnection" => Some(EventType::NetworkDisconnection),
            "privilege.escalation" => Some(EventType::PrivilegeEscalation),
            "privilege.drop" => Some(EventType::PrivilegeDrop),
            "persistence.created" => Some(EventType::PersistenceCreated),
            "persistence.removed" => Some(EventType::PersistenceRemoved),
            "configuration.change" => Some(EventType::ConfigurationChange),
            "service.started" => Some(EventType::ServiceStarted),
            "service.stopped" => Some(EventType::ServiceStopped),
            "service.modified" => Some(EventType::ServiceModified),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSource {
    pub hostname: String,
    pub process_id: Option<u32>,
    pub process_name: Option<String>,
    pub user: Option<String>,
    pub ip_address: Option<String>,
}

impl EventSource {
    pub fn new(hostname: String) -> Self {
        EventSource {
            hostname,
            process_id: None,
            process_name: None,
            user: None,
            ip_address: None,
        }
    }

    pub fn with_process_id(mut self, pid: u32) -> Self {
        self.process_id = Some(pid);
        self
    }

    pub fn with_process_name(mut self, name: String) -> Self {
        self.process_name = Some(name);
        self
    }

    pub fn with_user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn with_ip_address(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }
}