use tracing::Level;

/// Trait for errors that can provide their own severity level for tracing
pub trait ErrorSeverity {
    /// Returns the tracing level that should be used when this error occurs
    fn severity(&self) -> Level;

    /// Returns the variant name of the error for aggregate tracking
    fn variant_name(&self) -> &'static str {
        "unknown"
    }
}

impl ErrorSeverity for es_entity::EsEntityError {
    fn severity(&self) -> Level {
        match self {
            es_entity::EsEntityError::ConcurrentModification => Level::WARN,
            _ => Level::ERROR,
        }
    }

    fn variant_name(&self) -> &'static str {
        match self {
            es_entity::EsEntityError::UninitializedFieldError(_) => "UninitializedFieldError",
            es_entity::EsEntityError::EventDeserialization(_) => "EventDeserialization",
            es_entity::EsEntityError::NotFound => "NotFound",
            es_entity::EsEntityError::ConcurrentModification => "ConcurrentModification",
        }
    }
}
