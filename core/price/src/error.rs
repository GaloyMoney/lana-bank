use thiserror::Error;
//error[E0277]: `?` couldn't convert the error to `PriceError`
// --> core/price/src/lib.rs:54:62
// |
// 54 |         let mut stream = self.outbox.listen_ephemeral().await?;
// |                          ------------------------------------^ the trait `From<sqlx_core::error::Error>` is not implemented for `PriceError`
// |                          |
// |                          this can't be annotated with `?` because it has type `Result<_, sqlx_core::error::Error>`
// |
// note: `PriceError` needs to implement `From<sqlx_core::error::Error>`
// --> core/price/src/error.rs:4:1
// |
// 4 | pub enum PriceError {
// | ^^^^^^^^^^^^^^^^^^^
// = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
// = help: the following other types implement trait `From<T>`:
//           `PriceError` implements `From<BfxClientError>`
//           `PriceError` implements `From<ConversionError>`
//           `PriceError` implements `From<JobError>`

#[derive(Error, Debug)]
pub enum PriceError {
    #[error("PriceError - BfxClientError: {0}")]
    BfxClientError(#[from] super::bfx_client::error::BfxClientError),
    #[error("PriceError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
    #[error("PriceError - Outbox not configured")]
    OutboxNotConfigured,
    #[error("PriceError - JobError: {0}")]
    Job(#[from] job::error::JobError),
    #[error("PriceError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}
