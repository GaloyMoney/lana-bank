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
