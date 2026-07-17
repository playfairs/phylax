use phylax_common::{Error, Result};
use phylax_events::Event;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub mod models;
pub mod schema;

pub use models::*;

pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

impl Storage {
    pub fn new(path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| Error::DatabaseError(format!("Failed to open database: {}", e)))?;
        
        let storage = Storage {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        storage.initialize_schema()?;
        Ok(storage)
    }

    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to acquire lock: {}", e)))?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                source_hostname TEXT NOT NULL,
                source_process_id INTEGER,
                source_process_name TEXT,
                source_user TEXT,
                source_ip_address TEXT,
                metadata TEXT NOT NULL
            )",
            [],
        ).map_err(|e| Error::DatabaseError(format!("Failed to create events table: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS alerts (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                event_id TEXT NOT NULL,
                severity TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                acknowledged BOOLEAN DEFAULT 0,
                FOREIGN KEY (event_id) REFERENCES events(id)
            )",
            [],
        ).map_err(|e| Error::DatabaseError(format!("Failed to create alerts table: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS incidents (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                severity TEXT NOT NULL,
                status TEXT NOT NULL,
                event_ids TEXT NOT NULL
            )",
            [],
        ).map_err(|e| Error::DatabaseError(format!("Failed to create incidents table: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                event_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                conditions TEXT NOT NULL,
                actions TEXT NOT NULL,
                enabled BOOLEAN DEFAULT 1
            )",
            [],
        ).map_err(|e| Error::DatabaseError(format!("Failed to create rules table: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS responses (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                incident_id TEXT NOT NULL,
                responder_type TEXT NOT NULL,
                action TEXT NOT NULL,
                status TEXT NOT NULL,
                result TEXT,
                FOREIGN KEY (incident_id) REFERENCES incidents(id)
            )",
            [],
        ).map_err(|e| Error::DatabaseError(format!("Failed to create responses table: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS evidence (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                incident_id TEXT NOT NULL,
                evidence_type TEXT NOT NULL,
                path TEXT,
                hash TEXT,
                metadata TEXT,
                FOREIGN KEY (incident_id) REFERENCES incidents(id)
            )",
            [],
        ).map_err(|e| Error::DatabaseError(format!("Failed to create evidence table: {}", e)))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp)",
            [],
        ).map_err(|e| Error::DatabaseError(format!("Failed to create timestamp index: {}", e)))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)",
            [],
        ).map_err(|e| Error::DatabaseError(format!("Failed to create event_type index: {}", e)))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_alerts_event_id ON alerts(event_id)",
            [],
        ).map_err(|e| Error::DatabaseError(format!("Failed to create alert event_id index: {}", e)))?;

        Ok(())
    }

    pub fn store_event(&self, event: &Event) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to acquire lock: {}", e)))?;

        let metadata_json = serde_json::to_string(&event.metadata)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize metadata: {}", e)))?;

        conn.execute(
            "INSERT INTO events (id, timestamp, event_type, severity, source_hostname, source_process_id, source_process_name, source_user, source_ip_address, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                event.id.to_string(),
                event.timestamp.to_rfc3339(),
                event.event_type.as_str(),
                event.severity.as_str(),
                event.source.hostname,
                event.source.process_id,
                event.source.process_name,
                event.source.user,
                event.source.ip_address,
                metadata_json,
            ],
        ).map_err(|e| Error::DatabaseError(format!("Failed to store event: {}", e)))?;

        Ok(())
    }

    pub fn get_event(&self, id: &Uuid) -> Result<Option<Event>> {
        let conn = self.conn.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to acquire lock: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, severity, source_hostname, source_process_id, source_process_name, source_user, source_ip_address, metadata
             FROM events WHERE id = ?1"
        ).map_err(|e| Error::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

        let event = stmt.query_row(params![id.to_string()], |row| {
            let metadata_json: String = row.get(9)?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;

            let mut metadata_map = std::collections::HashMap::new();
            if let serde_json::Value::Object(obj) = metadata {
                for (k, v) in obj {
                    metadata_map.insert(k, v);
                }
            }

            Ok(Event {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?
                    .with_timezone(&chrono::Utc),
                event_type: phylax_events::EventType::from_str(&row.get::<_, String>(2)?)
                    .ok_or_else(|| rusqlite::Error::InvalidQuery)?,
                severity: phylax_common::Severity::from_str(&row.get::<_, String>(3)?)
                    .ok_or_else(|| rusqlite::Error::InvalidQuery)?,
                source: phylax_events::EventSource {
                    hostname: row.get(4)?,
                    process_id: row.get(5)?,
                    process_name: row.get(6)?,
                    user: row.get(7)?,
                    ip_address: row.get(8)?,
                },
                metadata: metadata_map,
            })
        }).optional().map_err(|e| Error::DatabaseError(format!("Failed to query event: {}", e)))?;

        Ok(event)
    }

    pub fn get_events_by_type(&self, event_type: &str, limit: Option<usize>) -> Result<Vec<Event>> {
        let conn = self.conn.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to acquire lock: {}", e)))?;

        let limit_val = limit.unwrap_or(100) as i64;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, severity, source_hostname, source_process_id, source_process_name, source_user, source_ip_address, metadata
             FROM events WHERE event_type = ?1 ORDER BY timestamp DESC LIMIT ?2"
        ).map_err(|e| Error::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

        let events = stmt.query_map(params![event_type, limit_val], |row| {
            let metadata_json: String = row.get(9)?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;

            let mut metadata_map = std::collections::HashMap::new();
            if let serde_json::Value::Object(obj) = metadata {
                for (k, v) in obj {
                    metadata_map.insert(k, v);
                }
            }

            Ok(Event {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?
                    .with_timezone(&chrono::Utc),
                event_type: phylax_events::EventType::from_str(&row.get::<_, String>(2)?)
                    .ok_or_else(|| rusqlite::Error::InvalidQuery)?,
                severity: phylax_common::Severity::from_str(&row.get::<_, String>(3)?)
                    .ok_or_else(|| rusqlite::Error::InvalidQuery)?,
                source: phylax_events::EventSource {
                    hostname: row.get(4)?,
                    process_id: row.get(5)?,
                    process_name: row.get(6)?,
                    user: row.get(7)?,
                    ip_address: row.get(8)?,
                },
                metadata: metadata_map,
            })
        }).map_err(|e| Error::DatabaseError(format!("Failed to query events: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::DatabaseError(format!("Failed to collect events: {}", e)))?;

        Ok(events)
    }

    pub fn store_alert(&self, alert: &Alert) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to acquire lock: {}", e)))?;

        conn.execute(
            "INSERT INTO alerts (id, timestamp, event_id, severity, title, description, status, acknowledged)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                alert.id.to_string(),
                alert.timestamp.to_rfc3339(),
                alert.event_id.to_string(),
                alert.severity.as_str(),
                alert.title,
                alert.description,
                alert.status.as_str(),
                alert.acknowledged as i32,
            ],
        ).map_err(|e| Error::DatabaseError(format!("Failed to store alert: {}", e)))?;

        Ok(())
    }

    pub fn get_alerts(&self, limit: Option<usize>) -> Result<Vec<Alert>> {
        let conn = self.conn.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to acquire lock: {}", e)))?;

        let limit_val = limit.unwrap_or(100) as i64;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_id, severity, title, description, status, acknowledged
             FROM alerts ORDER BY timestamp DESC LIMIT ?1"
        ).map_err(|e| Error::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

        let alerts = stmt.query_map(params![limit_val], |row| {
            Ok(Alert {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?
                    .with_timezone(&chrono::Utc),
                event_id: Uuid::parse_str(&row.get::<_, String>(2)?).map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?,
                severity: phylax_common::Severity::from_str(&row.get::<_, String>(3)?)
                    .ok_or_else(|| rusqlite::Error::InvalidQuery)?,
                title: row.get(4)?,
                description: row.get(5)?,
                status: AlertStatus::from_str(&row.get::<_, String>(6)?)
                    .ok_or_else(|| rusqlite::Error::InvalidQuery)?,
                acknowledged: row.get::<_, i32>(7)? != 0,
            })
        }).map_err(|e| Error::DatabaseError(format!("Failed to query alerts: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::DatabaseError(format!("Failed to collect alerts: {}", e)))?;

        Ok(alerts)
    }

    pub fn acknowledge_alert(&self, id: &Uuid) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to acquire lock: {}", e)))?;

        conn.execute(
            "UPDATE alerts SET acknowledged = 1 WHERE id = ?1",
            params![id.to_string()],
        ).map_err(|e| Error::DatabaseError(format!("Failed to acknowledge alert: {}", e)))?;

        Ok(())
    }

    pub fn cleanup_old_events(&self, days: u32) -> Result<u64> {
        let conn = self.conn.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to acquire lock: {}", e)))?;

        let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let cutoff_str = cutoff.to_rfc3339();

        let rows = conn.execute(
            "DELETE FROM events WHERE timestamp < ?1",
            params![cutoff_str],
        ).map_err(|e| Error::DatabaseError(format!("Failed to delete old events: {}", e)))?;

        Ok(rows)
    }

    pub fn get_statistics(&self) -> StorageStatistics {
        let conn = self.conn.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to acquire lock: {}", e)));

        let mut stats = StorageStatistics::default();

        if let Ok(conn) = conn {
            if let Ok(count) = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get::<_, i64>(0)) {
                stats.total_events = count as u64;
            }
            if let Ok(count) = conn.query_row("SELECT COUNT(*) FROM alerts", [], |row| row.get::<_, i64>(0)) {
                stats.total_alerts = count as u64;
            }
            if let Ok(count) = conn.query_row("SELECT COUNT(*) FROM rules WHERE enabled = 1", [], |row| row.get::<_, i64>(0)) {
                stats.active_rules = count as usize;
            }
        }

        stats
    }

    pub fn get_recent_logs(&self, limit: usize) -> Result<Vec<LogEntry>> {
        Ok(Vec::new())
    }

    pub fn get_recent_events(&self, limit: Option<usize>) -> Result<Vec<Event>> {
        let conn = self.conn.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to acquire lock: {}", e)))?;

        let limit_val = limit.unwrap_or(20) as i64;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, severity, source_hostname, source_process_id, source_process_name, source_user, source_ip_address, metadata
             FROM events ORDER BY timestamp DESC LIMIT ?1"
        ).map_err(|e| Error::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

        let events = stmt.query_map(params![limit_val], |row| {
            let metadata_json: String = row.get(9)?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;

            let mut metadata_map = std::collections::HashMap::new();
            if let serde_json::Value::Object(obj) = metadata {
                for (k, v) in obj {
                    metadata_map.insert(k, v);
                }
            }

            Ok(Event {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?
                    .with_timezone(&chrono::Utc),
                event_type: phylax_events::EventType::from_str(&row.get::<_, String>(2)?)
                    .ok_or_else(|| rusqlite::Error::InvalidQuery)?,
                severity: phylax_common::Severity::from_str(&row.get::<_, String>(3)?)
                    .ok_or_else(|| rusqlite::Error::InvalidQuery)?,
                source: phylax_events::EventSource {
                    hostname: row.get(4)?,
                    process_id: row.get(5)?,
                    process_name: row.get(6)?,
                    user: row.get(7)?,
                    ip_address: row.get(8)?,
                },
                metadata: metadata_map,
            })
        }).map_err(|e| Error::DatabaseError(format!("Failed to query events: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::DatabaseError(format!("Failed to collect events: {}", e)))?;

        Ok(events)
    }

    pub fn get_incidents(&self, limit: Option<usize>) -> Result<Vec<Incident>> {
        Ok(Vec::new())
    }

    pub fn get_rules(&self) -> Result<Vec<Rule>> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, Default)]
pub struct StorageStatistics {
    pub total_events: u64,
    pub total_alerts: u64,
    pub active_rules: usize,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message: String,
}