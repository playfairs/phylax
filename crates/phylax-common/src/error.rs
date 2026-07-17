use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Permission error: {0}")]
    PermissionError(String),

    #[error("Platform error: {0}")]
    PlatformError(String),

    #[error("Rule error: {0}")]
    RuleError(String),

    #[error("Engine error: {0}")]
    EngineError(String),

    #[error("Collector error: {0}")]
    CollectorError(String),

    #[error("Responder error: {0}")]
    ResponderError(String),

    #[error("Alert error: {0}")]
    AlertError(String),

    #[error("IPC error: {0}")]
    IpcError(String),

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("Plugin error: {0}")]
    PluginError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Logging error: {0}")]
    LoggingError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
