use thiserror::Error;

#[derive(Debug, Error)]
pub enum DepositAccountError {
    #[error("Error from event store: {0}")]
    EventStoreError(#[from] es_entity::EsEntityError),
    #[error("Could not generate short code ID")]
    CouldNotGenerateShortCodeId,
    #[error("Sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Cursor error: {0}")]
    CursorError(#[from] es_entity::CursorDestructureError),
    #[error("Short code ID cannot be negative: {0}")]
    NegativeShortCodeId(i64),
    #[error("Short code ID cannot be greater than 9999999: {0}")]
    ShortCodeIdTooLarge(i64),
    #[error("Could not parse account short code")]
    AccountCodeParseError,
}

impl DepositAccountError {
    pub fn was_not_found(&self) -> bool {
        matches!(
            self,
            Self::EventStoreError(es_entity::EsEntityError::NotFound)
        )
    }
}
