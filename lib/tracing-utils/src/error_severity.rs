use tracing::Level;

/// Trait for errors that can provide their own severity level for tracing
pub trait ErrorSeverity {
    /// Returns the tracing level that should be used when this error occurs
    fn severity(&self) -> Level;
}

impl ErrorSeverity for es_entity::EsEntityError {
    fn severity(&self) -> Level {
        match self {
            es_entity::EsEntityError::ConcurrentModification => Level::WARN,
            _ => Level::ERROR,
        }
    }
}

/// Implementation for sqlx::Error - provides appropriate severity levels for database errors
impl ErrorSeverity for sqlx::Error {
    fn severity(&self) -> Level {
        match self {
            // Database errors are generally warnings (could be transient)
            sqlx::Error::Database(_) => Level::WARN,
            // Connection errors could be temporary
            sqlx::Error::PoolTimedOut => Level::WARN,
            sqlx::Error::PoolClosed => Level::WARN,
            sqlx::Error::Io(_) => Level::WARN,
            // Configuration/usage errors are more serious
            sqlx::Error::Configuration(_) => Level::ERROR,
            sqlx::Error::TypeNotFound { .. } => Level::ERROR,
            sqlx::Error::ColumnNotFound(_) => Level::ERROR,
            sqlx::Error::ColumnDecode { .. } => Level::ERROR,
            sqlx::Error::Decode(_) => Level::ERROR,
            sqlx::Error::Migrate(_) => Level::ERROR,
            // Other errors default to ERROR
            _ => Level::ERROR,
        }
    }
}
