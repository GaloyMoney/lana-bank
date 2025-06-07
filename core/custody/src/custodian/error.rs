use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustodianError {
    #[error("CustodianError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustodianError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CustodianError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CustodianError - Could not decrypt Custodian config: {0}")]
    CouldNotDecryptCustodianConfig(chacha20poly1305::Error),
    #[error("CustodianError - FromHex: {0}")]
    FromHex(#[from] hex::FromHexError),
    #[error("CustodianError - SerdeJsonError: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

es_entity::from_es_entity_error!(CustodianError);
