use phylax_common::{Error, Result, Severity};
use phylax_events::Event;
use tracing::{info, warn, error, debug};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use tracing_appender::{rolling, non_blocking, RollingFileAppender};
use std::path::PathBuf;

pub mod config;

pub use config::*;

pub struct Logger {
    _non_blocking: non_blocking::WorkerGuard,
}

impl Logger {
    pub fn new(config: LoggingConfig) -> Result<Self> {
        let log_dir = config.log_path.unwrap_or_else(|| PathBuf::from("/var/log/phylax"));
        std::fs::create_dir_all(&log_dir)
            .map_err(|e| Error::LoggingError(format!("Failed to create log directory: {}", e)))?;

        let file_appender = rolling::daily(&log_dir, "phylax");
        let (non_blocking, guard) = non_blocking::NonBlocking::new(file_appender);

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&config.level));

        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .json()
                    .with_writer(non_blocking)
                    .with_target(true)
                    .with_thread_ids(true)
            );

        if config.console_output {
            subscriber.with(fmt::layer().pretty()).init();
        } else {
            subscriber.init();
        }

        Ok(Logger {
            _non_blocking: guard,
        })
    }

    pub fn log_event(&self, event: &Event) {
        let level = match event.severity {
            Severity::Info => "info",
            Severity::Low => "info",
            Severity::Medium => "warn",
            Severity::High => "warn",
            Severity::Critical => "error",
        };

        let event_data = serde_json::json!({
            "event_id": event.id,
            "event_type": event.event_type.as_str(),
            "severity": event.severity.as_str(),
            "source": event.source,
            "metadata": event.metadata,
        });

        match level {
            "info" => info!(
                event_id = %event.id,
                event_type = %event.event_type.as_str(),
                severity = %event.severity.as_str(),
                "Event logged",
                event = %serde_json::to_string(&event_data).unwrap_or_default()
            ),
            "warn" => warn!(
                event_id = %event.id,
                event_type = %event.event_type.as_str(),
                severity = %event.severity.as_str(),
                "Security event detected",
                event = %serde_json::to_string(&event_data).unwrap_or_default()
            ),
            "error" => error!(
                event_id = %event.id,
                event_type = %event.event_type.as_str(),
                severity = %event.severity.as_str(),
                "Critical security event",
                event = %serde_json::to_string(&event_data).unwrap_or_default()
            ),
            _ => debug!(
                event_id = %event.id,
                event_type = %event.event_type.as_str(),
                "Event logged"
            ),
        }
    }

    pub fn log_security_event(&self, event: &Event) {
        let security_data = serde_json::json!({
            "event_id": event.id,
            "event_type": event.event_type.as_str(),
            "severity": event.severity.as_str(),
            "source": event.source,
            "metadata": event.metadata,
            "timestamp": event.timestamp.to_rfc3339(),
        });

        error!(
            event_id = %event.id,
            event_type = %event.event_type.as_str(),
            severity = %event.severity.as_str(),
            "SECURITY EVENT",
            security_event = %serde_json::to_string(&security_data).unwrap_or_default()
        );
    }

    pub fn log_error(&self, context: &str, error: &Error) {
        error!(
            context = context,
            error = %error,
            "Error occurred"
        );
    }

    pub fn log_info(&self, message: &str) {
        info!(message = message, "Info");
    }

    pub fn log_warning(&self, message: &str) {
        warn!(message = message, "Warning");
    }
}