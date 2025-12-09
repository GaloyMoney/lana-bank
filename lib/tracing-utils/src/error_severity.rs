use std::sync::Arc;

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

impl ErrorSeverity for std::io::Error {
    fn severity(&self) -> Level {
        Level::ERROR
    }
}

impl<E> ErrorSeverity for Arc<E>
where
    E: ErrorSeverity + ?Sized,
{
    fn severity(&self) -> tracing::Level {
        // delegate to the inner error
        (**self).severity()
    }
}
