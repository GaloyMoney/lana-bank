use outbox::Outbox;

#[derive(Debug, thiserror::Error)]
pub enum PriceListenerError {
    #[error("PriceListenerError - Outbox not configured")]
    OutboxNotConfigured,
    #[error("PriceListenerError - No price available")]
    NoPriceAvailable,
    #[error("PriceListenerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}
